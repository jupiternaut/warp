use std::collections::HashSet;
#[cfg(not(target_family = "wasm"))]
use std::io::{BufRead, BufReader, Write};
use std::process::Stdio;
#[cfg(not(target_family = "wasm"))]
use std::sync::mpsc;
use std::sync::Arc;
#[cfg(not(target_family = "wasm"))]
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, bail, Context};
use command::blocking::Command as BlockingCommand;
use command::r#async::Command;
use futures::channel::oneshot;
use futures::{StreamExt, TryFutureExt};
use instant::Instant;
use itertools::Itertools;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;
use warp_multi_agent_api as api;

use crate::ai::generate_code_review_content::api::OutputType;
use crate::ai::predict::generate_ai_input_suggestions::{
    AgentModeSuggestionV2, GenerateAIInputSuggestionsRequest, GenerateAIInputSuggestionsResponseV2,
};
use crate::ai::predict::generate_am_query_suggestions::{
    GenerateAMQuerySuggestionsRequest, GenerateAMQuerySuggestionsResponse, SimpleQuery, Suggestion,
};
use crate::ai::request_usage_model::RequestLimitInfo;
use crate::ai_assistant::requests::GenerateDialogueResult;
use crate::ai_assistant::utils::TranscriptPart;
use crate::drive::workflows::ai_assist::{
    GeneratedArgument, GeneratedCommandMetadata, GeneratedCommandMetadataError,
};
use crate::server::server_api::AIApiError;

const ENABLE_ENV: &str = "WARP_LOCAL_CODEX_AI";
const CODEX_BIN_ENV: &str = "WARP_LOCAL_CODEX_BIN";
const CODEX_RUNNER_ENV: &str = "WARP_LOCAL_CODEX_RUNNER";
const CODEX_TIMEOUT_MS_ENV: &str = "WARP_LOCAL_CODEX_TIMEOUT_MS";
const INTEGRATION_TEST_ENV: &str = "WARP_TEST_DISABLE_KEYBINDING_SAVE";
const LOCAL_CONVERSATION_PREFIX: &str = "local-codex-";
pub(crate) const DISPLAY_ONLY_TOOL_CALL_PREFIX: &str = "local-codex-display-";
const CODEX_TIMEOUT: Duration = Duration::from_secs(180);
pub const MODEL_ID: &str = "local-codex-chatgpt-oauth";
pub const MODEL_DISPLAY_NAME: &str = "Local Codex (ChatGPT OAuth)";

#[derive(Debug, Deserialize)]
struct CodexJsonEvent {
    #[serde(rename = "type")]
    kind: String,
    item: Option<CodexJsonItem>,
}

#[derive(Debug, Deserialize)]
struct CodexJsonItem {
    #[serde(rename = "type")]
    kind: String,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GeneratedCommandList {
    commands: Vec<GeneratedCommand>,
}

#[derive(Debug, Deserialize)]
pub struct GeneratedCommand {
    command: String,
    description: String,
    #[serde(default)]
    parameters: Vec<GeneratedCommandParameter>,
}

#[derive(Debug, Deserialize)]
pub struct GeneratedCommandParameter {
    #[serde(default)]
    id: String,
    #[serde(default)]
    description: String,
}

#[derive(Debug, Deserialize)]
struct GeneratedMetadata {
    parameterized_command: String,
    title: String,
    description: String,
    #[serde(default)]
    arguments: Vec<GeneratedMetadataArgument>,
}

#[derive(Debug, Deserialize)]
struct GeneratedMetadataArgument {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    default_value: String,
}

#[derive(Debug, Deserialize, Default)]
struct PassivePromptSuggestion {
    #[serde(default)]
    prompt: String,
    #[serde(default)]
    label: String,
    #[serde(default)]
    is_trigger_irrelevant: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CodexRunnerKind {
    Exec,
    AppServer,
}

impl CodexRunnerKind {
    fn selected() -> Self {
        match std::env::var(CODEX_RUNNER_ENV) {
            Ok(value) if value.eq_ignore_ascii_case("app-server") => Self::AppServer,
            _ => Self::Exec,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
enum LocalCodexEvent {
    Text(String),
    Reasoning(String),
    ToolCall(String),
    FileDiff(String),
    CommandResult(String),
    Unsupported(String),
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
enum LocalCodexRenderableEvent {
    AgentTextDelta {
        item_id: String,
        delta: String,
    },
    ReasoningDelta {
        item_id: String,
        delta: String,
    },
    CommandDisplayCard {
        item_id: String,
        command: String,
    },
    FileDiffDisplayCard {
        item_id: String,
        changes: Vec<CodexFilePatchChange>,
    },
    CommandTranscript {
        item_id: String,
        transcript: String,
    },
    FileDiffTranscript {
        item_id: String,
        diff: String,
    },
    ToolTranscript {
        item_id: String,
        transcript: String,
    },
    Unsupported {
        item_id: String,
        transcript: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CodexFilePatchChange {
    path: String,
    diff: String,
    move_to: String,
}

#[derive(Debug, Default)]
struct LocalCodexStreamingTranslator {
    created_messages: HashSet<String>,
}

#[derive(Debug)]
enum CodexRunOutcome {
    Completed(String),
    Cancelled,
}

#[cfg(not(target_family = "wasm"))]
#[derive(Debug)]
enum AppServerReadEvent {
    Message(Value),
    Eof,
    Error(String),
}

#[cfg(not(target_family = "wasm"))]
#[derive(Debug)]
enum AppServerStreamOutcome {
    Completed,
    Cancelled,
}

pub fn enabled() -> bool {
    match std::env::var(ENABLE_ENV) {
        Ok(value) => !matches!(
            value.to_ascii_lowercase().as_str(),
            "0" | "false" | "off" | "no"
        ),
        Err(_) => !cfg!(test) && std::env::var(INTEGRATION_TEST_ENV).is_err(),
    }
}

fn codex_program() -> String {
    std::env::var(CODEX_BIN_ENV).unwrap_or_else(|_| "codex".to_string())
}

fn codex_timeout() -> Duration {
    std::env::var(CODEX_TIMEOUT_MS_ENV)
        .ok()
        .and_then(|value| value.parse().ok())
        .map(Duration::from_millis)
        .unwrap_or(CODEX_TIMEOUT)
}

pub fn runner_description() -> &'static str {
    match CodexRunnerKind::selected() {
        CodexRunnerKind::Exec => "local codex exec",
        CodexRunnerKind::AppServer => "local codex app-server",
    }
}

pub fn is_local_conversation_token(token: &str) -> bool {
    token.starts_with(LOCAL_CONVERSATION_PREFIX)
}

fn agent_turn_prompt(user_prompt: &str) -> String {
    user_prompt.to_string()
}

fn passive_suggestion_prompt(user_prompt: &str) -> String {
    format!(
        "Given this Warp terminal context, suggest one useful follow-up prompt chip. \
         Return only JSON with schema {{\"prompt\":\"prompt to run\", \"label\":\"short label\", \
         \"is_trigger_irrelevant\":false}}. Context:\n{user_prompt}"
    )
}

fn local_stream_ids(request: &api::Request) -> (String, String, String) {
    let conversation_id = request
        .metadata
        .as_ref()
        .map(|metadata| metadata.conversation_id.clone())
        .filter(|id| !id.is_empty())
        .unwrap_or_else(|| format!("{LOCAL_CONVERSATION_PREFIX}{}", Uuid::new_v4()));
    let request_id = format!("{LOCAL_CONVERSATION_PREFIX}{}", Uuid::new_v4());
    let run_id = format!("{LOCAL_CONVERSATION_PREFIX}run-{}", Uuid::new_v4());

    (conversation_id, request_id, run_id)
}

pub fn log_startup_state() {
    let enabled = enabled();
    let current_exe = std::env::current_exe()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|err| format!("unknown ({err})"));
    let codex_program = codex_program();
    let login_status = if enabled {
        match BlockingCommand::new(&codex_program)
            .args(["login", "status"])
            .output()
        {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let status = if stdout.trim().is_empty() {
                    stderr.trim()
                } else {
                    stdout.trim()
                };
                status.to_string()
            }
            Ok(output) => format!("login status failed with {}", output.status),
            Err(err) => format!("login status unavailable: {err}"),
        }
    } else {
        "not checked; using Warp AI credits".to_string()
    };

    log::warn!(
        "Local Codex AI: {}; exe={}; codex_bin={}; login_status={}",
        if enabled { "enabled" } else { "disabled" },
        current_exe,
        codex_program,
        login_status
    );
}

pub fn log_local_route(entrypoint: &str) {
    log::warn!("{entrypoint}: routing to local Codex; using 0 Warp AI credits");
}

pub fn log_warp_route(entrypoint: &str) {
    log::warn!("{entrypoint}: WARP_LOCAL_CODEX_AI disabled; using Warp AI credits");
}

pub async fn ensure_logged_in() -> anyhow::Result<()> {
    let codex_program = codex_program();
    command_output(&codex_program, ["--version"], Duration::from_secs(10))
        .await
        .context("Codex CLI is not installed or not on PATH. Install Codex, then run `codex` or `codex login --device-auth`.")?;

    let login_status = command_output(&codex_program, ["login", "status"], Duration::from_secs(15))
        .await
        .context("Unable to check Codex login status. Run `codex` or `codex login --device-auth`, then try again.")?;

    if !login_status.contains("Logged in") {
        bail!(
            "Codex CLI is not logged in. Run `codex` or `codex login --device-auth`, then try again."
        );
    }
    Ok(())
}

async fn command_output<const N: usize>(
    program: &str,
    args: [&str; N],
    timeout: Duration,
) -> anyhow::Result<String> {
    let mut command = Command::new(program);
    command.args(args);
    let output = futures_lite::future::race(command.output().map_err(anyhow::Error::from), async {
        warpui::r#async::Timer::after(timeout).await;
        Err(anyhow!("command timed out after {timeout:?}"))
    })
    .await?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !output.status.success() {
        bail!(
            "{program} failed with status {}: {}",
            output.status,
            stderr.trim()
        );
    }
    Ok(if stdout.trim().is_empty() {
        stderr
    } else {
        stdout
    })
}

async fn codex_exec(prompt: &str) -> anyhow::Result<String> {
    codex_exec_with_cwd(prompt, None).await
}

async fn codex_exec_with_cwd(prompt: &str, cwd: Option<&str>) -> anyhow::Result<String> {
    ensure_logged_in().await?;

    let codex_program = codex_program();
    let timeout = codex_timeout();
    log::warn!(
        "Local Codex AI: running codex exec --json; cwd={}",
        cwd.unwrap_or("<unset>")
    );

    let mut command = Command::new(&codex_program);
    command.args([
        "exec",
        "--json",
        "--dangerously-bypass-approvals-and-sandbox",
        "--skip-git-repo-check",
    ]);
    if let Some(cwd) = cwd.filter(|cwd| !cwd.trim().is_empty()) {
        command.arg("--cd").arg(cwd);
    }
    command.arg(prompt);

    let output = futures_lite::future::race(command.output().map_err(anyhow::Error::from), async {
        warpui::r#async::Timer::after(timeout).await;
        Err(anyhow!("Codex request timed out after {timeout:?}"))
    })
    .await?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !output.status.success() {
        bail!(
            "Codex CLI failed with status {}: {}",
            output.status,
            stderr.trim()
        );
    }

    log::warn!("Local Codex AI: codex exec completed successfully");
    Ok(extract_last_agent_message(&stdout).unwrap_or_else(|| stdout.trim().to_string()))
}

#[cfg(not(target_family = "wasm"))]
async fn codex_exec_cancellable(
    prompt: String,
    cwd: Option<String>,
    cancellation_rx: oneshot::Receiver<()>,
) -> anyhow::Result<CodexRunOutcome> {
    ensure_logged_in().await?;
    codex_exec_cancellable_inner(prompt, cwd, cancellation_rx).await
}

#[cfg(not(target_family = "wasm"))]
async fn codex_exec_cancellable_inner(
    prompt: String,
    cwd: Option<String>,
    cancellation_rx: oneshot::Receiver<()>,
) -> anyhow::Result<CodexRunOutcome> {
    enum RaceOutcome {
        Output(std::process::Output),
        Cancelled,
    }

    let codex_program = codex_program();
    let timeout = codex_timeout();
    log::warn!(
        "Local Codex AI: running cancellable codex exec --json; cwd={}",
        cwd.as_deref().unwrap_or("<unset>")
    );

    let mut command = Command::new_with_process_group(&codex_program);
    command
        .args([
            "exec",
            "--json",
            "--dangerously-bypass-approvals-and-sandbox",
            "--skip-git-repo-check",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    if let Some(cwd) = cwd.as_deref().filter(|cwd| !cwd.trim().is_empty()) {
        command.arg("--cd").arg(cwd);
    }
    command.arg(prompt);

    let child = command
        .spawn()
        .with_context(|| format!("Failed to start Codex CLI: {codex_program}"))?;
    let output = child
        .output()
        .map_ok(RaceOutcome::Output)
        .map_err(anyhow::Error::from);
    let timeout = async move {
        warpui::r#async::Timer::after(timeout).await;
        Err(anyhow!("Codex request timed out after {timeout:?}"))
    };
    let cancelled = async move {
        let _ = cancellation_rx.await;
        Ok(RaceOutcome::Cancelled)
    };

    match futures_lite::future::race(futures_lite::future::race(output, timeout), cancelled).await?
    {
        RaceOutcome::Cancelled => {
            log::warn!(
                "Local Codex AI: cancellation received; Codex child dropped with kill_on_drop"
            );
            Ok(CodexRunOutcome::Cancelled)
        }
        RaceOutcome::Output(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if !output.status.success() {
                bail!(
                    "Codex CLI failed with status {}: {}",
                    output.status,
                    stderr.trim()
                );
            }

            log::warn!("Local Codex AI: cancellable codex exec completed successfully");
            Ok(CodexRunOutcome::Completed(
                extract_last_agent_message(&stdout).unwrap_or_else(|| stdout.trim().to_string()),
            ))
        }
    }
}

#[cfg(not(target_family = "wasm"))]
async fn run_codex_app_server_response_stream(
    prompt: String,
    cwd: Option<String>,
    mut cancellation_rx: oneshot::Receiver<()>,
    task_id: String,
    request_id: String,
    tx: async_channel::Sender<Result<api::ResponseEvent, Arc<AIApiError>>>,
) -> anyhow::Result<AppServerStreamOutcome> {
    ensure_logged_in().await?;

    let codex_program = codex_program();
    let cwd = cwd.filter(|cwd| !cwd.trim().is_empty());
    log::warn!(
        "Local Codex AI: running codex app-server --listen stdio://; cwd={}",
        cwd.as_deref().unwrap_or("<unset>")
    );

    let mut child = BlockingCommand::new(&codex_program)
        .args(["app-server", "--listen", "stdio://"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to start Codex app-server: {codex_program}"))?;

    let mut stdin = child
        .stdin
        .take()
        .context("Codex app-server stdin was unavailable")?;
    let stdout = child
        .stdout
        .take()
        .context("Codex app-server stdout was unavailable")?;
    if let Some(stderr) = child.stderr.take() {
        drain_app_server_stderr(stderr);
    }

    let (read_tx, read_rx) = mpsc::channel();
    thread::spawn(move || read_app_server_stdout(stdout, read_tx));

    let mut next_id = 1_u64;
    let initialize_id = app_server_send_request(
        &mut stdin,
        &mut next_id,
        "initialize",
        json!({
            "clientInfo": {
                "name": "warp-codex-oss",
                "title": "WarpCodexOss",
                "version": env!("CARGO_PKG_VERSION"),
            },
            "capabilities": null,
        }),
    )?;
    app_server_wait_for_response(&read_rx, initialize_id)?;

    let thread_start_id = app_server_send_request(
        &mut stdin,
        &mut next_id,
        "thread/start",
        json!({
            "cwd": cwd.clone(),
            "approvalPolicy": "never",
            "sandbox": "danger-full-access",
            "experimentalRawEvents": false,
            "persistExtendedHistory": false,
        }),
    )?;
    let thread_response = app_server_wait_for_response(&read_rx, thread_start_id)?;
    let thread_id = thread_response
        .pointer("/result/thread/id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .context("Codex app-server thread/start response did not include thread.id")?;

    let turn_start_id = app_server_send_request(
        &mut stdin,
        &mut next_id,
        "turn/start",
        json!({
            "threadId": thread_id.clone(),
            "input": [{
                "type": "text",
                "text": prompt,
                "text_elements": [],
            }],
            "cwd": cwd.clone(),
            "approvalPolicy": "never",
            "sandboxPolicy": { "type": "dangerFullAccess" },
        }),
    )?;

    let mut translator = LocalCodexStreamingTranslator::default();
    let mut active_turn_id = None;
    let timeout = codex_timeout();
    let start = Instant::now();
    let outcome = loop {
        if cancellation_received(&mut cancellation_rx) {
            interrupt_app_server_turn(
                &mut stdin,
                &mut next_id,
                &thread_id,
                active_turn_id.as_deref(),
            );
            let _ = child.kill();
            break AppServerStreamOutcome::Cancelled;
        }

        match read_rx.recv_timeout(Duration::from_millis(100)) {
            Ok(AppServerReadEvent::Message(message)) => {
                log_app_server_event_method(&message);
                if let Some(error) = app_server_error_message(&message) {
                    bail!("Codex app-server error: {error}");
                }

                if message.get("id").and_then(Value::as_u64) == Some(turn_start_id) {
                    active_turn_id = message
                        .pointer("/result/turn/id")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                    continue;
                }

                if let Some(turn_id) = message
                    .pointer("/params/turn/id")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                {
                    active_turn_id.get_or_insert(turn_id);
                }

                if app_server_is_server_request(&message) {
                    respond_to_unsupported_app_server_request(&mut stdin, &message);
                    continue;
                }

                if let Some(event) = app_server_renderable_event(&message) {
                    let actions = translator.translate_event(&task_id, &request_id, event);
                    if !actions.is_empty()
                        && !send_ok_event(&tx, client_actions_event(actions)).await
                    {
                        let _ = child.kill();
                        break AppServerStreamOutcome::Cancelled;
                    }
                }

                if message.get("method").and_then(Value::as_str) == Some("turn/completed") {
                    break AppServerStreamOutcome::Completed;
                }
            }
            Ok(AppServerReadEvent::Eof) => {
                bail!("Codex app-server exited before completing the turn");
            }
            Ok(AppServerReadEvent::Error(error)) => {
                bail!("Codex app-server output error: {error}");
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    bail!("Codex app-server request timed out after {timeout:?}");
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                bail!("Codex app-server output channel closed");
            }
        }
    };

    let _ = child.kill();
    let _ = child.wait();
    Ok(outcome)
}

#[cfg(not(target_family = "wasm"))]
fn cancellation_received(cancellation_rx: &mut oneshot::Receiver<()>) -> bool {
    matches!(cancellation_rx.try_recv(), Ok(Some(())) | Err(_))
}

#[cfg(not(target_family = "wasm"))]
fn drain_app_server_stderr(stderr: std::process::ChildStderr) {
    thread::spawn(move || {
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            log::debug!("Codex app-server stderr: {line}");
        }
    });
}

#[cfg(not(target_family = "wasm"))]
fn read_app_server_stdout(stdout: std::process::ChildStdout, tx: mpsc::Sender<AppServerReadEvent>) {
    for line in BufReader::new(stdout).lines() {
        match line {
            Ok(line) if line.trim().is_empty() => {}
            Ok(line) => match serde_json::from_str::<Value>(&line) {
                Ok(message) => {
                    if tx.send(AppServerReadEvent::Message(message)).is_err() {
                        return;
                    }
                }
                Err(err) => {
                    let _ = tx.send(AppServerReadEvent::Error(format!("{err}: {line}")));
                    return;
                }
            },
            Err(err) => {
                let _ = tx.send(AppServerReadEvent::Error(err.to_string()));
                return;
            }
        }
    }
    let _ = tx.send(AppServerReadEvent::Eof);
}

#[cfg(not(target_family = "wasm"))]
fn app_server_send_request(
    stdin: &mut std::process::ChildStdin,
    next_id: &mut u64,
    method: &str,
    params: Value,
) -> anyhow::Result<u64> {
    let id = *next_id;
    *next_id += 1;
    let request = json!({
        "id": id,
        "method": method,
        "params": params,
    });
    writeln!(stdin, "{}", serde_json::to_string(&request)?)?;
    stdin.flush()?;
    Ok(id)
}

#[cfg(not(target_family = "wasm"))]
fn app_server_wait_for_response(
    read_rx: &mpsc::Receiver<AppServerReadEvent>,
    id: u64,
) -> anyhow::Result<Value> {
    let timeout = codex_timeout();
    let start = Instant::now();
    loop {
        let remaining = timeout
            .checked_sub(start.elapsed())
            .unwrap_or_else(|| Duration::from_millis(0));
        if remaining.is_zero() {
            bail!("Codex app-server request {id} timed out after {timeout:?}");
        }

        match read_rx.recv_timeout(remaining.min(Duration::from_millis(250))) {
            Ok(AppServerReadEvent::Message(message)) => {
                if let Some(error) = app_server_error_message(&message) {
                    bail!("Codex app-server error: {error}");
                }
                if message.get("id").and_then(Value::as_u64) == Some(id) {
                    return Ok(message);
                }
            }
            Ok(AppServerReadEvent::Eof) => bail!("Codex app-server exited before response {id}"),
            Ok(AppServerReadEvent::Error(error)) => bail!("Codex app-server output error: {error}"),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                bail!("Codex app-server output channel closed")
            }
        }
    }
}

#[cfg(not(target_family = "wasm"))]
fn app_server_error_message(message: &Value) -> Option<String> {
    if let Some(error) = message.get("error") {
        return Some(serde_json::to_string(error).unwrap_or_else(|_| error.to_string()));
    }
    if message.get("method").and_then(Value::as_str) == Some("error") {
        return Some(
            message
                .pointer("/params/error/message")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| message.to_string()),
        );
    }
    None
}

fn log_app_server_event_method(message: &Value) {
    let Some(method) = message.get("method").and_then(Value::as_str) else {
        return;
    };

    let item_id = message
        .pointer("/params/itemId")
        .or_else(|| message.pointer("/params/item/id"))
        .and_then(Value::as_str)
        .unwrap_or("<none>");
    let item_type = message
        .pointer("/params/item/type")
        .or_else(|| message.pointer("/params/type"))
        .and_then(Value::as_str)
        .unwrap_or("<none>");

    log::warn!(
        "Local Codex app-server event: method={method}; item_id={item_id}; item_type={item_type}"
    );
}

fn app_server_renderable_event(message: &Value) -> Option<LocalCodexRenderableEvent> {
    let method = message.get("method").and_then(Value::as_str)?;
    let params = message.get("params")?;
    let item_id = params
        .get("itemId")
        .and_then(Value::as_str)
        .unwrap_or(method)
        .to_string();

    match method {
        "item/started" => app_server_started_item_renderable_event(params),
        "item/agentMessage/delta" => Some(LocalCodexRenderableEvent::AgentTextDelta {
            item_id,
            delta: params.get("delta")?.as_str()?.to_string(),
        }),
        "item/reasoning/textDelta" | "item/reasoning/summaryTextDelta" => {
            Some(LocalCodexRenderableEvent::ReasoningDelta {
                item_id,
                delta: params.get("delta")?.as_str()?.to_string(),
            })
        }
        "item/plan/delta" => Some(LocalCodexRenderableEvent::ReasoningDelta {
            item_id,
            delta: params.get("delta")?.as_str()?.to_string(),
        }),
        "item/commandExecution/outputDelta" => Some(LocalCodexRenderableEvent::CommandTranscript {
            item_id,
            transcript: params.get("delta")?.as_str()?.to_string(),
        }),
        "item/fileChange/outputDelta" => Some(LocalCodexRenderableEvent::FileDiffTranscript {
            item_id,
            diff: params.get("delta")?.as_str()?.to_string(),
        }),
        "item/fileChange/patchUpdated" => {
            if let Some(changes) = codex_file_patch_changes(params) {
                Some(LocalCodexRenderableEvent::FileDiffDisplayCard { item_id, changes })
            } else {
                Some(LocalCodexRenderableEvent::FileDiffTranscript {
                    item_id,
                    diff: pretty_json(params.get("changes").unwrap_or(params)),
                })
            }
        }
        "item/mcpToolCall/progress" => Some(LocalCodexRenderableEvent::ToolTranscript {
            item_id,
            transcript: params.get("message")?.as_str()?.to_string(),
        }),
        "process/outputDelta" => Some(LocalCodexRenderableEvent::CommandTranscript {
            item_id,
            transcript: format!(
                "process {} {} output chunk (base64): {}",
                params
                    .get("processHandle")
                    .and_then(Value::as_str)
                    .unwrap_or("<unknown>"),
                params
                    .get("stream")
                    .and_then(Value::as_str)
                    .unwrap_or("<unknown>"),
                params
                    .get("deltaBase64")
                    .and_then(Value::as_str)
                    .unwrap_or("")
            ),
        }),
        _ => None,
    }
}

fn app_server_started_item_renderable_event(params: &Value) -> Option<LocalCodexRenderableEvent> {
    let item = params.get("item")?;
    if item.get("type").and_then(Value::as_str) != Some("commandExecution") {
        return None;
    }

    let command = item.get("command")?.as_str()?.trim();
    if command.is_empty() {
        return None;
    }

    Some(LocalCodexRenderableEvent::CommandDisplayCard {
        item_id: item.get("id")?.as_str()?.to_string(),
        command: command.to_string(),
    })
}

fn pretty_json(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}

#[cfg(not(target_family = "wasm"))]
fn app_server_is_server_request(message: &Value) -> bool {
    message.get("id").is_some()
        && message.get("method").is_some()
        && message.get("result").is_none()
}

#[cfg(not(target_family = "wasm"))]
fn respond_to_unsupported_app_server_request(
    stdin: &mut std::process::ChildStdin,
    message: &Value,
) {
    let Some(id) = message.get("id").cloned() else {
        return;
    };
    let response = json!({
        "id": id,
        "error": {
            "code": -32601,
            "message": "WarpCodexOss app-server runner does not execute server-originated tool or approval requests in V5",
        }
    });
    let _ = writeln!(stdin, "{}", response);
    let _ = stdin.flush();
}

#[cfg(not(target_family = "wasm"))]
fn interrupt_app_server_turn(
    stdin: &mut std::process::ChildStdin,
    next_id: &mut u64,
    thread_id: &str,
    turn_id: Option<&str>,
) {
    let Some(turn_id) = turn_id else {
        return;
    };
    let _ = app_server_send_request(
        stdin,
        next_id,
        "turn/interrupt",
        json!({
            "threadId": thread_id,
            "turnId": turn_id,
        }),
    );
}

fn extract_last_agent_message(stdout: &str) -> Option<String> {
    stdout
        .lines()
        .filter_map(|line| serde_json::from_str::<CodexJsonEvent>(line).ok())
        .filter(|event| event.kind == "item.completed")
        .filter_map(|event| event.item)
        .filter(|item| item.kind == "agent_message")
        .filter_map(|item| item.text)
        .next_back()
}

fn extract_json<T: DeserializeOwned>(text: &str) -> anyhow::Result<T> {
    if let Ok(value) = serde_json::from_str::<T>(text.trim()) {
        return Ok(value);
    }

    let stripped = text
        .trim()
        .strip_prefix("```json")
        .or_else(|| text.trim().strip_prefix("```"))
        .and_then(|text| text.strip_suffix("```"))
        .map(str::trim)
        .unwrap_or_else(|| text.trim());
    if let Ok(value) = serde_json::from_str::<T>(stripped) {
        return Ok(value);
    }

    let value = first_json_value(stripped).context("Codex response did not contain JSON")?;
    serde_json::from_value(value).context("Codex response JSON had an unexpected shape")
}

fn first_json_value(text: &str) -> Option<Value> {
    let bytes = text.as_bytes();
    for (start, byte) in bytes.iter().enumerate() {
        if !matches!(byte, b'{' | b'[') {
            continue;
        }

        let mut depth = 0_i32;
        let mut in_string = false;
        let mut escaped = false;
        for (offset, next) in bytes[start..].iter().enumerate() {
            if in_string {
                if escaped {
                    escaped = false;
                } else if *next == b'\\' {
                    escaped = true;
                } else if *next == b'"' {
                    in_string = false;
                }
                continue;
            }

            match *next {
                b'"' => in_string = true,
                b'{' | b'[' => depth += 1,
                b'}' | b']' => {
                    depth -= 1;
                    if depth == 0 {
                        let candidate = &text[start..=start + offset];
                        if let Ok(value) = serde_json::from_str(candidate) {
                            return Some(value);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    None
}

fn codex_file_patch_changes(params: &Value) -> Option<Vec<CodexFilePatchChange>> {
    let changes = params.get("changes")?.as_array()?;
    let changes = changes
        .iter()
        .filter_map(|change| {
            let path = change.get("path")?.as_str()?.trim();
            let diff = change.get("diff")?.as_str()?.trim();
            if path.is_empty() || diff.is_empty() {
                return None;
            }

            Some(CodexFilePatchChange {
                path: path.to_string(),
                diff: diff.to_string(),
                move_to: change
                    .pointer("/kind/move_path")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            })
        })
        .collect::<Vec<_>>();

    (!changes.is_empty()).then_some(changes)
}

pub async fn generate_commands(prompt: &str) -> anyhow::Result<Vec<GeneratedCommand>> {
    let codex_prompt = format!(
        "You are powering Warp command search. Return only JSON with this schema: \
         {{\"commands\":[{{\"command\":\"shell command\",\"description\":\"short title\",\
         \"parameters\":[{{\"id\":\"ARG\",\"description\":\"argument description\"}}]}}]}}. \
         Generate at most 5 useful terminal commands. User request: {prompt}"
    );
    let response = codex_exec(&codex_prompt).await?;
    let commands: GeneratedCommandList = extract_json(&response)?;
    Ok(commands.commands)
}

pub async fn generate_dialogue_answer(
    transcript: Vec<TranscriptPart>,
    prompt: String,
) -> anyhow::Result<GenerateDialogueResult> {
    let transcript = transcript
        .into_iter()
        .map(|part| {
            format!(
                "User: {}\nAssistant: {}",
                part.raw_user_prompt(),
                part.raw_assistant_answer()
            )
        })
        .join("\n\n");
    let codex_prompt = format!(
        "Answer the user's Warp AI question clearly and concisely. Previous transcript:\n\
         {transcript}\n\nCurrent user question:\n{prompt}"
    );
    let answer = codex_exec(&codex_prompt).await?;
    Ok(GenerateDialogueResult::Success {
        answer,
        truncated: false,
        request_limit_info: RequestLimitInfo::default(),
        transcript_summarized: false,
    })
}

pub async fn generate_metadata_for_command(
    command: String,
) -> Result<GeneratedCommandMetadata, GeneratedCommandMetadataError> {
    let codex_prompt = format!(
        "Create Warp workflow metadata for this shell command. Return only JSON with schema \
         {{\"parameterized_command\":\"command with named placeholders if useful\",\
         \"title\":\"short title\",\"description\":\"one sentence\",\
         \"arguments\":[{{\"name\":\"ARG\",\"description\":\"description\",\
         \"default_value\":\"\"}}]}}. Command: {command}"
    );
    let response = codex_exec(&codex_prompt)
        .await
        .map_err(|_| GeneratedCommandMetadataError::AiProviderError)?;
    let metadata: GeneratedMetadata =
        extract_json(&response).map_err(|_| GeneratedCommandMetadataError::BadCommand)?;
    Ok(GeneratedCommandMetadata {
        command: metadata.parameterized_command,
        title: metadata.title,
        description: metadata.description,
        arguments: metadata
            .arguments
            .into_iter()
            .map(|argument| GeneratedArgument {
                name: argument.name,
                description: argument.description,
                default_value: argument.default_value,
            })
            .collect(),
    })
}

pub async fn generate_code_review_content(
    output_type: OutputType,
    diff: String,
    branch_name: String,
    commit_messages: Vec<String>,
) -> anyhow::Result<String> {
    let request = match output_type {
        OutputType::CommitMessage => "Write a concise commit message.",
        OutputType::PrTitle => "Write a concise pull request title.",
        OutputType::PrDescription => "Write a useful pull request description in Markdown.",
    };
    let codex_prompt = format!(
        "{request}\n\nBranch: {branch_name}\n\nExisting commit messages:\n{}\n\nDiff:\n{diff}\n\nReturn only the requested text.",
        commit_messages.join("\n")
    );
    codex_exec(&codex_prompt).await
}

pub async fn generate_input_suggestions(
    request: &GenerateAIInputSuggestionsRequest,
) -> anyhow::Result<GenerateAIInputSuggestionsResponseV2> {
    let serialized_request = serde_json::to_string(request).unwrap_or_default();
    let codex_prompt = format!(
        "Predict the user's next terminal action from this Warp context. Return only JSON with \
         schema {{\"commands\":[\"command\"],\"ai_queries\":[{{\"query\":\"agent query\",\
         \"context_block_ids\":[]}}],\"most_likely_action\":\"command|ai_query|none\"}}. \
         Keep commands executable and do not include explanations.\n\nContext JSON:\n{serialized_request}"
    );
    let response = codex_exec(&codex_prompt).await?;
    extract_json(&response).or_else(|_| {
        Ok(GenerateAIInputSuggestionsResponseV2 {
            commands: vec![],
            ai_queries: vec![AgentModeSuggestionV2 {
                query: response.trim().to_string(),
                context_block_ids: vec![],
            }],
            most_likely_action: "ai_query".to_string(),
        })
    })
}

pub async fn generate_am_query_suggestions(
    request: &GenerateAMQuerySuggestionsRequest,
) -> anyhow::Result<GenerateAMQuerySuggestionsResponse> {
    let serialized_request = serde_json::to_string(request).unwrap_or_default();
    let codex_prompt = format!(
        "Suggest one useful Warp Agent Mode follow-up query from this terminal context. Return only \
         JSON with schema {{\"query\":\"short user-facing query\",\"should_plan_task\":false}}.\n\n\
         Context JSON:\n{serialized_request}"
    );
    let response = codex_exec(&codex_prompt).await?;
    let simple: SimpleQuery = extract_json(&response).unwrap_or_else(|_| SimpleQuery {
        query: response.trim().to_string(),
        should_plan_task: false,
    });
    Ok(GenerateAMQuerySuggestionsResponse {
        id: Uuid::new_v4().to_string(),
        suggestion: (!simple.query.trim().is_empty()).then_some(Suggestion::Simple(simple)),
    })
}

#[allow(deprecated)]
pub async fn generate_multi_agent_output(
    request: &api::Request,
) -> anyhow::Result<Vec<api::ResponseEvent>> {
    let (conversation_id, request_id, run_id) = local_stream_ids(request);
    let (task_id, mut actions) = task_id_and_bootstrap_actions(request);

    let user_prompt = request_prompt(request);
    let cwd = request_cwd(request);
    let is_passive = request.input.as_ref().is_some_and(|input| {
        matches!(
            input.r#type,
            Some(api::request::input::Type::GeneratePassiveSuggestions(_))
        )
    });

    let answer = if is_passive {
        let prompt = passive_suggestion_prompt(&user_prompt);
        codex_exec_with_cwd(&prompt, cwd.as_deref()).await?
    } else {
        let prompt = agent_turn_prompt(&user_prompt);
        codex_exec_with_cwd(&prompt, cwd.as_deref()).await?
    };

    actions.extend(if is_passive {
        passive_suggestion_actions(&task_id, &request_id, &answer)
    } else {
        agent_text_actions(&task_id, &request_id, &answer)
    });

    Ok(vec![
        api::ResponseEvent {
            r#type: Some(api::response_event::Type::Init(
                api::response_event::StreamInit {
                    conversation_id,
                    request_id: request_id.clone(),
                    run_id,
                },
            )),
        },
        api::ResponseEvent {
            r#type: Some(api::response_event::Type::ClientActions(
                api::response_event::ClientActions { actions },
            )),
        },
        api::ResponseEvent {
            r#type: Some(api::response_event::Type::Finished(
                api::response_event::StreamFinished {
                    token_usage: vec![],
                    should_refresh_model_config: false,
                    request_cost: Some(api::response_event::stream_finished::RequestCost {
                        exact: 0.0,
                        platform_credits: 0.0,
                    }),
                    conversation_usage_metadata: Some(
                        api::response_event::stream_finished::ConversationUsageMetadata {
                            context_window_usage: 0.0,
                            summarized: false,
                            credits_spent: 0.0,
                            token_usage: vec![],
                            tool_usage_metadata: None,
                            warp_token_usage: Default::default(),
                            byok_token_usage: Default::default(),
                        },
                    ),
                    reason: Some(api::response_event::stream_finished::Reason::Done(
                        api::response_event::stream_finished::Done {},
                    )),
                },
            )),
        },
    ])
}

#[cfg(not(target_family = "wasm"))]
pub fn generate_multi_agent_response_stream(
    request: api::Request,
    cancellation_rx: oneshot::Receiver<()>,
) -> crate::ai::agent::api::ResponseStream {
    let (tx, rx) = async_channel::unbounded();
    thread::spawn(move || {
        futures_lite::future::block_on(async move {
            run_multi_agent_response_stream(request, cancellation_rx, tx).await;
        });
    });
    Box::pin(rx)
}

#[cfg(target_family = "wasm")]
pub fn generate_multi_agent_response_stream(
    _request: api::Request,
    _cancellation_rx: oneshot::Receiver<()>,
) -> crate::ai::agent::api::ResponseStream {
    let stream = futures::stream::iter([Err(Arc::new(AIApiError::Other(anyhow!(
        "Local Codex is not available in WASM builds"
    ))))]);
    Box::pin(stream)
}

#[cfg(not(target_family = "wasm"))]
async fn run_multi_agent_response_stream(
    request: api::Request,
    cancellation_rx: oneshot::Receiver<()>,
    tx: async_channel::Sender<Result<api::ResponseEvent, Arc<AIApiError>>>,
) {
    let (conversation_id, request_id, run_id) = local_stream_ids(&request);
    let (task_id, bootstrap_actions) = task_id_and_bootstrap_actions(&request);
    let user_prompt = request_prompt(&request);
    let cwd = request_cwd(&request);
    let is_passive = request.input.as_ref().is_some_and(|input| {
        matches!(
            input.r#type,
            Some(api::request::input::Type::GeneratePassiveSuggestions(_))
        )
    });

    if !send_ok_event(
        &tx,
        stream_init_event(conversation_id, request_id.clone(), run_id),
    )
    .await
    {
        return;
    }
    if !bootstrap_actions.is_empty()
        && !send_ok_event(&tx, client_actions_event(bootstrap_actions)).await
    {
        return;
    }

    let prompt = if is_passive {
        passive_suggestion_prompt(&user_prompt)
    } else {
        agent_turn_prompt(&user_prompt)
    };

    if !is_passive && CodexRunnerKind::selected() == CodexRunnerKind::AppServer {
        match run_codex_app_server_response_stream(
            prompt,
            cwd,
            cancellation_rx,
            task_id.clone(),
            request_id.clone(),
            tx.clone(),
        )
        .await
        {
            Ok(AppServerStreamOutcome::Completed) => {
                log::warn!("Local Codex AI: app-server completed; sending StreamFinished Done");
                let _ = send_ok_event(&tx, finished_event(finished_reason_done())).await;
            }
            Ok(AppServerStreamOutcome::Cancelled) => {
                log::warn!("Local Codex AI: app-server cancelled; sending StreamFinished Other");
                let _ = send_ok_event(&tx, finished_event(finished_reason_other())).await;
            }
            Err(err) => {
                log::warn!(
                    "Local Codex AI: app-server failed; sending StreamFinished InternalError: {err:#}"
                );
                let message = format!(
                    "Local Codex app-server error: {err:#}\n\nSet `WARP_LOCAL_CODEX_RUNNER=exec` to use the stable exec runner, or run `codex` / `codex login --device-auth` and try again."
                );
                let actions = local_codex_events_to_actions(
                    &task_id,
                    &request_id,
                    &[LocalCodexEvent::Unsupported(message.clone())],
                );
                if !actions.is_empty() && !send_ok_event(&tx, client_actions_event(actions)).await {
                    return;
                }
                let _ = send_ok_event(&tx, finished_event(finished_reason_internal_error(message)))
                    .await;
            }
        }
        return;
    }

    match codex_exec_cancellable(prompt, cwd, cancellation_rx).await {
        Ok(CodexRunOutcome::Completed(answer)) => {
            let actions = if is_passive {
                passive_suggestion_actions(&task_id, &request_id, &answer)
            } else {
                local_codex_events_to_actions(
                    &task_id,
                    &request_id,
                    &[LocalCodexEvent::Text(answer)],
                )
            };
            if !actions.is_empty() && !send_ok_event(&tx, client_actions_event(actions)).await {
                return;
            }
            let _ = send_ok_event(&tx, finished_event(finished_reason_done())).await;
        }
        Ok(CodexRunOutcome::Cancelled) => {
            let _ = send_ok_event(&tx, finished_event(finished_reason_other())).await;
        }
        Err(err) => {
            let message = format!(
                "Local Codex error: {err:#}\n\nRun `codex` or `codex login --device-auth`, then try again."
            );
            let actions = local_codex_events_to_actions(
                &task_id,
                &request_id,
                &[LocalCodexEvent::Unsupported(message.clone())],
            );
            if !actions.is_empty() && !send_ok_event(&tx, client_actions_event(actions)).await {
                return;
            }
            let _ =
                send_ok_event(&tx, finished_event(finished_reason_internal_error(message))).await;
        }
    }
}

async fn send_ok_event(
    tx: &async_channel::Sender<Result<api::ResponseEvent, Arc<AIApiError>>>,
    event: api::ResponseEvent,
) -> bool {
    tx.send(Ok(event)).await.is_ok()
}

fn stream_init_event(
    conversation_id: String,
    request_id: String,
    run_id: String,
) -> api::ResponseEvent {
    api::ResponseEvent {
        r#type: Some(api::response_event::Type::Init(
            api::response_event::StreamInit {
                conversation_id,
                request_id,
                run_id,
            },
        )),
    }
}

fn client_actions_event(actions: Vec<api::ClientAction>) -> api::ResponseEvent {
    api::ResponseEvent {
        r#type: Some(api::response_event::Type::ClientActions(
            api::response_event::ClientActions { actions },
        )),
    }
}

#[allow(deprecated)]
fn finished_event(reason: api::response_event::stream_finished::Reason) -> api::ResponseEvent {
    api::ResponseEvent {
        r#type: Some(api::response_event::Type::Finished(
            api::response_event::StreamFinished {
                token_usage: vec![],
                should_refresh_model_config: false,
                request_cost: Some(api::response_event::stream_finished::RequestCost {
                    exact: 0.0,
                    platform_credits: 0.0,
                }),
                conversation_usage_metadata: Some(
                    api::response_event::stream_finished::ConversationUsageMetadata {
                        context_window_usage: 0.0,
                        summarized: false,
                        credits_spent: 0.0,
                        token_usage: vec![],
                        tool_usage_metadata: None,
                        warp_token_usage: Default::default(),
                        byok_token_usage: Default::default(),
                    },
                ),
                reason: Some(reason),
            },
        )),
    }
}

fn finished_reason_done() -> api::response_event::stream_finished::Reason {
    api::response_event::stream_finished::Reason::Done(
        api::response_event::stream_finished::Done {},
    )
}

fn finished_reason_other() -> api::response_event::stream_finished::Reason {
    api::response_event::stream_finished::Reason::Other(
        api::response_event::stream_finished::Other {},
    )
}

fn finished_reason_internal_error(message: String) -> api::response_event::stream_finished::Reason {
    api::response_event::stream_finished::Reason::InternalError(
        api::response_event::stream_finished::InternalError { message },
    )
}

fn task_id_and_bootstrap_actions(request: &api::Request) -> (String, Vec<api::ClientAction>) {
    if let Some(task_id) = request
        .task_context
        .as_ref()
        .and_then(|task_context| task_context.tasks.first())
        .map(|task| task.id.clone())
        .filter(|id| !id.is_empty())
    {
        return (task_id, Vec::new());
    }

    let task_id = format!("local-codex-task-{}", Uuid::new_v4());
    (
        task_id.clone(),
        vec![api::ClientAction {
            action: Some(api::client_action::Action::CreateTask(
                api::client_action::CreateTask {
                    task: Some(api::Task {
                        id: task_id,
                        description: "Local Codex".to_string(),
                        dependencies: None,
                        messages: vec![],
                        summary: String::new(),
                        server_data: String::new(),
                    }),
                },
            )),
        }],
    )
}

fn agent_text_actions(task_id: &str, request_id: &str, answer: &str) -> Vec<api::ClientAction> {
    local_codex_events_to_actions(
        task_id,
        request_id,
        &[LocalCodexEvent::Text(answer.to_string())],
    )
}

fn local_codex_events_to_actions(
    task_id: &str,
    request_id: &str,
    events: &[LocalCodexEvent],
) -> Vec<api::ClientAction> {
    let mut translator = LocalCodexStreamingTranslator::default();
    let mut actions = Vec::new();
    for (index, event) in events.iter().enumerate() {
        let item_id = format!("exec-{index}");
        let Some(event) = local_codex_renderable_event(event, item_id) else {
            continue;
        };
        actions.extend(translator.translate_event(task_id, request_id, event));
    }
    actions
}

fn local_codex_renderable_event(
    event: &LocalCodexEvent,
    item_id: String,
) -> Option<LocalCodexRenderableEvent> {
    match event {
        LocalCodexEvent::Text(text) if !text.is_empty() => {
            Some(LocalCodexRenderableEvent::AgentTextDelta {
                item_id,
                delta: text.clone(),
            })
        }
        LocalCodexEvent::Reasoning(text) if !text.is_empty() => {
            Some(LocalCodexRenderableEvent::ReasoningDelta {
                item_id,
                delta: text.clone(),
            })
        }
        LocalCodexEvent::ToolCall(text) if !text.is_empty() => {
            Some(LocalCodexRenderableEvent::ToolTranscript {
                item_id,
                transcript: text.clone(),
            })
        }
        LocalCodexEvent::FileDiff(text) if !text.is_empty() => {
            Some(LocalCodexRenderableEvent::FileDiffTranscript {
                item_id,
                diff: text.clone(),
            })
        }
        LocalCodexEvent::CommandResult(text) if !text.is_empty() => {
            Some(LocalCodexRenderableEvent::CommandTranscript {
                item_id,
                transcript: text.clone(),
            })
        }
        LocalCodexEvent::Unsupported(text) if !text.is_empty() => {
            Some(LocalCodexRenderableEvent::Unsupported {
                item_id,
                transcript: text.clone(),
            })
        }
        _ => None,
    }
}

impl LocalCodexStreamingTranslator {
    fn translate_event(
        &mut self,
        task_id: &str,
        request_id: &str,
        event: LocalCodexRenderableEvent,
    ) -> Vec<api::ClientAction> {
        if let LocalCodexRenderableEvent::CommandDisplayCard { item_id, command } = event {
            let message_id = stable_local_codex_message_id("command-card", &item_id);
            if !self.created_messages.insert(message_id.clone()) {
                return vec![];
            }
            return vec![add_message_action(
                task_id,
                display_only_command_message(task_id, request_id, &message_id, &item_id, command),
            )];
        }

        if let LocalCodexRenderableEvent::FileDiffDisplayCard { item_id, changes } = event {
            let message_id = stable_local_codex_message_id("diff-card", &item_id);
            if !self.created_messages.insert(message_id.clone()) {
                return vec![];
            }
            return vec![add_message_action(
                task_id,
                display_only_file_diff_message(task_id, request_id, &message_id, &item_id, changes),
            )];
        }

        let (kind, item_id, message, path) = match event {
            LocalCodexRenderableEvent::AgentTextDelta { item_id, delta } => (
                "text",
                item_id,
                StreamMessage::AgentOutput(delta),
                "agent_output.text",
            ),
            LocalCodexRenderableEvent::ReasoningDelta { item_id, delta } => (
                "reasoning",
                item_id,
                StreamMessage::AgentReasoning(delta),
                "agent_reasoning.reasoning",
            ),
            LocalCodexRenderableEvent::CommandDisplayCard { .. } => unreachable!(),
            LocalCodexRenderableEvent::FileDiffDisplayCard { .. } => unreachable!(),
            LocalCodexRenderableEvent::CommandTranscript {
                item_id,
                transcript,
            } => (
                "command",
                item_id,
                StreamMessage::AgentOutput(format!("Codex command transcript:\n{transcript}")),
                "agent_output.text",
            ),
            LocalCodexRenderableEvent::FileDiffTranscript { item_id, diff } => (
                "diff",
                item_id,
                StreamMessage::AgentOutput(format!(
                    "Codex file diff transcript:\n```diff\n{diff}\n```"
                )),
                "agent_output.text",
            ),
            LocalCodexRenderableEvent::ToolTranscript {
                item_id,
                transcript,
            } => (
                "tool",
                item_id,
                StreamMessage::AgentOutput(format!("Codex tool transcript:\n{transcript}")),
                "agent_output.text",
            ),
            LocalCodexRenderableEvent::Unsupported {
                item_id,
                transcript,
            } => (
                "unsupported",
                item_id,
                StreamMessage::AgentOutput(format!("Codex unsupported event:\n{transcript}")),
                "agent_output.text",
            ),
        };

        if message.is_empty() {
            return vec![];
        }

        let message_id = stable_local_codex_message_id(kind, &item_id);
        let mut actions = Vec::new();
        if self.created_messages.insert(message_id.clone()) {
            actions.push(add_message_action(
                task_id,
                empty_stream_message(task_id, request_id, &message_id, &message),
            ));
        }
        actions.push(append_message_action(
            task_id,
            stream_message(task_id, request_id, &message_id, message),
            path,
        ));
        actions
    }
}

enum StreamMessage {
    AgentOutput(String),
    AgentReasoning(String),
}

impl StreamMessage {
    fn is_empty(&self) -> bool {
        match self {
            StreamMessage::AgentOutput(text) | StreamMessage::AgentReasoning(text) => {
                text.is_empty()
            }
        }
    }

    fn empty_like(&self) -> Self {
        match self {
            StreamMessage::AgentOutput(_) => Self::AgentOutput(String::new()),
            StreamMessage::AgentReasoning(_) => Self::AgentReasoning(String::new()),
        }
    }
}

fn stable_local_codex_message_id(kind: &str, item_id: &str) -> String {
    let sanitized = item_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>();
    let sanitized = sanitized.trim_matches('-');
    let item_id = if sanitized.is_empty() {
        "item"
    } else {
        sanitized
    };
    format!("local-codex-{kind}-{item_id}")
}

fn stable_local_codex_tool_call_id(kind: &str, item_id: &str) -> String {
    format!(
        "{}{}",
        DISPLAY_ONLY_TOOL_CALL_PREFIX,
        stable_local_codex_message_id(kind, item_id)
    )
}

fn add_message_action(task_id: &str, message: api::Message) -> api::ClientAction {
    api::ClientAction {
        action: Some(api::client_action::Action::AddMessagesToTask(
            api::client_action::AddMessagesToTask {
                task_id: task_id.to_string(),
                messages: vec![message],
            },
        )),
    }
}

fn display_only_command_message(
    task_id: &str,
    request_id: &str,
    message_id: &str,
    item_id: &str,
    command: String,
) -> api::Message {
    api::Message {
        id: message_id.to_string(),
        task_id: task_id.to_string(),
        request_id: request_id.to_string(),
        timestamp: None,
        server_message_data: String::new(),
        citations: vec![],
        message: Some(api::message::Message::ToolCall(api::message::ToolCall {
            tool_call_id: stable_local_codex_tool_call_id("command", item_id),
            tool: Some(api::message::tool_call::Tool::RunShellCommand(
                api::message::tool_call::RunShellCommand {
                    command,
                    is_read_only: false,
                    uses_pager: false,
                    citations: vec![],
                    is_risky: false,
                    risk_category: 0,
                    wait_until_complete_value: Some(
                        api::message::tool_call::run_shell_command::WaitUntilCompleteValue::WaitUntilComplete(
                            true,
                        ),
                    ),
                },
            )),
        })),
    }
}

fn display_only_file_diff_message(
    task_id: &str,
    request_id: &str,
    message_id: &str,
    item_id: &str,
    changes: Vec<CodexFilePatchChange>,
) -> api::Message {
    let summary = display_only_file_diff_summary(&changes);
    let v4a_updates = changes
        .into_iter()
        .map(
            |change| api::message::tool_call::apply_file_diffs::V4aFileUpdate {
                file_path: change.path.clone(),
                move_to: change.move_to,
                hunks: vec![
                    api::message::tool_call::apply_file_diffs::v4a_file_update::Hunk {
                        change_context: codex_diff_change_context(&change.diff, &change.path),
                        pre_context: String::new(),
                        old: String::new(),
                        new: change.diff,
                        post_context: String::new(),
                    },
                ],
            },
        )
        .collect();

    api::Message {
        id: message_id.to_string(),
        task_id: task_id.to_string(),
        request_id: request_id.to_string(),
        timestamp: None,
        server_message_data: String::new(),
        citations: vec![],
        message: Some(api::message::Message::ToolCall(api::message::ToolCall {
            tool_call_id: stable_local_codex_tool_call_id("diff", item_id),
            tool: Some(api::message::tool_call::Tool::ApplyFileDiffs(
                api::message::tool_call::ApplyFileDiffs {
                    summary,
                    diffs: vec![],
                    new_files: vec![],
                    deleted_files: vec![],
                    v4a_updates,
                },
            )),
        })),
    }
}

fn display_only_file_diff_summary(changes: &[CodexFilePatchChange]) -> String {
    match changes {
        [] => "Codex file changes".to_string(),
        [change] => format!("Codex changed {}", change.path),
        _ => format!("Codex changed {} files", changes.len()),
    }
}

fn codex_diff_change_context(diff: &str, path: &str) -> Vec<String> {
    let contexts = diff
        .lines()
        .filter(|line| line.starts_with("@@"))
        .take(4)
        .map(str::to_string)
        .collect::<Vec<_>>();

    if contexts.is_empty() {
        vec![format!("Codex file change: {path}")]
    } else {
        contexts
    }
}

fn append_message_action(task_id: &str, message: api::Message, path: &str) -> api::ClientAction {
    api::ClientAction {
        action: Some(api::client_action::Action::AppendToMessageContent(
            api::client_action::AppendToMessageContent {
                task_id: task_id.to_string(),
                message: Some(message),
                mask: Some(prost_types::FieldMask {
                    paths: vec![path.to_string()],
                }),
            },
        )),
    }
}

fn empty_stream_message(
    task_id: &str,
    request_id: &str,
    message_id: &str,
    message: &StreamMessage,
) -> api::Message {
    stream_message(task_id, request_id, message_id, message.empty_like())
}

fn stream_message(
    task_id: &str,
    request_id: &str,
    message_id: &str,
    message: StreamMessage,
) -> api::Message {
    api::Message {
        id: message_id.to_string(),
        task_id: task_id.to_string(),
        request_id: request_id.to_string(),
        timestamp: None,
        server_message_data: String::new(),
        citations: vec![],
        message: Some(match message {
            StreamMessage::AgentOutput(text) => {
                api::message::Message::AgentOutput(api::message::AgentOutput { text })
            }
            StreamMessage::AgentReasoning(reasoning) => {
                api::message::Message::AgentReasoning(api::message::AgentReasoning {
                    reasoning,
                    finished_duration: None,
                })
            }
        }),
    }
}

fn passive_suggestion_actions(
    task_id: &str,
    request_id: &str,
    answer: &str,
) -> Vec<api::ClientAction> {
    let suggestion: PassivePromptSuggestion = extract_json(answer).unwrap_or_default();
    if suggestion.prompt.trim().is_empty() {
        return vec![];
    }

    let tool_call = api::message::ToolCall {
        tool_call_id: format!("local-codex-suggest-{}", Uuid::new_v4()),
        tool: Some(api::message::tool_call::Tool::SuggestPrompt(
            api::message::tool_call::SuggestPrompt {
                is_trigger_irrelevant: suggestion.is_trigger_irrelevant,
                display_mode: Some(
                    api::message::tool_call::suggest_prompt::DisplayMode::PromptChip(
                        api::message::tool_call::suggest_prompt::PromptChip {
                            prompt: suggestion.prompt,
                            label: suggestion.label,
                        },
                    ),
                ),
            },
        )),
    };

    vec![api::ClientAction {
        action: Some(api::client_action::Action::AddMessagesToTask(
            api::client_action::AddMessagesToTask {
                task_id: task_id.to_string(),
                messages: vec![api::Message {
                    id: format!("local-codex-message-{}", Uuid::new_v4()),
                    task_id: task_id.to_string(),
                    request_id: request_id.to_string(),
                    timestamp: None,
                    server_message_data: String::new(),
                    citations: vec![],
                    message: Some(api::message::Message::ToolCall(tool_call)),
                }],
            },
        )),
    }]
}

fn request_prompt(request: &api::Request) -> String {
    let mut parts = Vec::new();
    if let Some(input) = &request.input {
        if let Some(context) = &input.context {
            if let Some(directory) = &context.directory {
                if !directory.pwd.is_empty() {
                    parts.push(format!("Working directory: {}", directory.pwd));
                }
            }
        }
        if let Some(input_type) = &input.r#type {
            collect_input_prompt(input_type, &mut parts);
        }
    }
    if parts.is_empty() {
        "Continue the Warp Agent conversation.".to_string()
    } else {
        parts.join("\n\n")
    }
}

fn request_cwd(request: &api::Request) -> Option<String> {
    request
        .input
        .as_ref()
        .and_then(|input| input.context.as_ref())
        .and_then(|context| context.directory.as_ref())
        .map(|directory| directory.pwd.clone())
        .filter(|pwd| !pwd.trim().is_empty())
}

#[allow(deprecated)]
fn collect_input_prompt(input_type: &api::request::input::Type, parts: &mut Vec<String>) {
    use api::request::input;
    match input_type {
        input::Type::UserInputs(inputs) => {
            for item in &inputs.inputs {
                if let Some(item) = &item.input {
                    match item {
                        input::user_inputs::user_input::Input::UserQuery(query) => {
                            parts.push(format!("User query: {}", query.query));
                        }
                        input::user_inputs::user_input::Input::CliAgentUserQuery(query) => {
                            if let Some(user_query) = &query.user_query {
                                parts.push(format!("CLI agent query: {}", user_query.query));
                            }
                            if let Some(command) = &query.running_command {
                                parts.push(format!("Running command: {}", command.command));
                            }
                        }
                        input::user_inputs::user_input::Input::ToolCallResult(_) => {
                            parts.push("A tool call result was returned. Continue.".to_string());
                        }
                        input::user_inputs::user_input::Input::MessagesReceivedFromAgents(
                            messages,
                        ) => {
                            for message in &messages.messages {
                                parts.push(format!(
                                    "Message from agent {}: {}",
                                    message.sender_agent_id, message.message_body
                                ));
                            }
                        }
                        input::user_inputs::user_input::Input::EventsFromAgents(_) => {
                            parts.push(
                                "Agent lifecycle events were returned. Continue.".to_string(),
                            );
                        }
                        input::user_inputs::user_input::Input::PassiveSuggestionResult(_) => {
                            parts.push("A passive suggestion was accepted. Continue.".to_string());
                        }
                        input::user_inputs::user_input::Input::OrchestrationConfigUpdate(_) => {
                            parts.push("The orchestration plan was updated. Continue.".to_string());
                        }
                    }
                }
            }
        }
        input::Type::QueryWithCannedResponse(_) => {
            parts.push("Start from the selected Warp canned response.".to_string());
        }
        input::Type::AutoCodeDiffQuery(query) => parts.push(query.query.clone()),
        input::Type::ResumeConversation(_) => parts.push("Resume the conversation.".to_string()),
        input::Type::InitProjectRules(_) => {
            parts.push("Generate project rules for this workspace.".to_string());
        }
        input::Type::GeneratePassiveSuggestions(suggestion) => {
            parts.push(passive_suggestion_context(suggestion));
        }
        input::Type::CreateNewProject(project) => {
            parts.push(format!("Create a new project: {}", project.query));
        }
        input::Type::CloneRepository(repo) => {
            parts.push(format!("Clone repository: {}", repo.url));
        }
        input::Type::CodeReview(_) => {
            parts.push("Review this code change.".to_string());
        }
        input::Type::SummarizeConversation(summary) => {
            parts.push(format!("Summarize the conversation. {}", summary.prompt));
        }
        input::Type::CreateEnvironment(env) => {
            parts.push(format!(
                "Create environment for repos: {}",
                env.repo_paths.join(", ")
            ));
        }
        input::Type::FetchReviewComments(review) => {
            parts.push(format!(
                "Fetch review comments for repo: {}",
                review.repo_path
            ));
        }
        input::Type::StartFromAmbientRunPrompt(run) => {
            parts.push(format!(
                "{}\nAmbient run id: {}",
                run.runtime_base_prompt, run.ambient_run_id
            ));
        }
        input::Type::InvokeSkill(skill) => {
            if let Some(user_query) = &skill.user_query {
                parts.push(format!("Invoke skill query: {}", user_query.query));
            } else {
                parts.push("Invoke the selected skill.".to_string());
            }
        }
        input::Type::UserQuery(query) => parts.push(query.query.clone()),
        input::Type::ToolCallResult(_) => {
            parts.push("A tool call result was returned. Continue.".to_string());
        }
    }
}

fn passive_suggestion_context(
    suggestion: &api::request::input::GeneratePassiveSuggestions,
) -> String {
    use api::request::input::generate_passive_suggestions::Trigger;

    match &suggestion.trigger {
        Some(Trigger::ShellCommandCompleted(command)) => {
            let Some(command) = &command.executed_shell_command else {
                return "A shell command completed.".to_string();
            };
            format!(
                "A shell command completed.\nCommand: {}\nExit code: {}\nOutput:\n{}",
                command.command, command.exit_code, command.output
            )
        }
        Some(Trigger::AgentResponseCompleted(_)) => {
            "An agent response completed. Suggest a useful follow-up.".to_string()
        }
        Some(Trigger::FilesChanged(_)) => "Files changed. Suggest a useful follow-up.".to_string(),
        Some(Trigger::CommandRun(_)) => "A command ran. Suggest a useful follow-up.".to_string(),
        None => "Suggest a useful Warp follow-up prompt.".to_string(),
    }
}

pub fn into_graphql_generated_command(
    command: GeneratedCommand,
) -> warp_graphql::mutations::generate_commands::GeneratedCommand {
    warp_graphql::mutations::generate_commands::GeneratedCommand {
        command: command.command,
        description: command.description,
        parameters: command
            .parameters
            .into_iter()
            .map(|parameter| {
                warp_graphql::mutations::generate_commands::GeneratedCommandParameter {
                    id: parameter.id,
                    description: parameter.description,
                }
            })
            .collect(),
    }
}

pub fn stream_from_events(
    events: Vec<api::ResponseEvent>,
) -> crate::server::server_api::AIOutputStream<api::ResponseEvent> {
    let stream = futures::stream::iter(events.into_iter().map(Ok::<_, Arc<AIApiError>>));
    cfg_if::cfg_if! {
        if #[cfg(target_family = "wasm")] {
            stream.boxed_local()
        } else {
            stream.boxed()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    struct EnvGuard {
        key: &'static str,
        old: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: impl AsRef<std::ffi::OsStr>) -> Self {
            let old = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key, old }
        }

        fn remove(key: &'static str) -> Self {
            let old = std::env::var(key).ok();
            std::env::remove_var(key);
            Self { key, old }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.old {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }

    fn write_fake_codex(dir: &Path, body: &str) -> std::path::PathBuf {
        let path = dir.join("codex");
        fs::write(&path, body).expect("fake codex script should be written");
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
        path
    }

    fn empty_request() -> api::Request {
        api::Request {
            task_context: None,
            input: None,
            settings: None,
            metadata: None,
            existing_suggestions: None,
            mcp_context: None,
        }
    }

    fn request_with_ambient_task_id() -> api::Request {
        api::Request {
            metadata: Some(api::request::Metadata {
                conversation_id: "existing-conversation".to_string(),
                ambient_agent_task_id: "c239995a-e35d-441b-a828-a2a8dcceeacb".to_string(),
                ..Default::default()
            }),
            ..empty_request()
        }
    }

    #[test]
    fn agent_turn_prompt_does_not_inject_bridge_answer_template() {
        let prompt = agent_turn_prompt("User query: say a random sentence");

        assert_eq!(prompt, "User query: say a random sentence");
        assert!(!prompt.contains("native bridge"));
        assert!(!prompt.contains("safe textual planning"));
    }

    fn agent_output_texts(actions: &[api::ClientAction]) -> Vec<String> {
        coalesce_messages(actions)
            .into_iter()
            .filter_map(|message| match message.message {
                Some(api::message::Message::AgentOutput(output)) => Some(output.text.clone()),
                _ => None,
            })
            .collect()
    }

    fn coalesce_messages(actions: &[api::ClientAction]) -> Vec<api::Message> {
        use field_mask::FieldMaskOperation;
        use std::collections::HashMap;

        let mut messages_by_id: HashMap<String, api::Message> = HashMap::new();
        let mut order = Vec::new();

        for action in actions {
            match &action.action {
                Some(api::client_action::Action::AddMessagesToTask(add)) => {
                    for message in &add.messages {
                        if !messages_by_id.contains_key(&message.id) {
                            order.push(message.id.clone());
                        }
                        messages_by_id.insert(message.id.clone(), message.clone());
                    }
                }
                Some(api::client_action::Action::AppendToMessageContent(append)) => {
                    let Some(message) = &append.message else {
                        continue;
                    };
                    let mask = append.mask.clone().unwrap_or_default();
                    if let Some(existing) = messages_by_id.get_mut(&message.id) {
                        let merged = FieldMaskOperation::append(
                            &api::MESSAGE_DESCRIPTOR,
                            existing,
                            message,
                            mask,
                        )
                        .apply()
                        .expect("append mask should merge");
                        *existing = merged;
                    } else {
                        order.push(message.id.clone());
                        messages_by_id.insert(message.id.clone(), message.clone());
                    }
                }
                _ => {}
            }
        }

        order
            .into_iter()
            .filter_map(|id| messages_by_id.remove(&id))
            .collect()
    }

    fn coalesced_agent_output_texts(actions: &[api::ClientAction]) -> Vec<String> {
        coalesce_messages(actions)
            .into_iter()
            .filter_map(|message| match message.message {
                Some(api::message::Message::AgentOutput(output)) => Some(output.text),
                _ => None,
            })
            .collect()
    }

    fn coalesced_reasoning_texts(actions: &[api::ClientAction]) -> Vec<String> {
        coalesce_messages(actions)
            .into_iter()
            .filter_map(|message| match message.message {
                Some(api::message::Message::AgentReasoning(reasoning)) => Some(reasoning.reasoning),
                _ => None,
            })
            .collect()
    }

    fn actions_contain_tool_call(actions: &[api::ClientAction]) -> bool {
        coalesce_messages(actions)
            .into_iter()
            .any(|message| matches!(message.message, Some(api::message::Message::ToolCall(_))))
    }

    fn tool_call_ids(actions: &[api::ClientAction]) -> Vec<String> {
        coalesce_messages(actions)
            .into_iter()
            .filter_map(|message| match message.message {
                Some(api::message::Message::ToolCall(tool_call)) => Some(tool_call.tool_call_id),
                _ => None,
            })
            .collect()
    }

    fn actions_contain_executable_tool_call(actions: &[api::ClientAction]) -> bool {
        tool_call_ids(actions)
            .into_iter()
            .any(|id| !id.starts_with(DISPLAY_ONLY_TOOL_CALL_PREFIX))
    }

    fn apply_file_diffs_tool_calls(
        actions: &[api::ClientAction],
    ) -> Vec<api::message::tool_call::ApplyFileDiffs> {
        coalesce_messages(actions)
            .into_iter()
            .filter_map(|message| match message.message {
                Some(api::message::Message::ToolCall(tool_call)) => match tool_call.tool {
                    Some(api::message::tool_call::Tool::ApplyFileDiffs(diffs)) => Some(diffs),
                    _ => None,
                },
                _ => None,
            })
            .collect()
    }

    #[test]
    fn extracts_json_inside_fenced_response() {
        let parsed: SimpleQuery =
            extract_json("```json\n{\"query\":\"run tests\",\"should_plan_task\":false}\n```")
                .unwrap();
        assert_eq!(parsed.query, "run tests");
        assert!(!parsed.should_plan_task);
    }

    #[test]
    fn extracts_last_codex_agent_message() {
        let output = r#"{"type":"thread.started","thread_id":"1"}
{"type":"item.completed","item":{"id":"item_0","type":"agent_message","text":"first"}}
{"type":"item.completed","item":{"id":"item_1","type":"agent_message","text":"second"}}"#;
        assert_eq!(
            extract_last_agent_message(output).as_deref(),
            Some("second")
        );
    }

    #[test]
    fn bootstraps_task_when_request_has_no_task_context() {
        let request = api::Request {
            task_context: None,
            input: None,
            settings: None,
            metadata: None,
            existing_suggestions: None,
            mcp_context: None,
        };

        let (task_id, actions) = task_id_and_bootstrap_actions(&request);

        assert!(task_id.starts_with("local-codex-task-"));
        assert_eq!(actions.len(), 1);
        match &actions[0].action {
            Some(api::client_action::Action::CreateTask(create_task)) => {
                assert_eq!(create_task.task.as_ref().unwrap().id, task_id);
            }
            action => panic!("expected CreateTask action, got {action:?}"),
        }
    }

    #[test]
    fn reuses_existing_task_context_without_bootstrap() {
        let request = api::Request {
            task_context: Some(api::request::TaskContext {
                tasks: vec![api::Task {
                    id: "root-task".to_string(),
                    description: String::new(),
                    dependencies: None,
                    messages: vec![],
                    summary: String::new(),
                    server_data: String::new(),
                }],
            }),
            input: None,
            settings: None,
            metadata: None,
            existing_suggestions: None,
            mcp_context: None,
        };

        let (task_id, actions) = task_id_and_bootstrap_actions(&request);

        assert_eq!(task_id, "root-task");
        assert!(actions.is_empty());
    }

    #[test]
    fn translates_local_codex_events_to_agent_text_fallbacks() {
        let actions = local_codex_events_to_actions(
            "task",
            "request",
            &[
                LocalCodexEvent::Text("hello".to_string()),
                LocalCodexEvent::Reasoning("thinking".to_string()),
                LocalCodexEvent::ToolCall("read file".to_string()),
                LocalCodexEvent::FileDiff("--- a\n+++ b".to_string()),
                LocalCodexEvent::CommandResult("exit 0".to_string()),
                LocalCodexEvent::Unsupported("raw event".to_string()),
            ],
        );

        let texts = agent_output_texts(&actions);
        let reasoning = coalesced_reasoning_texts(&actions);

        assert_eq!(texts.len(), 5);
        assert_eq!(reasoning, vec!["thinking"]);
        assert_eq!(texts[0], "hello");
        assert!(texts[1].contains("Codex tool transcript"));
        assert!(texts[2].contains("Codex file diff transcript"));
        assert!(texts[3].contains("Codex command transcript"));
        assert!(texts[4].contains("Codex unsupported event"));
    }

    #[test]
    fn streaming_translator_appends_agent_output_deltas() {
        let mut translator = LocalCodexStreamingTranslator::default();
        let mut actions = translator.translate_event(
            "task",
            "request",
            LocalCodexRenderableEvent::AgentTextDelta {
                item_id: "item-1".to_string(),
                delta: "hello ".to_string(),
            },
        );
        actions.extend(translator.translate_event(
            "task",
            "request",
            LocalCodexRenderableEvent::AgentTextDelta {
                item_id: "item-1".to_string(),
                delta: "world".to_string(),
            },
        ));

        let texts = coalesced_agent_output_texts(&actions);

        assert_eq!(texts, vec!["hello world"]);
        assert_eq!(
            actions
                .iter()
                .filter(|action| matches!(
                    &action.action,
                    Some(api::client_action::Action::AddMessagesToTask(_))
                ))
                .count(),
            1
        );
    }

    #[test]
    fn streaming_translator_appends_reasoning_deltas_as_reasoning() {
        let mut translator = LocalCodexStreamingTranslator::default();
        let mut actions = translator.translate_event(
            "task",
            "request",
            LocalCodexRenderableEvent::ReasoningDelta {
                item_id: "reasoning-1".to_string(),
                delta: "step ".to_string(),
            },
        );
        actions.extend(translator.translate_event(
            "task",
            "request",
            LocalCodexRenderableEvent::ReasoningDelta {
                item_id: "reasoning-1".to_string(),
                delta: "two".to_string(),
            },
        ));

        let texts = coalesced_reasoning_texts(&actions);

        assert_eq!(texts, vec!["step two"]);
        assert!(coalesced_agent_output_texts(&actions).is_empty());
    }

    #[test]
    fn streaming_translator_keeps_executed_tool_events_as_text() {
        let mut translator = LocalCodexStreamingTranslator::default();
        let mut actions = Vec::new();
        for event in [
            LocalCodexRenderableEvent::CommandTranscript {
                item_id: "cmd-1".to_string(),
                transcript: "$ echo ok\nok".to_string(),
            },
            LocalCodexRenderableEvent::FileDiffTranscript {
                item_id: "diff-1".to_string(),
                diff: "--- a/file\n+++ b/file".to_string(),
            },
            LocalCodexRenderableEvent::ToolTranscript {
                item_id: "tool-1".to_string(),
                transcript: "read_file completed".to_string(),
            },
            LocalCodexRenderableEvent::Unsupported {
                item_id: "raw-1".to_string(),
                transcript: "{\"method\":\"unknown\"}".to_string(),
            },
        ] {
            actions.extend(translator.translate_event("task", "request", event));
        }

        let texts = coalesced_agent_output_texts(&actions);

        assert!(!actions_contain_tool_call(&actions));
        assert!(texts.iter().any(|text| text.contains("command transcript")));
        assert!(texts
            .iter()
            .any(|text| text.contains("file diff transcript")));
        assert!(texts.iter().any(|text| text.contains("tool transcript")));
        assert!(texts.iter().any(|text| text.contains("unsupported event")));
    }

    #[test]
    fn streaming_translator_creates_display_only_command_card() {
        let mut translator = LocalCodexStreamingTranslator::default();
        let actions = translator.translate_event(
            "task",
            "request",
            LocalCodexRenderableEvent::CommandDisplayCard {
                item_id: "cmd-1".to_string(),
                command: "echo already-ran".to_string(),
            },
        );

        let ids = tool_call_ids(&actions);

        assert_eq!(ids.len(), 1);
        assert!(ids[0].starts_with(DISPLAY_ONLY_TOOL_CALL_PREFIX));
        assert!(!actions_contain_executable_tool_call(&actions));
    }

    #[test]
    fn streaming_translator_creates_display_only_diff_card() {
        let mut translator = LocalCodexStreamingTranslator::default();
        let actions = translator.translate_event(
            "task",
            "request",
            LocalCodexRenderableEvent::FileDiffDisplayCard {
                item_id: "diff-1".to_string(),
                changes: vec![CodexFilePatchChange {
                    path: "src/main.rs".to_string(),
                    diff: "@@ -1 +1 @@\n-old\n+new".to_string(),
                    move_to: String::new(),
                }],
            },
        );

        let ids = tool_call_ids(&actions);
        let diff_calls = apply_file_diffs_tool_calls(&actions);

        assert_eq!(ids.len(), 1);
        assert!(ids[0].starts_with(DISPLAY_ONLY_TOOL_CALL_PREFIX));
        assert!(!actions_contain_executable_tool_call(&actions));
        assert_eq!(diff_calls.len(), 1);
        assert_eq!(diff_calls[0].summary, "Codex changed src/main.rs");
        assert_eq!(diff_calls[0].v4a_updates[0].file_path, "src/main.rs");
        assert_eq!(
            diff_calls[0].v4a_updates[0].hunks[0].change_context,
            vec!["@@ -1 +1 @@".to_string()]
        );
    }

    #[test]
    fn app_server_patch_updated_uses_card_when_structured() {
        let event = app_server_renderable_event(&json!({
            "method": "item/fileChange/patchUpdated",
            "params": {
                "itemId": "diff-1",
                "changes": [{
                    "path": "src/lib.rs",
                    "diff": "@@ -1 +1 @@\n-a\n+b",
                }],
            },
        }))
        .unwrap();

        assert!(matches!(
            event,
            LocalCodexRenderableEvent::FileDiffDisplayCard { .. }
        ));
    }

    #[test]
    fn app_server_patch_updated_falls_back_when_unstructured() {
        let event = app_server_renderable_event(&json!({
            "method": "item/fileChange/patchUpdated",
            "params": {
                "itemId": "diff-1",
                "changes": [{ "path": "src/lib.rs" }],
            },
        }))
        .unwrap();

        assert!(matches!(
            event,
            LocalCodexRenderableEvent::FileDiffTranscript { .. }
        ));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn app_server_runner_streams_display_only_command_cards() {
        let dir = tempfile::tempdir().unwrap();
        let fake_codex = write_fake_codex(
            dir.path(),
            r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "codex fake"
  exit 0
fi
if [ "$1" = "login" ] && [ "$2" = "status" ]; then
  echo "Logged in using ChatGPT"
  exit 0
fi
if [ "$1" = "app-server" ]; then
  exec python3 -c '
import json
import sys

THREAD_ID = "thread-1"
TURN_ID = "turn-1"

def send(obj):
    print(json.dumps(obj, separators=(",", ":")), flush=True)

for line in sys.stdin:
    message = json.loads(line)
    request_id = message.get("id")
    method = message.get("method")
    if method == "initialize":
        send({"id": request_id, "result": {
            "userAgent": "fake-app-server",
            "codexHome": "/tmp/codex",
            "platformFamily": "unix",
            "platformOs": "macos",
        }})
    elif method == "thread/start":
        send({"id": request_id, "result": {
            "thread": {"id": THREAD_ID},
            "model": "gpt-5.5",
            "modelProvider": "openai",
            "serviceTier": None,
            "cwd": "/tmp",
            "instructionSources": [],
            "approvalPolicy": "never",
            "approvalsReviewer": "user",
            "sandbox": {"type": "dangerFullAccess"},
            "permissionProfile": {"type": "disabled"},
            "activePermissionProfile": None,
            "reasoningEffort": "xhigh",
        }})
    elif method == "turn/start":
        send({"id": request_id, "result": {"turn": {"id": TURN_ID}}})
        send({"method": "turn/started", "params": {"threadId": THREAD_ID, "turn": {"id": TURN_ID}}})
        send({"method": "item/agentMessage/delta", "params": {
            "threadId": THREAD_ID, "turnId": TURN_ID, "itemId": "msg-1", "delta": "app-server ",
        }})
        send({"method": "item/agentMessage/delta", "params": {
            "threadId": THREAD_ID, "turnId": TURN_ID, "itemId": "msg-1", "delta": "ok",
        }})
        send({"method": "item/reasoning/textDelta", "params": {
            "threadId": THREAD_ID, "turnId": TURN_ID, "itemId": "reason-1", "delta": "thinking", "contentIndex": 0,
        }})
        send({"method": "item/started", "params": {
            "threadId": THREAD_ID, "turnId": TURN_ID, "startedAtMs": 1,
            "item": {
                "id": "cmd-1",
                "type": "commandExecution",
                "command": "echo ok",
                "commandActions": [],
                "cwd": "/tmp",
                "status": "inProgress",
            },
        }})
        send({"method": "item/commandExecution/outputDelta", "params": {
            "threadId": THREAD_ID, "turnId": TURN_ID, "itemId": "cmd-1", "delta": "$ echo ok\nok",
        }})
        send({"method": "item/fileChange/outputDelta", "params": {
            "threadId": THREAD_ID, "turnId": TURN_ID, "itemId": "diff-output-1", "delta": "@@ -1 +1 @@\n-old\n+new",
        }})
        send({"method": "item/fileChange/patchUpdated", "params": {
            "threadId": THREAD_ID, "turnId": TURN_ID, "itemId": "diff-1",
            "changes": [{"path": "a.txt", "diff": "@@ -1 +1 @@\n-old\n+new"}],
        }})
        send({"method": "item/mcpToolCall/progress", "params": {
            "threadId": THREAD_ID, "turnId": TURN_ID, "itemId": "tool-1", "message": "tool progress",
        }})
        send({"method": "turn/completed", "params": {"threadId": THREAD_ID, "turn": {"id": TURN_ID}}})
    elif method == "turn/interrupt":
        send({"id": request_id, "result": {}})
'
fi
exit 64
"#,
        );
        let _bin = EnvGuard::set(CODEX_BIN_ENV, fake_codex);
        let _runner = EnvGuard::set(CODEX_RUNNER_ENV, "app-server");
        let _timeout = EnvGuard::set(CODEX_TIMEOUT_MS_ENV, "5000");
        let (_cancel_tx, cancel_rx) = oneshot::channel();

        let mut stream = generate_multi_agent_response_stream(empty_request(), cancel_rx);
        let mut actions = Vec::new();
        let mut saw_done = false;
        while let Some(event) = stream.next().await {
            let event = event.unwrap();
            match event.r#type {
                Some(api::response_event::Type::ClientActions(client_actions)) => {
                    actions.extend(client_actions.actions);
                }
                Some(api::response_event::Type::Finished(
                    api::response_event::StreamFinished {
                        reason: Some(api::response_event::stream_finished::Reason::Done(_)),
                        ..
                    },
                )) => {
                    saw_done = true;
                    break;
                }
                _ => {}
            }
        }

        let texts = coalesced_agent_output_texts(&actions);
        let reasoning = coalesced_reasoning_texts(&actions);

        assert!(saw_done);
        assert!(texts.iter().any(|text| text == "app-server ok"));
        assert_eq!(reasoning, vec!["thinking"]);
        assert!(texts.iter().any(|text| text.contains("command transcript")));
        assert!(texts
            .iter()
            .any(|text| text.contains("file diff transcript")));
        assert!(texts.iter().any(|text| text.contains("tool transcript")));
        assert!(actions_contain_tool_call(&actions));
        assert!(!actions_contain_executable_tool_call(&actions));
        assert_eq!(apply_file_diffs_tool_calls(&actions).len(), 1);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn app_server_runner_cancels_promptly() {
        let dir = tempfile::tempdir().unwrap();
        let fake_codex = write_fake_codex(
            dir.path(),
            r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "codex fake"
  exit 0
fi
if [ "$1" = "login" ] && [ "$2" = "status" ]; then
  echo "Logged in using ChatGPT"
  exit 0
fi
if [ "$1" = "app-server" ]; then
  exec python3 -c '
import json
import sys
import time

THREAD_ID = "thread-1"
TURN_ID = "turn-1"

def send(obj):
    print(json.dumps(obj, separators=(",", ":")), flush=True)

for line in sys.stdin:
    message = json.loads(line)
    request_id = message.get("id")
    method = message.get("method")
    if method == "initialize":
        send({"id": request_id, "result": {
            "userAgent": "fake-app-server",
            "codexHome": "/tmp/codex",
            "platformFamily": "unix",
            "platformOs": "macos",
        }})
    elif method == "thread/start":
        send({"id": request_id, "result": {"thread": {"id": THREAD_ID}}})
    elif method == "turn/start":
        send({"id": request_id, "result": {"turn": {"id": TURN_ID}}})
        while True:
            time.sleep(1)
    elif method == "turn/interrupt":
        send({"id": request_id, "result": {}})
'
fi
exit 64
"#,
        );
        let _bin = EnvGuard::set(CODEX_BIN_ENV, fake_codex);
        let _runner = EnvGuard::set(CODEX_RUNNER_ENV, "app-server");
        let _timeout = EnvGuard::set(CODEX_TIMEOUT_MS_ENV, "10000");
        let (cancel_tx, cancel_rx) = oneshot::channel();

        let result = tokio::time::timeout(std::time::Duration::from_secs(3), async move {
            let mut stream = generate_multi_agent_response_stream(empty_request(), cancel_rx);
            let first = stream.next().await.unwrap().unwrap();
            assert!(matches!(
                first.r#type,
                Some(api::response_event::Type::Init(_))
            ));

            cancel_tx.send(()).unwrap();

            let mut saw_cancel_finish = false;
            while let Some(event) = stream.next().await {
                let event = event.unwrap();
                if matches!(
                    event.r#type,
                    Some(api::response_event::Type::Finished(
                        api::response_event::StreamFinished {
                            reason: Some(api::response_event::stream_finished::Reason::Other(_)),
                            ..
                        }
                    ))
                ) {
                    saw_cancel_finish = true;
                    break;
                }
            }
            assert!(saw_cancel_finish);
        })
        .await;

        assert!(result.is_ok(), "app-server stream did not cancel promptly");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn codex_exec_uses_fake_codex_and_extracts_response() {
        let dir = tempfile::tempdir().unwrap();
        let fake_codex = write_fake_codex(
            dir.path(),
            r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "codex fake"
  exit 0
fi
if [ "$1" = "login" ] && [ "$2" = "status" ]; then
  echo "Logged in using ChatGPT"
  exit 0
fi
if [ "$1" = "exec" ]; then
  echo '{"type":"item.completed","item":{"id":"item_0","type":"agent_message","text":"fake-ok"}}'
  exit 0
fi
exit 64
"#,
        );
        let _bin = EnvGuard::set(CODEX_BIN_ENV, fake_codex);
        let _timeout = EnvGuard::remove(CODEX_TIMEOUT_MS_ENV);

        let response = codex_exec("test prompt").await.unwrap();

        assert_eq!(response, "fake-ok");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn ensure_logged_in_rejects_not_logged_in_fake_codex() {
        let dir = tempfile::tempdir().unwrap();
        let fake_codex = write_fake_codex(
            dir.path(),
            r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "codex fake"
  exit 0
fi
if [ "$1" = "login" ] && [ "$2" = "status" ]; then
  echo "Not logged in"
  exit 0
fi
exit 64
"#,
        );
        let _bin = EnvGuard::set(CODEX_BIN_ENV, fake_codex);

        let err = ensure_logged_in().await.unwrap_err().to_string();

        assert!(err.contains("Codex CLI is not logged in"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn codex_exec_reports_nonzero_exit_without_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let fake_codex = write_fake_codex(
            dir.path(),
            r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "codex fake"
  exit 0
fi
if [ "$1" = "login" ] && [ "$2" = "status" ]; then
  echo "Logged in using ChatGPT"
  exit 0
fi
if [ "$1" = "exec" ]; then
  echo "boom" >&2
  exit 42
fi
exit 64
"#,
        );
        let _bin = EnvGuard::set(CODEX_BIN_ENV, fake_codex);
        let _timeout = EnvGuard::remove(CODEX_TIMEOUT_MS_ENV);

        let err = codex_exec("test prompt").await.unwrap_err().to_string();

        assert!(err.contains("Codex CLI failed with status"));
        assert!(err.contains("boom"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn codex_exec_times_out() {
        let dir = tempfile::tempdir().unwrap();
        let fake_codex = write_fake_codex(
            dir.path(),
            r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "codex fake"
  exit 0
fi
if [ "$1" = "login" ] && [ "$2" = "status" ]; then
  echo "Logged in using ChatGPT"
  exit 0
fi
if [ "$1" = "exec" ]; then
  sleep 2
  exit 0
fi
exit 64
"#,
        );
        let _bin = EnvGuard::set(CODEX_BIN_ENV, fake_codex);
        let _timeout = EnvGuard::set(CODEX_TIMEOUT_MS_ENV, "20");

        let err = codex_exec("test prompt").await.unwrap_err().to_string();

        assert!(err.contains("Codex request timed out"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn local_response_stream_starts_before_codex_exec_finishes_and_cancels() {
        let dir = tempfile::tempdir().unwrap();
        let fake_codex = write_fake_codex(
            dir.path(),
            r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "codex fake"
  exit 0
fi
if [ "$1" = "login" ] && [ "$2" = "status" ]; then
  echo "Logged in using ChatGPT"
  exit 0
fi
if [ "$1" = "exec" ]; then
  sleep 5
  echo '{"type":"item.completed","item":{"id":"item_0","type":"agent_message","text":"too-late"}}'
  exit 0
fi
exit 64
"#,
        );
        let _bin = EnvGuard::set(CODEX_BIN_ENV, fake_codex);
        let _runner = EnvGuard::remove(CODEX_RUNNER_ENV);
        let _timeout = EnvGuard::set(CODEX_TIMEOUT_MS_ENV, "10000");
        let (cancel_tx, cancel_rx) = oneshot::channel();

        let result = tokio::time::timeout(std::time::Duration::from_secs(2), async move {
            let mut stream = generate_multi_agent_response_stream(empty_request(), cancel_rx);
            let first = stream.next().await.unwrap().unwrap();
            assert!(matches!(
                first.r#type,
                Some(api::response_event::Type::Init(_))
            ));

            cancel_tx.send(()).unwrap();

            let mut saw_cancel_finish = false;
            while let Some(event) = stream.next().await {
                let event = event.unwrap();
                if matches!(
                    event.r#type,
                    Some(api::response_event::Type::Finished(
                        api::response_event::StreamFinished {
                            reason: Some(api::response_event::stream_finished::Reason::Other(_)),
                            ..
                        }
                    ))
                ) {
                    saw_cancel_finish = true;
                    break;
                }
            }
            assert!(saw_cancel_finish);
        })
        .await;

        assert!(result.is_ok(), "local Codex stream did not cancel promptly");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn local_response_stream_reports_login_error_without_warp_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let fake_codex = write_fake_codex(
            dir.path(),
            r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "codex fake"
  exit 0
fi
if [ "$1" = "login" ] && [ "$2" = "status" ]; then
  echo "Not logged in"
  exit 0
fi
if [ "$1" = "exec" ]; then
  echo "should not run" >&2
  exit 99
fi
exit 64
"#,
        );
        let _bin = EnvGuard::set(CODEX_BIN_ENV, fake_codex);
        let _runner = EnvGuard::remove(CODEX_RUNNER_ENV);
        let (_cancel_tx, cancel_rx) = oneshot::channel();

        let mut stream = generate_multi_agent_response_stream(empty_request(), cancel_rx);
        let mut texts = Vec::new();
        let mut saw_internal_error = false;
        while let Some(event) = stream.next().await {
            let event = event.unwrap();
            match event.r#type {
                Some(api::response_event::Type::ClientActions(actions)) => {
                    texts.extend(agent_output_texts(&actions.actions));
                }
                Some(api::response_event::Type::Finished(
                    api::response_event::StreamFinished {
                        reason: Some(api::response_event::stream_finished::Reason::InternalError(_)),
                        ..
                    },
                )) => {
                    saw_internal_error = true;
                    break;
                }
                _ => {}
            }
        }

        assert!(texts.iter().any(|text| text.contains("Local Codex error")));
        assert!(saw_internal_error);
    }

    #[test]
    fn identifies_local_codex_conversation_tokens() {
        assert!(is_local_conversation_token("local-codex-123"));
        assert!(!is_local_conversation_token("server-conversation-123"));
    }

    #[test]
    fn local_stream_ids_ignore_request_ambient_task_id() {
        let request = request_with_ambient_task_id();
        let (conversation_id, request_id, run_id) = local_stream_ids(&request);

        assert_eq!(conversation_id, "existing-conversation");
        assert!(request_id.starts_with(LOCAL_CONVERSATION_PREFIX));
        assert!(run_id.starts_with(LOCAL_CONVERSATION_PREFIX));
        assert_ne!(
            run_id, "c239995a-e35d-441b-a828-a2a8dcceeacb",
            "local Codex must not reuse Warp cloud ambient task ids"
        );
    }
}

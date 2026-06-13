use std::{
    fmt,
    sync::OnceLock,
    time::{Duration, Instant},
};

const ENV_VAR: &str = "WARP_MARKDOWN_PERF_LOG";
const LOG_TARGET: &str = "warpcodexoss.markdown_perf";

fn env_enabled() -> bool {
    let Ok(value) = std::env::var(ENV_VAR) else {
        return false;
    };
    !matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "" | "0" | "false" | "off" | "no"
    )
}

pub fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(env_enabled)
}

pub fn start() -> Option<Instant> {
    enabled().then(Instant::now)
}

pub fn log(stage: &str, elapsed: Duration, details: impl fmt::Display) {
    if !enabled() {
        return;
    }

    log::warn!(
        target: LOG_TARGET,
        "stage={stage} elapsed_ms={:.3} {details}",
        elapsed.as_secs_f64() * 1000.0
    );
}

pub fn log_instant(stage: &str, start: Option<Instant>, details: impl fmt::Display) {
    if let Some(start) = start {
        log(stage, start.elapsed(), details);
    }
}

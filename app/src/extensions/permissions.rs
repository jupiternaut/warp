/// Explicit permission model for VSIX capabilities. Defaults deny every capability.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ExtensionPermissions {
    pub(crate) filesystem: Vec<FilesystemPermission>,
    pub(crate) network: NetworkPermission,
    pub(crate) terminal: TerminalPermission,
    pub(crate) secrets: SecretPermission,
    pub(crate) webview: WebviewPermission,
    pub(crate) authentication: AuthenticationPermission,
    pub(crate) ai: AiAccessPermission,
}

impl ExtensionPermissions {
    pub(crate) fn is_deny_by_default(&self) -> bool {
        self.filesystem.is_empty()
            && self.network == NetworkPermission::None
            && self.terminal == TerminalPermission::None
            && self.secrets == SecretPermission::None
            && self.webview == WebviewPermission::None
            && self.authentication == AuthenticationPermission::None
            && self.ai == AiAccessPermission::Denied
    }

    pub(crate) fn requested_capabilities(&self) -> Vec<&'static str> {
        let mut capabilities = Vec::new();
        if !self.filesystem.is_empty() {
            capabilities.push("filesystem");
        }
        if self.network != NetworkPermission::None {
            capabilities.push("network");
        }
        if self.terminal != TerminalPermission::None {
            capabilities.push("terminal");
        }
        if self.secrets != SecretPermission::None {
            capabilities.push("secrets");
        }
        if self.webview != WebviewPermission::None {
            capabilities.push("webview");
        }
        if self.authentication != AuthenticationPermission::None {
            capabilities.push("authentication");
        }
        if self.ai != AiAccessPermission::Denied {
            capabilities.push("ai");
        }
        capabilities
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FilesystemPermission {
    ReadExtensionDirectory,
    ReadWorkspace,
    WriteWorkspace,
    ReadUserSelectedFiles,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) enum NetworkPermission {
    #[default]
    None,
    Hosts(Vec<String>),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) enum TerminalPermission {
    #[default]
    None,
    CommandProposals,
    UserApprovedCommandExecution,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) enum SecretPermission {
    #[default]
    None,
    ExtensionScopedStorage,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) enum WebviewPermission {
    #[default]
    None,
    LocalContentOnly,
    RemoteContent(Vec<String>),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) enum AuthenticationPermission {
    #[default]
    None,
    UserMediatedProviderTokens,
    /// Explicitly diagnosed as unsupported: Codex OAuth tokens are not available to extensions.
    CodexOAuthToken,
}

impl AuthenticationPermission {
    pub(crate) fn requests_codex_oauth(&self) -> bool {
        matches!(self, Self::CodexOAuthToken)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) enum AiAccessPermission {
    #[default]
    Denied,
    RequiresExplicitUserApproval,
}

impl AiAccessPermission {
    pub(crate) fn requires_user_approval(&self) -> bool {
        matches!(self, Self::RequiresExplicitUserApproval)
    }

    pub(crate) fn can_consume_warp_ai_credits_without_prompt(&self) -> bool {
        false
    }
}

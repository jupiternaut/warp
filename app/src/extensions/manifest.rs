use super::{ExtensionIdentity, ExtensionPermissions};

/// Final-shaped subset of VS Code manifest metadata Warp needs before runtime support exists.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExtensionManifest {
    pub(crate) identity: ExtensionIdentity,
    pub(crate) metadata: ExtensionManifestMetadata,
    pub(crate) engines: ExtensionEngineCompatibility,
    pub(crate) extension_kind: Vec<ExtensionKind>,
    pub(crate) activation_events: Vec<ActivationEvent>,
    pub(crate) contributes: Vec<ContributionPoint>,
    pub(crate) permissions: ExtensionPermissions,
}

impl ExtensionManifest {
    pub(crate) fn new(
        identity: ExtensionIdentity,
        metadata: ExtensionManifestMetadata,
        engines: ExtensionEngineCompatibility,
    ) -> Self {
        Self {
            identity,
            metadata,
            engines,
            extension_kind: Vec::new(),
            activation_events: Vec::new(),
            contributes: Vec::new(),
            permissions: ExtensionPermissions::default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExtensionManifestMetadata {
    pub(crate) display_name: String,
    pub(crate) version: String,
    pub(crate) description: Option<String>,
    pub(crate) license: Option<String>,
    pub(crate) repository: Option<String>,
}

impl ExtensionManifestMetadata {
    pub(crate) fn new(display_name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            display_name: display_name.into(),
            version: version.into(),
            description: None,
            license: None,
            repository: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExtensionEngineCompatibility {
    pub(crate) vscode: String,
}

impl ExtensionEngineCompatibility {
    pub(crate) fn vscode(version_range: impl Into<String>) -> Self {
        Self {
            vscode: version_range.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ExtensionKind {
    Ui,
    Workspace,
    Web,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ActivationEvent {
    OnStartupFinished,
    OnCommand(String),
    OnLanguage(String),
    WorkspaceContains(String),
    Raw(String),
}

impl ActivationEvent {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::OnStartupFinished => "onStartupFinished",
            Self::OnCommand(value)
            | Self::OnLanguage(value)
            | Self::WorkspaceContains(value)
            | Self::Raw(value) => value,
        }
    }

    pub(crate) fn is_contract_supported(&self) -> bool {
        matches!(
            self,
            Self::OnStartupFinished | Self::OnCommand(_) | Self::OnLanguage(_)
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ContributionPoint {
    Commands,
    Configuration,
    Languages,
    Grammars,
    Themes,
    Keybindings,
    Raw(String),
}

impl ContributionPoint {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::Commands => "commands",
            Self::Configuration => "configuration",
            Self::Languages => "languages",
            Self::Grammars => "grammars",
            Self::Themes => "themes",
            Self::Keybindings => "keybindings",
            Self::Raw(value) => value,
        }
    }

    pub(crate) fn is_contract_supported(&self) -> bool {
        matches!(
            self,
            Self::Commands | Self::Configuration | Self::Languages | Self::Grammars | Self::Themes
        )
    }
}

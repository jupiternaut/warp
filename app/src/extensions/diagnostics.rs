use super::ExtensionIdentity;

/// Severity for compatibility findings surfaced before any untrusted extension code can run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CompatibilitySeverity {
    Error,
    Warning,
    Info,
}

/// Stable diagnostic codes for unsupported VSIX features and security boundaries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CompatibilityDiagnosticCode {
    FeatureDisabled,
    InvalidManifest,
    UntrustedPackage,
    UnsupportedEngine,
    UnsupportedRuntime,
    NodeRuntimeUnavailable,
    QuickJsHostOutOfScope,
    UnsupportedActivationEvent,
    UnsupportedContribution,
    UnsupportedPermission,
    CodexOAuthUnavailable,
    AiAccessRequiresApproval,
}

/// One structured compatibility diagnostic. Unsupported features must be represented here rather
/// than being silently ignored.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompatibilityDiagnostic {
    pub(crate) severity: CompatibilitySeverity,
    pub(crate) code: CompatibilityDiagnosticCode,
    pub(crate) message: String,
    pub(crate) extension: Option<ExtensionIdentity>,
    pub(crate) subject: Option<String>,
}

impl CompatibilityDiagnostic {
    pub(crate) fn error(code: CompatibilityDiagnosticCode, message: impl Into<String>) -> Self {
        Self::new(CompatibilitySeverity::Error, code, message)
    }

    pub(crate) fn warning(code: CompatibilityDiagnosticCode, message: impl Into<String>) -> Self {
        Self::new(CompatibilitySeverity::Warning, code, message)
    }

    pub(crate) fn info(code: CompatibilityDiagnosticCode, message: impl Into<String>) -> Self {
        Self::new(CompatibilitySeverity::Info, code, message)
    }

    pub(crate) fn new(
        severity: CompatibilitySeverity,
        code: CompatibilityDiagnosticCode,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            code,
            message: message.into(),
            extension: None,
            subject: None,
        }
    }

    pub(crate) fn with_extension(mut self, extension: ExtensionIdentity) -> Self {
        self.extension = Some(extension);
        self
    }

    pub(crate) fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }
}

/// Collection wrapper for compatibility diagnostics.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CompatibilityDiagnosticSet {
    diagnostics: Vec<CompatibilityDiagnostic>,
}

impl CompatibilityDiagnosticSet {
    pub(crate) fn push(&mut self, diagnostic: CompatibilityDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &CompatibilityDiagnostic> {
        self.diagnostics.iter()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub(crate) fn has_errors(&self) -> bool {
        self.iter()
            .any(|diagnostic| diagnostic.severity == CompatibilitySeverity::Error)
    }

    pub(crate) fn contains_code(&self, code: CompatibilityDiagnosticCode) -> bool {
        self.iter().any(|diagnostic| diagnostic.code == code)
    }
}

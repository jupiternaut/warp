//! Contract boundary for future VSIX / VS Code extension support in WarpCodexOss.
#![allow(dead_code, unused_imports)]
//!
//! This module is intentionally a shape-only integration layer: it defines the
//! identities, manifest metadata, permission declarations, diagnostics, and
//! registry/store handles that later VSIX work can build on. It does **not** run
//! Node extension hosts, extract VSIX archives, contact Open VSX, or reuse the
//! existing QuickJS Warp-native plugin host in `crate::plugin`.

mod diagnostics;
mod identity;
mod manifest;
mod permissions;
mod registry;

pub(crate) use diagnostics::{
    CompatibilityDiagnostic, CompatibilityDiagnosticCode, CompatibilityDiagnosticSet,
    CompatibilitySeverity,
};
pub(crate) use identity::{ExtensionIdentity, ExtensionIdentityError, ExtensionInstanceIdentity};
pub(crate) use manifest::{
    ActivationEvent, ContributionPoint, ExtensionEngineCompatibility, ExtensionKind,
    ExtensionManifest, ExtensionManifestMetadata,
};
pub(crate) use permissions::{
    AiAccessPermission, AuthenticationPermission, ExtensionPermissions, FilesystemPermission,
    NetworkPermission, SecretPermission, TerminalPermission, WebviewPermission,
};
pub(crate) use registry::{
    ExtensionRegistry, ExtensionRegistryHandle, ExtensionRegistryRecord, ExtensionStoreHandle,
    ExtensionStoreSource, InMemoryExtensionRegistry, RegistryError, VsixPackageHandle,
};

use crate::features::FeatureFlag;

/// Canonical feature flag for the VSIX / VS Code extension contract layer.
pub(crate) const VS_CODE_EXTENSIONS_FEATURE_FLAG: FeatureFlag = FeatureFlag::VsCodeExtensions;

/// Returns whether the VSIX contract layer is enabled for the current process.
pub(crate) fn vs_code_extensions_enabled() -> bool {
    VS_CODE_EXTENSIONS_FEATURE_FLAG.is_enabled()
}

/// Produces the structured diagnostic used whenever VSIX support is reached while disabled.
pub(crate) fn feature_gate_diagnostic() -> Option<CompatibilityDiagnostic> {
    (!vs_code_extensions_enabled()).then(|| {
        CompatibilityDiagnostic::error(
            CompatibilityDiagnosticCode::FeatureDisabled,
            "VS Code extensions are disabled by FeatureFlag::VsCodeExtensions",
        )
    })
}

/// Builds the baseline compatibility diagnostics for a manifest without launching extension code.
pub(crate) fn diagnose_manifest_compatibility(
    manifest: &ExtensionManifest,
) -> CompatibilityDiagnosticSet {
    let mut diagnostics = CompatibilityDiagnosticSet::default();

    if let Some(diagnostic) = feature_gate_diagnostic() {
        diagnostics.push(diagnostic.with_extension(manifest.identity.clone()));
    }

    for activation_event in &manifest.activation_events {
        if !activation_event.is_contract_supported() {
            diagnostics.push(
                CompatibilityDiagnostic::error(
                    CompatibilityDiagnosticCode::UnsupportedActivationEvent,
                    format!(
                        "activation event `{}` is not supported by the VSIX contract layer",
                        activation_event.as_str()
                    ),
                )
                .with_extension(manifest.identity.clone())
                .with_subject(activation_event.as_str()),
            );
        }
    }

    for contribution in &manifest.contributes {
        if !contribution.is_contract_supported() {
            diagnostics.push(
                CompatibilityDiagnostic::warning(
                    CompatibilityDiagnosticCode::UnsupportedContribution,
                    format!(
                        "contribution point `{}` is recorded but not supported yet",
                        contribution.as_str()
                    ),
                )
                .with_extension(manifest.identity.clone())
                .with_subject(contribution.as_str()),
            );
        }
    }

    if manifest.permissions.ai.requires_user_approval() {
        diagnostics.push(
            CompatibilityDiagnostic::warning(
                CompatibilityDiagnosticCode::AiAccessRequiresApproval,
                "extension AI access requires explicit user approval and must not silently consume Warp AI credits",
            )
            .with_extension(manifest.identity.clone()),
        );
    }

    if manifest.permissions.authentication.requests_codex_oauth() {
        diagnostics.push(
            CompatibilityDiagnostic::error(
                CompatibilityDiagnosticCode::CodexOAuthUnavailable,
                "Codex OAuth tokens are never exposed to extensions",
            )
            .with_extension(manifest.identity.clone()),
        );
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest() -> ExtensionManifest {
        ExtensionManifest::new(
            ExtensionIdentity::new("example", "hello-warp").unwrap(),
            ExtensionManifestMetadata::new("Hello Warp", "0.1.0"),
            ExtensionEngineCompatibility::vscode("^1.90.0"),
        )
    }

    #[test]
    fn feature_gate_emits_structured_diagnostic_when_disabled() {
        let _guard = FeatureFlag::VsCodeExtensions.override_enabled(false);

        let diagnostic = feature_gate_diagnostic().expect("disabled flag should diagnose");

        assert_eq!(
            diagnostic.code,
            CompatibilityDiagnosticCode::FeatureDisabled
        );
        assert_eq!(diagnostic.severity, CompatibilitySeverity::Error);
    }

    #[test]
    fn feature_gate_allows_enabled_contract_layer() {
        let _guard = FeatureFlag::VsCodeExtensions.override_enabled(true);

        assert!(vs_code_extensions_enabled());
        assert!(feature_gate_diagnostic().is_none());
    }

    #[test]
    fn default_permissions_are_deny_by_default_and_never_silent_ai_credit_use() {
        let permissions = ExtensionPermissions::default();

        assert!(permissions.is_deny_by_default());
        assert!(!permissions.ai.can_consume_warp_ai_credits_without_prompt());
        assert!(!permissions.authentication.requests_codex_oauth());
        assert!(permissions.requested_capabilities().is_empty());
    }

    #[test]
    fn requested_permissions_are_explicitly_reported() {
        let permissions = ExtensionPermissions {
            filesystem: vec![FilesystemPermission::ReadWorkspace],
            network: NetworkPermission::Hosts(vec!["https://open-vsx.org".into()]),
            terminal: TerminalPermission::CommandProposals,
            secrets: SecretPermission::ExtensionScopedStorage,
            webview: WebviewPermission::LocalContentOnly,
            authentication: AuthenticationPermission::UserMediatedProviderTokens,
            ai: AiAccessPermission::RequiresExplicitUserApproval,
        };

        assert_eq!(
            permissions.requested_capabilities(),
            vec![
                "filesystem",
                "network",
                "terminal",
                "secrets",
                "webview",
                "authentication",
                "ai",
            ]
        );
        assert!(!permissions.ai.can_consume_warp_ai_credits_without_prompt());
    }

    #[test]
    fn unsupported_manifest_features_are_structured_diagnostics() {
        let _guard = FeatureFlag::VsCodeExtensions.override_enabled(true);
        let mut manifest = sample_manifest();
        manifest.activation_events = vec![ActivationEvent::Raw("onDebug".into())];
        manifest.contributes = vec![ContributionPoint::Raw("debuggers".into())];
        manifest.permissions.authentication = AuthenticationPermission::CodexOAuthToken;

        let diagnostics = diagnose_manifest_compatibility(&manifest);

        assert!(diagnostics.has_errors());
        assert!(diagnostics.contains_code(CompatibilityDiagnosticCode::UnsupportedActivationEvent));
        assert!(diagnostics.contains_code(CompatibilityDiagnosticCode::UnsupportedContribution));
        assert!(diagnostics.contains_code(CompatibilityDiagnosticCode::CodexOAuthUnavailable));
    }

    #[test]
    fn registry_stores_manifest_with_diagnostics_without_running_code() {
        let _guard = FeatureFlag::VsCodeExtensions.override_enabled(false);
        let mut registry = InMemoryExtensionRegistry::default();
        let manifest = sample_manifest();

        let handle = registry.register_manifest(manifest).unwrap();
        let diagnostics = registry.compatibility_diagnostics(handle).unwrap();

        assert!(diagnostics.contains_code(CompatibilityDiagnosticCode::FeatureDisabled));
    }
}

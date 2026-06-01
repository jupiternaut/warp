use std::{collections::BTreeMap, error::Error, fmt, path::PathBuf};

use super::{diagnose_manifest_compatibility, CompatibilityDiagnosticSet, ExtensionManifest};

/// Opaque handle for registry records. Handles are local process identifiers, not marketplace IDs.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub(crate) struct ExtensionRegistryHandle(u64);

impl ExtensionRegistryHandle {
    pub(crate) fn raw(self) -> u64 {
        self.0
    }
}

/// Opaque handle for extension store references. Open VSX networking is deliberately not implemented.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExtensionStoreHandle {
    pub(crate) source: ExtensionStoreSource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ExtensionStoreSource {
    LocalVsix(VsixPackageHandle),
    OpenVsx { namespace: String, name: String },
}

/// Local VSIX package reference. The package is untrusted until a future verifier proves otherwise.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VsixPackageHandle {
    pub(crate) path: PathBuf,
    pub(crate) sha256: Option<String>,
    pub(crate) trusted: bool,
}

impl VsixPackageHandle {
    pub(crate) fn untrusted(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            sha256: None,
            trusted: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExtensionRegistryRecord {
    pub(crate) handle: ExtensionRegistryHandle,
    pub(crate) manifest: ExtensionManifest,
    pub(crate) diagnostics: CompatibilityDiagnosticSet,
    pub(crate) store: Option<ExtensionStoreHandle>,
}

pub(crate) trait ExtensionRegistry {
    fn register_manifest(
        &mut self,
        manifest: ExtensionManifest,
    ) -> Result<ExtensionRegistryHandle, RegistryError>;

    fn get_manifest(&self, handle: ExtensionRegistryHandle) -> Option<&ExtensionManifest>;

    fn compatibility_diagnostics(
        &self,
        handle: ExtensionRegistryHandle,
    ) -> Option<&CompatibilityDiagnosticSet>;
}

#[derive(Debug, Default)]
pub(crate) struct InMemoryExtensionRegistry {
    next_handle: u64,
    records: BTreeMap<ExtensionRegistryHandle, ExtensionRegistryRecord>,
}

impl ExtensionRegistry for InMemoryExtensionRegistry {
    fn register_manifest(
        &mut self,
        manifest: ExtensionManifest,
    ) -> Result<ExtensionRegistryHandle, RegistryError> {
        self.next_handle = self
            .next_handle
            .checked_add(1)
            .ok_or(RegistryError::HandleSpaceExhausted)?;
        let handle = ExtensionRegistryHandle(self.next_handle);
        let diagnostics = diagnose_manifest_compatibility(&manifest);
        self.records.insert(
            handle,
            ExtensionRegistryRecord {
                handle,
                manifest,
                diagnostics,
                store: None,
            },
        );
        Ok(handle)
    }

    fn get_manifest(&self, handle: ExtensionRegistryHandle) -> Option<&ExtensionManifest> {
        self.records.get(&handle).map(|record| &record.manifest)
    }

    fn compatibility_diagnostics(
        &self,
        handle: ExtensionRegistryHandle,
    ) -> Option<&CompatibilityDiagnosticSet> {
        self.records.get(&handle).map(|record| &record.diagnostics)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RegistryError {
    HandleSpaceExhausted,
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HandleSpaceExhausted => write!(f, "extension registry handle space exhausted"),
        }
    }
}

impl Error for RegistryError {}

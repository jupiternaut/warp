use std::{error::Error, fmt};

/// Publisher/name identity used by VS Code marketplaces and VSIX manifests.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub(crate) struct ExtensionIdentity {
    pub(crate) publisher: String,
    pub(crate) name: String,
}

impl ExtensionIdentity {
    pub(crate) fn new(
        publisher: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<Self, ExtensionIdentityError> {
        let publisher = publisher.into();
        let name = name.into();
        validate_component("publisher", &publisher)?;
        validate_component("name", &name)?;
        Ok(Self { publisher, name })
    }

    pub(crate) fn qualified_id(&self) -> String {
        format!("{}.{}", self.publisher, self.name)
    }
}

/// Concrete installed extension identity, including the manifest version string.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct ExtensionInstanceIdentity {
    pub(crate) identity: ExtensionIdentity,
    pub(crate) version: String,
}

impl ExtensionInstanceIdentity {
    pub(crate) fn new(identity: ExtensionIdentity, version: impl Into<String>) -> Self {
        Self {
            identity,
            version: version.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ExtensionIdentityError {
    Empty {
        component: &'static str,
    },
    InvalidCharacter {
        component: &'static str,
        value: String,
    },
}

impl fmt::Display for ExtensionIdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty { component } => write!(f, "extension {component} cannot be empty"),
            Self::InvalidCharacter { component, value } => {
                write!(
                    f,
                    "extension {component} contains unsupported characters: {value}"
                )
            }
        }
    }
}

impl Error for ExtensionIdentityError {}

fn validate_component(component: &'static str, value: &str) -> Result<(), ExtensionIdentityError> {
    if value.is_empty() {
        return Err(ExtensionIdentityError::Empty { component });
    }

    let valid = value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'));
    if !valid {
        return Err(ExtensionIdentityError::InvalidCharacter {
            component,
            value: value.to_string(),
        });
    }

    Ok(())
}

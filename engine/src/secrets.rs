use std::collections::BTreeMap;

pub const SECRET_SERVICE_NAME: &str = "org.voxtype-personas";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SecretRef(String);

impl SecretRef {
    pub fn for_provider(provider_id: &str) -> Result<Self, SecretError> {
        if provider_id.is_empty()
            || !provider_id
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return Err(SecretError::InvalidReference);
        }
        Ok(Self(format!(
            "{SECRET_SERVICE_NAME}/provider/{provider_id}"
        )))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub trait SecretStore {
    fn read(&self, reference: &SecretRef) -> Result<Vec<u8>, SecretError>;
    fn write(&mut self, reference: SecretRef, secret: &[u8]) -> Result<(), SecretError>;
    fn verify(&self, reference: &SecretRef) -> Result<(), SecretError>;
    fn replace(&mut self, reference: SecretRef, secret: &[u8]) -> Result<(), SecretError>;
    fn delete(&mut self, reference: &SecretRef) -> Result<(), SecretError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SecretError {
    InvalidReference,
    NotFound,
    Unavailable,
    Locked,
    OperationFailed,
}

impl std::fmt::Display for SecretError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidReference => "secret reference is invalid",
            Self::NotFound => "secret is not available",
            Self::Unavailable => "Secret Service is unavailable",
            Self::Locked => "Secret Service is locked",
            Self::OperationFailed => "Secret Service operation failed",
        };
        write!(formatter, "{message}")
    }
}

impl std::error::Error for SecretError {}

#[derive(Default)]
pub struct InMemorySecretStore {
    values: BTreeMap<SecretRef, Vec<u8>>,
}

impl SecretStore for InMemorySecretStore {
    fn read(&self, reference: &SecretRef) -> Result<Vec<u8>, SecretError> {
        self.values
            .get(reference)
            .cloned()
            .ok_or(SecretError::NotFound)
    }

    fn write(&mut self, reference: SecretRef, secret: &[u8]) -> Result<(), SecretError> {
        if self.values.contains_key(&reference) {
            return Err(SecretError::OperationFailed);
        }
        self.values.insert(reference, secret.to_vec());
        Ok(())
    }

    fn verify(&self, reference: &SecretRef) -> Result<(), SecretError> {
        self.read(reference).map(|_| ())
    }

    fn replace(&mut self, reference: SecretRef, secret: &[u8]) -> Result<(), SecretError> {
        if !self.values.contains_key(&reference) {
            return Err(SecretError::NotFound);
        }
        self.values.insert(reference, secret.to_vec());
        Ok(())
    }

    fn delete(&mut self, reference: &SecretRef) -> Result<(), SecretError> {
        self.values
            .remove(reference)
            .map(|_| ())
            .ok_or(SecretError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_lifecycle_uses_an_opaque_reference() {
        let reference = SecretRef::for_provider("openai").expect("reference should be valid");
        let mut store = InMemorySecretStore::default();

        store
            .write(reference.clone(), b"first")
            .expect("write should succeed");
        store.verify(&reference).expect("secret should verify");
        assert_eq!(
            store.read(&reference).expect("secret should read"),
            b"first"
        );
        store
            .replace(reference.clone(), b"second")
            .expect("replace should succeed");
        assert_eq!(
            store.read(&reference).expect("secret should read"),
            b"second"
        );
        store.delete(&reference).expect("delete should succeed");
        assert_eq!(store.read(&reference), Err(SecretError::NotFound));
    }

    #[test]
    fn errors_do_not_include_secret_values() {
        let error = SecretError::OperationFailed;
        assert!(!error.to_string().contains("secret-value"));
    }
}

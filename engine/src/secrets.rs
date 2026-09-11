use std::collections::BTreeMap;
use std::collections::HashMap;

pub const SECRET_SERVICE_NAME: &str = "org.voxtype-personas";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SecretRef(String);

impl SecretRef {
    pub fn from_config(value: &str) -> Result<Self, SecretError> {
        let provider_id = value
            .strip_prefix("org.voxtype-personas/provider/")
            .ok_or(SecretError::InvalidReference)?;
        Self::for_provider(provider_id)
    }

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

pub struct SecretServiceStore;

impl SecretServiceStore {
    pub fn new() -> Self {
        Self
    }

    fn connect() -> Result<secret_service::blocking::SecretService<'static>, SecretError> {
        secret_service::blocking::SecretService::connect(secret_service::EncryptionType::Dh)
            .map_err(map_secret_service_error)
    }

    fn attributes(reference: &SecretRef) -> HashMap<&str, &str> {
        HashMap::from([
            ("application", SECRET_SERVICE_NAME),
            ("reference", reference.as_str()),
        ])
    }

    fn find_unlocked<'a>(
        service: &'a secret_service::blocking::SecretService<'a>,
        reference: &SecretRef,
    ) -> Result<secret_service::blocking::Item<'a>, SecretError> {
        let items = service
            .search_items(Self::attributes(reference))
            .map_err(map_secret_service_error)?;
        if !items.locked.is_empty() {
            return Err(SecretError::Locked);
        }
        items
            .unlocked
            .into_iter()
            .next()
            .ok_or(SecretError::NotFound)
    }
}

impl Default for SecretServiceStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretStore for SecretServiceStore {
    fn read(&self, reference: &SecretRef) -> Result<Vec<u8>, SecretError> {
        let service = Self::connect()?;
        Self::find_unlocked(&service, reference)?
            .get_secret()
            .map_err(map_secret_service_error)
    }

    fn write(&mut self, reference: SecretRef, secret: &[u8]) -> Result<(), SecretError> {
        let service = Self::connect()?;
        if Self::find_unlocked(&service, &reference).is_ok() {
            return Err(SecretError::OperationFailed);
        }
        service
            .get_default_collection()
            .map_err(map_secret_service_error)?
            .create_item(
                "VoxTypePersonas provider key",
                Self::attributes(&reference),
                secret,
                false,
                "text/plain",
            )
            .map_err(map_secret_service_error)?;
        Ok(())
    }

    fn verify(&self, reference: &SecretRef) -> Result<(), SecretError> {
        self.read(reference).map(|_| ())
    }

    fn replace(&mut self, reference: SecretRef, secret: &[u8]) -> Result<(), SecretError> {
        let service = Self::connect()?;
        service
            .get_default_collection()
            .map_err(map_secret_service_error)?
            .create_item(
                "VoxTypePersonas provider key",
                Self::attributes(&reference),
                secret,
                true,
                "text/plain",
            )
            .map_err(map_secret_service_error)?;
        Ok(())
    }

    fn delete(&mut self, reference: &SecretRef) -> Result<(), SecretError> {
        let service = Self::connect()?;
        Self::find_unlocked(&service, reference)?
            .delete()
            .map_err(map_secret_service_error)
    }
}

fn map_secret_service_error(error: secret_service::Error) -> SecretError {
    match error {
        secret_service::Error::Locked => SecretError::Locked,
        secret_service::Error::NoResult => SecretError::NotFound,
        secret_service::Error::Unavailable => SecretError::Unavailable,
        _ => SecretError::OperationFailed,
    }
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

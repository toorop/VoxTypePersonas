use crate::catalog::PortableProfile;
use crate::config::{
    CURRENT_SCHEMA_VERSION, Config, Profile, ProfileState, Prompt, RAW_PROFILE_ID,
};
use crate::paths::AppPaths;
use crate::secrets::{SecretError, SecretRef, SecretServiceStore, SecretStore};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct ConfigStore {
    paths: AppPaths,
}

impl ConfigStore {
    pub fn discover() -> Result<Self, StoreError> {
        Ok(Self::new(AppPaths::discover()?))
    }

    pub fn new(paths: AppPaths) -> Self {
        Self { paths }
    }

    pub fn config_path(&self) -> &Path {
        &self.paths.config_file
    }

    pub fn load_or_create(&self) -> Result<Config, StoreError> {
        self.paths.ensure_private_directories()?;

        match fs::read_to_string(&self.paths.config_file) {
            Ok(source) => self.load_source(&source),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let config = Config::defaults();
                config.validate()?;
                self.write_atomically(&config)?;
                Ok(config)
            }
            Err(error) => Err(StoreError::ReadConfig {
                path: self.paths.config_file.clone(),
                source: error,
            }),
        }
    }

    pub fn set_active_profile(&self, id: &str) -> Result<Config, StoreError> {
        self.set_active_profile_with_secret_store(id, &SecretServiceStore::new())
    }

    pub fn set_active_profile_with_secret_store(
        &self,
        id: &str,
        secret_store: &impl SecretStore,
    ) -> Result<Config, StoreError> {
        let mut config = self.load_or_create()?;
        if !config.profiles.contains_key(id) {
            return Err(StoreError::UnknownProfile);
        }
        if config.profile_state(id) == Some(ProfileState::Draft) {
            return Err(StoreError::ProfileNotReady(id.to_owned()));
        }
        verify_profile_secret(&config, id, secret_store)?;

        config.active_profile = id.to_owned();
        config.validate()?;
        self.write_atomically(&config)?;
        Ok(config)
    }

    pub fn import_draft_profiles(
        &self,
        portable_profiles: &[PortableProfile],
    ) -> Result<Config, StoreError> {
        let mut config = self.load_or_create()?;

        for portable in portable_profiles {
            if config.profiles.contains_key(&portable.id) {
                return Err(StoreError::ProfileAlreadyExists(portable.id.clone()));
            }
            if config.prompts.contains_key(&portable.id) {
                return Err(StoreError::PromptAlreadyExists(portable.id.clone()));
            }
        }

        for portable in portable_profiles {
            config.prompts.insert(
                portable.id.clone(),
                Prompt {
                    name: portable.name.clone(),
                    system: portable.system_prompt.clone(),
                },
            );
            config.profiles.insert(
                portable.id.clone(),
                Profile {
                    name: portable.name.clone(),
                    provider: None,
                    model: None,
                    prompt: Some(portable.id.clone()),
                    max_input_chars: portable.limits.max_input_chars,
                    max_output_tokens: portable.limits.max_output_tokens,
                    timeout_ms: portable.limits.timeout_ms,
                    output_policy: portable.output_policy.clone(),
                },
            );
        }

        config.validate()?;
        self.write_atomically(&config)?;
        Ok(config)
    }

    pub fn create_draft_profile(&self, id: &str, name: &str) -> Result<Config, StoreError> {
        let mut config = self.load_or_create()?;
        ensure_new_profile_id(&config, id)?;
        ensure_profile_name_available(&config, name, None)?;
        config.prompts.insert(
            id.to_owned(),
            Prompt {
                name: name.to_owned(),
                system: "Configure this prompt before activating the profile.".to_owned(),
            },
        );
        config.profiles.insert(
            id.to_owned(),
            Profile {
                name: name.to_owned(),
                provider: None,
                model: None,
                prompt: Some(id.to_owned()),
                max_input_chars: crate::config::DEFAULT_MAX_INPUT_CHARS,
                max_output_tokens: crate::config::DEFAULT_MAX_OUTPUT_TOKENS,
                timeout_ms: crate::config::DEFAULT_TIMEOUT_MS,
                output_policy: Default::default(),
            },
        );
        config.validate()?;
        self.write_atomically(&config)?;
        Ok(config)
    }

    pub fn duplicate_profile(
        &self,
        source_id: &str,
        target_id: &str,
        name: &str,
    ) -> Result<Config, StoreError> {
        let mut config = self.load_or_create()?;
        ensure_new_profile_id(&config, target_id)?;
        ensure_profile_name_available(&config, name, None)?;
        let mut duplicate = config
            .profiles
            .get(source_id)
            .cloned()
            .ok_or(StoreError::UnknownProfile)?;
        duplicate.name = name.to_owned();
        if let Some(source_prompt_id) = &duplicate.prompt {
            let source_prompt = config
                .prompts
                .get(source_prompt_id)
                .cloned()
                .ok_or(StoreError::InvalidProfileReference)?;
            if config.prompts.contains_key(target_id) {
                return Err(StoreError::PromptAlreadyExists(target_id.to_owned()));
            }
            config.prompts.insert(
                target_id.to_owned(),
                Prompt {
                    name: name.to_owned(),
                    system: source_prompt.system,
                },
            );
            duplicate.prompt = Some(target_id.to_owned());
        }
        config.profiles.insert(target_id.to_owned(), duplicate);
        config.validate()?;
        self.write_atomically(&config)?;
        Ok(config)
    }

    pub fn rename_profile(&self, id: &str, name: &str) -> Result<Config, StoreError> {
        let mut config = self.load_or_create()?;
        ensure_profile_name_available(&config, name, Some(id))?;
        let profile = config
            .profiles
            .get_mut(id)
            .ok_or(StoreError::UnknownProfile)?;
        profile.name = name.to_owned();
        config.validate()?;
        self.write_atomically(&config)?;
        Ok(config)
    }

    pub fn delete_profile(&self, id: &str) -> Result<Config, StoreError> {
        if id == RAW_PROFILE_ID {
            return Err(StoreError::ProtectedRawProfile);
        }
        let mut config = self.load_or_create()?;
        let removed = config
            .profiles
            .remove(id)
            .ok_or(StoreError::UnknownProfile)?;
        if config.active_profile == id {
            config.active_profile = RAW_PROFILE_ID.to_owned();
        }
        if let Some(prompt_id) = removed.prompt
            && !config
                .profiles
                .values()
                .any(|profile| profile.prompt.as_deref() == Some(&prompt_id))
        {
            config.prompts.remove(&prompt_id);
        }
        config.validate()?;
        self.write_atomically(&config)?;
        Ok(config)
    }

    pub fn update_prompt(&self, id: &str, system: &str) -> Result<Config, StoreError> {
        let mut config = self.load_or_create()?;
        let prompt = config
            .prompts
            .get_mut(id)
            .ok_or(StoreError::UnknownPrompt)?;
        prompt.system = system.to_owned();
        config.validate()?;
        self.write_atomically(&config)?;
        Ok(config)
    }

    fn load_source(&self, source: &str) -> Result<Config, StoreError> {
        let schema_version = read_schema_version(source)?;
        match schema_version {
            CURRENT_SCHEMA_VERSION => {
                let config = Config::parse(source)?;
                config.validate()?;
                Ok(config)
            }
            version if version < CURRENT_SCHEMA_VERSION => {
                let migrated = migrate_to_current(source, version)?;
                migrated.validate()?;
                self.back_up_source(schema_version, source)?;
                self.write_atomically(&migrated)?;
                Ok(migrated)
            }
            version => Err(StoreError::UnsupportedSchemaVersion(version)),
        }
    }

    fn back_up_source(&self, schema_version: u32, source: &str) -> Result<(), StoreError> {
        let path = self
            .paths
            .backup_dir
            .join(format!("config.schema-{schema_version}.toml"));
        let mut backup = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|source| StoreError::CreateBackup {
                path: path.clone(),
                source,
            })?;
        set_private_file_permissions(&path)?;
        backup
            .write_all(source.as_bytes())
            .map_err(|source| StoreError::WriteBackup {
                path: path.clone(),
                source,
            })?;
        backup
            .sync_all()
            .map_err(|source| StoreError::WriteBackup { path, source })?;
        Ok(())
    }

    fn write_atomically(&self, config: &Config) -> Result<(), StoreError> {
        let parent = self
            .paths
            .config_file
            .parent()
            .ok_or_else(|| StoreError::InvalidConfigPath(self.paths.config_file.clone()))?;
        let source = toml::to_string_pretty(config)?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        set_private_file_permissions(temporary.path())?;
        temporary.write_all(source.as_bytes())?;
        temporary.as_file().sync_all()?;
        temporary
            .persist(&self.paths.config_file)
            .map_err(|error| StoreError::PersistConfig {
                path: self.paths.config_file.clone(),
                source: error.error,
            })?;
        set_private_file_permissions(&self.paths.config_file)?;
        Ok(())
    }
}

fn ensure_new_profile_id(config: &Config, id: &str) -> Result<(), StoreError> {
    if id == RAW_PROFILE_ID {
        return Err(StoreError::ProtectedRawProfile);
    }
    if config.profiles.contains_key(id) {
        return Err(StoreError::ProfileAlreadyExists(id.to_owned()));
    }
    if config.prompts.contains_key(id) {
        return Err(StoreError::PromptAlreadyExists(id.to_owned()));
    }
    Ok(())
}

fn ensure_profile_name_available(
    config: &Config,
    name: &str,
    excluded_id: Option<&str>,
) -> Result<(), StoreError> {
    let normalized = name.trim().to_lowercase();
    if normalized.is_empty() {
        return Err(StoreError::InvalidProfileName);
    }
    if config.profiles.iter().any(|(id, profile)| {
        Some(id.as_str()) != excluded_id && profile.name.trim().to_lowercase() == normalized
    }) {
        return Err(StoreError::ProfileNameAlreadyExists(name.trim().to_owned()));
    }
    Ok(())
}

fn verify_profile_secret(
    config: &Config,
    profile_id: &str,
    secret_store: &impl SecretStore,
) -> Result<(), StoreError> {
    let profile = config
        .profiles
        .get(profile_id)
        .expect("existing profile should be available");
    let Some(provider_id) = &profile.provider else {
        return Ok(());
    };
    let provider = config
        .providers
        .get(provider_id)
        .expect("ready profile should reference an existing provider");
    if !provider.kind.requires_secret() {
        return Ok(());
    }
    let reference = provider
        .secret_ref
        .as_deref()
        .expect("ready remote provider should have a secret reference");
    let reference = SecretRef::from_config(reference)
        .expect("ready remote provider should have a valid secret reference");
    secret_store
        .verify(&reference)
        .map_err(StoreError::SecretVerification)
}

fn read_schema_version(source: &str) -> Result<u32, StoreError> {
    #[derive(serde::Deserialize)]
    struct SchemaHeader {
        schema_version: u32,
    }

    let value: toml::Value = toml::from_str(source)?;
    let header: SchemaHeader = value.try_into()?;
    Ok(header.schema_version)
}

fn migrate_to_current(source: &str, starting_version: u32) -> Result<Config, StoreError> {
    let mut config = Config::parse(source)?;
    let mut version = starting_version;

    while version < CURRENT_SCHEMA_VERSION {
        config = match version {
            0 => migrate_v0_to_v1(config),
            unsupported => return Err(StoreError::UnsupportedSchemaVersion(unsupported)),
        };
        version = config.schema_version;
    }

    Ok(config)
}

fn migrate_v0_to_v1(mut config: Config) -> Config {
    config.schema_version = 1;
    config
}

#[cfg(unix)]
fn set_private_file_permissions(path: &Path) -> Result<(), io::Error> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn set_private_file_permissions(_: &Path) -> Result<(), io::Error> {
    Ok(())
}

#[derive(Debug)]
pub enum StoreError {
    Paths(crate::paths::PathError),
    Io(io::Error),
    ReadConfig { path: PathBuf, source: io::Error },
    ParseToml(toml::de::Error),
    SerializeToml(toml::ser::Error),
    InvalidConfig(crate::config::ValidationErrors),
    UnsupportedSchemaVersion(u32),
    UnknownProfile,
    ProfileNotReady(String),
    ProfileAlreadyExists(String),
    PromptAlreadyExists(String),
    ProtectedRawProfile,
    InvalidProfileReference,
    InvalidProfileName,
    ProfileNameAlreadyExists(String),
    UnknownPrompt,
    SecretVerification(SecretError),
    InvalidConfigPath(PathBuf),
    CreateBackup { path: PathBuf, source: io::Error },
    WriteBackup { path: PathBuf, source: io::Error },
    PersistConfig { path: PathBuf, source: io::Error },
}

impl From<crate::paths::PathError> for StoreError {
    fn from(value: crate::paths::PathError) -> Self {
        Self::Paths(value)
    }
}

impl From<io::Error> for StoreError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<toml::de::Error> for StoreError {
    fn from(value: toml::de::Error) -> Self {
        Self::ParseToml(value)
    }
}

impl From<toml::ser::Error> for StoreError {
    fn from(value: toml::ser::Error) -> Self {
        Self::SerializeToml(value)
    }
}

impl From<crate::config::ValidationErrors> for StoreError {
    fn from(value: crate::config::ValidationErrors) -> Self {
        Self::InvalidConfig(value)
    }
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Paths(error) => write!(formatter, "could not resolve application paths: {error}"),
            Self::Io(error) => write!(formatter, "file operation failed: {error}"),
            Self::ReadConfig { path, source } => {
                write!(
                    formatter,
                    "could not read configuration '{}': {source}",
                    path.display()
                )
            }
            Self::ParseToml(error) => write!(formatter, "could not parse configuration: {error}"),
            Self::SerializeToml(error) => {
                write!(formatter, "could not serialize configuration: {error}")
            }
            Self::InvalidConfig(error) => write!(formatter, "{error}"),
            Self::UnsupportedSchemaVersion(version) => {
                write!(formatter, "schema version {version} is unsupported")
            }
            Self::UnknownProfile => write!(formatter, "the requested profile does not exist"),
            Self::ProfileNotReady(id) => {
                write!(
                    formatter,
                    "profile '{id}' is a draft and cannot be activated"
                )
            }
            Self::ProfileAlreadyExists(id) => {
                write!(formatter, "profile '{id}' already exists")
            }
            Self::PromptAlreadyExists(id) => {
                write!(formatter, "prompt '{id}' already exists")
            }
            Self::ProtectedRawProfile => write!(
                formatter,
                "the mandatory Raw profile cannot be changed this way"
            ),
            Self::InvalidProfileReference => write!(
                formatter,
                "profile configuration has an invalid prompt reference"
            ),
            Self::InvalidProfileName => write!(formatter, "profile name cannot be empty"),
            Self::ProfileNameAlreadyExists(name) => {
                write!(formatter, "profile name '{name}' already exists")
            }
            Self::UnknownPrompt => write!(formatter, "the requested prompt does not exist"),
            Self::SecretVerification(error) => {
                write!(formatter, "could not verify the provider key: {error}")
            }
            Self::InvalidConfigPath(path) => {
                write!(
                    formatter,
                    "configuration path '{}' has no parent directory",
                    path.display()
                )
            }
            Self::CreateBackup { path, source } => {
                write!(
                    formatter,
                    "could not create migration backup '{}': {source}",
                    path.display()
                )
            }
            Self::WriteBackup { path, source } => {
                write!(
                    formatter,
                    "could not write migration backup '{}': {source}",
                    path.display()
                )
            }
            Self::PersistConfig { path, source } => {
                write!(
                    formatter,
                    "could not replace configuration '{}': {source}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for StoreError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        DEFAULT_MAX_INPUT_CHARS, DEFAULT_MAX_OUTPUT_TOKENS, DEFAULT_TIMEOUT_MS, Provider,
        ProviderKind, RAW_PROFILE_ID,
    };
    use crate::secrets::InMemorySecretStore;

    fn test_store() -> (tempfile::TempDir, ConfigStore) {
        let temporary_root = tempfile::tempdir().expect("temporary directory should exist");
        let root = temporary_root.path();
        let paths = AppPaths {
            config_dir: root.join("config"),
            config_file: root.join("config/config.toml"),
            state_dir: root.join("state"),
            engine_dir: root.join("data/bin"),
            temporary_dir: root.join("cache/downloads"),
            backup_dir: root.join("state/backups"),
            log_dir: root.join("state/logs"),
        };
        (temporary_root, ConfigStore::new(paths))
    }

    #[test]
    fn creates_and_persists_defaults_on_first_use() {
        let (_root, store) = test_store();

        let config = store.load_or_create().expect("defaults should be created");

        assert_eq!(config.active_profile, RAW_PROFILE_ID);
        assert!(store.config_path().is_file());
        let persisted =
            fs::read_to_string(store.config_path()).expect("configuration should be readable");
        assert!(persisted.contains("schema_version = 1"));
    }

    #[test]
    fn example_profile_is_persisted_but_remains_unconfigured() {
        let (_root, store) = test_store();
        let config = store.load_or_create().expect("defaults should be created");

        assert!(config.profiles.contains_key("example"));
        assert_eq!(config.active_profile, RAW_PROFILE_ID);
    }

    #[test]
    fn profile_crud_preserves_raw_and_cleans_an_unshared_prompt() {
        let (_root, store) = test_store();
        store
            .create_draft_profile("notes", "Notes")
            .expect("a draft should be created");
        let renamed = store
            .rename_profile("notes", "Meeting notes")
            .expect("a profile should be renamed");
        assert_eq!(renamed.profiles["notes"].name, "Meeting notes");

        let duplicated = store
            .duplicate_profile("example", "example-copy", "Example copy")
            .expect("a profile and its prompt should be duplicated");
        assert_eq!(
            duplicated.profiles["example-copy"].prompt.as_deref(),
            Some("example-copy")
        );
        assert!(duplicated.prompts.contains_key("example-copy"));

        let deleted = store
            .delete_profile("example-copy")
            .expect("the duplicate should be deleted");
        assert!(!deleted.profiles.contains_key("example-copy"));
        assert!(!deleted.prompts.contains_key("example-copy"));
        assert!(matches!(
            store.delete_profile(RAW_PROFILE_ID),
            Err(StoreError::ProtectedRawProfile)
        ));
    }

    #[test]
    fn profile_names_are_unique_without_blocking_legacy_duplicate_cleanup() {
        let (_root, store) = test_store();
        assert!(matches!(
            store.create_draft_profile("raw-2", "Raw"),
            Err(StoreError::ProfileNameAlreadyExists(name)) if name == "Raw"
        ));

        let mut legacy = Config::defaults();
        legacy.prompts.insert(
            "raw-2".to_owned(),
            Prompt {
                name: "Raw".to_owned(),
                system: "Legacy prompt.".to_owned(),
            },
        );
        legacy.profiles.insert(
            "raw-2".to_owned(),
            Profile {
                name: "Raw".to_owned(),
                provider: None,
                model: None,
                prompt: Some("raw-2".to_owned()),
                max_input_chars: DEFAULT_MAX_INPUT_CHARS,
                max_output_tokens: DEFAULT_MAX_OUTPUT_TOKENS,
                timeout_ms: DEFAULT_TIMEOUT_MS,
                output_policy: Default::default(),
            },
        );
        store
            .paths
            .ensure_private_directories()
            .expect("directories exist");
        store
            .write_atomically(&legacy)
            .expect("legacy fixture writes");
        store
            .delete_profile("raw-2")
            .expect("legacy duplicate remains removable");
    }

    #[test]
    fn unknown_profile_does_not_replace_valid_configuration() {
        let (_root, store) = test_store();
        store.load_or_create().expect("defaults should be created");
        let original =
            fs::read_to_string(store.config_path()).expect("configuration should be readable");

        let error = store
            .set_active_profile("missing")
            .expect_err("unknown profile must fail");

        assert!(matches!(error, StoreError::UnknownProfile));
        let current =
            fs::read_to_string(store.config_path()).expect("configuration should remain readable");
        assert_eq!(current, original);
    }

    #[test]
    fn draft_profile_does_not_replace_valid_configuration() {
        let (_root, store) = test_store();
        store.load_or_create().expect("defaults should be created");
        let original =
            fs::read_to_string(store.config_path()).expect("configuration should be readable");

        let error = store
            .set_active_profile("example")
            .expect_err("draft profile activation must fail");

        assert!(matches!(error, StoreError::ProfileNotReady(id) if id == "example"));
        let current =
            fs::read_to_string(store.config_path()).expect("configuration should remain readable");
        assert_eq!(current, original);
    }

    #[test]
    fn imports_a_portable_profile_as_a_draft() {
        let (_root, store) = test_store();
        let portable = crate::catalog::parse_portable_profile(include_str!(
            "../tests/fixtures/catalog/duplicate-a.md"
        ))
        .expect("fixture should parse");

        let imported = store
            .import_draft_profiles(&[portable])
            .expect("portable profile should import");

        assert_eq!(imported.active_profile, RAW_PROFILE_ID);
        assert_eq!(
            imported.profile_state("duplicate"),
            Some(ProfileState::Draft)
        );
        assert_eq!(
            imported
                .prompts
                .get("duplicate")
                .expect("imported prompt should exist")
                .name,
            "Duplicate A"
        );
    }

    #[test]
    fn failed_import_preserves_the_existing_configuration() {
        let (_root, store) = test_store();
        let portable = crate::catalog::parse_portable_profile(include_str!(
            "../tests/fixtures/catalog/duplicate-a.md"
        ))
        .expect("fixture should parse");
        store
            .import_draft_profiles(&[portable.clone()])
            .expect("first import should succeed");
        let original =
            fs::read_to_string(store.config_path()).expect("configuration should be readable");

        let error = store
            .import_draft_profiles(&[portable])
            .expect_err("duplicate import should fail");

        assert!(matches!(error, StoreError::ProfileAlreadyExists(id) if id == "duplicate"));
        let current =
            fs::read_to_string(store.config_path()).expect("configuration should be readable");
        assert_eq!(current, original);
    }

    #[test]
    fn remote_profile_activation_requires_a_verifiable_key_without_replacing_configuration() {
        let (_root, store) = test_store();
        let mut config = Config::defaults();
        config.providers.insert(
            "remote".to_owned(),
            Provider {
                kind: ProviderKind::Openai,
                endpoint: Some("https://example.invalid".to_owned()),
                secret_ref: Some("org.voxtype-personas/provider/remote".to_owned()),
                timeout_ms: Some(DEFAULT_TIMEOUT_MS),
                models: vec!["test-model".to_owned()],
            },
        );
        let example = config
            .profiles
            .get_mut("example")
            .expect("example profile should exist");
        example.provider = Some("remote".to_owned());
        example.model = Some("test-model".to_owned());
        store
            .paths
            .ensure_private_directories()
            .expect("directories should exist");
        store
            .write_atomically(&config)
            .expect("fixture configuration should be written");
        let original =
            fs::read_to_string(store.config_path()).expect("configuration should be readable");

        let missing_store = InMemorySecretStore::default();
        let error = store
            .set_active_profile_with_secret_store("example", &missing_store)
            .expect_err("missing provider key must reject activation");

        assert!(matches!(
            error,
            StoreError::SecretVerification(SecretError::NotFound)
        ));
        assert!(
            !error
                .to_string()
                .contains("org.voxtype-personas/provider/remote")
        );
        let current =
            fs::read_to_string(store.config_path()).expect("configuration should be readable");
        assert_eq!(current, original);
    }

    #[test]
    fn remote_profile_activation_succeeds_when_the_key_verifies() {
        let (_root, store) = test_store();
        let mut config = Config::defaults();
        config.providers.insert(
            "remote".to_owned(),
            Provider {
                kind: ProviderKind::Openai,
                endpoint: Some("https://example.invalid".to_owned()),
                secret_ref: Some("org.voxtype-personas/provider/remote".to_owned()),
                timeout_ms: Some(DEFAULT_TIMEOUT_MS),
                models: vec!["test-model".to_owned()],
            },
        );
        let example = config
            .profiles
            .get_mut("example")
            .expect("example profile should exist");
        example.provider = Some("remote".to_owned());
        example.model = Some("test-model".to_owned());
        store
            .paths
            .ensure_private_directories()
            .expect("directories should exist");
        store
            .write_atomically(&config)
            .expect("fixture configuration should be written");
        let reference = SecretRef::for_provider("remote").expect("reference should be valid");
        let mut secret_store = InMemorySecretStore::default();
        secret_store
            .write(reference, b"test-key")
            .expect("test key should be stored");

        let activated = store
            .set_active_profile_with_secret_store("example", &secret_store)
            .expect("verified provider key should allow activation");

        assert_eq!(activated.active_profile, "example");
    }

    #[test]
    fn malformed_configuration_is_not_replaced() {
        let (_root, store) = test_store();
        store
            .paths
            .ensure_private_directories()
            .expect("directories should exist");
        let malformed = "schema_version = [";
        fs::write(store.config_path(), malformed).expect("malformed fixture should be written");

        assert!(matches!(
            store.load_or_create(),
            Err(StoreError::ParseToml(_))
        ));
        let current =
            fs::read_to_string(store.config_path()).expect("configuration should remain readable");
        assert_eq!(current, malformed);
    }

    #[test]
    fn migrates_v0_after_creating_a_backup() {
        let (_root, store) = test_store();
        store
            .paths
            .ensure_private_directories()
            .expect("directories should exist");
        let legacy = toml::to_string_pretty(&Config::defaults())
            .expect("defaults should serialize")
            .replacen("schema_version = 1", "schema_version = 0", 1);
        fs::write(store.config_path(), &legacy).expect("legacy fixture should be written");

        let migrated = store
            .load_or_create()
            .expect("legacy configuration should migrate");

        assert_eq!(migrated.schema_version, CURRENT_SCHEMA_VERSION);
        let backup = store.paths.backup_dir.join("config.schema-0.toml");
        assert_eq!(
            fs::read_to_string(backup).expect("backup should exist"),
            legacy
        );
        let current =
            fs::read_to_string(store.config_path()).expect("configuration should be readable");
        assert!(current.contains("schema_version = 1"));
    }

    #[cfg(unix)]
    #[test]
    fn configuration_file_uses_owner_only_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let (_root, store) = test_store();
        store.load_or_create().expect("defaults should be created");
        let mode = fs::metadata(store.config_path())
            .expect("configuration should exist")
            .permissions()
            .mode()
            & 0o777;

        assert_eq!(mode, 0o600);
    }
}

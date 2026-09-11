use crate::config::{CURRENT_SCHEMA_VERSION, Config};
use crate::paths::AppPaths;
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
        let mut config = self.load_or_create()?;
        if !config.profiles.contains_key(id) {
            return Err(StoreError::UnknownProfile);
        }

        config.active_profile = id.to_owned();
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
    use crate::config::RAW_PROFILE_ID;

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
    fn active_profile_selection_is_persisted() {
        let (_root, store) = test_store();
        store.load_or_create().expect("defaults should be created");

        store
            .set_active_profile("chat")
            .expect("known profile should be selected");
        let reloaded = store.load_or_create().expect("configuration should reload");

        assert_eq!(reloaded.active_profile, "chat");
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

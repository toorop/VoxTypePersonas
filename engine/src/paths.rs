use std::env;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppPaths {
    pub config_dir: PathBuf,
    pub config_file: PathBuf,
    pub state_dir: PathBuf,
    pub engine_dir: PathBuf,
    pub temporary_dir: PathBuf,
    pub backup_dir: PathBuf,
    pub log_dir: PathBuf,
}

impl AppPaths {
    pub fn discover() -> Result<Self, PathError> {
        Self::from_environment(|key| env::var_os(key))
    }

    fn from_environment<F>(getenv: F) -> Result<Self, PathError>
    where
        F: Fn(&str) -> Option<OsString>,
    {
        let home = getenv("HOME").ok_or(PathError::MissingHomeDirectory)?;
        let config_home = getenv("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(&home).join(".config"));
        let data_home = getenv("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(&home).join(".local/share"));
        let state_home = getenv("XDG_STATE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(&home).join(".local/state"));
        let cache_home = getenv("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(&home).join(".cache"));

        for (name, path) in [
            ("XDG_CONFIG_HOME", &config_home),
            ("XDG_DATA_HOME", &data_home),
            ("XDG_STATE_HOME", &state_home),
            ("XDG_CACHE_HOME", &cache_home),
        ] {
            if !path.is_absolute() {
                return Err(PathError::NonAbsoluteDirectory(name.to_owned()));
            }
        }

        let config_dir = config_home.join("voxtype-personas");
        let state_dir = state_home.join("voxtype-personas");

        Ok(Self {
            config_file: config_dir.join("config.toml"),
            config_dir,
            engine_dir: data_home.join("voxtype-personas/bin"),
            temporary_dir: cache_home.join("voxtype-personas/downloads"),
            backup_dir: state_dir.join("backups"),
            log_dir: state_dir.join("logs"),
            state_dir,
        })
    }

    pub fn ensure_private_directories(&self) -> Result<(), io::Error> {
        for directory in [
            &self.config_dir,
            &self.state_dir,
            &self.engine_dir,
            &self.temporary_dir,
            &self.backup_dir,
            &self.log_dir,
        ] {
            fs::create_dir_all(directory)?;
            set_private_permissions(directory)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathError {
    MissingHomeDirectory,
    NonAbsoluteDirectory(String),
}

impl std::fmt::Display for PathError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingHomeDirectory => {
                write!(formatter, "could not determine the home directory")
            }
            Self::NonAbsoluteDirectory(variable) => {
                write!(formatter, "{variable} must be an absolute path")
            }
        }
    }
}

impl std::error::Error for PathError {}

#[cfg(unix)]
fn set_private_permissions(path: &PathBuf) -> Result<(), io::Error> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn set_private_permissions(_: &PathBuf) -> Result<(), io::Error> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_environment(key: &str) -> Option<OsString> {
        match key {
            "HOME" => Some(OsString::from("/home/tester")),
            "XDG_CONFIG_HOME" => Some(OsString::from("/tmp/config")),
            "XDG_DATA_HOME" => Some(OsString::from("/tmp/data")),
            "XDG_STATE_HOME" => Some(OsString::from("/tmp/state")),
            "XDG_CACHE_HOME" => Some(OsString::from("/tmp/cache")),
            _ => None,
        }
    }

    #[test]
    fn discovers_all_xdg_locations() {
        let paths = AppPaths::from_environment(test_environment).expect("paths should resolve");

        assert_eq!(
            paths.config_file,
            PathBuf::from("/tmp/config/voxtype-personas/config.toml")
        );
        assert_eq!(
            paths.engine_dir,
            PathBuf::from("/tmp/data/voxtype-personas/bin")
        );
        assert_eq!(
            paths.backup_dir,
            PathBuf::from("/tmp/state/voxtype-personas/backups")
        );
        assert_eq!(
            paths.log_dir,
            PathBuf::from("/tmp/state/voxtype-personas/logs")
        );
        assert_eq!(
            paths.temporary_dir,
            PathBuf::from("/tmp/cache/voxtype-personas/downloads")
        );
    }

    #[test]
    fn rejects_relative_xdg_directories() {
        let error = AppPaths::from_environment(|key| match key {
            "HOME" => Some(OsString::from("/home/tester")),
            "XDG_CONFIG_HOME" => Some(OsString::from("relative")),
            _ => None,
        })
        .expect_err("relative XDG path must fail");

        assert_eq!(
            error,
            PathError::NonAbsoluteDirectory("XDG_CONFIG_HOME".to_owned())
        );
    }

    #[test]
    fn creates_private_directories() {
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

        paths
            .ensure_private_directories()
            .expect("directories should be created");

        assert!(paths.config_dir.is_dir());
        assert!(paths.engine_dir.is_dir());
        assert!(paths.backup_dir.is_dir());
    }

    #[cfg(unix)]
    #[test]
    fn private_directories_use_owner_only_permissions() {
        use std::os::unix::fs::PermissionsExt;

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

        paths
            .ensure_private_directories()
            .expect("directories should be created");

        let mode = fs::metadata(&paths.config_dir)
            .expect("configuration directory should exist")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o700);
    }
}

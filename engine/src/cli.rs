use crate::config::Config;
use crate::storage::ConfigStore;
use clap::{Args, Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Parser)]
#[command(name = "voxtype-personas", version = VERSION, about = "Persona-aware post-processing for Voxtype")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Version,
    Process(ProcessArgs),
    Profiles(ProfilesArgs),
    Providers(ProvidersArgs),
    Config(ConfigArgs),
}

#[derive(Debug, Args)]
struct ProcessArgs {
    #[arg(long)]
    profile: Option<String>,
}

#[derive(Debug, Args)]
struct ProfilesArgs {
    #[command(subcommand)]
    command: ProfilesCommand,
}

#[derive(Debug, Subcommand)]
enum ProfilesCommand {
    List,
    SetActive { id: String },
}

#[derive(Debug, Args)]
struct ProvidersArgs {
    #[command(subcommand)]
    command: ProvidersCommand,
}

#[derive(Debug, Subcommand)]
enum ProvidersCommand {
    Test { id: String },
}

#[derive(Debug, Args)]
struct ConfigArgs {
    #[command(subcommand)]
    command: ConfigCommand,
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    Validate(ValidateArgs),
}

#[derive(Debug, Args)]
struct ValidateArgs {
    #[arg(long, conflicts_with = "defaults")]
    file: Option<PathBuf>,
    #[arg(long)]
    defaults: bool,
}

pub fn run() -> Result<(), CliError> {
    run_from(Cli::parse())
}

fn run_from(cli: Cli) -> Result<(), CliError> {
    match cli.command {
        Command::Version => {
            println!("{VERSION}");
            Ok(())
        }
        Command::Config(ConfigArgs {
            command: ConfigCommand::Validate(args),
        }) => validate(args),
        Command::Process(args) => Err(CliError::NotImplemented(format!(
            "process{} is not available until Phase 3",
            args.profile
                .as_deref()
                .map(|profile| format!(" for profile '{profile}'"))
                .unwrap_or_default()
        ))),
        Command::Profiles(ProfilesArgs { command }) => profiles(command),
        Command::Providers(ProvidersArgs {
            command: ProvidersCommand::Test { id },
        }) => Err(CliError::NotImplemented(format!(
            "providers test for '{id}' is not available until Phase 6"
        ))),
    }
}

fn validate(args: ValidateArgs) -> Result<(), CliError> {
    let config = match args.file {
        Some(path) => {
            let text = fs::read_to_string(&path)
                .map_err(|source| CliError::ReadConfig { path, source })?;
            Config::parse(&text).map_err(CliError::ParseConfig)?
        }
        None if args.defaults => Config::defaults(),
        None => ConfigStore::discover()
            .and_then(|store| store.load_or_create())
            .map_err(CliError::Store)?,
    };

    config.validate().map_err(CliError::InvalidConfig)?;
    if args.defaults {
        println!("Built-in default configuration is valid.");
    } else {
        println!("Configuration is valid.");
    }
    Ok(())
}

fn profiles(command: ProfilesCommand) -> Result<(), CliError> {
    let store = ConfigStore::discover().map_err(CliError::Store)?;
    match command {
        ProfilesCommand::List => {
            let config = store.load_or_create().map_err(CliError::Store)?;
            for (id, profile) in config.profiles {
                let marker = if id == config.active_profile {
                    "*"
                } else {
                    " "
                };
                println!("{marker} {id}\t{}", profile.name);
            }
            Ok(())
        }
        ProfilesCommand::SetActive { id } => {
            let config = store.set_active_profile(&id).map_err(CliError::Store)?;
            println!("Active profile set to '{}'.", config.active_profile);
            Ok(())
        }
    }
}

#[derive(Debug)]
pub enum CliError {
    NotImplemented(String),
    ReadConfig {
        path: PathBuf,
        source: std::io::Error,
    },
    ParseConfig(toml::de::Error),
    InvalidConfig(crate::config::ValidationErrors),
    Store(crate::storage::StoreError),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplemented(message) => write!(formatter, "{message}"),
            Self::ReadConfig { path, source } => {
                write!(
                    formatter,
                    "could not read configuration file '{}': {source}",
                    path.display()
                )
            }
            Self::ParseConfig(error) => write!(formatter, "could not parse configuration: {error}"),
            Self::InvalidConfig(error) => write!(formatter, "{error}"),
            Self::Store(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for CliError {}

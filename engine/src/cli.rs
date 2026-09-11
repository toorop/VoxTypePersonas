use crate::config::Config;
use crate::storage::ConfigStore;
use clap::{Args, Parser, Subcommand};
use std::fs;
use std::io::{self, Read, Write};
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
        Command::Process(args) => process(args),
        Command::Profiles(ProfilesArgs { command }) => profiles(command),
        Command::Providers(ProvidersArgs {
            command: ProvidersCommand::Test { id },
        }) => Err(CliError::NotImplemented(format!(
            "providers test for '{id}' is not available until Phase 6"
        ))),
    }
}

fn process(args: ProcessArgs) -> Result<(), CliError> {
    let mut transcription = Vec::new();
    io::stdin()
        .read_to_end(&mut transcription)
        .map_err(CliError::ReadInput)?;

    let store = match ConfigStore::discover() {
        Ok(store) => store,
        Err(error) => return preserve_raw_and_fail(&transcription, CliError::Store(error)),
    };
    let config = match store.load_or_create() {
        Ok(config) => config,
        Err(error) => return preserve_raw_and_fail(&transcription, CliError::Store(error)),
    };
    let profile_id = args.profile.unwrap_or(config.active_profile);
    let profile = match config.profiles.get(&profile_id) {
        Some(profile) => profile,
        None => return preserve_raw_and_fail(&transcription, CliError::UnknownProfile),
    };

    if profile_id == crate::config::RAW_PROFILE_ID {
        return write_pasteable_output(&transcription);
    }

    preserve_raw_and_fail(
        &transcription,
        CliError::NotImplemented(format!(
            "profile '{}' is not available until provider processing is implemented",
            profile.name
        )),
    )
}

fn preserve_raw_and_fail(transcription: &[u8], error: CliError) -> Result<(), CliError> {
    write_pasteable_output(transcription)?;
    Err(error)
}

fn write_pasteable_output(output: &[u8]) -> Result<(), CliError> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(output).map_err(CliError::WriteOutput)?;
    stdout.flush().map_err(CliError::WriteOutput)
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
    ReadInput(io::Error),
    WriteOutput(io::Error),
    UnknownProfile,
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
            Self::ReadInput(error) => write!(formatter, "could not read standard input: {error}"),
            Self::WriteOutput(error) => {
                write!(formatter, "could not write standard output: {error}")
            }
            Self::UnknownProfile => write!(formatter, "the requested profile does not exist"),
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

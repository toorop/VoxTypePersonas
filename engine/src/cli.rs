use crate::config::Config;
use crate::processing::{ProcessingOutput, ProviderFailure, finalize_provider_response};
use crate::providers::{
    AnthropicAdapter, GeminiAdapter, ProviderAdapter, ProviderRequest, ReqwestTransport,
    compatible_from_provider, ollama_from_provider,
};
use crate::secrets::{SecretRef, SecretServiceStore, SecretStore};
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
        }) => provider_test(&id),
    }
}

fn provider_test(id: &str) -> Result<(), CliError> {
    let config = ConfigStore::discover()
        .and_then(|store| store.load_or_create())
        .map_err(CliError::Store)?;
    let provider = config.providers.get(id).ok_or(CliError::UnknownProfile)?;
    let model = provider.models.first().ok_or_else(|| {
        CliError::NotImplemented("provider has no configured model for testing".to_owned())
    })?;
    let adapter = ollama_from_provider(provider, ReqwestTransport)
        .map_err(|_| CliError::NotImplemented("provider kind is not available yet".to_owned()))?;
    adapter
        .test(model, provider.timeout_ms.unwrap_or(30_000))
        .map_err(|_| CliError::NotImplemented("provider test failed".to_owned()))?;
    println!("Provider test succeeded.");
    Ok(())
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
    let profile_id = args
        .profile
        .unwrap_or_else(|| config.active_profile.clone());
    let profile = match config.profiles.get(&profile_id) {
        Some(profile) => profile,
        None => return preserve_raw_and_fail(&transcription, CliError::UnknownProfile),
    };

    if profile_id == crate::config::RAW_PROFILE_ID {
        return write_pasteable_output(&transcription);
    }
    if config.profile_state(&profile_id) == Some(crate::config::ProfileState::Draft) {
        return write_raw_fallback(
            &transcription,
            "voxtype-personas: selected profile is not ready; returned Raw profile output",
        );
    }

    let response = match (&profile.provider, &profile.model, &profile.prompt) {
        (Some(provider_id), Some(model), Some(prompt_id)) => {
            let Some(provider) = config.providers.get(provider_id) else {
                return write_processing_output(finalize_provider_response(
                    &transcription,
                    Err(ProviderFailure::Unavailable),
                    &profile.output_policy,
                ));
            };
            let Some(prompt) = config.prompts.get(prompt_id) else {
                return write_processing_output(finalize_provider_response(
                    &transcription,
                    Err(ProviderFailure::Unavailable),
                    &profile.output_policy,
                ));
            };
            let user_text = String::from_utf8(transcription.clone()).map_err(|_| {
                CliError::NotImplemented("provider processing requires UTF-8 input".to_owned())
            })?;
            let request = ProviderRequest {
                model: model.clone(),
                system_prompt: prompt.system.clone(),
                user_text,
                timeout_ms: profile.timeout_ms,
                max_output_tokens: profile.max_output_tokens,
            };
            if provider.kind == crate::config::ProviderKind::Ollama {
                ollama_from_provider(provider, ReqwestTransport)
                    .and_then(|adapter| adapter.process(&request))
            } else {
                let secret = provider
                    .secret_ref
                    .as_deref()
                    .ok_or(ProviderFailure::Authentication)
                    .and_then(|reference| {
                        SecretRef::from_config(reference)
                            .map_err(|_| ProviderFailure::Authentication)
                    })
                    .and_then(|reference| {
                        SecretServiceStore::new()
                            .read(&reference)
                            .map_err(|_| ProviderFailure::Authentication)
                    })
                    .and_then(|bytes| {
                        String::from_utf8(bytes).map_err(|_| ProviderFailure::Authentication)
                    });
                secret.and_then(|api_key| match provider.kind {
                    crate::config::ProviderKind::Anthropic => {
                        AnthropicAdapter::new(api_key, ReqwestTransport)
                            .and_then(|adapter| adapter.process(&request))
                    }
                    crate::config::ProviderKind::Gemini => {
                        GeminiAdapter::new(api_key, ReqwestTransport)
                            .and_then(|adapter| adapter.process(&request))
                    }
                    _ => compatible_from_provider(provider, api_key, ReqwestTransport)
                        .and_then(|adapter| adapter.process(&request)),
                })
            }
        }
        _ => Err(ProviderFailure::Unavailable),
    };
    let output = finalize_provider_response(&transcription, response, &profile.output_policy);
    write_processing_output(output)
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

fn write_processing_output(output: ProcessingOutput) -> Result<(), CliError> {
    write_pasteable_output(output.output_bytes())?;
    if output.fallback_reason().is_some() {
        eprintln!("voxtype-personas: post-processing was unavailable; returned raw transcription");
    }
    Ok(())
}

fn write_raw_fallback(output: &[u8], diagnostic: &str) -> Result<(), CliError> {
    write_pasteable_output(output)?;
    eprintln!("{diagnostic}");
    Ok(())
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
            for (id, profile) in &config.profiles {
                let state = config
                    .profile_state(id)
                    .expect("listed profiles must have a state");
                let marker = if state == crate::config::ProfileState::Active {
                    "*"
                } else {
                    " "
                };
                println!("{marker} {id}\t{}\t{}", profile.name, state.label());
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

use crate::config::{
    CURRENT_SCHEMA_VERSION, DEFAULT_MAX_INPUT_CHARS, DEFAULT_MAX_OUTPUT_TOKENS, DEFAULT_TIMEOUT_MS,
    OutputPolicy, ProviderKind, RAW_PROFILE_ID,
};
use serde::Deserialize;
use std::fmt;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PortableProfile {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub status: PortableProfileStatus,
    pub provider: PortableProvider,
    pub compatibility: Compatibility,
    pub limits: Limits,
    #[serde(default)]
    pub output_policy: OutputPolicy,
    #[serde(skip)]
    pub system_prompt: String,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PortableProfileStatus {
    Draft,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PortableProvider {
    pub kind: Option<ProviderKind>,
    pub endpoint: Option<String>,
    pub model: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Compatibility {
    #[serde(default)]
    pub model_families: Vec<String>,
    pub notes: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    #[serde(default = "default_max_input_chars")]
    pub max_input_chars: u32,
    #[serde(default = "default_max_output_tokens")]
    pub max_output_tokens: u32,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

const fn default_max_input_chars() -> u32 {
    DEFAULT_MAX_INPUT_CHARS
}

const fn default_max_output_tokens() -> u32 {
    DEFAULT_MAX_OUTPUT_TOKENS
}

const fn default_timeout_ms() -> u64 {
    DEFAULT_TIMEOUT_MS
}

pub fn parse_portable_profile(source: &str) -> Result<PortableProfile, CatalogError> {
    let (front_matter, system_prompt) = split_front_matter(source)?;
    reject_prohibited_content(front_matter, system_prompt)?;

    let mut profile: PortableProfile =
        serde_yaml::from_str(front_matter).map_err(CatalogError::InvalidFrontMatter)?;
    profile.system_prompt = system_prompt.to_owned();
    validate_portable_profile(&profile)?;
    Ok(profile)
}

pub fn parse_portable_profiles<'a>(
    sources: impl IntoIterator<Item = &'a str>,
) -> Result<Vec<PortableProfile>, CatalogError> {
    let profiles = sources
        .into_iter()
        .map(parse_portable_profile)
        .collect::<Result<Vec<_>, _>>()?;
    validate_unique_profile_ids(&profiles)?;
    Ok(profiles)
}

fn split_front_matter(source: &str) -> Result<(&str, &str), CatalogError> {
    let source = source
        .strip_prefix("---\n")
        .ok_or(CatalogError::MissingFrontMatter)?;
    let Some(separator) = source.find("\n---\n") else {
        return Err(CatalogError::MissingFrontMatter);
    };
    let front_matter = &source[..separator];
    let system_prompt = &source[separator + "\n---\n".len()..];
    Ok((front_matter, system_prompt))
}

fn reject_prohibited_content(front_matter: &str, system_prompt: &str) -> Result<(), CatalogError> {
    let combined = format!("{front_matter}\n{system_prompt}").to_ascii_lowercase();
    for prohibited in ["secret_ref", "api_key", "authorization:", "bearer "] {
        if combined.contains(prohibited) {
            return Err(CatalogError::ProhibitedContent(prohibited));
        }
    }
    Ok(())
}

fn validate_portable_profile(profile: &PortableProfile) -> Result<(), CatalogError> {
    if profile.schema_version != CURRENT_SCHEMA_VERSION {
        return Err(CatalogError::UnsupportedSchemaVersion(
            profile.schema_version,
        ));
    }
    if !is_valid_identifier(&profile.id) {
        return Err(CatalogError::InvalidField("id"));
    }
    if profile.id == RAW_PROFILE_ID {
        return Err(CatalogError::ReservedProfileId);
    }
    if profile.name.trim().is_empty() {
        return Err(CatalogError::InvalidField("name"));
    }
    if profile.system_prompt.trim().is_empty() {
        return Err(CatalogError::InvalidField("system prompt"));
    }
    if profile.compatibility.notes.trim().is_empty() {
        return Err(CatalogError::InvalidField("compatibility notes"));
    }
    if profile.limits.max_input_chars == 0
        || profile.limits.max_output_tokens == 0
        || profile.limits.timeout_ms == 0
    {
        return Err(CatalogError::InvalidField("execution limits"));
    }
    if profile
        .compatibility
        .model_families
        .iter()
        .any(|family| family.trim().is_empty())
    {
        return Err(CatalogError::InvalidField("model family"));
    }
    if let Some(endpoint) = &profile.provider.endpoint {
        if !(endpoint.starts_with("https://") || endpoint.starts_with("http://")) {
            return Err(CatalogError::InvalidField("provider endpoint"));
        }
    }
    if profile
        .provider
        .model
        .as_deref()
        .is_some_and(|model| model.trim().is_empty())
    {
        return Err(CatalogError::InvalidField("provider model"));
    }
    Ok(())
}

fn validate_unique_profile_ids(profiles: &[PortableProfile]) -> Result<(), CatalogError> {
    let mut ids = std::collections::BTreeSet::new();
    for profile in profiles {
        if !ids.insert(&profile.id) {
            return Err(CatalogError::DuplicateProfileId(profile.id.clone()));
        }
    }
    Ok(())
}

fn is_valid_identifier(id: &str) -> bool {
    !id.is_empty()
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

#[derive(Debug)]
pub enum CatalogError {
    MissingFrontMatter,
    InvalidFrontMatter(serde_yaml::Error),
    UnsupportedSchemaVersion(u32),
    InvalidField(&'static str),
    ReservedProfileId,
    DuplicateProfileId(String),
    ProhibitedContent(&'static str),
}

impl fmt::Display for CatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingFrontMatter => {
                write!(formatter, "portable profile requires YAML front matter")
            }
            Self::InvalidFrontMatter(_) => {
                write!(formatter, "portable profile front matter is invalid")
            }
            Self::UnsupportedSchemaVersion(version) => {
                write!(
                    formatter,
                    "portable profile schema version {version} is unsupported"
                )
            }
            Self::InvalidField(field) => {
                write!(formatter, "portable profile has an invalid {field}")
            }
            Self::ReservedProfileId => {
                write!(
                    formatter,
                    "portable profile cannot replace the mandatory Raw profile"
                )
            }
            Self::DuplicateProfileId(id) => {
                write!(
                    formatter,
                    "portable profile identifier '{id}' is duplicated"
                )
            }
            Self::ProhibitedContent(kind) => {
                write!(
                    formatter,
                    "portable profile contains prohibited {kind} content"
                )
            }
        }
    }
}

impl std::error::Error for CatalogError {}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_PROFILE: &str = r#"---
schema_version: 1
id: example
name: Example profile
status: draft

provider:
  kind: null
  endpoint: null
  model: null

compatibility:
  model_families: []
  notes: Adapt this prompt for the selected model.

limits:
  max_input_chars: 20000
  max_output_tokens: 2048
  timeout_ms: 30000

output_policy:
  reject_markdown: true
  reject_obvious_preambles: true
---
Correct the transcription while preserving the speaker's intent.
"#;

    #[test]
    fn parses_a_portable_draft_profile() {
        let profile = parse_portable_profile(VALID_PROFILE).expect("fixture should be valid");

        assert_eq!(profile.id, "example");
        assert_eq!(profile.status, PortableProfileStatus::Draft);
        assert_eq!(
            profile.system_prompt,
            "Correct the transcription while preserving the speaker's intent.\n"
        );
    }

    #[test]
    fn shipped_example_matches_the_portable_format() {
        let profile = parse_portable_profile(include_str!("../../profiles/example.md"))
            .expect("shipped example should be valid");

        assert_eq!(profile.id, "example");
        assert_eq!(profile.status, PortableProfileStatus::Draft);
    }

    #[test]
    fn rejects_missing_or_unknown_front_matter_fields() {
        assert!(matches!(
            parse_portable_profile("No front matter"),
            Err(CatalogError::MissingFrontMatter)
        ));
        let unknown =
            VALID_PROFILE.replacen("status: draft", "status: draft\nsecret_ref: private", 1);
        assert!(matches!(
            parse_portable_profile(&unknown),
            Err(CatalogError::ProhibitedContent("secret_ref"))
        ));
    }

    #[test]
    fn rejects_unsupported_schema_and_invalid_metadata() {
        let unsupported = VALID_PROFILE.replacen("schema_version: 1", "schema_version: 2", 1);
        assert!(matches!(
            parse_portable_profile(&unsupported),
            Err(CatalogError::UnsupportedSchemaVersion(2))
        ));
        let invalid_id = VALID_PROFILE.replacen("id: example", "id: Example", 1);
        assert!(matches!(
            parse_portable_profile(&invalid_id),
            Err(CatalogError::InvalidField("id"))
        ));
        let raw_id = VALID_PROFILE.replacen("id: example", "id: raw", 1);
        assert!(matches!(
            parse_portable_profile(&raw_id),
            Err(CatalogError::ReservedProfileId)
        ));
    }

    #[test]
    fn rejects_secret_like_content_without_echoing_it() {
        let unsafe_profile = VALID_PROFILE.replacen(
            "Correct the transcription while preserving the speaker's intent.",
            "Authorization: Bearer private-token",
            1,
        );

        let error = parse_portable_profile(&unsafe_profile).expect_err("secret-like data fails");
        assert!(matches!(
            error,
            CatalogError::ProhibitedContent("authorization:")
        ));
        assert!(!error.to_string().contains("private-token"));
    }

    #[test]
    fn rejects_duplicate_ids_across_a_profile_set() {
        let first = include_str!("../tests/fixtures/catalog/duplicate-a.md");
        let second = include_str!("../tests/fixtures/catalog/duplicate-b.md");

        let error = parse_portable_profiles([first, second])
            .expect_err("duplicate profile identifiers must be rejected");

        assert!(matches!(error, CatalogError::DuplicateProfileId(id) if id == "duplicate"));
    }

    #[test]
    fn invalid_catalog_fixtures_are_rejected_without_sensitive_diagnostics() {
        let unsafe_profile = include_str!("../tests/fixtures/catalog/secret-reference.md");
        let missing_front_matter = include_str!("../tests/fixtures/catalog/no-front-matter.md");

        let secret_error = parse_portable_profile(unsafe_profile)
            .expect_err("secret-reference fixture must be rejected");
        assert!(matches!(
            secret_error,
            CatalogError::ProhibitedContent("secret_ref")
        ));
        assert!(!secret_error.to_string().contains("private-reference"));
        assert!(matches!(
            parse_portable_profile(missing_front_matter),
            Err(CatalogError::MissingFrontMatter)
        ));
    }
}

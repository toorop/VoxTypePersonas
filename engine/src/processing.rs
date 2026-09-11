use crate::config::OutputPolicy;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderFailure {
    Unavailable,
    Timeout,
    Network,
    Authentication,
    HttpStatus(u16),
    ResponseParsing,
    InvalidResponse(ResponseValidationError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResponseValidationError {
    Empty,
    ContainsNul,
    ObviousPreamble,
    Markdown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProcessingOutput {
    Processed(String),
    Fallback {
        text: Vec<u8>,
        reason: ProviderFailure,
    },
}

impl ProcessingOutput {
    pub fn output_bytes(&self) -> &[u8] {
        match self {
            Self::Processed(text) => text.as_bytes(),
            Self::Fallback { text, .. } => text,
        }
    }

    pub fn fallback_reason(&self) -> Option<&ProviderFailure> {
        match self {
            Self::Processed(_) => None,
            Self::Fallback { reason, .. } => Some(reason),
        }
    }
}

pub fn finalize_provider_response(
    original: &[u8],
    response: Result<String, ProviderFailure>,
    policy: &OutputPolicy,
) -> ProcessingOutput {
    match response.and_then(|text| validate_response(&text, policy).map(|_| text)) {
        Ok(text) => ProcessingOutput::Processed(text),
        Err(reason) => ProcessingOutput::Fallback {
            text: original.to_vec(),
            reason,
        },
    }
}

pub fn validate_response(response: &str, policy: &OutputPolicy) -> Result<(), ProviderFailure> {
    let trimmed = response.trim();
    if trimmed.is_empty() {
        return Err(ProviderFailure::InvalidResponse(
            ResponseValidationError::Empty,
        ));
    }
    if response.contains('\0') {
        return Err(ProviderFailure::InvalidResponse(
            ResponseValidationError::ContainsNul,
        ));
    }
    if policy.reject_obvious_preambles && has_obvious_preamble(trimmed) {
        return Err(ProviderFailure::InvalidResponse(
            ResponseValidationError::ObviousPreamble,
        ));
    }
    if policy.reject_markdown && has_markdown_framing(trimmed) {
        return Err(ProviderFailure::InvalidResponse(
            ResponseValidationError::Markdown,
        ));
    }
    Ok(())
}

fn has_obvious_preamble(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    [
        "here is ",
        "here's ",
        "sure,",
        "certainly,",
        "corrected transcription:",
        "corrected text:",
    ]
    .iter()
    .any(|prefix| lower.starts_with(prefix))
}

fn has_markdown_framing(text: &str) -> bool {
    text.starts_with("```")
        || text.starts_with("# ")
        || text.starts_with("## ")
        || text.starts_with("- ")
        || text.starts_with("* ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_response_is_processed_output() {
        let output = finalize_provider_response(
            b"original",
            Ok("Natural corrected text.".to_owned()),
            &OutputPolicy::default(),
        );

        assert_eq!(
            output,
            ProcessingOutput::Processed("Natural corrected text.".to_owned())
        );
    }

    #[test]
    fn every_recoverable_provider_failure_returns_original_bytes() {
        let failures = [
            ProviderFailure::Unavailable,
            ProviderFailure::Timeout,
            ProviderFailure::Network,
            ProviderFailure::Authentication,
            ProviderFailure::HttpStatus(503),
            ProviderFailure::ResponseParsing,
        ];

        for failure in failures {
            let output = finalize_provider_response(
                b"original\ntext",
                Err(failure.clone()),
                &OutputPolicy::default(),
            );
            assert_eq!(output.output_bytes(), b"original\ntext");
            assert_eq!(output.fallback_reason(), Some(&failure));
        }
    }

    #[test]
    fn invalid_responses_fall_back_to_original_bytes() {
        let invalid = [
            "",
            "  \n",
            "Here is the corrected text.",
            "```text\noutput\n```",
        ];

        for response in invalid {
            let output = finalize_provider_response(
                b"original",
                Ok(response.to_owned()),
                &OutputPolicy::default(),
            );
            assert_eq!(output.output_bytes(), b"original");
            assert!(matches!(
                output.fallback_reason(),
                Some(ProviderFailure::InvalidResponse(_))
            ));
        }
    }

    #[test]
    fn profile_policy_can_allow_markdown_or_preambles() {
        let policy = OutputPolicy {
            reject_markdown: false,
            reject_obvious_preambles: false,
        };

        assert!(validate_response("# Heading", &policy).is_ok());
        assert!(validate_response("Here is the text.", &policy).is_ok());
    }
}

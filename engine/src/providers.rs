use crate::config::{Provider, ProviderKind};
use crate::processing::ProviderFailure;
use serde::{Deserialize, Serialize};

pub const DEFAULT_OLLAMA_ENDPOINT: &str = "http://127.0.0.1:11434";

pub const OPENAI_ENDPOINT: &str = "https://api.openai.com/v1";
pub const MISTRAL_ENDPOINT: &str = "https://api.mistral.ai/v1";
pub const GROQ_ENDPOINT: &str = "https://api.groq.com/openai/v1";
pub const OPENROUTER_ENDPOINT: &str = "https://openrouter.ai/api/v1";
pub const ANTHROPIC_ENDPOINT: &str = "https://api.anthropic.com/v1/messages";
pub const GEMINI_ENDPOINT: &str = "https://generativelanguage.googleapis.com/v1beta";

pub fn default_endpoint(kind: &ProviderKind) -> Option<&'static str> {
    match kind {
        ProviderKind::Openai => Some(OPENAI_ENDPOINT),
        ProviderKind::Mistral => Some(MISTRAL_ENDPOINT),
        ProviderKind::Groq => Some(GROQ_ENDPOINT),
        ProviderKind::Openrouter => Some(OPENROUTER_ENDPOINT),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderRequest {
    pub model: String,
    pub system_prompt: String,
    pub user_text: String,
    pub timeout_ms: u64,
    pub max_output_tokens: u32,
}

pub trait ProviderAdapter {
    fn list_models(&self) -> Result<Vec<String>, ProviderFailure>;
    fn test(&self, model: &str, timeout_ms: u64) -> Result<(), ProviderFailure>;
    fn process(&self, request: &ProviderRequest) -> Result<String, ProviderFailure>;
}

pub trait HttpTransport {
    fn send(&self, request: HttpRequest) -> Result<HttpResponse, ProviderFailure>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: &'static str,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
    pub timeout_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ReqwestTransport;

impl HttpTransport for ReqwestTransport {
    fn send(&self, request: HttpRequest) -> Result<HttpResponse, ProviderFailure> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_millis(request.timeout_ms))
            .build()
            .map_err(|_| ProviderFailure::Unavailable)?;
        let request_builder = match request.method {
            "GET" => client.get(&request.path),
            "POST" => client.post(&request.path),
            _ => return Err(ProviderFailure::Network),
        };
        let response = request_builder
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .headers(
                request
                    .headers
                    .iter()
                    .filter_map(|(name, value)| {
                        let name = reqwest::header::HeaderName::from_bytes(name.as_bytes()).ok()?;
                        let value = reqwest::header::HeaderValue::from_str(value).ok()?;
                        Some((name, value))
                    })
                    .collect(),
            )
            .body(request.body.unwrap_or_default())
            .send()
            .map_err(|error| {
                if error.is_timeout() {
                    ProviderFailure::Timeout
                } else {
                    ProviderFailure::Unavailable
                }
            })?;
        let status = response.status().as_u16();
        let body = response
            .text()
            .map_err(|_| ProviderFailure::ResponseParsing)?;
        Ok(HttpResponse { status, body })
    }
}

pub struct OllamaAdapter<T> {
    endpoint: String,
    transport: T,
}

pub struct ChatCompletionsAdapter<T> {
    endpoint: String,
    api_key: String,
    transport: T,
}

pub struct AnthropicAdapter<T> {
    api_key: String,
    transport: T,
}
impl<T> AnthropicAdapter<T> {
    pub fn new(api_key: String, transport: T) -> Result<Self, ProviderFailure> {
        if api_key.is_empty() {
            return Err(ProviderFailure::Authentication);
        }
        Ok(Self { api_key, transport })
    }
}
impl<T: HttpTransport> ProviderAdapter for AnthropicAdapter<T> {
    fn list_models(&self) -> Result<Vec<String>, ProviderFailure> {
        Err(ProviderFailure::Unavailable)
    }
    fn test(&self, model: &str, timeout_ms: u64) -> Result<(), ProviderFailure> {
        self.process(&ProviderRequest {
            model: model.to_owned(),
            system_prompt: "Return only OK.".to_owned(),
            user_text: "ping".to_owned(),
            timeout_ms,
            max_output_tokens: 1,
        })
        .map(|_| ())
    }
    fn process(&self, request: &ProviderRequest) -> Result<String, ProviderFailure> {
        let body = serde_json::json!({"model":request.model,"max_tokens":request.max_output_tokens,"system":request.system_prompt,"messages":[{"role":"user","content":request.user_text}]}).to_string();
        let response = self.transport.send(HttpRequest {
            method: "POST",
            path: ANTHROPIC_ENDPOINT.to_owned(),
            headers: vec![
                ("x-api-key".to_owned(), self.api_key.clone()),
                ("anthropic-version".to_owned(), "2023-06-01".to_owned()),
            ],
            body: Some(body),
            timeout_ms: request.timeout_ms,
        })?;
        ensure_success(&response)?;
        let payload: serde_json::Value =
            serde_json::from_str(&response.body).map_err(|_| ProviderFailure::ResponseParsing)?;
        payload["content"][0]["text"]
            .as_str()
            .map(str::to_owned)
            .ok_or(ProviderFailure::ResponseParsing)
    }
}

pub struct GeminiAdapter<T> {
    api_key: String,
    transport: T,
}
impl<T> GeminiAdapter<T> {
    pub fn new(api_key: String, transport: T) -> Result<Self, ProviderFailure> {
        if api_key.is_empty() {
            Err(ProviderFailure::Authentication)
        } else {
            Ok(Self { api_key, transport })
        }
    }
}
impl<T: HttpTransport> ProviderAdapter for GeminiAdapter<T> {
    fn list_models(&self) -> Result<Vec<String>, ProviderFailure> {
        Err(ProviderFailure::Unavailable)
    }
    fn test(&self, model: &str, timeout_ms: u64) -> Result<(), ProviderFailure> {
        self.process(&ProviderRequest {
            model: model.to_owned(),
            system_prompt: "Return only OK.".to_owned(),
            user_text: "ping".to_owned(),
            timeout_ms,
            max_output_tokens: 1,
        })
        .map(|_| ())
    }
    fn process(&self, request: &ProviderRequest) -> Result<String, ProviderFailure> {
        let body = serde_json::json!({"system_instruction":{"parts":[{"text":request.system_prompt}]},"contents":[{"role":"user","parts":[{"text":request.user_text}]}],"generationConfig":{"maxOutputTokens":request.max_output_tokens}}).to_string();
        let response = self.transport.send(HttpRequest {
            method: "POST",
            path: format!(
                "{}/models/{}:generateContent",
                GEMINI_ENDPOINT, request.model
            ),
            headers: vec![("x-goog-api-key".to_owned(), self.api_key.clone())],
            body: Some(body),
            timeout_ms: request.timeout_ms,
        })?;
        ensure_success(&response)?;
        let payload: serde_json::Value =
            serde_json::from_str(&response.body).map_err(|_| ProviderFailure::ResponseParsing)?;
        payload["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .map(str::to_owned)
            .ok_or(ProviderFailure::ResponseParsing)
    }
}

impl<T> ChatCompletionsAdapter<T> {
    pub fn new(endpoint: &str, api_key: String, transport: T) -> Result<Self, ProviderFailure> {
        if !(endpoint.starts_with("https://") || endpoint.starts_with("http://")) {
            return Err(ProviderFailure::Unavailable);
        }
        if api_key.is_empty() {
            return Err(ProviderFailure::Authentication);
        }
        Ok(Self {
            endpoint: endpoint.trim_end_matches('/').to_owned(),
            api_key,
            transport,
        })
    }

    fn headers(&self) -> Vec<(String, String)> {
        vec![(
            "authorization".to_owned(),
            format!("Bearer {}", self.api_key),
        )]
    }
}

impl<T: HttpTransport> ProviderAdapter for ChatCompletionsAdapter<T> {
    fn list_models(&self) -> Result<Vec<String>, ProviderFailure> {
        let response = self.transport.send(HttpRequest {
            method: "GET",
            path: format!("{}/models", self.endpoint),
            headers: self.headers(),
            body: None,
            timeout_ms: 10_000,
        })?;
        ensure_success(&response)?;
        let payload: OpenAiModelsResponse =
            serde_json::from_str(&response.body).map_err(|_| ProviderFailure::ResponseParsing)?;
        Ok(payload
            .data
            .into_iter()
            .map(|model| model.id)
            .filter(|id| is_text_model(id))
            .collect())
    }

    fn test(&self, model: &str, timeout_ms: u64) -> Result<(), ProviderFailure> {
        self.process(&ProviderRequest {
            model: model.to_owned(),
            system_prompt: "Return only OK.".to_owned(),
            user_text: "ping".to_owned(),
            timeout_ms,
            max_output_tokens: 1,
        })
        .map(|_| ())
    }

    fn process(&self, request: &ProviderRequest) -> Result<String, ProviderFailure> {
        let body = serde_json::to_string(&OpenAiChatRequest {
            model: &request.model,
            messages: vec![
                OpenAiMessage {
                    role: "system",
                    content: &request.system_prompt,
                },
                OpenAiMessage {
                    role: "user",
                    content: &request.user_text,
                },
            ],
            max_tokens: request.max_output_tokens,
        })
        .map_err(|_| ProviderFailure::ResponseParsing)?;
        let response = self.transport.send(HttpRequest {
            method: "POST",
            path: format!("{}/chat/completions", self.endpoint),
            headers: self.headers(),
            body: Some(body),
            timeout_ms: request.timeout_ms,
        })?;
        ensure_success(&response)?;
        let payload: OpenAiChatResponse =
            serde_json::from_str(&response.body).map_err(|_| ProviderFailure::ResponseParsing)?;
        payload
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .ok_or(ProviderFailure::ResponseParsing)
    }
}

fn is_text_model(id: &str) -> bool {
    let lower = id.to_ascii_lowercase();
    ![
        "image",
        "audio",
        "whisper",
        "tts",
        "embedding",
        "moderation",
    ]
    .iter()
    .any(|term| lower.contains(term))
}

#[derive(Deserialize)]
struct OpenAiModelsResponse {
    data: Vec<OpenAiModel>,
}
#[derive(Deserialize)]
struct OpenAiModel {
    id: String,
}
#[derive(Deserialize)]
struct OpenAiChatResponse {
    choices: Vec<OpenAiChoice>,
}
#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiResponseMessage,
}
#[derive(Deserialize)]
struct OpenAiResponseMessage {
    content: String,
}
#[derive(Serialize)]
struct OpenAiChatRequest<'a> {
    model: &'a str,
    messages: Vec<OpenAiMessage<'a>>,
    max_tokens: u32,
}
#[derive(Serialize)]
struct OpenAiMessage<'a> {
    role: &'a str,
    content: &'a str,
}

impl<T> OllamaAdapter<T> {
    pub fn new(endpoint: Option<&str>, transport: T) -> Self {
        Self {
            endpoint: endpoint
                .unwrap_or(DEFAULT_OLLAMA_ENDPOINT)
                .trim_end_matches('/')
                .to_owned(),
            transport,
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

impl<T: HttpTransport> ProviderAdapter for OllamaAdapter<T> {
    fn list_models(&self) -> Result<Vec<String>, ProviderFailure> {
        let response = self.transport.send(HttpRequest {
            method: "GET",
            path: format!("{}/api/tags", self.endpoint),
            headers: Vec::new(),
            body: None,
            timeout_ms: 5_000,
        })?;
        ensure_success(&response)?;
        let payload: OllamaTagsResponse =
            serde_json::from_str(&response.body).map_err(|_| ProviderFailure::ResponseParsing)?;
        Ok(payload.models.into_iter().map(|model| model.name).collect())
    }

    fn test(&self, model: &str, timeout_ms: u64) -> Result<(), ProviderFailure> {
        self.process(&ProviderRequest {
            model: model.to_owned(),
            system_prompt: "Return only OK.".to_owned(),
            user_text: "ping".to_owned(),
            timeout_ms,
            max_output_tokens: 1,
        })
        .map(|_| ())
    }

    fn process(&self, request: &ProviderRequest) -> Result<String, ProviderFailure> {
        let body = serde_json::to_string(&OllamaChatRequest {
            model: &request.model,
            stream: false,
            messages: vec![
                OllamaMessage {
                    role: "system",
                    content: &request.system_prompt,
                },
                OllamaMessage {
                    role: "user",
                    content: &request.user_text,
                },
            ],
            options: OllamaOptions {
                num_predict: request.max_output_tokens,
            },
        })
        .map_err(|_| ProviderFailure::ResponseParsing)?;
        let response = self.transport.send(HttpRequest {
            method: "POST",
            path: format!("{}/api/chat", self.endpoint),
            headers: Vec::new(),
            body: Some(body),
            timeout_ms: request.timeout_ms,
        })?;
        ensure_success(&response)?;
        let payload: OllamaChatResponse =
            serde_json::from_str(&response.body).map_err(|_| ProviderFailure::ResponseParsing)?;
        Ok(payload.message.content)
    }
}

pub fn ollama_from_provider<T>(
    provider: &Provider,
    transport: T,
) -> Result<OllamaAdapter<T>, ProviderFailure> {
    if provider.kind != ProviderKind::Ollama {
        return Err(ProviderFailure::Unavailable);
    }
    Ok(OllamaAdapter::new(provider.endpoint.as_deref(), transport))
}

pub fn compatible_from_provider<T>(
    provider: &Provider,
    api_key: String,
    transport: T,
) -> Result<ChatCompletionsAdapter<T>, ProviderFailure> {
    let endpoint = provider
        .endpoint
        .as_deref()
        .or_else(|| default_endpoint(&provider.kind))
        .ok_or(ProviderFailure::Unavailable)?;
    match provider.kind {
        ProviderKind::Openai
        | ProviderKind::Mistral
        | ProviderKind::Groq
        | ProviderKind::Openrouter
        | ProviderKind::OpenaiCompatible => {
            ChatCompletionsAdapter::new(endpoint, api_key, transport)
        }
        _ => Err(ProviderFailure::Unavailable),
    }
}

fn ensure_success(response: &HttpResponse) -> Result<(), ProviderFailure> {
    if (200..300).contains(&response.status) {
        Ok(())
    } else if response.status == 401 || response.status == 403 {
        Err(ProviderFailure::Authentication)
    } else {
        Err(ProviderFailure::HttpStatus(response.status))
    }
}

#[derive(Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}
#[derive(Deserialize)]
struct OllamaModel {
    name: String,
}
#[derive(Deserialize)]
struct OllamaChatResponse {
    message: OllamaResponseMessage,
}
#[derive(Deserialize)]
struct OllamaResponseMessage {
    content: String,
}
#[derive(Serialize)]
struct OllamaChatRequest<'a> {
    model: &'a str,
    stream: bool,
    messages: Vec<OllamaMessage<'a>>,
    options: OllamaOptions,
}
#[derive(Serialize)]
struct OllamaMessage<'a> {
    role: &'a str,
    content: &'a str,
}
#[derive(Serialize)]
struct OllamaOptions {
    num_predict: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct MockTransport {
        response: Result<HttpResponse, ProviderFailure>,
        requests: RefCell<Vec<HttpRequest>>,
    }
    impl HttpTransport for MockTransport {
        fn send(&self, request: HttpRequest) -> Result<HttpResponse, ProviderFailure> {
            self.requests.borrow_mut().push(request);
            self.response.clone()
        }
    }

    #[test]
    fn detects_installed_ollama_models() {
        let transport = MockTransport {
            response: Ok(HttpResponse {
                status: 200,
                body: r#"{"models":[{"name":"llama3"},{"name":"qwen"}]}"#.to_owned(),
            }),
            requests: RefCell::new(Vec::new()),
        };
        let adapter = OllamaAdapter::new(None, transport);
        assert_eq!(
            adapter.list_models().expect("models should parse"),
            ["llama3", "qwen"]
        );
    }

    #[test]
    fn unavailable_ollama_is_reported_without_a_network_fallback() {
        let transport = MockTransport {
            response: Err(ProviderFailure::Unavailable),
            requests: RefCell::new(Vec::new()),
        };
        let adapter = OllamaAdapter::new(None, transport);
        assert_eq!(adapter.list_models(), Err(ProviderFailure::Unavailable));
    }

    #[test]
    fn sends_system_and_user_content_separately() {
        let transport = MockTransport {
            response: Ok(HttpResponse {
                status: 200,
                body: r#"{"message":{"content":"final"}}"#.to_owned(),
            }),
            requests: RefCell::new(Vec::new()),
        };
        let adapter = OllamaAdapter::new(Some("http://localhost:11434/"), transport);
        let output = adapter
            .process(&ProviderRequest {
                model: "llama3".to_owned(),
                system_prompt: "system rules".to_owned(),
                user_text: "dictated text".to_owned(),
                timeout_ms: 123,
                max_output_tokens: 7,
            })
            .expect("response should parse");
        assert_eq!(output, "final");
        let request = adapter.transport.requests.borrow();
        assert_eq!(request[0].path, "http://localhost:11434/api/chat");
        let body = request[0].body.as_ref().expect("body should exist");
        assert!(body.contains("system rules"));
        assert!(body.contains("dictated text"));
        assert!(body.contains("\"num_predict\":7"));
    }

    #[test]
    fn maps_auth_and_http_errors() {
        let auth = HttpResponse {
            status: 401,
            body: String::new(),
        };
        let unavailable = HttpResponse {
            status: 503,
            body: String::new(),
        };
        assert_eq!(ensure_success(&auth), Err(ProviderFailure::Authentication));
        assert_eq!(
            ensure_success(&unavailable),
            Err(ProviderFailure::HttpStatus(503))
        );
    }

    #[test]
    fn compatible_adapter_filters_non_text_models() {
        let transport = MockTransport {
            response: Ok(HttpResponse {
                status: 200,
                body:
                    r#"{"data":[{"id":"gpt-text"},{"id":"whisper-1"},{"id":"text-embedding-3"}]}"#
                        .to_owned(),
            }),
            requests: RefCell::new(Vec::new()),
        };
        let adapter =
            ChatCompletionsAdapter::new(OPENAI_ENDPOINT, "private-key".to_owned(), transport)
                .expect("adapter should construct");
        assert_eq!(
            adapter.list_models().expect("models should parse"),
            ["gpt-text"]
        );
        let request = adapter.transport.requests.borrow();
        assert_eq!(request[0].headers[0].0, "authorization");
        assert_eq!(request[0].headers[0].1, "Bearer private-key");
    }

    #[test]
    fn compatible_adapter_sends_separate_messages() {
        let transport = MockTransport {
            response: Ok(HttpResponse {
                status: 200,
                body: r#"{"choices":[{"message":{"content":"final"}}]}"#.to_owned(),
            }),
            requests: RefCell::new(Vec::new()),
        };
        let adapter = ChatCompletionsAdapter::new(
            "https://example.test/v1",
            "private-key".to_owned(),
            transport,
        )
        .expect("adapter should construct");
        assert_eq!(
            adapter
                .process(&ProviderRequest {
                    model: "text-model".to_owned(),
                    system_prompt: "system".to_owned(),
                    user_text: "user".to_owned(),
                    timeout_ms: 1000,
                    max_output_tokens: 2
                })
                .expect("response should parse"),
            "final"
        );
        let body = adapter.transport.requests.borrow()[0]
            .body
            .clone()
            .expect("request body should exist");
        assert!(body.contains("\"system\""));
        assert!(body.contains("\"user\""));
        assert!(body.contains("\"max_tokens\":2"));
    }

    #[test]
    fn built_in_endpoints_are_stable() {
        assert_eq!(
            default_endpoint(&ProviderKind::Openai),
            Some(OPENAI_ENDPOINT)
        );
        assert_eq!(
            default_endpoint(&ProviderKind::Mistral),
            Some(MISTRAL_ENDPOINT)
        );
        assert_eq!(default_endpoint(&ProviderKind::Groq), Some(GROQ_ENDPOINT));
        assert_eq!(
            default_endpoint(&ProviderKind::Openrouter),
            Some(OPENROUTER_ENDPOINT)
        );
    }

    #[test]
    fn anthropic_uses_top_level_system_and_user_message() {
        let transport = MockTransport {
            response: Ok(HttpResponse {
                status: 200,
                body: r#"{"content":[{"text":"final"}]}"#.to_owned(),
            }),
            requests: RefCell::new(Vec::new()),
        };
        let adapter =
            AnthropicAdapter::new("key".to_owned(), transport).expect("adapter should construct");
        assert_eq!(
            adapter
                .process(&ProviderRequest {
                    model: "claude".to_owned(),
                    system_prompt: "system".to_owned(),
                    user_text: "user".to_owned(),
                    timeout_ms: 1_000,
                    max_output_tokens: 2
                })
                .expect("response should parse"),
            "final"
        );
        let request = adapter.transport.requests.borrow();
        assert_eq!(request[0].path, ANTHROPIC_ENDPOINT);
        assert!(
            request[0]
                .body
                .as_ref()
                .expect("body")
                .contains("\"system\":\"system\"")
        );
        assert!(
            request[0]
                .headers
                .iter()
                .any(|(name, _)| name == "x-api-key")
        );
    }

    #[test]
    fn gemini_uses_system_instruction_and_contents() {
        let transport = MockTransport {
            response: Ok(HttpResponse {
                status: 200,
                body: r#"{"candidates":[{"content":{"parts":[{"text":"final"}]}}]}"#.to_owned(),
            }),
            requests: RefCell::new(Vec::new()),
        };
        let adapter =
            GeminiAdapter::new("key".to_owned(), transport).expect("adapter should construct");
        assert_eq!(
            adapter
                .process(&ProviderRequest {
                    model: "gemini-test".to_owned(),
                    system_prompt: "system".to_owned(),
                    user_text: "user".to_owned(),
                    timeout_ms: 1_000,
                    max_output_tokens: 2
                })
                .expect("response should parse"),
            "final"
        );
        let request = adapter.transport.requests.borrow();
        assert_eq!(
            request[0].path,
            format!("{}/models/gemini-test:generateContent", GEMINI_ENDPOINT)
        );
        let body = request[0].body.as_ref().expect("body");
        assert!(body.contains("system_instruction"));
        assert!(body.contains("contents"));
    }
}

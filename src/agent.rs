use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const DEFAULT_OPENAI_MODEL: &str = "gpt-4o-mini";
pub const DEFAULT_GEMINI_MODEL: &str = "gemini-flash-latest";
pub const OPENAI_API_URL: &str = "https://api.openai.com";
pub const GEMINI_API_URL: &str = "https://generativelanguage.googleapis.com";

#[derive(Debug)]
pub struct AgentError {
    pub message: String,
}

impl std::fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AgentError {}

pub trait LlmAgent {
    fn send_request(&self, prompt: &str) -> Result<AgentResponse, AgentError>;
}

pub struct OpenAiAgent {
    api_key: String,
    timeout: Duration,
    base_url: String,
    model: String,
}

impl OpenAiAgent {
    pub fn new(api_key: String, timeout: Duration) -> Self {
        Self {
            api_key,
            timeout,
            base_url: OPENAI_API_URL.to_string(),
            model: DEFAULT_OPENAI_MODEL.to_string(),
        }
    }

    pub fn with_config(api_key: String, timeout: Duration, base_url: Option<String>, model: Option<String>) -> Self {
        Self {
            api_key,
            timeout,
            base_url: base_url.unwrap_or_else(|| OPENAI_API_URL.to_string()),
            model: model.unwrap_or_else(|| DEFAULT_OPENAI_MODEL.to_string()),
        }
    }

    pub fn build_chat_request(&self, prompt: &str) -> ChatRequest {
        ChatRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        }
    }

    fn parse_response(&self, json: &str) -> Result<AgentResponse, AgentError> {
        let api_response: OpenAiApiResponse = serde_json::from_str(json).map_err(|e| AgentError {
            message: format!("Failed to parse response: {}", e),
        })?;

        match api_response {
            OpenAiApiResponse::Success(chat_response) => {
                let content = chat_response
                    .choices
                    .first()
                    .map(|c| c.message.content.clone())
                    .ok_or_else(|| AgentError {
                        message: "No choices in response".to_string(),
                    })?;
                Ok(AgentResponse { content })
            }
            OpenAiApiResponse::Error { error } => Err(AgentError {
                message: format!("API error: {}", error.message),
            }),
        }
    }
}

impl LlmAgent for OpenAiAgent {
    fn send_request(&self, prompt: &str) -> Result<AgentResponse, AgentError> {
        let request = self.build_chat_request(prompt);
        let base = self.base_url.trim_end_matches('/');
        let url = format!("{}/v1/chat/completions", base);

        let agent = ureq::AgentBuilder::new()
            .timeout(self.timeout)
            .build();

        let response = agent
            .post(&url)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Content-Type", "application/json")
            .send_json(&request);

        match response {
            Ok(resp) => {
                let body = resp.into_string().map_err(|e| AgentError {
                    message: format!("Failed to read response body: {}", e),
                })?;
                self.parse_response(&body)
            }
            Err(ureq::Error::Status(status, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                let error_msg = match status {
                    401 => "Unauthorized: Invalid API key".to_string(),
                    429 => "Rate limit exceeded".to_string(),
                    s if s >= 500 => format!("Server error: {}", s),
                    _ => {
                        if let Ok(parsed) = self.parse_response(&body) {
                            return Ok(parsed);
                        }
                        format!("HTTP error {}: {}", status, body)
                    }
                };
                Err(AgentError { message: error_msg })
            }
            Err(e) => Err(AgentError {
                message: format!("Request failed: {}", e),
            }),
        }
    }
}

pub struct GeminiAgent {
    api_key: String,
    timeout: Duration,
    base_url: String,
    model: String,
}

impl GeminiAgent {
    pub fn new(api_key: String, timeout: Duration) -> Self {
        Self {
            api_key,
            timeout,
            base_url: GEMINI_API_URL.to_string(),
            model: DEFAULT_GEMINI_MODEL.to_string(),
        }
    }

    pub fn with_config(api_key: String, timeout: Duration, base_url: Option<String>, model: Option<String>) -> Self {
        Self {
            api_key,
            timeout,
            base_url: base_url.unwrap_or_else(|| GEMINI_API_URL.to_string()),
            model: model.unwrap_or_else(|| DEFAULT_GEMINI_MODEL.to_string()),
        }
    }

    pub fn build_request(&self, prompt: &str) -> GeminiRequest {
        GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: prompt.to_string(),
                }],
            }],
        }
    }

    pub fn build_url(&self) -> String {
        let base = self.base_url.trim_end_matches('/');
        format!("{}/v1beta/models/{}:generateContent", base, self.model)
    }

    fn parse_response(&self, json: &str) -> Result<AgentResponse, AgentError> {
        let api_response: GeminiApiResponse = serde_json::from_str(json).map_err(|e| AgentError {
            message: format!("Failed to parse response: {}", e),
        })?;

        match api_response {
            GeminiApiResponse::Success(response) => {
                let content = response
                    .candidates
                    .first()
                    .and_then(|c| c.content.parts.first())
                    .map(|p| p.text.clone())
                    .ok_or_else(|| AgentError {
                        message: "No content in response".to_string(),
                    })?;
                Ok(AgentResponse { content })
            }
            GeminiApiResponse::Error { error } => Err(AgentError {
                message: format!("API error: {}", error.message),
            }),
        }
    }
}

impl LlmAgent for GeminiAgent {
    fn send_request(&self, prompt: &str) -> Result<AgentResponse, AgentError> {
        let request = self.build_request(prompt);
        let url = self.build_url();

        let agent = ureq::AgentBuilder::new()
            .timeout(self.timeout)
            .build();

        let response = agent
            .post(&url)
            .set("X-goog-api-key", &self.api_key)
            .set("Content-Type", "application/json")
            .send_json(&request);

        match response {
            Ok(resp) => {
                let body = resp.into_string().map_err(|e| AgentError {
                    message: format!("Failed to read response body: {}", e),
                })?;
                self.parse_response(&body)
            }
            Err(ureq::Error::Status(status, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                let error_msg = match status {
                    401 => "Unauthorized: Invalid API key".to_string(),
                    429 => "Rate limit exceeded".to_string(),
                    s if s >= 500 => format!("Server error: {}", s),
                    _ => {
                        if let Ok(parsed) = self.parse_response(&body) {
                            return Ok(parsed);
                        }
                        format!("HTTP error {}: {}", status, body)
                    }
                };
                Err(AgentError { message: error_msg })
            }
            Err(e) => Err(AgentError {
                message: format!("Request failed: {}", e),
            }),
        }
    }
}

pub fn create_agent(
    vendor: &str,
    api_key: String,
    timeout: Duration,
    base_url: Option<String>,
    model: Option<String>,
) -> Result<Box<dyn LlmAgent>, AgentError> {
    match vendor {
        "openai" => Ok(Box::new(OpenAiAgent::with_config(api_key, timeout, base_url, model))),
        "gemini" => Ok(Box::new(GeminiAgent::with_config(api_key, timeout, base_url, model))),
        _ => Err(AgentError {
            message: format!("Unknown vendor '{}'. Valid vendors: openai, gemini", vendor),
        }),
    }
}

#[derive(Debug)]
pub struct AgentResponse {
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: Message,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum OpenAiApiResponse {
    Success(ChatResponse),
    Error { error: ApiErrorDetail },
}

#[derive(Debug, Deserialize)]
struct ApiErrorDetail {
    message: String,
}

#[derive(Debug, Serialize)]
pub struct GeminiRequest {
    pub contents: Vec<GeminiContent>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiContent {
    pub parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiPart {
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct GeminiResponse {
    pub candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
pub struct GeminiCandidate {
    pub content: GeminiContent,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum GeminiApiResponse {
    Success(GeminiResponse),
    Error { error: ApiErrorDetail },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_implements_trait() {
        let agent = OpenAiAgent::new("key".to_string(), Duration::from_secs(10));
        let _: &dyn LlmAgent = &agent;
    }

    #[test]
    fn test_create_openai_agent() {
        let api_key = "test_api_key".to_string();
        let timeout = Duration::from_secs(30);

        let agent = OpenAiAgent::new(api_key.clone(), timeout);

        assert_eq!(agent.api_key, api_key);
        assert_eq!(agent.timeout, timeout);
        assert_eq!(agent.base_url, OPENAI_API_URL);
        assert_eq!(agent.model, DEFAULT_OPENAI_MODEL);
    }

    #[test]
    fn test_openai_uses_configured_model() {
        let agent = OpenAiAgent::with_config(
            "key".to_string(),
            Duration::from_secs(10),
            None,
            Some("gpt-4".to_string()),
        );

        let request = agent.build_chat_request("Hello");

        assert_eq!(request.model, "gpt-4");
    }

    #[test]
    fn test_openai_uses_configured_base_url() {
        let agent = OpenAiAgent::with_config(
            "key".to_string(),
            Duration::from_secs(10),
            Some("https://custom.openai.com".to_string()),
            None,
        );

        assert_eq!(agent.base_url, "https://custom.openai.com");
    }

    #[test]
    fn test_build_chat_request() {
        let agent = OpenAiAgent::new("key".to_string(), Duration::from_secs(10));

        let request = agent.build_chat_request("Hello");

        assert_eq!(request.model, DEFAULT_OPENAI_MODEL);
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.messages[0].role, "user");
        assert_eq!(request.messages[0].content, "Hello");
    }

    #[test]
    fn test_parse_openai_success_response() {
        let agent = OpenAiAgent::new("key".to_string(), Duration::from_secs(10));
        let json = r#"{
            "choices": [
                {
                    "message": {
                        "role": "assistant",
                        "content": "Hello! How can I help you?"
                    }
                }
            ]
        }"#;

        let result = agent.parse_response(json);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().content, "Hello! How can I help you?");
    }

    #[test]
    fn test_parse_openai_error_response() {
        let agent = OpenAiAgent::new("key".to_string(), Duration::from_secs(10));
        let json = r#"{
            "error": {
                "message": "Invalid API key",
                "type": "invalid_request_error",
                "code": "invalid_api_key"
            }
        }"#;

        let result = agent.parse_response(json);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("Invalid API key"));
    }

    #[test]
    fn test_gemini_implements_trait() {
        let agent = GeminiAgent::new("key".to_string(), Duration::from_secs(10));
        let _: &dyn LlmAgent = &agent;
    }

    #[test]
    fn test_create_gemini_agent() {
        let api_key = "test_api_key".to_string();
        let timeout = Duration::from_secs(30);

        let agent = GeminiAgent::new(api_key.clone(), timeout);

        assert_eq!(agent.api_key, api_key);
        assert_eq!(agent.timeout, timeout);
        assert_eq!(agent.base_url, GEMINI_API_URL);
        assert_eq!(agent.model, DEFAULT_GEMINI_MODEL);
    }

    #[test]
    fn test_gemini_request_format() {
        let agent = GeminiAgent::new("key".to_string(), Duration::from_secs(10));

        let request = agent.build_request("Hello");

        assert_eq!(request.contents.len(), 1);
        assert_eq!(request.contents[0].parts.len(), 1);
        assert_eq!(request.contents[0].parts[0].text, "Hello");
    }

    #[test]
    fn test_gemini_url_format() {
        let agent = GeminiAgent::new("key".to_string(), Duration::from_secs(10));

        let url = agent.build_url();

        assert_eq!(
            url,
            format!("{}/v1beta/models/{}:generateContent", GEMINI_API_URL, DEFAULT_GEMINI_MODEL)
        );
    }

    #[test]
    fn test_gemini_url_with_custom_model() {
        let agent = GeminiAgent::with_config(
            "key".to_string(),
            Duration::from_secs(10),
            None,
            Some("gemini-pro".to_string()),
        );

        let url = agent.build_url();

        assert!(url.contains("gemini-pro:generateContent"));
    }

    #[test]
    fn test_gemini_response_parsing() {
        let agent = GeminiAgent::new("key".to_string(), Duration::from_secs(10));
        let json = r#"{
            "candidates": [
                {
                    "content": {
                        "parts": [
                            {"text": "Hello from Gemini!"}
                        ]
                    }
                }
            ]
        }"#;

        let result = agent.parse_response(json);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().content, "Hello from Gemini!");
    }

    #[test]
    fn test_gemini_error_response() {
        let agent = GeminiAgent::new("key".to_string(), Duration::from_secs(10));
        let json = r#"{
            "error": {
                "message": "API key not valid",
                "status": "INVALID_ARGUMENT"
            }
        }"#;

        let result = agent.parse_response(json);

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("API key not valid"));
    }

    #[test]
    fn test_select_vendor_openai() {
        let result = create_agent(
            "openai",
            "key".to_string(),
            Duration::from_secs(10),
            None,
            None,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_select_vendor_gemini() {
        let result = create_agent(
            "gemini",
            "key".to_string(),
            Duration::from_secs(10),
            None,
            None,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_vendor_error() {
        let result = create_agent(
            "unknown",
            "key".to_string(),
            Duration::from_secs(10),
            None,
            None,
        );

        match result {
            Ok(_) => panic!("Expected error for unknown vendor"),
            Err(error) => {
                assert!(error.message.contains("Unknown vendor"));
                assert!(error.message.contains("openai"));
                assert!(error.message.contains("gemini"));
            }
        }
    }
}

#[cfg(test)]
mod pact_tests {
    use super::*;
    use pact_consumer::prelude::*;

    #[test]
    fn pact_openai_successful_completion() {
        let mock_server = PactBuilder::new_v4("agent-run", "openai-api")
            .interaction("successful chat completion", "", |mut i| {
                i.request
                    .post()
                    .path("/v1/chat/completions")
                    .header("Authorization", "Bearer test_api_key")
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!({
                        "model": like!("gpt-4o-mini"),
                        "messages": each_like!({
                            "role": like!("user"),
                            "content": like!("Hello")
                        })
                    }));
                i.response
                    .ok()
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!({
                        "choices": [{
                            "message": {
                                "role": "assistant",
                                "content": "Hello! How can I help you today?"
                            }
                        }]
                    }));
                i
            })
            .start_mock_server(None, None);

        let agent = OpenAiAgent::with_config(
            "test_api_key".to_string(),
            Duration::from_secs(10),
            Some(mock_server.url().to_string()),
            None,
        );

        let result = agent.send_request("Hello");

        assert!(result.is_ok());
        assert_eq!(result.unwrap().content, "Hello! How can I help you today?");
    }

    #[test]
    fn pact_openai_invalid_api_key() {
        let mock_server = PactBuilder::new_v4("agent-run", "openai-api")
            .interaction("invalid API key", "", |mut i| {
                i.request
                    .post()
                    .path("/v1/chat/completions")
                    .header("Authorization", "Bearer invalid_key");
                i.response
                    .status(401)
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!({
                        "error": {
                            "message": "Incorrect API key provided",
                            "type": "invalid_request_error",
                            "code": "invalid_api_key"
                        }
                    }));
                i
            })
            .start_mock_server(None, None);

        let agent = OpenAiAgent::with_config(
            "invalid_key".to_string(),
            Duration::from_secs(10),
            Some(mock_server.url().to_string()),
            None,
        );

        let result = agent.send_request("Hello");

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Unauthorized"));
    }

    #[test]
    fn pact_openai_rate_limit() {
        let mock_server = PactBuilder::new_v4("agent-run", "openai-api")
            .interaction("rate limit exceeded", "", |mut i| {
                i.request
                    .post()
                    .path("/v1/chat/completions");
                i.response
                    .status(429)
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!({
                        "error": {
                            "message": "Rate limit exceeded",
                            "type": "rate_limit_error"
                        }
                    }));
                i
            })
            .start_mock_server(None, None);

        let agent = OpenAiAgent::with_config(
            "test_key".to_string(),
            Duration::from_secs(10),
            Some(mock_server.url().to_string()),
            None,
        );

        let result = agent.send_request("Hello");

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Rate limit"));
    }

    #[test]
    fn pact_openai_server_error() {
        let mock_server = PactBuilder::new_v4("agent-run", "openai-api")
            .interaction("server error", "", |mut i| {
                i.request
                    .post()
                    .path("/v1/chat/completions");
                i.response
                    .status(500)
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!({
                        "error": {
                            "message": "Internal server error",
                            "type": "server_error"
                        }
                    }));
                i
            })
            .start_mock_server(None, None);

        let agent = OpenAiAgent::with_config(
            "test_key".to_string(),
            Duration::from_secs(10),
            Some(mock_server.url().to_string()),
            None,
        );

        let result = agent.send_request("Hello");

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Server error"));
    }

    #[test]
    fn pact_gemini_successful_completion() {
        let mock_server = PactBuilder::new_v4("agent-run", "gemini-api")
            .interaction("successful gemini completion", "", |mut i| {
                i.request
                    .post()
                    .path(format!("/v1beta/models/{}:generateContent", DEFAULT_GEMINI_MODEL))
                    .header("X-goog-api-key", "test_gemini_key")
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!({
                        "contents": each_like!({
                            "parts": each_like!({
                                "text": like!("Hello")
                            })
                        })
                    }));
                i.response
                    .ok()
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!({
                        "candidates": [{
                            "content": {
                                "parts": [{
                                    "text": "Hello from Gemini!"
                                }]
                            }
                        }]
                    }));
                i
            })
            .start_mock_server(None, None);

        let agent = GeminiAgent::with_config(
            "test_gemini_key".to_string(),
            Duration::from_secs(10),
            Some(mock_server.url().to_string()),
            None,
        );

        let result = agent.send_request("Hello");

        assert!(result.is_ok());
        assert_eq!(result.unwrap().content, "Hello from Gemini!");
    }

    #[test]
    fn pact_gemini_invalid_api_key() {
        let mock_server = PactBuilder::new_v4("agent-run", "gemini-api")
            .interaction("gemini invalid API key", "", |mut i| {
                i.request
                    .post()
                    .path(format!("/v1beta/models/{}:generateContent", DEFAULT_GEMINI_MODEL))
                    .header("X-goog-api-key", "invalid_key");
                i.response
                    .status(401)
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!({
                        "error": {
                            "message": "API key not valid. Please pass a valid API key.",
                            "status": "INVALID_ARGUMENT"
                        }
                    }));
                i
            })
            .start_mock_server(None, None);

        let agent = GeminiAgent::with_config(
            "invalid_key".to_string(),
            Duration::from_secs(10),
            Some(mock_server.url().to_string()),
            None,
        );

        let result = agent.send_request("Hello");

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Unauthorized"));
    }

    #[test]
    fn pact_gemini_rate_limit() {
        let mock_server = PactBuilder::new_v4("agent-run", "gemini-api")
            .interaction("gemini rate limit exceeded", "", |mut i| {
                i.request
                    .post()
                    .path(format!("/v1beta/models/{}:generateContent", DEFAULT_GEMINI_MODEL));
                i.response
                    .status(429)
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!({
                        "error": {
                            "message": "Resource has been exhausted",
                            "status": "RESOURCE_EXHAUSTED"
                        }
                    }));
                i
            })
            .start_mock_server(None, None);

        let agent = GeminiAgent::with_config(
            "test_key".to_string(),
            Duration::from_secs(10),
            Some(mock_server.url().to_string()),
            None,
        );

        let result = agent.send_request("Hello");

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Rate limit"));
    }

    #[test]
    fn pact_gemini_server_error() {
        let mock_server = PactBuilder::new_v4("agent-run", "gemini-api")
            .interaction("gemini server error", "", |mut i| {
                i.request
                    .post()
                    .path(format!("/v1beta/models/{}:generateContent", DEFAULT_GEMINI_MODEL));
                i.response
                    .status(500)
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!({
                        "error": {
                            "message": "Internal error encountered",
                            "status": "INTERNAL"
                        }
                    }));
                i
            })
            .start_mock_server(None, None);

        let agent = GeminiAgent::with_config(
            "test_key".to_string(),
            Duration::from_secs(10),
            Some(mock_server.url().to_string()),
            None,
        );

        let result = agent.send_request("Hello");

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Server error"));
    }
}

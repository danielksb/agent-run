use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const DEFAULT_MODEL: &str = "gpt-4o-mini";

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

pub const OPENAI_API_URL: &str = "https://api.openai.com";

pub struct Agent {
    api_key: String,
    timeout: Duration,
    base_url: String,
}

impl Agent {
    pub fn new(api_key: String, timeout: Duration) -> Self {
        Self {
            api_key,
            timeout,
            base_url: OPENAI_API_URL.to_string(),
        }
    }

    pub fn with_base_url(api_key: String, timeout: Duration, base_url: String) -> Self {
        Self {
            api_key,
            timeout,
            base_url,
        }
    }

    pub fn build_chat_request(&self, prompt: &str) -> ChatRequest {
        ChatRequest {
            model: DEFAULT_MODEL.to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        }
    }

    pub fn send_request(&self, prompt: &str) -> Result<AgentResponse, AgentError> {
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

    pub fn parse_response(&self, json: &str) -> Result<AgentResponse, AgentError> {
        let api_response: ApiResponse = serde_json::from_str(json).map_err(|e| AgentError {
            message: format!("Failed to parse response: {}", e),
        })?;

        match api_response {
            ApiResponse::Success(chat_response) => {
                let content = chat_response
                    .choices
                    .first()
                    .map(|c| c.message.content.clone())
                    .ok_or_else(|| AgentError {
                        message: "No choices in response".to_string(),
                    })?;
                Ok(AgentResponse { content })
            }
            ApiResponse::Error { error } => Err(AgentError {
                message: format!("API error: {}", error.message),
            }),
        }
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
enum ApiResponse {
    Success(ChatResponse),
    Error { error: ApiErrorDetail },
}

#[derive(Debug, Deserialize)]
struct ApiErrorDetail {
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_agent() {
        let api_key = "test_api_key".to_string();
        let timeout = Duration::from_secs(30);

        let agent = Agent::new(api_key.clone(), timeout);

        assert_eq!(agent.api_key, api_key);
        assert_eq!(agent.timeout, timeout);
        assert_eq!(agent.base_url, OPENAI_API_URL);
    }

    #[test]
    fn test_create_agent_with_custom_base_url() {
        let agent = Agent::with_base_url(
            "key".to_string(),
            Duration::from_secs(10),
            "http://localhost:8080".to_string(),
        );

        assert_eq!(agent.base_url, "http://localhost:8080");
    }

    #[test]
    fn test_build_chat_request() {
        let agent = Agent::new("key".to_string(), Duration::from_secs(10));

        let request = agent.build_chat_request("Hello");

        assert_eq!(request.model, DEFAULT_MODEL);
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.messages[0].role, "user");
        assert_eq!(request.messages[0].content, "Hello");
    }

    #[test]
    fn test_parse_success_response() {
        let agent = Agent::new("key".to_string(), Duration::from_secs(10));
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
    fn test_parse_error_response() {
        let agent = Agent::new("key".to_string(), Duration::from_secs(10));
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
}

#[cfg(test)]
mod pact_tests {
    use super::*;
    use pact_consumer::prelude::*;

    #[test]
    fn pact_successful_completion() {
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

        let agent = Agent::with_base_url(
            "test_api_key".to_string(),
            Duration::from_secs(10),
            mock_server.url().to_string(),
        );

        let result = agent.send_request("Hello");

        assert!(result.is_ok());
        assert_eq!(result.unwrap().content, "Hello! How can I help you today?");
    }

    #[test]
    fn pact_invalid_api_key() {
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

        let agent = Agent::with_base_url(
            "invalid_key".to_string(),
            Duration::from_secs(10),
            mock_server.url().to_string(),
        );

        let result = agent.send_request("Hello");

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Unauthorized"));
    }

    #[test]
    fn pact_rate_limit() {
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

        let agent = Agent::with_base_url(
            "test_key".to_string(),
            Duration::from_secs(10),
            mock_server.url().to_string(),
        );

        let result = agent.send_request("Hello");

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Rate limit"));
    }

    #[test]
    fn pact_server_error() {
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

        let agent = Agent::with_base_url(
            "test_key".to_string(),
            Duration::from_secs(10),
            mock_server.url().to_string(),
        );

        let result = agent.send_request("Hello");

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Server error"));
    }
}

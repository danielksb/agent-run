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

pub struct Agent {
    api_key: String,
    timeout: Duration,
}

impl Agent {
    pub fn new(api_key: String, timeout: Duration) -> Self {
        Self { api_key, timeout }
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

use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const DEFAULT_MODEL: &str = "gpt-4o-mini";

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
}

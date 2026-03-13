use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct Agent {
    api_key: String,
    timeout: Duration,
}

impl Agent {
    pub fn new(api_key: String, timeout: Duration) -> Self {
        Self { api_key, timeout }
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
}

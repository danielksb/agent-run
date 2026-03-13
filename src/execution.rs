use std::time::Duration;

use crate::agent::{Agent, AgentError, AgentResponse};
use crate::config::AppConfig;

pub fn execute(config: AppConfig) -> Result<AgentResponse, AgentError> {
    let timeout = Duration::from_secs(config.timeout_secs);
    let agent = match config.base_url {
        Some(base_url) => Agent::with_base_url(config.api_key, timeout, base_url),
        None => Agent::new(config.api_key, timeout),
    };
    agent.send_request(&config.prompt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pact_consumer::prelude::*;

    #[test]
    fn test_successful_execution() {
        let mock_server = PactBuilder::new_v4("agent-run", "openai-api")
            .interaction("execution successful completion", "", |mut i| {
                i.request
                    .post()
                    .path("/v1/chat/completions")
                    .header("Authorization", "Bearer test_execution_key")
                    .header("Content-Type", "application/json");
                i.response
                    .ok()
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!({
                        "choices": [{
                            "message": {
                                "role": "assistant",
                                "content": "Execution test response"
                            }
                        }]
                    }));
                i
            })
            .start_mock_server(None, None);

        let config = AppConfig {
            api_key: "test_execution_key".to_string(),
            prompt: "Test prompt".to_string(),
            timeout_secs: 10,
            base_url: Some(mock_server.url().to_string()),
        };

        let result = execute(config);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().content, "Execution test response");
    }

    #[test]
    fn test_execution_propagates_errors() {
        let mock_server = PactBuilder::new_v4("agent-run", "openai-api")
            .interaction("execution error propagation", "", |mut i| {
                i.request
                    .post()
                    .path("/v1/chat/completions");
                i.response
                    .status(401)
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!({
                        "error": {
                            "message": "Invalid API key",
                            "type": "invalid_request_error"
                        }
                    }));
                i
            })
            .start_mock_server(None, None);

        let config = AppConfig {
            api_key: "invalid_key".to_string(),
            prompt: "Test prompt".to_string(),
            timeout_secs: 10,
            base_url: Some(mock_server.url().to_string()),
        };

        let result = execute(config);

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Unauthorized"));
    }
}

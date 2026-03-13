use std::time::Duration;

use crate::agent::{create_agent, AgentError, AgentResponse};
use crate::config::AppConfig;

pub fn execute(config: AppConfig) -> Result<AgentResponse, AgentError> {
    let timeout = Duration::from_secs(config.timeout_secs);
    let agent = create_agent(
        &config.vendor,
        config.api_key,
        timeout,
        config.base_url,
        config.model,
    )?;
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
            vendor: "openai".to_string(),
            model: None,
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
            vendor: "openai".to_string(),
            model: None,
            base_url: Some(mock_server.url().to_string()),
        };

        let result = execute(config);

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Unauthorized"));
    }

    #[test]
    fn test_gemini_execution() {
        let mock_server = PactBuilder::new_v4("agent-run", "gemini-api")
            .interaction("execution gemini completion", "", |mut i| {
                i.request
                    .post()
                    .path("/v1beta/models/gemini-flash-latest:generateContent")
                    .header("X-goog-api-key", "test_gemini_key")
                    .header("Content-Type", "application/json");
                i.response
                    .ok()
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!({
                        "candidates": [{
                            "content": {
                                "parts": [{
                                    "text": "Gemini execution response"
                                }]
                            }
                        }]
                    }));
                i
            })
            .start_mock_server(None, None);

        let config = AppConfig {
            api_key: "test_gemini_key".to_string(),
            prompt: "Test prompt".to_string(),
            timeout_secs: 10,
            vendor: "gemini".to_string(),
            model: None,
            base_url: Some(mock_server.url().to_string()),
        };

        let result = execute(config);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().content, "Gemini execution response");
    }

    #[test]
    fn test_invalid_vendor_returns_error() {
        let config = AppConfig {
            api_key: "key".to_string(),
            prompt: "Test".to_string(),
            timeout_secs: 10,
            vendor: "invalid".to_string(),
            model: None,
            base_url: None,
        };

        let result = execute(config);

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Unknown vendor"));
    }
}

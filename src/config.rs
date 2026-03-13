use clap::Parser;
use std::env;

pub const API_KEY_ENV_VAR: &str = "AGENTRUN_API_KEY";

#[derive(Parser, Debug)]
#[command(name = "agent-run")]
#[command(version, about = "Run LLM agents with a given prompt")]
pub struct Cli {
    /// The prompt to send to the LLM agent
    #[arg(short, long)]
    pub prompt: Option<String>,

    /// Timeout in seconds for the API request
    #[arg(short, long, default_value = "10")]
    pub timeout: u64,
}

#[derive(Debug)]
pub struct AppConfig {
    pub api_key: String,
    pub prompt: String,
    pub timeout_secs: u64,
}

#[derive(Debug)]
pub struct ConfigError {
    pub message: String,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ConfigError {}

pub fn load_api_key() -> Result<String, ConfigError> {
    dotenvy::dotenv().ok();
    env::var(API_KEY_ENV_VAR).map_err(|_| ConfigError {
        message: format!("Environment variable {} is not set", API_KEY_ENV_VAR),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::env;

    #[test]
    fn test_parse_prompt_argument() {
        let cli = Cli::parse_from(["agent-run", "--prompt", "Hello"]);
        assert_eq!(cli.prompt, Some("Hello".to_string()));
    }

    #[test]
    fn test_parse_timeout_argument() {
        let cli = Cli::parse_from(["agent-run", "-p", "Hi", "--timeout", "30"]);
        assert_eq!(cli.timeout, 30);
    }

    #[test]
    fn test_default_timeout() {
        let cli = Cli::parse_from(["agent-run", "-p", "Hi"]);
        assert_eq!(cli.timeout, 10);
    }

    #[test]
    fn test_load_api_key_from_env() {
        // SAFETY: Test runs in single-threaded context
        unsafe { env::set_var(API_KEY_ENV_VAR, "test_key") };
        let result = load_api_key();
        unsafe { env::remove_var(API_KEY_ENV_VAR) };
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_key");
    }

    #[test]
    fn test_missing_api_key_error() {
        // SAFETY: Test runs in single-threaded context
        unsafe { env::remove_var(API_KEY_ENV_VAR) };
        let result = load_api_key();
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains(API_KEY_ENV_VAR));
    }
}

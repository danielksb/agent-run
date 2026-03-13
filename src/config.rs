use clap::Parser;

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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

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
}

mod agent;
mod config;
mod execution;

use std::io::{self, Write};
use std::process::ExitCode;

use agent::AgentResponse;
use clap::Parser;
use config::{get_prompt, load_api_key, AppConfig, Cli};
use execution::execute;

fn write_success<W: Write>(response: &AgentResponse, writer: &mut W) -> io::Result<()> {
    write!(writer, "{}", response.content)
}

fn write_error<W: Write>(error: &str, writer: &mut W) -> io::Result<()> {
    writeln!(writer, "agent-run: error: {}", error)
}

fn run() -> Result<AgentResponse, String> {
    let cli = Cli::parse();

    let api_key = load_api_key().map_err(|e| e.message)?;

    let stdin = io::stdin();
    let prompt = get_prompt(cli.prompt, stdin.lock()).map_err(|e| e.message)?;

    let config = AppConfig {
        api_key,
        prompt,
        timeout_secs: cli.timeout,
        base_url: None,
    };

    execute(config).map_err(|e| e.message)
}

fn main() -> ExitCode {
    match run() {
        Ok(response) => {
            let mut stdout = io::stdout();
            if write_success(&response, &mut stdout).is_err() {
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            let mut stderr = io::stderr();
            let _ = write_error(&error, &mut stderr);
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_success_to_stdout() {
        let response = AgentResponse {
            content: "Hello from LLM!".to_string(),
        };
        let mut output = Vec::new();

        let result = write_success(&response, &mut output);

        assert!(result.is_ok());
        assert_eq!(String::from_utf8(output).unwrap(), "Hello from LLM!");
    }

    #[test]
    fn test_output_error_to_stderr() {
        let error_message = "API key not found";
        let mut output = Vec::new();

        let result = write_error(error_message, &mut output);

        assert!(result.is_ok());
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "agent-run: error: API key not found\n"
        );
    }
}

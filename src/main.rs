mod agent;
mod config;
mod execution;

use std::io::Write;

use agent::AgentResponse;

fn write_success<W: Write>(response: &AgentResponse, writer: &mut W) -> std::io::Result<()> {
    write!(writer, "{}", response.content)
}

fn write_error<W: Write>(error: &str, writer: &mut W) -> std::io::Result<()> {
    writeln!(writer, "agent-run: error: {}", error)
}

fn main() {
    println!("Hello, world!");
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

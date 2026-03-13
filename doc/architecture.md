# agent-run

Small terminal application to run LLM agents with a provided prompt or skill. Designed for easy integration in Bash/PowerShell scripts, unix pipes, cron jobs, or manual command-line use. Supports connecting to different LLM vendors (OpenAI, Google Gemini) or a locally run model (e.g., ollama).


## Execution

- If `--prompt` / `-p` is provided, use it as the system prompt; otherwise read prompt from stdin.
- If environment variable `AGENTRUN_API_KEY` is set, use it as the API key; otherwise exit with an error.
- Load configuration from TOML file (see Configuration File section).
- Send the prompt to the selected LLM vendor via the raw HTTP API.
- Wait for a response (timeout configurable, default 10 seconds).
- On success: write response text to stdout and exit 0.
- On error: write an error message to stderr and exit nonzero.

## Components

- **Configuration**: gather settings from TOML file, CLI arguments, and environment variables into an `AppConfig` object.
- **Agent**: trait-based abstraction for LLM vendor communication.
- **Vendors**: concrete implementations for each LLM provider (OpenAI, Gemini).
- **Execution**: takes `AppConfig` and returns `AgentResponse` using the appropriate vendor.

## Configuration File

The application uses a TOML configuration file for vendor-specific settings.

### File Location

1. Path specified via `--config <path>` CLI argument (highest priority)
2. `~/.agent-run.toml` in user's home directory (default)
3. If no config file found, use built-in defaults

### TOML Structure

```toml
[general]
timeout = 30          # Request timeout in seconds (default: 10)
default_vendor = "openai"  # Default vendor if --vendor not specified

[openai]
base_url = "https://api.openai.com"
model = "gpt-4o-mini"

[gemini]
base_url = "https://generativelanguage.googleapis.com"
model = "gemini-2.0-flash"
```

### Configuration Priority

Settings are merged with the following priority (highest to lowest):
1. CLI arguments (`--timeout`, `--vendor`, `--model`)
2. Environment variables (`AGENTRUN_API_KEY`)
3. TOML configuration file
4. Built-in defaults

## Supported Vendors

| Vendor | API Endpoint | Auth Header |
|--------|-------------|-------------|
| OpenAI | `/v1/chat/completions` | `Authorization: Bearer <key>` |
| Gemini | `/v1beta/models/{model}:generateContent` | `x-goog-api-key: <key>` |

## Test plan

Follow test-driven development (TDD):
1. Write the test first.
2. Confirm the test fails with the expected error.
3. Implement functionality to pass the test.

Use pact tests to validate LLM interactions.
The program is the consumer and stores pact files in the `pacts` directory.

## Technical decisions

- Use rust edition 2024
- Use dotenvy for retrieving environment variables
- Use ureq for http requests, since no async requests are necessary
- Use toml crate for configuration file parsing
- Use dirs crate for cross-platform home directory resolution
- Use trait-based abstraction for vendor implementations

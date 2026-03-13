# agent-run

Small, single-file terminal application to run LLM agents with a provided prompt or skill. Designed for easy integration in Bash/PowerShell scripts, unix pipes, cron jobs, or manual command-line use. Supports connecting to different LLM vendors or a locally run model (e.g., ollama).


## Execution

- If `--prompt` / `-p` is provided, use it as the system prompt; otherwise read prompt from stdin.
- If environment variable `AGENTRUN_API_KEY` is set, use it as the API key; otherwise exit with an error.
- Send the prompt to the LLM agent (currently OpenAI only) via the raw HTTP API.
- Wait up to 10 seconds for a response.
- On success: write response text to stdout and exit 0.
- On error: write an error message to stderr and exit nonzero.

## Components

- Configuration: gather command-line arguments and environment variables into an `AppConfig` object.
- Agent: encapsulate agent communication behind a simple API.
- Execution: takes `AppConfig` and returns `AgentResponse` using the `Agent` component.

## Test plan

Follow test-driven development (TDD):
1. Write the test first.
2. Confirm the test fails with the expected error.
3. Implement functionality to pass the test.

Use pact tests to validate LLM interactions.
The program is the consumer and stores pact files in the `pacts` directory.

## Technical decisions

- Use rust edition 2024
- Use dotenv for retrieving environment variables
- Use ureq for http requests, since no async requests are necessary.

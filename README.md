# agent-run

A command-line tool to send prompts to LLM APIs (OpenAI, Google Gemini) and receive responses.

## What it does

`agent-run` takes a text prompt and sends it to an LLM API, then outputs the response. It supports:

- **OpenAI** (GPT models)
- **Google Gemini**

## Installation

```bash
cargo build --release
```

The binary will be at `target/release/agent-run`.

## Configuration

### API Key

Set your API key as an environment variable:

```bash
# Linux/macOS
export AGENTRUN_API_KEY=your_api_key_here

# Windows PowerShell
$env:AGENTRUN_API_KEY = "your_api_key_here"
```

Or create a `.env` file in the working directory:

```
AGENTRUN_API_KEY=your_api_key_here
```

### Configuration File (Optional)

Create `~/.agent-run.toml` or specify a path with `--config`:

```toml
[general]
timeout = 30                    # Request timeout in seconds
default_vendor = "gemini"       # Default: "openai"

[openai]
base_url = "https://api.openai.com"
model = "gpt-4o-mini"

[gemini]
base_url = "https://generativelanguage.googleapis.com"
model = "gemini-flash-latest"
```

## Usage

### Basic usage

```bash
# Using --prompt flag
agent-run --prompt "What is the capital of France?"

# Using stdin
echo "What is 2+2?" | agent-run

# Pipe file contents
cat question.txt | agent-run
```

### CLI Options

```
Options:
  -p, --prompt <PROMPT>    The prompt to send to the LLM
  -t, --timeout <TIMEOUT>  Timeout in seconds for the API request
  -c, --config <CONFIG>    Path to configuration file
  -v, --vendor <VENDOR>    LLM vendor: "openai" or "gemini"
  -m, --model <MODEL>      Model name (overrides config file)
  -h, --help               Print help
  -V, --version            Print version
```

### Examples

```bash
# Use Gemini instead of OpenAI
agent-run --vendor gemini --prompt "Hello"

# Use a specific model
agent-run --vendor openai --model gpt-4 --prompt "Explain quantum computing"

# Use a custom config file
agent-run --config ./my-config.toml --prompt "Hello"

# Longer timeout for complex queries
agent-run --timeout 60 --prompt "Write a detailed essay about climate change"
```

## Configuration Priority

Settings are applied in this order (highest priority first):

1. CLI arguments (`--timeout`, `--vendor`, `--model`)
2. Configuration file (`~/.agent-run.toml` or `--config`)
3. Built-in defaults

## License

MIT

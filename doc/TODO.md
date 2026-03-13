# Implementation Plan for agent-run

This document outlines the step-by-step plan to implement the `agent-run` CLI application as described in `architecture.md`.

**Development Approach:** Test-Driven Development (TDD)
- Write the test first
- Verify the test fails with the expected error message
- Implement the functionality to satisfy the test conditions

---

## Architecture Components

The application is structured into three main components:

| Component | Responsibility | Input | Output |
|-----------|---------------|-------|--------|
| **AppConfig** | Collects config from CLI args and environment variables | CLI args, env vars | `AppConfig` struct |
| **Agent** | Wraps communication with the LLM in a convenient API | Prompt, config | `AgentResponse` |
| **Execution** | Orchestrates the application flow | `AppConfig` | `AgentResponse` |

---

## Phase 1: Project Setup ✓

### 1.1 Update Cargo.toml
- [x] Keep Rust edition 2024 as specified
- [x] Add required dependencies:
  - `clap` - CLI argument parsing with derive macros
  - `ureq` - Synchronous HTTP client (no async complexity needed)
  - `serde` / `serde_json` - JSON serialization for OpenAI API
  - `dotenvy` - Environment variable loading from `.env` files
  - `pact_consumer` - Pact testing framework (dev dependency)

### 1.2 Create .env.template
- [x] Create `.env.template` file with `AGENTRUN_API_KEY=your_key_here`
- [x] Add `.env` to `.gitignore` to prevent accidental commits of secrets
- [x] Add `pacts/` to `.gitignore`

---

## Phase 2: AppConfig Component (TDD)

The `AppConfig` component collects all configuration from CLI arguments and environment variables.

### 2.1 Define AppConfig Struct ✓
```rust
struct AppConfig {
    api_key: String,
    prompt: String,
    timeout_secs: u64,
    // Future: model, vendor
}
```
Also defined `Cli` struct with clap derive macros for argument parsing.

### 2.2 TDD: CLI Argument Parsing ✓

#### Test: Parse --prompt argument ✓
- [x] **Write test:** `test_parse_prompt_argument`
  - Given: CLI args `["agent-run", "--prompt", "Hello"]`
  - Expected: `cli.prompt == Some("Hello")`
- [x] **Verify test passes** (Cli struct already implemented in 2.1)
- [x] **Implement:** Cli struct with clap derive macros already has `-p`/`--prompt` argument

#### Test: Parse --timeout argument ✓
- [x] **Write test:** `test_parse_timeout_argument`
  - Given: CLI args `["agent-run", "-p", "Hi", "--timeout", "30"]`
  - Expected: `cli.timeout == 30`
- [x] **Verify test passes**
- [x] **Implement:** `-t`/`--timeout` argument with default value of 10

#### Test: Default timeout value ✓
- [x] **Write test:** `test_default_timeout`
  - Given: CLI args without `--timeout`
  - Expected: `cli.timeout == 10`
- [x] **Verify test passes**
- [x] **Implement:** Default value set in clap

### 2.3 TDD: Environment Variable Loading ✓

#### Test: Load API key from environment ✓
- [x] **Write test:** `test_load_api_key_from_env`
  - Given: `AGENTRUN_API_KEY=test_key` in environment
  - Expected: `load_api_key() == Ok("test_key")`
- [x] **Verify test passes**
- [x] **Implement:** `load_api_key()` uses `dotenvy::dotenv().ok()` then `std::env::var("AGENTRUN_API_KEY")`

#### Test: Missing API key error ✓
- [x] **Write test:** `test_missing_api_key_error`
  - Given: No `AGENTRUN_API_KEY` in environment
  - Expected: Error with message containing "AGENTRUN_API_KEY"
- [x] **Verify test passes**
- [x] **Implement:** Returns `ConfigError` with descriptive message when env var missing

### 2.4 TDD: Stdin Input Handling ✓

#### Test: Read prompt from stdin when no --prompt ✓
- [x] **Write test:** `test_read_prompt_from_stdin`
  - Given: No `--prompt` arg, stdin contains "Hello from stdin"
  - Expected: `get_prompt(None, reader) == Ok("Hello from stdin")`
- [x] **Verify test passes**
- [x] **Implement:** `get_prompt()` reads from provided reader when prompt arg is None

#### Test: Empty prompt error ✓
- [x] **Write test:** `test_empty_prompt_error`
  - Given: Empty prompt (whitespace only)
  - Expected: Error with message about empty prompt
- [x] **Verify test passes**
- [x] **Implement:** Validates prompt is not empty after trimming

**Additional tests added:**
- `test_prompt_from_cli_argument` - CLI argument takes precedence over stdin
- `test_prompt_trimmed` - Prompt whitespace is trimmed

---

## Phase 3: Agent Component (TDD)

The `Agent` component wraps communication with the LLM API.

### 3.1 Define Agent Types ✓
Created `src/agent.rs` with:
```rust
struct Agent {
    api_key: String,
    timeout: Duration,
}

struct AgentResponse {
    content: String,
}

// OpenAI API types (with Serialize/Deserialize)
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

struct Message {
    role: String,
    content: String,
}

struct ChatResponse {
    choices: Vec<Choice>,
}

struct Choice {
    message: Message,
}
```

### 3.2 TDD: Agent Creation ✓

#### Test: Create agent with config ✓
- [x] **Write test:** `test_create_agent`
  - Given: API key and timeout
  - Expected: Agent instance created successfully
- [x] **Verify test passes**
- [x] **Implement:** `Agent::new(api_key, timeout)` constructor

### 3.3 TDD: Request Building ✓

#### Test: Build chat request ✓
- [x] **Write test:** `test_build_chat_request`
  - Given: Prompt "Hello"
  - Expected: `ChatRequest` with user message containing "Hello"
- [x] **Verify test passes**
- [x] **Implement:** `Agent::build_chat_request(prompt)` method

### 3.4 TDD: Response Parsing ✓

#### Test: Parse successful response ✓
- [x] **Write test:** `test_parse_success_response`
  - Given: Valid JSON response with `choices[0].message.content`
  - Expected: `AgentResponse.content` extracted correctly
- [x] **Verify test passes**
- [x] **Implement:** `Agent::parse_response(json)` with `ApiResponse` enum

#### Test: Parse error response ✓
- [x] **Write test:** `test_parse_error_response`
  - Given: JSON with error object
  - Expected: `AgentError` with message containing API error
- [x] **Verify test passes**
- [x] **Implement:** Handle API error responses via `#[serde(untagged)]` enum

### 3.5 Pact Tests: LLM API Interaction ✓

**Pact file output:**
- Default location: `target/pacts/` (used when running `cargo test`)
- Custom location: Set `PACT_OUTPUT_DIR=pacts` in `.env` (copy from `.env.template`)
- Generated file: `agent-run-openai-api.json`

#### Pact: Successful chat completion ✓
- [x] **Write pact test:** `pact_successful_completion`
  - Define expected request: POST to `/v1/chat/completions`
  - Define expected response: 200 with choices array
  - Mock OpenAI endpoint
- [x] **Verify pact test works with mock**
- [x] **Implement:** `Agent::send_request()` using `ureq`
  - Build POST request to `{base_url}/v1/chat/completions`
  - Set `Authorization: Bearer <API_KEY>` header
  - Set `Content-Type: application/json` header
  - Use `.send_json()` for request body
  - Configure timeout via `ureq::AgentBuilder`

#### Pact: Invalid API key (401) ✓
- [x] **Write pact test:** `pact_invalid_api_key`
  - Expected response: 401 Unauthorized
- [x] **Verify test passes**
- [x] **Implement:** Handle 401 response with "Unauthorized: Invalid API key"

#### Pact: Rate limit exceeded (429) ✓
- [x] **Write pact test:** `pact_rate_limit`
  - Expected response: 429 Too Many Requests
- [x] **Verify test passes**
- [x] **Implement:** Handle 429 response with "Rate limit exceeded"

#### Pact: Server error (500) ✓
- [x] **Write pact test:** `pact_server_error`
  - Expected response: 500 Internal Server Error
- [x] **Verify test passes**
- [x] **Implement:** Handle 5xx responses with "Server error: {status}"

---

## Phase 4: Execution Component (TDD) ✓

The `Execution` component orchestrates the application flow.

### 4.1 Define Execution Interface ✓
Created `src/execution.rs` with:
```rust
pub fn execute(config: AppConfig) -> Result<AgentResponse, AgentError>;
```

### 4.2 TDD: Execution Flow ✓

#### Test: Successful execution ✓
- [x] **Write test:** `test_successful_execution`
  - Given: Valid `AppConfig` with prompt and API key
  - Expected: `AgentResponse` with content
- [x] **Verify test passes**
- [x] **Implement:** 
  1. Create `Agent` from config (with optional base_url)
  2. Call `agent.send_request()` with prompt
  3. Return response

#### Test: Propagate agent errors ✓
- [x] **Write test:** `test_execution_propagates_errors`
  - Given: Agent returns error (401)
  - Expected: Error propagated from execution
- [x] **Verify test passes**
- [x] **Implement:** Error propagation via `Result` type

**Note:** Added `base_url: Option<String>` to `AppConfig` for test mocking

---

## Phase 5: Main Function & Output

### 5.1 TDD: Output Formatting ✓

#### Test: Write success to stdout ✓
- [x] **Write test:** `test_output_success_to_stdout`
  - Given: Successful `AgentResponse`
  - Expected: Content written to writer (no trailing newline)
- [x] **Verify test passes**
- [x] **Implement:** `write_success(response, writer)` using generic `Write` trait

#### Test: Write error to stderr ✓
- [x] **Write test:** `test_output_error_to_stderr`
  - Given: Error message
  - Expected: Formatted error message written to writer
- [x] **Verify test passes**
- [x] **Implement:** `write_error(error, writer)` with format `agent-run: error: <message>`

### 5.2 Implement main() ✓
- [x] Wire together all components:
```rust
fn run() -> Result<AgentResponse, String> {
    let cli = Cli::parse();
    let api_key = load_api_key()?;
    let prompt = get_prompt(cli.prompt, stdin.lock())?;
    let config = AppConfig { api_key, prompt, timeout_secs, base_url: None };
    execute(config)
}

fn main() -> ExitCode {
    match run() {
        Ok(response) -> write to stdout, return SUCCESS
        Err(error) -> write to stderr, return FAILURE
    }
}
```

### 5.3 Error Message Format ✓
- [x] Use format: `agent-run: error: <description>`
- [x] Error messages include relevant details (API error messages propagated)

---

## Phase 6: Integration & Manual Testing

### 6.1 Integration Tests
- [ ] Test full flow with mock server
- [ ] Test error scenarios end-to-end

### 6.2 Manual Testing
- [ ] Test with `--prompt "Hello"` argument
- [ ] Test with stdin input: `echo "Hello" | agent-run`
- [ ] Test missing API key error
- [ ] Test empty prompt error
- [ ] Test network timeout
- [ ] Test invalid API key error

### 6.3 Script Integration
- [ ] Test in PowerShell script
- [ ] Test in Bash script
- [ ] Test pipe chaining: `cat file.txt | agent-run | other-command`

---

## Phase 7: TOML Configuration File (TDD)

Support for configuration via TOML file with vendor-specific settings.

### 7.1 Add Dependencies
- [ ] Add `toml` crate for TOML parsing
- [ ] Add `dirs` crate for cross-platform home directory

### 7.2 Define Configuration Structs
```rust
#[derive(Deserialize, Default)]
struct TomlConfig {
    general: Option<GeneralConfig>,
    openai: Option<VendorConfig>,
    gemini: Option<VendorConfig>,
}

#[derive(Deserialize, Default)]
struct GeneralConfig {
    timeout: Option<u64>,
    default_vendor: Option<String>,
}

#[derive(Deserialize, Default)]
struct VendorConfig {
    base_url: Option<String>,
    model: Option<String>,
}
```

### 7.3 TDD: Config File Loading

#### Test: Load config from specified path
- [ ] **Write test:** `test_load_config_from_path`
  - Given: `--config /path/to/config.toml`
  - Expected: Config loaded from specified path

#### Test: Load config from home directory
- [ ] **Write test:** `test_load_config_from_home`
  - Given: No `--config` arg, `~/.agent-run.toml` exists
  - Expected: Config loaded from home directory

#### Test: Default config when file missing
- [ ] **Write test:** `test_default_config_when_missing`
  - Given: No config file exists
  - Expected: Built-in defaults used

#### Test: Parse general section
- [ ] **Write test:** `test_parse_general_section`
  - Given: TOML with `[general]` section
  - Expected: timeout and default_vendor parsed

#### Test: Parse vendor sections
- [ ] **Write test:** `test_parse_vendor_sections`
  - Given: TOML with `[openai]` and `[gemini]` sections
  - Expected: base_url and model parsed for each vendor

### 7.4 TDD: CLI Config Flag

#### Test: --config flag
- [ ] **Write test:** `test_config_cli_flag`
  - Given: CLI args with `--config path/to/file.toml`
  - Expected: Config path stored in CLI struct

#### Test: --vendor flag
- [ ] **Write test:** `test_vendor_cli_flag`
  - Given: CLI args with `--vendor gemini`
  - Expected: Vendor selection stored in CLI struct

#### Test: --model flag
- [ ] **Write test:** `test_model_cli_flag`
  - Given: CLI args with `--model gpt-4`
  - Expected: Model override stored in CLI struct

### 7.5 TDD: Configuration Merging

#### Test: CLI overrides config file
- [ ] **Write test:** `test_cli_overrides_config`
  - Given: Config file has timeout=30, CLI has --timeout 60
  - Expected: Final timeout is 60

#### Test: Config file overrides defaults
- [ ] **Write test:** `test_config_overrides_defaults`
  - Given: Config file has timeout=30, no CLI override
  - Expected: Final timeout is 30

---

## Phase 8: Multi-Vendor Agent Abstraction (TDD)

Abstract agent communication behind a trait to support multiple vendors.

### 8.1 Define Agent Trait
```rust
pub trait LlmAgent {
    fn send_request(&self, prompt: &str) -> Result<AgentResponse, AgentError>;
}
```

### 8.2 TDD: Refactor OpenAI Agent

#### Test: OpenAI implements LlmAgent trait
- [ ] **Write test:** `test_openai_implements_trait`
  - Given: OpenAI agent instance
  - Expected: Can call trait methods

#### Test: OpenAI uses configured model
- [ ] **Write test:** `test_openai_uses_configured_model`
  - Given: Config with model="gpt-4"
  - Expected: Request uses specified model

#### Test: OpenAI uses configured base_url
- [ ] **Write test:** `test_openai_uses_configured_base_url`
  - Given: Config with custom base_url
  - Expected: Request sent to custom URL

### 8.3 TDD: Implement Gemini Agent

#### Test: Gemini request format
- [ ] **Write test:** `test_gemini_request_format`
  - Given: Prompt "Hello"
  - Expected: Request body matches Gemini API format
  ```json
  {
    "contents": [{
      "parts": [{"text": "Hello"}]
    }]
  }
  ```

#### Test: Gemini response parsing
- [ ] **Write test:** `test_gemini_response_parsing`
  - Given: Valid Gemini API response
  - Expected: Content extracted from `candidates[0].content.parts[0].text`

#### Test: Gemini authentication header
- [ ] **Write test:** `test_gemini_auth_header`
  - Given: API key
  - Expected: Request has `x-goog-api-key` header (not Bearer token)

#### Test: Gemini URL format
- [ ] **Write test:** `test_gemini_url_format`
  - Given: Model "gemini-2.0-flash"
  - Expected: URL is `{base_url}/v1beta/models/gemini-2.0-flash:generateContent`

### 8.4 Pact Tests: Gemini API

#### Pact: Gemini successful completion
- [ ] **Write pact test:** `pact_gemini_successful_completion`
  - Define expected Gemini request/response format
  - Mock Gemini endpoint

#### Pact: Gemini error responses
- [ ] **Write pact test:** `pact_gemini_error_responses`
  - Test 401, 429, 500 error handling

### 8.5 TDD: Vendor Selection

#### Test: Select vendor from config
- [ ] **Write test:** `test_select_vendor_from_config`
  - Given: Config with default_vendor="gemini"
  - Expected: Gemini agent used

#### Test: Select vendor from CLI flag
- [ ] **Write test:** `test_select_vendor_from_cli`
  - Given: --vendor openai
  - Expected: OpenAI agent used (overrides config)

#### Test: Invalid vendor error
- [ ] **Write test:** `test_invalid_vendor_error`
  - Given: --vendor unknown
  - Expected: Error with list of valid vendors

---

## Phase 9: Update Execution & Main

### 9.1 Update AppConfig
- [ ] Add `vendor: String` field
- [ ] Add `model: Option<String>` field
- [ ] Add `config_path: Option<PathBuf>` field

### 9.2 Update main() Flow
```
1. Parse CLI args
2. Load TOML config (from --config or ~/.agent-run.toml)
3. Merge config: CLI > TOML > defaults
4. Load API key from environment
5. Get prompt from CLI or stdin
6. Create appropriate vendor agent
7. Execute and handle result
```

### 9.3 Integration Testing
- [ ] Test OpenAI with config file
- [ ] Test Gemini with config file
- [ ] Test vendor switching via CLI
- [ ] Test config file in home directory

---

## Future Phases

### Phase 10: Additional Vendors
- [ ] Implement Ollama support (local LLM)
- [ ] Implement Anthropic Claude support
- [ ] Implement Azure OpenAI support

### Phase 11: Tool Support
- [ ] Implement web search tool
- [ ] Implement filesystem access tools
- [ ] Handle tool calls in API response
- [ ] Execute tools and continue conversation

### Phase 12: Enhanced Features
- [ ] Streaming response output
- [ ] Conversation history from file
- [ ] Custom system prompts from file
- [ ] JSON output mode

---

## Implementation Order (TDD)

**MVP Implementation Sequence:**

| Step | Phase | Component | TDD Cycle |
|------|-------|-----------|-----------|
| 1 | Phase 1 | Setup | Dependencies & project structure |
| 2 | Phase 2.2 | AppConfig | CLI argument parsing |
| 3 | Phase 2.3 | AppConfig | Environment variable loading |
| 4 | Phase 2.4 | AppConfig | Stdin input handling |
| 5 | Phase 3.2-3.4 | Agent | Unit tests for request/response |
| 6 | Phase 3.5 | Agent | Pact tests for HTTP API |
| 7 | Phase 4.2 | Execution | Orchestration logic |
| 8 | Phase 5.1 | Main | Output formatting |
| 9 | Phase 5.2 | Main | Wire everything together |
| 10 | Phase 6 | All | Integration & manual testing |

**For each TDD step:**
1. Write the test
2. Run test → verify it fails
3. Write minimal code to pass
4. Refactor if needed
5. Move to next test

---

## Technical Decisions (from architecture.md)

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Rust Edition | 2024 | As specified in architecture |
| HTTP Client | `ureq` | No async needed, simpler synchronous API |
| Environment Variables | `dotenvy` | Load from `.env` files for local development |
| Testing | TDD + Pact | Unit tests first, Pact for API contract testing |
| Architecture | 3 Components | AppConfig, Agent, Execution |

---

## Dependencies Summary

```toml
[package]
edition = "2024"

[dependencies]
clap = { version = "4", features = ["derive"] }
ureq = { version = "2", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
dotenvy = "0.15"

[dev-dependencies]
pact_consumer = "1"
```

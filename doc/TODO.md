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

### 3.3 TDD: Request Building

#### Test: Build chat request
- [ ] **Write test:** `test_build_chat_request`
  - Given: Prompt "Hello"
  - Expected: `ChatRequest` with user message containing "Hello"
- [ ] **Verify test fails**
- [ ] **Implement:** Method to build `ChatRequest` from prompt

### 3.4 TDD: Response Parsing

#### Test: Parse successful response
- [ ] **Write test:** `test_parse_success_response`
  - Given: Valid JSON response with `choices[0].message.content`
  - Expected: `AgentResponse.content` extracted correctly
- [ ] **Verify test fails**
- [ ] **Implement:** JSON deserialization into `ChatResponse`

#### Test: Parse error response
- [ ] **Write test:** `test_parse_error_response`
  - Given: JSON with error object
  - Expected: Appropriate error returned
- [ ] **Verify test fails**
- [ ] **Implement:** Handle API error responses

### 3.5 Pact Tests: LLM API Interaction

#### Pact: Successful chat completion
- [ ] **Write pact test:** `pact_successful_completion`
  - Define expected request: POST to `/v1/chat/completions`
  - Define expected response: 200 with choices array
  - Mock OpenAI endpoint
- [ ] **Verify pact test works with mock**
- [ ] **Implement:** HTTP request using `ureq`
  - Build POST request to `https://api.openai.com/v1/chat/completions`
  - Set `Authorization: Bearer <API_KEY>` header
  - Set `Content-Type: application/json` header
  - Use `.send_json()` for request body
  - Configure timeout via `ureq::agent()`

#### Pact: Invalid API key (401)
- [ ] **Write pact test:** `pact_invalid_api_key`
  - Expected response: 401 Unauthorized
- [ ] **Verify test fails**
- [ ] **Implement:** Handle 401 response with appropriate error

#### Pact: Rate limit exceeded (429)
- [ ] **Write pact test:** `pact_rate_limit`
  - Expected response: 429 Too Many Requests
- [ ] **Verify test fails**
- [ ] **Implement:** Handle 429 response with appropriate error

#### Pact: Server error (500)
- [ ] **Write pact test:** `pact_server_error`
  - Expected response: 500 Internal Server Error
- [ ] **Verify test fails**
- [ ] **Implement:** Handle 5xx responses with appropriate error

---

## Phase 4: Execution Component (TDD)

The `Execution` component orchestrates the application flow.

### 4.1 Define Execution Interface
```rust
fn execute(config: AppConfig) -> Result<AgentResponse, Error>;
```

### 4.2 TDD: Execution Flow

#### Test: Successful execution
- [ ] **Write test:** `test_successful_execution`
  - Given: Valid `AppConfig` with prompt and API key
  - Expected: `AgentResponse` with content
- [ ] **Verify test fails**
- [ ] **Implement:** 
  1. Create `Agent` from config
  2. Call agent with prompt
  3. Return response

#### Test: Propagate agent errors
- [ ] **Write test:** `test_execution_propagates_errors`
  - Given: Agent returns error
  - Expected: Error propagated from execution
- [ ] **Verify test fails**
- [ ] **Implement:** Error propagation

---

## Phase 5: Main Function & Output

### 5.1 TDD: Output Formatting

#### Test: Write success to stdout
- [ ] **Write test:** `test_output_success_to_stdout`
  - Given: Successful `AgentResponse`
  - Expected: Content written to stdout, exit code 0
- [ ] **Verify test fails**
- [ ] **Implement:** Print response content to stdout

#### Test: Write error to stderr
- [ ] **Write test:** `test_output_error_to_stderr`
  - Given: Error result
  - Expected: Error message written to stderr, exit code 1
- [ ] **Verify test fails**
- [ ] **Implement:** Print formatted error to stderr

### 5.2 Implement main()
- [ ] Wire together all components:
```
1. Build AppConfig from CLI args and env
2. Call execute(config)
3. Match result:
   - Ok(response) -> print to stdout, exit 0
   - Err(error) -> print to stderr, exit 1
```

### 5.3 Error Message Format
- [ ] Use format: `agent-run: error: <description>`
- [ ] Include relevant details (HTTP status code, API error message)

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

## Future Phases (Out of Initial Scope)

### Phase 7: Multiple LLM Vendors
- [ ] Abstract Agent behind trait
- [ ] Implement Ollama support (local LLM)
- [ ] Implement other vendors (Anthropic, etc.)
- [ ] Add `--vendor` CLI flag

### Phase 8: Tool Support
- [ ] Implement web search tool
- [ ] Implement filesystem access tools
- [ ] Handle tool calls in API response
- [ ] Execute tools and continue conversation

### Phase 9: Enhanced Features
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

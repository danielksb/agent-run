use clap::Parser;
use serde::Deserialize;
use std::env;
use std::fs;
use std::io::BufRead;
use std::path::PathBuf;

pub const API_KEY_ENV_VAR: &str = "AGENTRUN_API_KEY";
pub const DEFAULT_CONFIG_FILENAME: &str = ".agent-run.toml";
pub const DEFAULT_TIMEOUT: u64 = 10;
pub const DEFAULT_VENDOR: &str = "openai";

#[derive(Parser, Debug)]
#[command(name = "agent-run")]
#[command(version, about = "Run LLM agents with a given prompt")]
pub struct Cli {
    /// The prompt to send to the LLM agent
    #[arg(short, long)]
    pub prompt: Option<String>,

    /// Timeout in seconds for the API request
    #[arg(short, long)]
    pub timeout: Option<u64>,

    /// Path to configuration file
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// LLM vendor to use (openai, gemini)
    #[arg(short, long)]
    pub vendor: Option<String>,

    /// Model name to use (overrides config file)
    #[arg(short, long)]
    pub model: Option<String>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct TomlConfig {
    pub general: Option<GeneralConfig>,
    pub openai: Option<VendorConfig>,
    pub gemini: Option<VendorConfig>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct GeneralConfig {
    pub timeout: Option<u64>,
    pub default_vendor: Option<String>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct VendorConfig {
    pub base_url: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub api_key: String,
    pub prompt: String,
    pub timeout_secs: u64,
    pub vendor: String,
    pub model: Option<String>,
    pub base_url: Option<String>,
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

pub fn get_prompt<R: BufRead>(cli_prompt: Option<String>, mut reader: R) -> Result<String, ConfigError> {
    let prompt = match cli_prompt {
        Some(p) => p,
        None => {
            let mut input = String::new();
            reader.read_to_string(&mut input).map_err(|e| ConfigError {
                message: format!("Failed to read from stdin: {}", e),
            })?;
            input
        }
    };

    let trimmed = prompt.trim();
    if trimmed.is_empty() {
        return Err(ConfigError {
            message: "Prompt cannot be empty".to_string(),
        });
    }

    Ok(trimmed.to_string())
}

pub fn get_default_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(DEFAULT_CONFIG_FILENAME))
}

pub fn load_toml_config(path: Option<&PathBuf>) -> Result<TomlConfig, ConfigError> {
    let config_path = match path {
        Some(p) => {
            if !p.exists() {
                return Err(ConfigError {
                    message: format!("Config file not found: {}", p.display()),
                });
            }
            Some(p.clone())
        }
        None => get_default_config_path().filter(|p| p.exists()),
    };

    match config_path {
        Some(p) => {
            let content = fs::read_to_string(&p).map_err(|e| ConfigError {
                message: format!("Failed to read config file {}: {}", p.display(), e),
            })?;
            toml::from_str(&content).map_err(|e| ConfigError {
                message: format!("Failed to parse config file {}: {}", p.display(), e),
            })
        }
        None => Ok(TomlConfig::default()),
    }
}

pub fn get_vendor_config(toml_config: &TomlConfig, vendor: &str) -> VendorConfig {
    match vendor {
        "openai" => toml_config.openai.clone().unwrap_or_default(),
        "gemini" => toml_config.gemini.clone().unwrap_or_default(),
        _ => VendorConfig::default(),
    }
}

pub fn merge_config(cli: &Cli, toml_config: &TomlConfig) -> (u64, String, Option<String>, Option<String>) {
    let general = toml_config.general.clone().unwrap_or_default();

    let timeout = cli
        .timeout
        .or(general.timeout)
        .unwrap_or(DEFAULT_TIMEOUT);

    let vendor = cli
        .vendor
        .clone()
        .or(general.default_vendor)
        .unwrap_or_else(|| DEFAULT_VENDOR.to_string());

    let vendor_config = get_vendor_config(toml_config, &vendor);

    let model = cli.model.clone().or(vendor_config.model);
    let base_url = vendor_config.base_url;

    (timeout, vendor, model, base_url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::env;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_prompt_argument() {
        let cli = Cli::parse_from(["agent-run", "--prompt", "Hello"]);
        assert_eq!(cli.prompt, Some("Hello".to_string()));
    }

    #[test]
    fn test_parse_timeout_argument() {
        let cli = Cli::parse_from(["agent-run", "-p", "Hi", "--timeout", "30"]);
        assert_eq!(cli.timeout, Some(30));
    }

    #[test]
    fn test_default_timeout_is_none() {
        let cli = Cli::parse_from(["agent-run", "-p", "Hi"]);
        assert_eq!(cli.timeout, None);
    }

    #[test]
    fn test_config_cli_flag() {
        let cli = Cli::parse_from(["agent-run", "-p", "Hi", "--config", "/path/to/config.toml"]);
        assert_eq!(cli.config, Some(PathBuf::from("/path/to/config.toml")));
    }

    #[test]
    fn test_vendor_cli_flag() {
        let cli = Cli::parse_from(["agent-run", "-p", "Hi", "--vendor", "gemini"]);
        assert_eq!(cli.vendor, Some("gemini".to_string()));
    }

    #[test]
    fn test_model_cli_flag() {
        let cli = Cli::parse_from(["agent-run", "-p", "Hi", "--model", "gpt-4"]);
        assert_eq!(cli.model, Some("gpt-4".to_string()));
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

    #[test]
    fn test_read_prompt_from_stdin() {
        let input = b"Hello from stdin";
        let reader = &input[..];

        let result = get_prompt(None, reader);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello from stdin");
    }

    #[test]
    fn test_prompt_from_cli_argument() {
        let input = b"ignored stdin";
        let reader = &input[..];

        let result = get_prompt(Some("CLI prompt".to_string()), reader);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "CLI prompt");
    }

    #[test]
    fn test_empty_prompt_error() {
        let input = b"   \n\t  ";
        let reader = &input[..];

        let result = get_prompt(None, reader);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.to_lowercase().contains("empty"));
    }

    #[test]
    fn test_prompt_trimmed() {
        let input = b"  hello world  \n";
        let reader = &input[..];

        let result = get_prompt(None, reader);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello world");
    }

    #[test]
    fn test_load_config_from_path() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[general]
timeout = 30
default_vendor = "gemini"

[openai]
base_url = "https://api.openai.com"
model = "gpt-4o"

[gemini]
base_url = "https://generativelanguage.googleapis.com"
model = "gemini-2.0-flash"
"#
        )
        .unwrap();

        let path = file.path().to_path_buf();
        let result = load_toml_config(Some(&path));

        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.general.as_ref().unwrap().timeout, Some(30));
        assert_eq!(
            config.general.as_ref().unwrap().default_vendor,
            Some("gemini".to_string())
        );
    }

    #[test]
    fn test_default_config_when_missing() {
        let result = load_toml_config(None);

        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.general.is_none());
        assert!(config.openai.is_none());
        assert!(config.gemini.is_none());
    }

    #[test]
    fn test_error_when_specified_config_missing() {
        let path = PathBuf::from("/nonexistent/config.toml");
        let result = load_toml_config(Some(&path));

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("not found"));
    }

    #[test]
    fn test_parse_general_section() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[general]
timeout = 45
default_vendor = "openai"
"#
        )
        .unwrap();

        let path = file.path().to_path_buf();
        let config = load_toml_config(Some(&path)).unwrap();

        assert_eq!(config.general.as_ref().unwrap().timeout, Some(45));
        assert_eq!(
            config.general.as_ref().unwrap().default_vendor,
            Some("openai".to_string())
        );
    }

    #[test]
    fn test_parse_vendor_sections() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[openai]
base_url = "https://custom-openai.example.com"
model = "gpt-4-turbo"

[gemini]
base_url = "https://custom-gemini.example.com"
model = "gemini-pro"
"#
        )
        .unwrap();

        let path = file.path().to_path_buf();
        let config = load_toml_config(Some(&path)).unwrap();

        let openai = config.openai.as_ref().unwrap();
        assert_eq!(
            openai.base_url,
            Some("https://custom-openai.example.com".to_string())
        );
        assert_eq!(openai.model, Some("gpt-4-turbo".to_string()));

        let gemini = config.gemini.as_ref().unwrap();
        assert_eq!(
            gemini.base_url,
            Some("https://custom-gemini.example.com".to_string())
        );
        assert_eq!(gemini.model, Some("gemini-pro".to_string()));
    }

    #[test]
    fn test_cli_overrides_config() {
        let cli = Cli::parse_from(["agent-run", "-p", "Hi", "--timeout", "60", "--vendor", "gemini"]);
        let toml_config = TomlConfig {
            general: Some(GeneralConfig {
                timeout: Some(30),
                default_vendor: Some("openai".to_string()),
            }),
            openai: None,
            gemini: None,
        };

        let (timeout, vendor, _, _) = merge_config(&cli, &toml_config);

        assert_eq!(timeout, 60);
        assert_eq!(vendor, "gemini");
    }

    #[test]
    fn test_config_overrides_defaults() {
        let cli = Cli::parse_from(["agent-run", "-p", "Hi"]);
        let toml_config = TomlConfig {
            general: Some(GeneralConfig {
                timeout: Some(30),
                default_vendor: Some("gemini".to_string()),
            }),
            openai: None,
            gemini: Some(VendorConfig {
                base_url: Some("https://example.com".to_string()),
                model: Some("custom-model".to_string()),
            }),
        };

        let (timeout, vendor, model, base_url) = merge_config(&cli, &toml_config);

        assert_eq!(timeout, 30);
        assert_eq!(vendor, "gemini");
        assert_eq!(model, Some("custom-model".to_string()));
        assert_eq!(base_url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_defaults_when_no_config() {
        let cli = Cli::parse_from(["agent-run", "-p", "Hi"]);
        let toml_config = TomlConfig::default();

        let (timeout, vendor, model, base_url) = merge_config(&cli, &toml_config);

        assert_eq!(timeout, DEFAULT_TIMEOUT);
        assert_eq!(vendor, DEFAULT_VENDOR);
        assert_eq!(model, None);
        assert_eq!(base_url, None);
    }

    #[test]
    fn test_get_vendor_config_openai() {
        let toml_config = TomlConfig {
            general: None,
            openai: Some(VendorConfig {
                base_url: Some("https://openai.example.com".to_string()),
                model: Some("gpt-4".to_string()),
            }),
            gemini: None,
        };

        let vendor_config = get_vendor_config(&toml_config, "openai");

        assert_eq!(
            vendor_config.base_url,
            Some("https://openai.example.com".to_string())
        );
        assert_eq!(vendor_config.model, Some("gpt-4".to_string()));
    }

    #[test]
    fn test_get_vendor_config_gemini() {
        let toml_config = TomlConfig {
            general: None,
            openai: None,
            gemini: Some(VendorConfig {
                base_url: Some("https://gemini.example.com".to_string()),
                model: Some("gemini-pro".to_string()),
            }),
        };

        let vendor_config = get_vendor_config(&toml_config, "gemini");

        assert_eq!(
            vendor_config.base_url,
            Some("https://gemini.example.com".to_string())
        );
        assert_eq!(vendor_config.model, Some("gemini-pro".to_string()));
    }

    #[test]
    fn test_get_vendor_config_unknown() {
        let toml_config = TomlConfig::default();

        let vendor_config = get_vendor_config(&toml_config, "unknown");

        assert_eq!(vendor_config.base_url, None);
        assert_eq!(vendor_config.model, None);
    }

    #[test]
    fn test_model_cli_overrides_vendor_config() {
        let cli = Cli::parse_from(["agent-run", "-p", "Hi", "--model", "cli-model"]);
        let toml_config = TomlConfig {
            general: None,
            openai: Some(VendorConfig {
                base_url: None,
                model: Some("config-model".to_string()),
            }),
            gemini: None,
        };

        let (_, _, model, _) = merge_config(&cli, &toml_config);

        assert_eq!(model, Some("cli-model".to_string()));
    }
}

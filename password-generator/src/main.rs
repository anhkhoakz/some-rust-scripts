//! Password generator that creates cryptographically secure passwords.
//!
//! Generates passwords using random.org API.

use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};

// Constants
const DEFAULT_PASSWORD_COUNT: usize = 10;
const DEFAULT_PASSWORD_LENGTH: usize = 20;
const DEFAULT_API_TIMEOUT_SECONDS: u64 = 5;
const DEFAULT_MAX_RETRIES: u32 = 3;
const CONNECT_TIMEOUT_SECONDS: u64 = 2;
const POOL_IDLE_TIMEOUT_SECONDS: u64 = 30;
const POOL_MAX_IDLE_PER_HOST: usize = 2;
const INITIAL_RETRY_DELAY_MS: u64 = 100;
const CLIPBOARD_SUBTITLE: &str = "Click to copy to clipboard";

// Output type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputType {
        Plain,
        Alfred,
}

impl std::str::FromStr for OutputType {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_lowercase().as_str() {
                        "plain" => Ok(OutputType::Plain),
                        "alfred" => Ok(OutputType::Alfred),
                        _ => Err(format!(
                                "Invalid output type: {}. Must be 'plain' or 'alfred'",
                                s
                        )),
                }
        }
}

// Configuration
#[derive(Debug, Clone, Parser)]
#[command(
        name = "password-generator",
        about = "Generate secure passwords",
        version
)]
struct Config {
        #[arg(short, long, default_value_t = DEFAULT_PASSWORD_COUNT, help = "Number of passwords to generate")]
        count: usize,

        #[arg(short, long, default_value_t = DEFAULT_PASSWORD_LENGTH, help = "Length of each password")]
        length: usize,

        #[arg(long, default_value = "plain", help = "Output type")]
        #[arg(value_parser = clap::value_parser!(OutputType))]
        r#type: OutputType,

        #[arg(long, default_value_t = DEFAULT_API_TIMEOUT_SECONDS, help = "Timeout for API requests in seconds")]
        api_timeout: u64,

        #[arg(long, default_value_t = DEFAULT_MAX_RETRIES, help = "Maximum number of retries for API requests")]
        max_retries: u32,
}

// Data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AlfredItem {
        title: String,
        subtitle: String,
        arg: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AlfredOutput {
        items: Vec<AlfredItem>,
}

// Password generation
struct PasswordGenerator {
        config: Config,
        http_client: Client,
}

impl PasswordGenerator {
        fn new(config: Config) -> Result<Self> {
                let http_client = create_http_client(config.api_timeout)?;

                Ok(Self {
                        config,
                        http_client,
                })
        }

        async fn generate_passwords(&self) -> Result<Vec<String>> {
                generate_via_api(self).await
        }
}

fn create_http_client(timeout_seconds: u64) -> Result<Client> {
        Client::builder()
                .timeout(Duration::from_secs(timeout_seconds))
                .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECONDS))
                .pool_max_idle_per_host(POOL_MAX_IDLE_PER_HOST)
                .pool_idle_timeout(Duration::from_secs(
                        POOL_IDLE_TIMEOUT_SECONDS,
                ))
                .build()
                .context("Failed to create HTTP client")
}

async fn generate_via_api(
        generator: &PasswordGenerator,
) -> Result<Vec<String>> {
        let url: String = build_api_url(&generator.config);
        let mut last_error: Option<anyhow::Error> = None;

        for attempt in 1..=generator.config.max_retries {
                match try_api_request(&generator.http_client, &url).await {
                        Ok(passwords) => {
                                if is_valid_password_count(
                                        &passwords,
                                        generator.config.count,
                                ) {
                                        return Ok(passwords);
                                }
                        }
                        Err(e) => {
                                last_error = Some(e);
                                if should_retry(
                                        attempt,
                                        generator.config.max_retries,
                                ) {
                                        wait_before_retry(attempt).await;
                                }
                        }
                }
        }

        Err(last_error.unwrap_or_else(|| {
                anyhow::anyhow!(
                        "Failed after {} attempts",
                        generator.config.max_retries
                )
        }))
}

fn build_api_url(config: &Config) -> String {
        format!(
                "https://www.random.org/passwords/?num={}&len={}&format=plain&rnd=new",
                config.count, config.length
        )
}

async fn try_api_request(client: &Client, url: &str) -> Result<Vec<String>> {
        let response: reqwest::Response = client.get(url).send().await?;

        if !response.status().is_success() {
                return Err(anyhow::anyhow!(
                        "API returned status: {}",
                        response.status()
                ));
        }

        let body: String = response
                .text()
                .await
                .context("Failed to read response body")?;
        parse_api_response(&body)
}

fn parse_api_response(body: &str) -> Result<Vec<String>> {
        let passwords: Vec<String> = body
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(String::from)
                .collect();

        Ok(passwords)
}

fn is_valid_password_count(
        passwords: &[String],
        expected_count: usize,
) -> bool {
        passwords.len() == expected_count
}

fn should_retry(current_attempt: u32, max_retries: u32) -> bool {
        current_attempt < max_retries
}

async fn wait_before_retry(attempt: u32) {
        let delay_ms: u64 = calculate_retry_delay(attempt);
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
}

fn calculate_retry_delay(attempt: u32) -> u64 {
        INITIAL_RETRY_DELAY_MS * 2_u64.pow(attempt - 1)
}

// Output formatting
fn format_output(
        passwords: Vec<String>,
        output_type: OutputType,
) -> Result<String> {
        match output_type {
                OutputType::Plain => format_plain_text(passwords),
                OutputType::Alfred => format_alfred_json(passwords),
        }
}

fn format_plain_text(passwords: Vec<String>) -> Result<String> {
        Ok(passwords
                .into_iter()
                .filter(|password: &String| !password.is_empty())
                .collect::<Vec<String>>()
                .join("\n"))
}

fn format_alfred_json(passwords: Vec<String>) -> Result<String> {
        let items: Vec<AlfredItem> = create_alfred_items(passwords);
        let output: AlfredOutput = AlfredOutput { items };
        serde_json::to_string(&output).context("Failed to serialize JSON")
}

fn create_alfred_items(passwords: Vec<String>) -> Vec<AlfredItem> {
        passwords
                .into_iter()
                .filter(|password: &String| !password.is_empty())
                .map(|password: String| AlfredItem {
                        title: password.clone(),
                        subtitle: CLIPBOARD_SUBTITLE.to_string(),
                        arg: password,
                })
                .collect()
}

// Validation
fn validate_config(config: &Config) {
        if config.count == 0 {
                eprintln!("Warning: Password count is zero");
        }
        if config.length == 0 {
                eprintln!("Warning: Password length is zero");
        }
        if config.length > 1000 {
                eprintln!("Warning: Password length exceeds 1000 characters");
        }
}

// Main entry point
fn main() {
        if let Err(error) = run() {
                eprintln!("Error: {}", error);
                std::process::exit(1);
        }
}

async fn run_async() -> Result<()> {
        let config: Config = Config::parse();
        validate_config(&config);

        let generator: PasswordGenerator =
                PasswordGenerator::new(config.clone())?;
        let passwords: Vec<String> = generator.generate_passwords().await?;
        let output: String = format_output(passwords, config.r#type)?;

        println!("{}", output);
        Ok(())
}

fn run() -> Result<()> {
        tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("Failed to create async runtime")?
                .block_on(run_async())
}

// Tests
#[cfg(test)]
mod tests {
        use super::*;

        #[test]
        fn test_parse_api_response() {
                let body: &str = "password1\npassword2\npassword3\n";
                let passwords: Vec<String> = parse_api_response(body).unwrap();

                assert_eq!(passwords.len(), 3);
                assert_eq!(passwords[0], "password1");
                assert_eq!(passwords[1], "password2");
                assert_eq!(passwords[2], "password3");
        }

        #[test]
        fn test_parse_api_response_with_empty_lines() {
                let body = "pass1\n\npass2\n  \npass3\n";
                let passwords = parse_api_response(body).unwrap();

                assert_eq!(passwords.len(), 3);
        }

        #[test]
        fn test_format_alfred_json() {
                let passwords = vec!["test1".to_string(), "test2".to_string()];
                let json = format_alfred_json(passwords).unwrap();

                assert!(json.contains("test1"));
                assert!(json.contains("test2"));
                assert!(json.contains("items"));
                assert!(json.contains(CLIPBOARD_SUBTITLE));
        }

        #[test]
        fn test_is_valid_password_count() {
                let passwords =
                        vec!["a".to_string(), "b".to_string(), "c".to_string()];
                assert!(is_valid_password_count(&passwords, 3));
                assert!(!is_valid_password_count(&passwords, 2));
        }

        #[test]
        fn test_should_retry() {
                assert!(should_retry(1, 3));
                assert!(should_retry(2, 3));
                assert!(!should_retry(3, 3));
        }

        #[test]
        fn test_calculate_retry_delay() {
                assert_eq!(calculate_retry_delay(1), 100);
                assert_eq!(calculate_retry_delay(2), 200);
                assert_eq!(calculate_retry_delay(3), 400);
        }
}

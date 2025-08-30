use crate::{config::Config, errors::CreateError};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::env;

pub async fn get_suggestions(config: &Config, diff: &str) -> Result<Suggestions, CreateError> {
    let response = get_suggestions_with_usage(config, diff).await?;
    Ok(response.suggestions)
}

pub async fn get_suggestions_with_usage(
    config: &Config,
    diff: &str,
) -> Result<SuggestionsWithUsage, CreateError> {
    let response = prompt_with_usage(config, diff).await?;
    let suggestions = parse_suggestions(&response.content)?;

    // Calculate estimated cost based on Claude 3.7 Sonnet pricing
    let estimated_cost = calculate_cost(response.usage.input_tokens, response.usage.output_tokens);

    let usage = TokenUsage {
        input_tokens: response.usage.input_tokens,
        output_tokens: response.usage.output_tokens,
        total_tokens: response.usage.input_tokens + response.usage.output_tokens,
        estimated_cost: Some(estimated_cost),
    };

    Ok(SuggestionsWithUsage { suggestions, usage })
}

fn parse_suggestions(llm_response: &str) -> Result<Suggestions, CreateError> {
    let stripped = llm_response.lines().collect::<Vec<&str>>().join("");

    let json = match Regex::new(r##"\{.+}"##).unwrap().find(&stripped) {
        Some(s) => s.as_str(),
        None => return Err(CreateError::FailedToMatch(stripped)),
    };

    serde_json::from_str(json).map_err(CreateError::FailedToParse)
}

async fn prompt_with_usage(config: &Config, diff: &str) -> Result<AnthropicResponse, CreateError> {
    let api_key = env::var("ANTHROPIC_API_KEY")
        .map_err(|_| CreateError::MissingApiKey)?;
    
    let prompt = format!("{}\n{}", include_str!("diff_prompt.txt"), config);
    
    let request_body = AnthropicRequest {
        model: "claude-3-7-sonnet-20240924".to_string(),
        max_tokens: 1000,
        messages: vec![
            AnthropicMessage {
                role: "user".to_string(),
                content: format!("{}\n{}", prompt, diff),
            }
        ],
    };

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("Content-Type", "application/json")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| CreateError::ApiError(e.to_string()))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(CreateError::ApiError(format!("API request failed: {}", error_text)));
    }

    let api_response: AnthropicApiResponse = response
        .json()
        .await
        .map_err(|e| CreateError::ApiError(e.to_string()))?;

    // Extract text content from the response
    let content = api_response.content
        .iter()
        .filter_map(|c| {
            if c.content_type == "text" {
                Some(c.text.clone())
            } else {
                None
            }
        })
        .collect::<Vec<String>>()
        .join("");

    Ok(AnthropicResponse {
        content,
        usage: api_response.usage,
    })
}

fn calculate_cost(input_tokens: u64, output_tokens: u64) -> f64 {
    // Claude 3.7 Sonnet pricing: $3 per million input tokens, $15 per million output tokens
    let input_cost = (input_tokens as f64 / 1_000_000.0) * 3.0;
    let output_cost = (output_tokens as f64 / 1_000_000.0) * 15.0;
    input_cost + output_cost
}

#[derive(Debug, Default, Deserialize)]
pub struct Suggestions {
    pub category: String,
    pub change_type: String,
    pub title: String,
    pub pr_description: String,
}

#[derive(Debug, Default)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub estimated_cost: Option<f64>,
}

#[derive(Debug)]
pub struct SuggestionsWithUsage {
    pub suggestions: Suggestions,
    pub usage: TokenUsage,
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u64,
    messages: Vec<AnthropicMessage>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicApiResponse {
    content: Vec<AnthropicContent>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u64,
    output_tokens: u64,
}

#[derive(Debug)]
struct AnthropicResponse {
    content: String,
    usage: AnthropicUsage,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::unpack_config;

    fn load_example_config() -> Config {
        unpack_config(include_str!("testdata/example_config.json"))
            .expect("failed to load example config")
    }

    #[cfg(not(feature = "remote"))]
    #[tokio::test]
    async fn test_parse_prompt() {
        let example_config = load_example_config();
        let diff = include_str!("./testdata/example_git_diff.txt");
        let result = prompt(&example_config, diff).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(parse_suggestions(&response).is_ok());
    }

    #[test]
    fn test_parse_suggestions() {
        let response = r##"
            ```json
            {
                "category": "",
                "change_type": "Improvements",
                "title": "Add tests for diff prompt functionality",
                "pr_description": "This PR adds unit tests for the diff prompt functionality. \n\nChanges include:\n- Added a test module in src/diff_prompt.rs\n- Created a basic test case that verifies prompt behavior using a fixture file\n\nThis helps ensure the prompt functionality works correctly and provides a foundation for future testing."
            }
            ```
        "##;

        let result = parse_suggestions(&response);
        assert!(result.is_ok());
        let suggestions = result.unwrap();
        assert_eq!(suggestions.title, "Add tests for diff prompt functionality");
        assert_eq!(suggestions.pr_description, "This PR adds unit tests for the diff prompt functionality. \n\nChanges include:\n- Added a test module in src/diff_prompt.rs\n- Created a basic test case that verifies prompt behavior using a fixture file\n\nThis helps ensure the prompt functionality works correctly and provides a foundation for future testing.");
        assert_eq!(suggestions.category, "");
        assert_eq!(suggestions.change_type, "Improvements");
    }
}

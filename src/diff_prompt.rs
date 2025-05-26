use crate::{config::Config, errors::CreateError};
use regex::Regex;
use rig::{
    completion::Prompt,
    providers::anthropic::{self, CLAUDE_3_7_SONNET},
};
use serde::Deserialize;

// TODO: might make sense to refactor this to just take in the diff instead of getting the diff manually as well
pub async fn get_suggestions(config: &Config, diff: &str) -> Result<Suggestions, CreateError> {
    parse_suggestions(prompt(config, diff).await?.as_str())
}

fn parse_suggestions(llm_response: &str) -> Result<Suggestions, CreateError> {
    let stripped = llm_response.lines().collect::<Vec<&str>>().join("");

    let json = match Regex::new(r##"\{.+}"##).unwrap().find(&stripped) {
        Some(s) => s.as_str(),
        None => return Err(CreateError::FailedToMatch(stripped)),
    };

    serde_json::from_str(json).map_err(CreateError::FailedToParse)
}

async fn prompt(config: &Config, diff: &str) -> Result<String, CreateError> {
    let prompt = format!("{}\n{}", include_str!("diff_prompt.txt"), config);
    let anthropic_client = anthropic::Client::from_env();
    let sonnet = anthropic_client
        .agent(CLAUDE_3_7_SONNET)
        .preamble(&prompt)
        .max_tokens(1e3 as u64)
        .build();

    Ok(sonnet.prompt(diff).await?)
}

#[derive(Debug, Default, Deserialize)]
pub struct Suggestions {
    pub category: String,
    pub change_type: String,
    pub title: String,
    pub pr_description: String,
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

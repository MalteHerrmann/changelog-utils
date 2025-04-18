use crate::{config::Config, errors::CreateError, github};
use rig::{
    completion::Prompt,
    providers::anthropic::{self, CLAUDE_3_7_SONNET},
};
use serde::Deserialize;

pub async fn get_suggestions(
    config: &Config,
    work_branch: &str,
    pr_target: &str,
) -> Result<Suggestions, CreateError> {
    let diff = github::get_diff(work_branch, pr_target)?;
    let response = prompt(config, diff.as_str()).await?;

    Ok(serde_json::from_str(&response)?)
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

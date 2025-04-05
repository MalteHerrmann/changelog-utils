use crate::{config::Config, errors::CreateError};
use rig::{
    completion::Prompt,
    providers::anthropic::{self, CLAUDE_3_7_SONNET},
};

pub async fn prompt(config: &Config, diff: &str) -> Result<String, CreateError> {
    let prompt = format!("{}\n{}", include_str!("diff_prompt.txt"), config);
    let anthropic_client = anthropic::Client::from_env();
    let sonnet = anthropic_client
        .agent(CLAUDE_3_7_SONNET)
        .preamble(prompt.as_str())
        .max_tokens(1e3 as u64)
        .build();

    Ok(sonnet.prompt(diff).await?)
}

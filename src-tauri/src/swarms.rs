use std::{env, path::PathBuf};

use color_eyre::eyre::eyre;
use rig::{
    client::{CompletionClient, ProviderClient},
    completion::Prompt,
    providers::openai,
    tool::Tool,
};
use serde::{Deserialize, Serialize};

use crate::commands::MCPServer;
use crate::error::Result;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct AgentOptions {
    pub name: Option<String>,
    pub model: String,
    pub url: String,
    pub api_key: Option<String>,
    pub task: String,
    pub plan: Option<String>,
    pub user_name: Option<String>,
    pub max_loops: Option<u32>,
    pub save_state_dir: Option<PathBuf>,
    pub system_prompt: Option<String>,
    pub enable_autosave: bool,
}

pub async fn run_agent<T: Tool + 'static>(
    opts: AgentOptions,
    tools: impl IntoIterator<Item = T>,
    _mcp_servers: impl IntoIterator<Item = MCPServer>, // Not supported yet in rig-core
) -> Result<String> {
    // Create rig-core OpenAI client
    let openai_client = if let Some(api_key) = &opts.api_key {
        openai::Client::new(api_key)
    } else {
        // Check if environment variable exists
        if env::var("OPENAI_API_KEY").is_err() {
            return Err(eyre!(
                "No valid API key found in options or OPENAI_API_KEY environment variable"
            )
            .into());
        }
        openai::Client::from_env()
    };

    // Build agent with rig-core
    let mut agent_builder = openai_client.agent(&opts.model);

    // Set system prompt/preamble
    if let Some(system_prompt) = opts.system_prompt {
        agent_builder = agent_builder.preamble(&system_prompt);
    } else {
        // Default system prompt
        let name = opts.name.as_deref().unwrap_or("Assistant");
        let default_prompt =
            format!("You are {name}, an AI assistant. You are helpful, harmless, and honest.");
        agent_builder = agent_builder.preamble(&default_prompt);
    }

    // Add tools to the agent
    for tool in tools {
        agent_builder = agent_builder.tool(tool);
    }

    // Build the agent
    let agent = agent_builder.build();

    // Run the task
    let response = agent
        .prompt(&opts.task)
        .await
        .map_err(|e| eyre!("Failed to run agent: {}", e))?;

    Ok(response)
}

use std::sync::Arc;

use color_eyre::eyre::eyre;
use futures_util::StreamExt;
use kameo::prelude::{ActorRef as LocalActorRef, *};
use kameo_actors::{message_bus::Publish, pool::ActorPool};
use rig::{
    agent::AgentBuilder,
    client::{CompletionClient, ProviderClient, completion::CompletionModelHandle},
    completion::{CompletionModel, Message as RigMessage, ToolDefinition},
    message::{AssistantContent, ToolCall},
    providers::{anthropic, cohere, deepseek, gemini, ollama, openai, perplexity, xai},
    streaming::StreamingChat,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sync_wrapper::SyncFuture;
use uuid::Uuid;

use crate::{
    actors::{
        ActorRef, SystemEventBus,
        tools::{ToolExecutorActor, UseTool},
    },
    entities::{Agent, Model, ModelProvider},
    error::{AppError, Result},
    utils::tell_ask,
};

#[derive(Actor)]
pub struct AgentActor {
    pub bus: LocalActorRef<SystemEventBus>,
}

#[derive(Actor)]
pub struct AgentManagerActor {
    pub bus: LocalActorRef<SystemEventBus>,
    pub pool: LocalActorRef<ActorPool<AgentActor>>,
}

fn provider_to_model<'a>(
    provider: ModelProvider,
    name: &'a str,
    _api_key: Option<&'a str>,
) -> impl CompletionModel {
    match provider {
        ModelProvider::OpenAI => CompletionModelHandle {
            inner: Arc::new(openai::Client::from_env().completion_model(name)),
        },
        ModelProvider::Cohere => CompletionModelHandle {
            inner: Arc::new(cohere::Client::from_env().completion_model(name)),
        },
        ModelProvider::Anthropic => CompletionModelHandle {
            inner: Arc::new(anthropic::Client::from_env().completion_model(name)),
        },
        ModelProvider::Perplexity => CompletionModelHandle {
            inner: Arc::new(perplexity::Client::from_env().completion_model(name)),
        },
        ModelProvider::Gemini => CompletionModelHandle {
            inner: Arc::new(gemini::Client::from_env().completion_model(name)),
        },
        ModelProvider::XAi => CompletionModelHandle {
            inner: Arc::new(xai::Client::from_env().completion_model(name)),
        },
        ModelProvider::DeepSeek => CompletionModelHandle {
            inner: Arc::new(deepseek::Client::from_env().completion_model(name)),
        },
        ModelProvider::Ollama => CompletionModelHandle {
            inner: Arc::new(ollama::Client::from_env().completion_model(name)),
        },
    }
}

impl Message<AgentRequest> for AgentActor {
    type Reply = Result<()>;
    async fn handle(
        &mut self,
        msg: AgentRequest,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let mut agent_res = AgentResponseEvent {
            agent_id: msg.agent.id,
            conversation_id: msg.conversation_id,
            workspace_id: msg.agent.workspace_id.unwrap_or_default(),
            response: StreamedPart::Error("Unstarted".to_string()),
        };
        let Some(tool_ref) = msg.tool_ref else {
            return Err(eyre!("No tool ref provided").into());
        };
        let model = provider_to_model(msg.model.provider, &msg.model.name, None);
        let mut agent = AgentBuilder::new(model);
        for tool in msg.tool_definitions {
            agent = agent.tool(SandboxedTool {
                definition: tool,
                actor_ref: tool_ref.clone(),
            });
        }
        let agent = agent.build();
        let mut stream = agent.stream_chat(msg.prompt, msg.history).await?;
        while let Some(part) = stream.next().await {
            let part = match part {
                Ok(k) => k,
                Err(e) => {
                    agent_res.response = StreamedPart::Error(e.to_string());
                    self.bus.tell(Publish(agent_res.clone())).await.ok();
                    return Err(e.into());
                }
            };
            let stream_part = match part {
                AssistantContent::Text(text) => StreamedPart::Token(text.text),
                AssistantContent::ToolCall(tool_call) => StreamedPart::ToolCall(tool_call),
            };
            agent_res.response = stream_part;
            self.bus.tell(Publish(agent_res.clone())).await.ok();
        }
        let (full_response, tool_calls) = stream.choice.into_iter().fold(
            (String::new(), Vec::new()),
            |(mut full, mut tools), cur| {
                match cur {
                    AssistantContent::Text(text) => full.push_str(&text.text),
                    AssistantContent::ToolCall(tool_call) => tools.push(tool_call),
                }
                (full, tools)
            },
        );
        agent_res.response = StreamedPart::EndOfStream {
            full_response,
            tool_calls,
        };
        self.bus.tell(Publish(agent_res.clone())).await.ok();
        Ok(())
    }
}

#[derive(Clone)]
pub struct SandboxedTool {
    pub definition: ToolDefinition,
    pub actor_ref: ActorRef<ToolExecutorActor>,
}

impl rig::tool::Tool for SandboxedTool {
    const NAME: &'static str = "SandboxedTool";

    type Error = AppError;

    type Args = String;

    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        self.definition.clone()
    }

    fn call(
        &self,
        args: Self::Args,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send + Sync {
        SyncFuture::new(async move {
            let msg = UseTool {
                name: self.definition.name.clone().into(),
                args,
            };
            Ok(match &self.actor_ref {
                ActorRef::Local(actor_ref) => actor_ref.ask(msg).await?,
                ActorRef::Remote(actor_ref) => {
                    tell_ask::<_, ToolExecutorActor>(actor_ref, msg).await??
                }
            })
        })
    }

    fn name(&self) -> String {
        self.definition.name.clone()
    }
}

#[skip_serializing_none]
#[derive(Clone, Serialize, Deserialize)]
pub struct AgentRequest {
    pub agent: Agent,
    pub model: Model,
    pub prompt: String,
    pub history: Vec<RigMessage>,
    pub tool_definitions: Vec<ToolDefinition>,
    pub conversation_id: Uuid,
    pub participants: Vec<Uuid>,
    #[serde(skip, default)]
    pub tool_ref: Option<ActorRef<ToolExecutorActor>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponseEvent {
    pub agent_id: Uuid,
    pub conversation_id: Uuid,
    pub workspace_id: Uuid,
    pub response: StreamedPart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "type",
    content = "data"
)]
pub enum StreamedPart {
    /// A piece of the text response.
    Token(String),
    /// The agent is now calling a tool. The UI can use this
    /// to show a "thinking" or "using tool" indicator.
    ToolCall(ToolCall),
    /// The result from a tool call.
    ToolResult {
        tool_name: String,
        tool_output: serde_json::Value,
    },
    /// The agent has finished its response for this turn.
    EndOfStream {
        full_response: String,
        tool_calls: Vec<ToolCall>,
    },
    /// An error occurred that should be displayed to the user.
    Error(String),
}

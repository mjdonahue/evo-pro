use kameo::prelude::ActorRef as LocalActorRef;
use tauri::AppHandle;

use crate::actors::{
    SystemEventBus, agents::AgentManagerActor, conversation::ConversationManagerActor,
    database::DatabaseActor, tools::ToolExecutorActor,
};

#[derive(Clone)]
pub struct AppState {
    pub app: AppHandle,
    pub actors: ActorManager, // Direct reference to ActorManager
}

#[derive(Clone)]
pub struct ActorManager {
    pub bus: LocalActorRef<SystemEventBus>,
    pub db: LocalActorRef<DatabaseActor>, // ActorRef to DatabaseActor
    pub agent_manager: LocalActorRef<AgentManagerActor>,
    pub tool_ref: LocalActorRef<ToolExecutorActor>,
    pub conversation_manager: LocalActorRef<ConversationManagerActor>,
}

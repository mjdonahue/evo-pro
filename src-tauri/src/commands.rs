use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    actors::{
        conversation::SendMessage,
        database::{
            CreateBatchParticipants, DeleteP2pNode, DeleteParticipant, DeleteTask, DeleteUser, ListAgents, ListConversations, ListParticipants, ListTasks, ListUsers, UpdateAgent, UpdateP2pNode, UpdateParticipant, UpdateTask, UpdateUser
        },
    },
    entities::{
        Agent, AgentFilter, Conversation, ConversationFilter, CreateAgent, CreateConversation, CreateConversationParticipant, CreateP2pNode, CreateParticipant, CreateTask, CreateUser, P2pNode, Participant, ParticipantFilter, ParticipantRole, Task, TaskFilter, User, UserFilter
    },
    error::Result,
    keys::{PubKeyWrapper, KEY_PAIR, PEER_ID},
    state::AppState,
};

#[tauri::command]
pub async fn create_task(task: CreateTask, state: State<'_, AppState>) -> Result<Task> {
    Ok(state.actors.db.ask(task).await?)
}

#[tauri::command]
pub async fn list_tasks(filter: TaskFilter, state: State<'_, AppState>) -> Result<Vec<Task>> {
    Ok(state.actors.db.ask(ListTasks(filter)).await?)
}

#[tauri::command]
pub async fn delete_tasks(id: Uuid, state: State<'_, AppState>) -> Result<()> {
    Ok(state.actors.db.ask(DeleteTask(id)).await?)
}

#[tauri::command]
pub async fn update_task(task: Task, state: State<'_, AppState>) -> Result<()> {
    Ok(state.actors.db.ask(UpdateTask(task)).await?)
}

#[tauri::command]
pub async fn create_user(user: CreateUser, state: State<'_, AppState>) -> Result<User> {
    Ok(state.actors.db.ask(user).await?)
}

#[tauri::command]
pub async fn list_users(filter: UserFilter, state: State<'_, AppState>) -> Result<Vec<User>> {
    Ok(state.actors.db.ask(ListUsers(filter)).await?)
}

#[tauri::command]
pub async fn update_user(user: User, state: State<'_, AppState>) -> Result<()> {
    Ok(state.actors.db.ask(UpdateUser(user)).await?)
}

#[tauri::command]
pub async fn delete_user(id: Uuid, state: State<'_, AppState>) -> Result<()> {
    Ok(state.actors.db.ask(DeleteUser(id)).await?)
}

#[tauri::command]
pub async fn create_conversation(
    conversation: CreateConversation,
    participants: Vec<Uuid>,
    state: State<'_, AppState>,
) -> Result<Conversation> {
    let conv = state.actors.db.ask(conversation).await?;
    state
        .actors
        .db
        .ask(CreateBatchParticipants(
            participants
                .into_iter()
                .map(|p| CreateConversationParticipant {
                    conversation_id: conv.id,
                    participant_id: p,
                    is_active: true,
                    role: ParticipantRole::Member,
                })
                .collect(),
        ))
        .await?;
    Ok(conv)
}

#[tauri::command]
pub async fn create_agent(agent: CreateAgent, state: State<'_, AppState>) -> Result<Agent> {
    Ok(state.actors.db.ask(agent).await?)
}

#[tauri::command]
pub async fn update_agent(agent: Agent, state: State<'_, AppState>) -> Result<()> {
    Ok(state.actors.db.ask(UpdateAgent(agent)).await?)
}

#[tauri::command]
pub async fn list_agents(filter: AgentFilter, state: State<'_, AppState>) -> Result<Vec<Agent>> {
    Ok(state.actors.db.ask(ListAgents(filter)).await?)
}

#[tauri::command]
pub fn get_public_key() -> PubKeyWrapper {
    PubKeyWrapper(KEY_PAIR.read().unwrap().public())
}

#[tauri::command]
pub fn get_peer_id() -> PeerId {
    PEER_ID.get().cloned().unwrap()
}

#[tauri::command]
pub async fn create_p2p_node(
    p2p_node: CreateP2pNode,
    state: State<'_, AppState>,
) -> Result<P2pNode> {
    Ok(state.actors.db.ask(p2p_node).await?)
}

#[tauri::command]
pub async fn update_p2p_node(p2p_node: P2pNode, state: State<'_, AppState>) -> Result<()> {
    Ok(state.actors.db.ask(UpdateP2pNode(p2p_node)).await?)
}

#[tauri::command]
pub async fn delete_p2p_node(
    peer_id: Uuid,
    participant_id: Uuid,
    state: State<'_, AppState>,
) -> Result<()> {
    Ok(state
        .actors
        .db
        .ask(DeleteP2pNode(peer_id, participant_id))
        .await?)
}

#[tauri::command]
pub async fn create_participant(
    participant: CreateParticipant,
    state: State<'_, AppState>,
) -> Result<Participant> {
    Ok(state.actors.db.ask(participant).await?)
}

#[tauri::command]
pub async fn update_participant(
    participant: Participant,
    state: State<'_, AppState>,
) -> Result<Participant> {
    Ok(state.actors.db.ask(UpdateParticipant(participant)).await?)
}

#[tauri::command]
pub async fn delete_participant(participant: Uuid, state: State<'_, AppState>) -> Result<()> {
    Ok(state.actors.db.ask(DeleteParticipant(participant)).await?)
}

#[tauri::command(async)]
pub async fn send_message(msg: SendMessage, state: State<'_, AppState>) -> Result<()> {
    todo!()
}

#[tauri::command]
pub async fn list_participants(
    filter: ParticipantFilter,
    state: State<'_, AppState>,
) -> Result<Vec<Participant>> {
    Ok(state.actors.db.ask(ListParticipants(filter)).await?)
}

#[tauri::command]
pub async fn list_conversations(
    filter: ConversationFilter,
    state: State<'_, AppState>,
) -> Result<Vec<Conversation>> {
    Ok(state.actors.db.ask(ListConversations(filter)).await?)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    level: String,
    message: String,
    timestamp: String,
    source: String,
    metadata: Option<serde_json::Value>,
}

#[tauri::command]
pub fn log_frontend_message(entry: LogEntry) -> Result<(), String> {
    // Log to Tauri's console with proper formatting
    match entry.level.as_str() {
        "ERROR" => {
            error!(
                "[{}] {} - {}: {}",
                entry.timestamp,
                entry.source,
                entry.message,
                entry.metadata.map_or("".to_string(), |m| m.to_string())
            );
        }
        "WARN" => {
            warn!(
                "[{}] {} - {}: {}",
                entry.timestamp,
                entry.source,
                entry.message,
                entry.metadata.map_or("".to_string(), |m| m.to_string())
            );
        }
        "INFO" => {
            info!(
                "[{}] {} - {}: {}",
                entry.timestamp,
                entry.source,
                entry.message,
                entry.metadata.map_or("".to_string(), |m| m.to_string())
            );
        }
        "DEBUG" => {
            debug!(
                "[{}] {} - {}: {}",
                entry.timestamp,
                entry.source,
                entry.message,
                entry.metadata.map_or("".to_string(), |m| m.to_string())
            );
        }
        _ => {
            info!(
                "[{}] {} - {}: {}",
                entry.timestamp,
                entry.source,
                entry.message,
                entry.metadata.map_or("".to_string(), |m| m.to_string())
            );
        }
    }

    Ok(())
}

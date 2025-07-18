-- This migration adds triggers to automatically update the `updated_at` timestamp
-- for all tables that have an `updated_at` column.

-- Trigger for the 'accounts' table
CREATE TRIGGER trigger_accounts_updated_at
AFTER UPDATE ON accounts
FOR EACH ROW
BEGIN
    UPDATE accounts SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'addresses' table
CREATE TRIGGER trigger_addresses_updated_at
AFTER UPDATE ON addresses
FOR EACH ROW
BEGIN
    UPDATE addresses SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'agents' table
CREATE TRIGGER trigger_agents_updated_at
AFTER UPDATE ON agents
FOR EACH ROW
BEGIN
    UPDATE agents SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'agent_capabilities' table
CREATE TRIGGER trigger_agent_capabilities_updated_at
AFTER UPDATE ON agent_capabilities
FOR EACH ROW
BEGIN
    UPDATE agent_capabilities SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'agent_context' table
CREATE TRIGGER trigger_agent_context_updated_at
AFTER UPDATE ON agent_context
FOR EACH ROW
BEGIN
    UPDATE agent_context SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'agent_collaborations' table
CREATE TRIGGER trigger_agent_collaborations_updated_at
AFTER UPDATE ON agent_collaborations
FOR EACH ROW
BEGIN
    UPDATE agent_collaborations SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'api_keys' table
CREATE TRIGGER trigger_api_keys_updated_at
AFTER UPDATE ON api_keys
FOR EACH ROW
BEGIN
    UPDATE api_keys SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'attachments' table
CREATE TRIGGER trigger_attachments_updated_at
AFTER UPDATE ON attachments
FOR EACH ROW
BEGIN
    UPDATE attachments SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'comments' table
CREATE TRIGGER trigger_comments_updated_at
AFTER UPDATE ON comments
FOR EACH ROW
BEGIN
    UPDATE comments SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'contacts' table
CREATE TRIGGER trigger_contacts_updated_at
AFTER UPDATE ON contacts
FOR EACH ROW
BEGIN
    UPDATE contacts SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'contexts' table
CREATE TRIGGER trigger_contexts_updated_at
AFTER UPDATE ON contexts
FOR EACH ROW
BEGIN
    UPDATE contexts SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'conversations' table
CREATE TRIGGER trigger_conversations_updated_at
AFTER UPDATE ON conversations
FOR EACH ROW
BEGIN
    UPDATE conversations SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'conversation_participants' table
CREATE TRIGGER trigger_conversation_participants_updated_at
AFTER UPDATE ON conversation_participants
FOR EACH ROW
BEGIN
    UPDATE conversation_participants SET updated_at = CURRENT_TIMESTAMP WHERE conversation_id = OLD.conversation_id AND participant_id = OLD.participant_id;
END;

-- Trigger for the 'credentials' table
CREATE TRIGGER trigger_credentials_updated_at
AFTER UPDATE ON credentials
FOR EACH ROW
BEGIN
    UPDATE credentials SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'documents' table
CREATE TRIGGER trigger_documents_updated_at
AFTER UPDATE ON documents
FOR EACH ROW
BEGIN
    UPDATE documents SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'document_chunks' table
CREATE TRIGGER trigger_document_chunks_updated_at
AFTER UPDATE ON document_chunks
FOR EACH ROW
BEGIN
    UPDATE document_chunks SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'events' table
CREATE TRIGGER trigger_events_updated_at
AFTER UPDATE ON events
FOR EACH ROW
BEGIN
    UPDATE events SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'event_participants' table
CREATE TRIGGER trigger_event_participants_updated_at
AFTER UPDATE ON event_participants
FOR EACH ROW
BEGIN
    UPDATE event_participants SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'files' table
CREATE TRIGGER trigger_files_updated_at
AFTER UPDATE ON files
FOR EACH ROW
BEGIN
    UPDATE files SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'groups' table
CREATE TRIGGER trigger_groups_updated_at
AFTER UPDATE ON groups
FOR EACH ROW
BEGIN
    UPDATE groups SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'mcp_servers' table
CREATE TRIGGER trigger_mcp_servers_updated_at
AFTER UPDATE ON mcp_servers
FOR EACH ROW
BEGIN
    UPDATE mcp_servers SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'mcp_tools' table
CREATE TRIGGER trigger_mcp_tools_updated_at
AFTER UPDATE ON mcp_tools
FOR EACH ROW
BEGIN
    UPDATE mcp_tools SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'memories' table
CREATE TRIGGER trigger_memories_updated_at
AFTER UPDATE ON memories
FOR EACH ROW
BEGIN
    UPDATE memories SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'memory_vectors' table
CREATE TRIGGER trigger_memory_vectors_updated_at
AFTER UPDATE ON memory_vectors
FOR EACH ROW
BEGIN
    UPDATE memory_vectors SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'messages' table
CREATE TRIGGER trigger_messages_updated_at
AFTER UPDATE ON messages
FOR EACH ROW
BEGIN
    UPDATE messages SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'models' table
CREATE TRIGGER trigger_models_updated_at
AFTER UPDATE ON models
FOR EACH ROW
BEGIN
    UPDATE models SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'agent_models' table
CREATE TRIGGER trigger_agent_models_updated_at
AFTER UPDATE ON agent_models
FOR EACH ROW
BEGIN
    UPDATE agent_models SET updated_at = CURRENT_TIMESTAMP WHERE agent_id = OLD.agent_id AND model_id = OLD.model_id;
END;

-- Trigger for the 'notes' table
CREATE TRIGGER trigger_notes_updated_at
AFTER UPDATE ON notes
FOR EACH ROW
BEGIN
    UPDATE notes SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'notifications' table
CREATE TRIGGER trigger_notifications_updated_at
AFTER UPDATE ON notifications
FOR EACH ROW
BEGIN
    UPDATE notifications SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'p2p_nodes' table
CREATE TRIGGER trigger_p2p_nodes_updated_at
AFTER UPDATE ON p2p_nodes
FOR EACH ROW
BEGIN
    UPDATE p2p_nodes SET updated_at = CURRENT_TIMESTAMP WHERE participant_id = OLD.participant_id AND peer_id = OLD.peer_id;
END;

-- Trigger for the 'participants' table
CREATE TRIGGER trigger_participants_updated_at
AFTER UPDATE ON participants
FOR EACH ROW
BEGIN
    UPDATE participants SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'plans' table
CREATE TRIGGER trigger_plans_updated_at
AFTER UPDATE ON plans
FOR EACH ROW
BEGIN
    UPDATE plans SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'procedures' table
CREATE TRIGGER trigger_procedures_updated_at
AFTER UPDATE ON procedures
FOR EACH ROW
BEGIN
    UPDATE procedures SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'prompts' table
CREATE TRIGGER trigger_prompts_updated_at
AFTER UPDATE ON prompts
FOR EACH ROW
BEGIN
    UPDATE prompts SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'registry' table
CREATE TRIGGER trigger_registry_updated_at
AFTER UPDATE ON registry
FOR EACH ROW
BEGIN
    UPDATE registry SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'tasks' table
CREATE TRIGGER trigger_tasks_updated_at
AFTER UPDATE ON tasks
FOR EACH ROW
BEGIN
    UPDATE tasks SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'task_assignees' table
CREATE TRIGGER trigger_task_assignees_updated_at
AFTER UPDATE ON task_assignees
FOR EACH ROW
BEGIN
    UPDATE task_assignees SET updated_at = CURRENT_TIMESTAMP WHERE task_id = OLD.task_id AND participant_id = OLD.participant_id;
END;

-- Trigger for the 'tools' table
CREATE TRIGGER trigger_tools_updated_at
AFTER UPDATE ON tools
FOR EACH ROW
BEGIN
    UPDATE tools SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'users' table
CREATE TRIGGER trigger_users_updated_at
AFTER UPDATE ON users
FOR EACH ROW
BEGIN
    UPDATE users SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'workflows' table
CREATE TRIGGER trigger_workflows_updated_at
AFTER UPDATE ON workflows
FOR EACH ROW
BEGIN
    UPDATE workflows SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'workflow_steps' table
CREATE TRIGGER trigger_workflow_steps_updated_at
AFTER UPDATE ON workflow_steps
FOR EACH ROW
BEGIN
    UPDATE workflow_steps SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'workspaces' table
CREATE TRIGGER trigger_workspaces_updated_at
AFTER UPDATE ON workspaces
FOR EACH ROW
BEGIN
    UPDATE workspaces SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Trigger for the 'settings' table
CREATE TRIGGER trigger_settings_updated_at
AFTER UPDATE ON settings
FOR EACH ROW
BEGIN
    UPDATE settings SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

CREATE TABLE accounts (
    id BLOB PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    workspace_id BLOB,
    primary_address_id BLOB,
    account_type INTEGER NOT NULL DEFAULT 0,  -- 'PERSONAL', 'GROUP', 'ORGANIZATION'
    status INTEGER NOT NULL DEFAULT 0,  -- 'ACTIVE', 'ARCHIVED', 'DELETED'
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),  -- JSON object with additional metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL,
    FOREIGN KEY (primary_address_id) REFERENCES addresses(id) ON DELETE SET NULL
);

CREATE INDEX idx_accounts_workspace_id ON accounts(workspace_id);
CREATE INDEX idx_accounts_name ON accounts(name);
CREATE INDEX idx_accounts_primary_address_id ON accounts(primary_address_id);

CREATE TABLE addresses (
    id BLOB PRIMARY KEY NOT NULL,
    user_id BLOB,
    contact_id BLOB,
    account_id BLOB,
    address_type INTEGER NOT NULL DEFAULT 0,  -- 'HOME', 'WORK', 'OTHER'
    status INTEGER NOT NULL DEFAULT 0,  -- 'ACTIVE', 'ARCHIVED', 'DELETED'
    street TEXT,
    city TEXT,
    state TEXT,
    postal_code TEXT,
    country TEXT,
    country_code TEXT,
    latitude TEXT,
    longitude TEXT,
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),  -- JSON object with additional metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    workspace_id BLOB,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE SET NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE SET NULL
);

CREATE INDEX idx_addresses_contact_id ON addresses(contact_id) WHERE contact_id IS NOT NULL;
CREATE INDEX idx_addresses_user_id ON addresses(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX idx_addresses_account_id ON addresses(account_id) WHERE account_id IS NOT NULL;
CREATE INDEX idx_addresses_type ON addresses(address_type);
CREATE INDEX idx_addresses_status ON addresses(status);
CREATE INDEX idx_addresses_workspace_id ON addresses(workspace_id);

CREATE TABLE agents (
    id BLOB PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    avatar_url TEXT,
    -- Core classification
    agent_type INTEGER NOT NULL DEFAULT 0, -- 0: 'WORKER', 1: 'OPERATOR', 2: 'SYSTEM', 3: 'USER_PROXY'
    status INTEGER NOT NULL DEFAULT 0,       -- 0: 'ACTIVE', 1: 'INACTIVE', 2: 'DELETED'
    version TEXT NOT NULL DEFAULT '1.0.0',  -- version of the agent
    -- Agent configuration
    config TEXT CHECK (config IS NULL OR json_valid(config)),         -- JSON object with configuration
    tool_config TEXT CHECK (tool_config IS NULL OR json_valid(tool_config)),         -- JSON object with tool configuration for the agent
    context_window INTEGER NOT NULL DEFAULT 4000, -- The number of tokens that the agent can process at once
    -- Hierarchy and delegation
    parent_agent_id BLOB, -- parent_agent_id is the agent id of the parent of the agent
    operator_level INTEGER NOT NULL DEFAULT 0, -- Hierarchy level (0 = root)
    delegation_rules TEXT CHECK (delegation_rules IS NULL OR json_valid(delegation_rules)), -- JSON array of delegation rules
    -- Performance metrics
    performance_metrics TEXT CHECK (performance_metrics IS NULL OR json_valid(performance_metrics)), -- Combines various metrics
    last_interaction_at TIMESTAMP, -- The timestamp of the last user interaction with the agent
    -- Timestamps
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- Relationships
    model_id BLOB, -- model_id is the model id of the agent
    participant_id BLOB, -- participant_id is the participant id of the agent
    workspace_id BLOB, -- workspace_id is the workspace id of the agent
    registry_id BLOB, -- registry_id is the registry id of the agent
    created_by_id BLOB, -- created_by_id is the user id of the user who created the agent
    operator_user_id BLOB, -- operator_user_id is the user id of the operator of the agent  -- if the agent is a user operator
    -- Foreign keys
    FOREIGN KEY (model_id) REFERENCES models(id) ON DELETE SET NULL
    FOREIGN KEY (participant_id) REFERENCES participants(id) ON DELETE SET NULL,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL,
    FOREIGN KEY (registry_id) REFERENCES registry(id) ON DELETE SET NULL,
    FOREIGN KEY (created_by_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (operator_user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (parent_agent_id) REFERENCES agents(id) ON DELETE SET NULL
);

CREATE INDEX idx_agents_name ON agents(name);
CREATE INDEX idx_agents_model_id ON agents(model_id);
CREATE INDEX idx_agents_participant_id ON agents(participant_id);
CREATE INDEX idx_agents_workspace_id ON agents(workspace_id);
CREATE INDEX idx_agents_operator_user_id ON agents(operator_user_id);
CREATE INDEX idx_agents_created_by_id ON agents(created_by_id);
CREATE INDEX idx_agents_registry_id ON agents(registry_id);

-- Separate table for managing agent capabilities

CREATE TABLE agent_capabilities (
    id BLOB PRIMARY KEY NOT NULL,
    agent_id BLOB NOT NULL,
    capability_name TEXT NOT NULL,
    capability_type INTEGER NOT NULL DEFAULT 0, -- 0: 'TOOL', 1: 'SKILL', 2: 'KNOWLEDGE', 3: 'API'
    
    -- Configuration
    config TEXT CHECK (config IS NULL OR json_valid(config)),
    requirements TEXT CHECK (requirements IS NULL OR json_valid(requirements)), -- Dependencies
    
    -- Performance
    success_rate REAL DEFAULT 0.0,
    average_execution_time REAL DEFAULT 0.0,
    last_used_at TIMESTAMP,
    usage_count INTEGER DEFAULT 0,
    
    -- Status
    is_enabled BOOLEAN NOT NULL DEFAULT 1,
    is_verified BOOLEAN NOT NULL DEFAULT 0,
    
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE,
    UNIQUE(agent_id, capability_name)
);

CREATE INDEX idx_agent_capabilities_agent_id ON agent_capabilities(agent_id);
CREATE INDEX idx_agent_capabilities_capability_name ON agent_capabilities(capability_name);

-- More flexible state management

CREATE TABLE agent_context (
    id BLOB PRIMARY KEY NOT NULL,
    agent_id BLOB NOT NULL,
    context_type INTEGER NOT NULL DEFAULT 0, -- 0: 'SESSION', 1: 'WORKFLOW', 2: 'PERSISTENT', 3: 'SHARED'
    
    -- Scope
    conversation_id BLOB,
    execution_id BLOB,
    workspace_id BLOB,
    
    -- State data
    context_key TEXT NOT NULL,
    context_value TEXT NOT NULL CHECK (json_valid(context_value)),
    value_type TEXT NOT NULL DEFAULT 'json', -- 'json', 'string', 'number', 'boolean'
    
    -- Lifecycle
    expires_at TIMESTAMP,
    is_encrypted BOOLEAN NOT NULL DEFAULT 0,
    access_level INTEGER NOT NULL DEFAULT 0, -- 0: 'PRIVATE', 1: 'SHARED', 2: 'PUBLIC'
    
    -- Versioning
    version INTEGER NOT NULL DEFAULT 1,
    
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
    FOREIGN KEY (execution_id) REFERENCES executions(id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
    
    -- Composite unique constraint
    UNIQUE(agent_id, context_type, conversation_id, execution_id, context_key)
);

-- Streamlined collaboration tracking

CREATE TABLE agent_collaborations (
    id BLOB PRIMARY KEY NOT NULL,
    workspace_id BLOB,
    name TEXT NOT NULL,
    collaboration_type INTEGER NOT NULL DEFAULT 0, -- 0: 'DISCUSSION', 1: 'PROBLEM_SOLVING', 2: 'WORKFLOW'
    
    -- Participants (JSON array of agent IDs with roles)
    participants TEXT NOT NULL CHECK (json_valid(participants)),
    coordinator_agent_id BLOB,
    
    -- Session data
    objective TEXT,
    context_data TEXT CHECK (context_data IS NULL OR json_valid(context_data)),
    results TEXT CHECK (results IS NULL OR json_valid(results)),
    
    -- Status
    status INTEGER NOT NULL DEFAULT 0, -- 0: 'ACTIVE', 1: 'PAUSED', 2: 'COMPLETED', 3: 'CANCELLED'
    
    -- Timing
    started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at TIMESTAMP,
    
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL,
    FOREIGN KEY (coordinator_agent_id) REFERENCES agents(id) ON DELETE SET NULL
);

-- Single table for all execution types

CREATE TABLE executions (
    id BLOB PRIMARY KEY NOT NULL,
    execution_type INTEGER NOT NULL DEFAULT 0, -- 'AGENT_CALL', 'TOOL_CALL', 'CHAIN_STEP'
    
    -- Context
    agent_id BLOB,
    conversation_id BLOB,
    parent_execution_id BLOB,
    
    -- Execution details
    status INTEGER NOT NULL DEFAULT 0, -- 'PENDING', 'RUNNING', 'COMPLETED', 'FAILED'
    
    -- Data
    input_data TEXT CHECK (input_data IS NULL OR json_valid(input_data)),
    output_data TEXT CHECK (output_data IS NULL OR json_valid(output_data)),
    error_details TEXT,
    
    -- Metrics
    started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP,
    duration_ms INTEGER,
    tokens_used INTEGER DEFAULT 0,
    cost_estimate REAL DEFAULT 0.0,
    
    -- Tool-specific fields
    tool_id BLOB,
    tool_parameters TEXT CHECK (tool_parameters IS NULL OR json_valid(tool_parameters)),
    
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),
    
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE SET NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE SET NULL,
    FOREIGN KEY (parent_execution_id) REFERENCES executions(id) ON DELETE SET NULL,
    FOREIGN KEY (tool_id) REFERENCES tools(id) ON DELETE SET NULL
);

CREATE INDEX idx_executions_agent_id ON executions(agent_id);
CREATE INDEX idx_executions_conversation_id ON executions(conversation_id);
CREATE INDEX idx_executions_parent_execution_id ON executions(parent_execution_id);
CREATE INDEX idx_executions_tool_id ON executions(tool_id);
CREATE INDEX idx_executions_status ON executions(status);

-- Comprehensive agent metrics

CREATE TABLE agent_performance_metrics (
    id BLOB PRIMARY KEY NOT NULL,
    agent_id BLOB NOT NULL,
    conversation_id BLOB,
    metric_type INTEGER NOT NULL, -- response_time, accuracy, user_satisfaction
    value REAL NOT NULL,
    context TEXT CHECK (context IS NULL OR json_valid(context)), -- JSON with metric context
    measured_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE SET NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE SET NULL
);

CREATE INDEX idx_agent_performance_metrics_agent_id ON agent_performance_metrics(agent_id);

CREATE TABLE api_keys (
    id BLOB PRIMARY KEY NOT NULL,
    account_id BLOB NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    key_hash TEXT NOT NULL,
    scopes TEXT NOT NULL,
    expires_at TIMESTAMP,
    rate_limit INTEGER,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_used_at TIMESTAMP,
    UNIQUE(account_id, name),
    UNIQUE(key_hash)
);

CREATE INDEX idx_api_keys_account_id ON api_keys(account_id);

CREATE TABLE attachments (
    id BLOB PRIMARY KEY NOT NULL,
    workspace_id BLOB,
    message_id BLOB NOT NULL,
    file_id BLOB NOT NULL,
    attachment_type INTEGER NOT NULL DEFAULT 0,  -- 'IMAGE', 'VIDEO', 'AUDIO', 'FILE', 'LINK'
    url TEXT NOT NULL,
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),  -- JSON with mimeType, dimensions, etc.
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL,
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE SET NULL,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE SET NULL
);

CREATE INDEX idx_attachments_workspace_id ON attachments(workspace_id);
CREATE INDEX idx_attachments_message_id ON attachments(message_id);
CREATE INDEX idx_attachments_file_id ON attachments(file_id);

CREATE TABLE audit_log (
    id BLOB PRIMARY KEY NOT NULL,
    table_name TEXT NOT NULL,
    record_id BLOB NOT NULL,
    operation TEXT NOT NULL, -- INSERT, UPDATE, DELETE
    user_id BLOB,
    old_values TEXT CHECK (old_values IS NULL OR json_valid(old_values)), -- JSON
    new_values TEXT CHECK (new_values IS NULL OR json_valid(new_values)), -- JSON
    ip_address TEXT,
    user_agent TEXT,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_audit_log_table_record ON audit_log(table_name, record_id);
CREATE INDEX idx_audit_log_user_time ON audit_log(user_id, timestamp DESC);

CREATE TABLE comments (
    id BLOB PRIMARY KEY NOT NULL,
    workspace_id BLOB,
    message_id BLOB,
    parent_comment_id BLOB,
    conversation_id BLOB,
    task_id BLOB,
    event_id BLOB,
    type INTEGER NOT NULL DEFAULT 0, -- 'COMMENT', 'TASK', 'EVENT', 'OTHER'
    content TEXT NOT NULL,
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),  -- JSON with metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL,
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE SET NULL,
    FOREIGN KEY (parent_comment_id) REFERENCES comments(id) ON DELETE SET NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE SET NULL,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE SET NULL,
    FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE SET NULL
);

CREATE INDEX idx_comments_workspace_id ON comments(workspace_id);
CREATE INDEX idx_comments_message_id ON comments(message_id);
CREATE INDEX idx_comments_parent_comment_id ON comments(parent_comment_id);
CREATE INDEX idx_comments_conversation_id ON comments(conversation_id);
CREATE INDEX idx_comments_task_id ON comments(task_id);
CREATE INDEX idx_comments_event_id ON comments(event_id);
CREATE INDEX idx_comments_created_at ON comments(created_at);
CREATE INDEX idx_comments_updated_at ON comments(updated_at);

CREATE TABLE contacts (
    id BLOB PRIMARY KEY NOT NULL,
    workspace_id BLOB,
    user_id BLOB, -- Link to a user if the contact is a registered user
    name TEXT NOT NULL,
    first_name TEXT,
    last_name TEXT,
    mobile_phone TEXT,
    home_phone TEXT,
    work_phone TEXT,
    email TEXT,
    website TEXT,
    job_title TEXT,
    company TEXT,
    department TEXT,
    primary_address_id BLOB,
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),  -- JSON object with additional metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (primary_address_id) REFERENCES addresses(id) ON DELETE SET NULL
);

CREATE INDEX idx_contacts_workspace_id ON contacts(workspace_id);
CREATE INDEX idx_contacts_name ON contacts(name);
CREATE INDEX idx_contacts_first_name ON contacts(first_name);
CREATE INDEX idx_contacts_last_name ON contacts(last_name);

CREATE TABLE contexts (
    id BLOB PRIMARY KEY NOT NULL,
    workspace_id BLOB,
    conversation_id BLOB NOT NULL,
    name TEXT NOT NULL,
    type INTEGER NOT NULL DEFAULT 0, -- 'SESSION', 'TOPIC', 'AGENT', 'PROJECT', 'TOOL', 'OTHER'
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)), -- JSON object with additional metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE SET NULL
);

CREATE INDEX idx_contexts_workspace_id ON contexts(workspace_id);
CREATE INDEX idx_contexts_conversation_id ON contexts(conversation_id);
CREATE INDEX idx_contexts_type ON contexts(type);

CREATE TABLE conversations (
    id BLOB PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    conversation_type INTEGER NOT NULL DEFAULT 0,  -- 0: 'DIRECT', 1: 'GROUP'
    status INTEGER NOT NULL DEFAULT 0,  -- 0: 'ACTIVE', 1: 'INACTIVE', 2: 'DELETED'
    parent_conversation_id BLOB,
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),  -- JSON object with additional attributes
    last_message_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    workspace_id BLOB,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_conversation_id) REFERENCES conversations(id) ON DELETE SET NULL
);

CREATE INDEX idx_conversations_title ON conversations(title);
CREATE INDEX idx_conversations_last_message_at ON conversations(last_message_at);
CREATE INDEX idx_conversations_created_at ON conversations(created_at);

CREATE TABLE conversation_participants (
    conversation_id BLOB NOT NULL,
    participant_id BLOB NOT NULL,
    role INTEGER NOT NULL DEFAULT 0,  -- 0: 'OWNER', 1: 'ADMIN', 2: 'MEMBER', 3: 'ASSISTANT', 4: 'OBSERVER'
    joined_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    left_at TIMESTAMP,
    is_active BOOLEAN NOT NULL DEFAULT 1, -- 0: false, 1: true
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE SET NULL,
    FOREIGN KEY (participant_id) REFERENCES participants(id) ON DELETE SET NULL,
    PRIMARY KEY(conversation_id, participant_id)
);

CREATE INDEX idx_conversation_participants_conversation_id ON conversation_participants(conversation_id);
CREATE INDEX idx_conversation_participants_participant_id ON conversation_participants(participant_id);
CREATE INDEX idx_conversation_participants_role ON conversation_participants(role);

CREATE TABLE credentials (
    id BLOB PRIMARY KEY NOT NULL,
    workspace_id BLOB,
    name TEXT NOT NULL,
    credential_name TEXT NOT NULL,
    encrypted_value TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL
);

CREATE INDEX idx_credentials_workspace_id ON credentials(workspace_id);

CREATE TABLE documents (
    id BLOB PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    document_type INTEGER NOT NULL DEFAULT 0,
    mime_type TEXT,
    size_bytes INTEGER NOT NULL DEFAULT 0,
    content TEXT,
    metadata TEXT,
    file_path TEXT,
    url TEXT,
    is_indexed BOOLEAN NOT NULL DEFAULT 0,
    is_embedded BOOLEAN NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    workspace_id BLOB,
    owner_id BLOB,
    FOREIGN KEY (owner_id) REFERENCES participants(id) ON DELETE SET NULL,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL
);

CREATE INDEX idx_documents_workspace_id ON documents(workspace_id);
CREATE INDEX idx_documents_document_type ON documents(document_type);

CREATE TABLE document_chunks (
    id BLOB PRIMARY KEY NOT NULL,
    document_id BLOB NOT NULL,
    parent_chunk_id BLOB, -- For hierarchical chunking
    content TEXT NOT NULL,
    content_hash TEXT NOT NULL, -- SHA256 for deduplication
    chunk_type INTEGER NOT NULL DEFAULT 0, -- 0: 'semantic', 1: 'fixed-size', 2: 'paragraph', 3: 'sentence', 4: 'other'
    order_index INTEGER NOT NULL, -- The order of the chunk in the document
    semantic_level INTEGER DEFAULT 0, -- 0: 'header', 1: 'paragraph', 2: 'sentence', 3: 'other'
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)), -- JSON with metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    embedding_id BLOB, -- Link to vector storage
    workspace_id BLOB,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE SET NULL,
    FOREIGN KEY (parent_chunk_id) REFERENCES document_chunks(id) ON DELETE SET NULL,
    FOREIGN KEY (embedding_id) REFERENCES memory_vectors(id) ON DELETE SET NULL
);

CREATE INDEX idx_document_chunks_order ON document_chunks(document_id, order_index);
CREATE INDEX idx_document_chunks_semantic ON document_chunks(document_id, semantic_level, order_index);
CREATE INDEX idx_document_chunks_hash ON document_chunks(content_hash);
CREATE INDEX idx_document_chunks_workspace_id ON document_chunks(workspace_id);

CREATE TABLE events (
    id BLOB PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    event_type INTEGER NOT NULL DEFAULT 0, -- 'MEETING', 'CALL', 'EMAIL', 'TASK', 'OTHER'
    status INTEGER NOT NULL DEFAULT 0, -- 'SCHEDULED', 'COMPLETED', 'CANCELLED', 'RESCHEDULED'
    -- Event timing
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP NOT NULL,
    is_all_day_event BOOLEAN NOT NULL DEFAULT 0, -- 0: false, 1: true   
    timezone_sid_key TEXT DEFAULT "UTC", -- Timezone identifier e.g. "America/New_York" 
    -- Location and format
    location TEXT, -- Physical location of the event
    virtual_meeting_url TEXT, -- URL for the virtual meeting e.g. video call, zoom, etc. url
    meeting_platform TEXT, -- Platform for the virtual meeting e.g. zoom, google meet, etc.
    -- Recurrence
    is_recurrence BOOLEAN NOT NULL DEFAULT 0, -- 0: false, 1: true
    recurrence_rule TEXT, -- Json object of the recurrence rule e.g. "FREQ=DAILY;INTERVAL=1;COUNT=10"
    recurrence_parent_id BLOB, -- Id of the parent event if this is a recurrence
    -- Agent participation configuration
    agent_participation TEXT, -- Json object of how the agent participated in the event e.g. "ATTENDEE", "ORGANIZER", "OTHER"
    requires_transcription BOOLEAN NOT NULL DEFAULT 0, -- 0: false, 1: true
    requires_summarization BOOLEAN NOT NULL DEFAULT 0, -- 0: false, 1: true
    agent_capabilities TEXT, -- Json object of the required agent capabilities e.g. "TRANSCRIPTION", "SUMMARIZATION", "OTHER"
    -- Event configuration related fields
    is_private BOOLEAN NOT NULL DEFAULT 0, -- 0: false, 1: true
    allow_guests BOOLEAN NOT NULL DEFAULT 0, -- 0: false, 1: true
    max_attendees INTEGER,
    requires_approval BOOLEAN NOT NULL DEFAULT 0, -- 0: false, 1: true
    -- Meeting specific fields
    agenda TEXT, -- Agenda of the event
    meeting_notes TEXT, -- Notes from the meeting
    transcription TEXT, -- Transcription of the meeting
    summary TEXT, -- Summary of the meeting
    action_items TEXT, -- Json object with extracted action items from the meeting
    -- Meeting specific fields
    is_child_event BOOLEAN NOT NULL DEFAULT 0, -- 0: false, 1: true
    is_group_event BOOLEAN NOT NULL DEFAULT 0, -- 0: false, 1: true
    is_archived BOOLEAN NOT NULL DEFAULT 0, -- 0: false, 1: true    
    event_relation TEXT, -- Json object of the event relation e.g. "ATTENDEE", "ORGANIZER", "OTHER"
    activity_date TIMESTAMP,
    duration_in_minutes INTEGER,
    show_as TEXT,
    is_reminder_set BOOLEAN NOT NULL DEFAULT 0, -- 0: false, 1: true
    reminder_date_time TIMESTAMP NOT NULL,
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)), -- JSON with metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- Relationships
    plan_id BLOB,
    task_id BLOB,
    created_by_user_id BLOB,
    last_modified_by_user_id BLOB,
    workspace_id BLOB,
    parent_event_id BLOB,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL,
    FOREIGN KEY (created_by_user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (last_modified_by_user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (parent_event_id) REFERENCES events(id) ON DELETE SET NULL,
    FOREIGN KEY (plan_id) REFERENCES plans(id) ON DELETE SET NULL,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE SET NULL
);

CREATE INDEX idx_events_workspace_id ON events(workspace_id);
CREATE INDEX idx_events_created_at ON events(created_at);
CREATE INDEX idx_events_updated_at ON events(updated_at);

CREATE TABLE event_participants (
    id BLOB PRIMARY KEY NOT NULL,
    event_id BLOB NOT NULL,
    -- Participant identity
    participant_id BLOB,
    email TEXT,
    -- Participation details
    role INTEGER NOT NULL DEFAULT 0, -- 'ATTENDEE', 'ORGANIZER', 'OTHER'
    status INTEGER NOT NULL DEFAULT 0, -- 'PENDING', 'ACCEPTED', 'DECLINED', 'TENTATIVE', 'OTHER'
    response_at TIMESTAMP,
    -- Agent-specific configuration
    agent_role TEXT, -- 'ATTENDEE', 'ORGANIZER', 'OTHER'
    agent_config TEXT, -- Json object of the agent configuration for the event
    -- Communication preferences
    notification_preferences TEXT, -- Json object of the notification preferences for the event
    joined_at TIMESTAMP,
    left_at TIMESTAMP,
    -- Meeting participation
    is_present BOOLEAN NOT NULL DEFAULT 0, -- 0: false, 1: true
    is_muted BOOLEAN NOT NULL DEFAULT 0, -- 0: false, 1: true
    is_video_on BOOLEAN NOT NULL DEFAULT 0, -- 0: false, 1: true
    speaking_time INTEGER, -- in seconds
    -- Participant metadata
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)), -- JSON with metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    --Relationships
    FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE SET NULL,
    FOREIGN KEY (participant_id) REFERENCES participants(id) ON DELETE SET NULL
);

CREATE INDEX idx_event_participants_event_id ON event_participants(event_id);

CREATE TABLE files (
    id BLOB PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    type_ INTEGER NOT NULL DEFAULT 0,
    metadata TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_files_type ON files(type_);

CREATE TABLE groups (
    id BLOB PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    avatar TEXT,
    type INTEGER NOT NULL DEFAULT 0, -- 'PRIVATE', 'PUBLIC', 'OTHER'
    status INTEGER NOT NULL DEFAULT 0, -- 'ACTIVE', 'ARCHIVED', 'DELETED'
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)), -- JSON with metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    workspace_id BLOB, 
    parent_group_id BLOB,
    FOREIGN KEY (parent_group_id) REFERENCES groups(id) ON DELETE SET NULL,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL
);

CREATE INDEX idx_groups_workspace_id ON groups(workspace_id);
CREATE INDEX idx_groups_type ON groups(type);
CREATE INDEX idx_groups_status ON groups(status);

CREATE TABLE group_members (
    group_id BLOB NOT NULL,
    participant_id BLOB NOT NULL,
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE SET NULL, 
    FOREIGN KEY (participant_id) REFERENCES participants(id) ON DELETE SET NULL,
    PRIMARY KEY (group_id, participant_id),
    UNIQUE(group_id, participant_id)
);

CREATE TABLE mcp_servers (
    id BLOB PRIMARY KEY NOT NULL,
    workspace_id BLOB,
    name TEXT NOT NULL,
    type TEXT NOT NULL,
    status INTEGER NOT NULL,       -- 'ACTIVE', 'INACTIVE', 'MAINTENANCE'
    url TEXT NOT NULL,
    capabilities TEXT CHECK (capabilities IS NULL OR json_valid(capabilities)),          -- JSON array of capabilities
    metrics TEXT CHECK (metrics IS NULL OR json_valid(metrics)),               -- JSON with server metrics
    configuration TEXT CHECK (configuration IS NULL OR json_valid(configuration)),         -- JSON with configuration
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL
);

CREATE INDEX idx_mcp_servers_workspace_id ON mcp_servers(workspace_id);
CREATE INDEX idx_mcp_servers_status ON mcp_servers(status);
CREATE INDEX idx_mcp_servers_type ON mcp_servers(type);

CREATE TABLE mcp_tools (
    id BLOB PRIMARY KEY NOT NULL,
    mcp_server_id BLOB NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    is_enabled BOOLEAN NOT NULL DEFAULT 0, -- 0: false, 1: true
    type TEXT NOT NULL, -- 'REST', 'GRPC', 'LOCAL'
    status INTEGER NOT NULL DEFAULT 0, -- 'ACTIVE', 'ARCHIVED', 'DELETED'
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (mcp_server_id) REFERENCES mcp_servers(id) ON DELETE SET NULL
);

-- Memory items are the items of the memory. They are the items of the memory.

CREATE TABLE memories (
    id BLOB PRIMARY KEY NOT NULL,
    workspace_id BLOB,
    participant_id BLOB,
    conversation_id BLOB,
    
    -- Memory classification
    memory_type INTEGER NOT NULL DEFAULT 0, -- 0: 'EPISODIC', 1: 'SEMANTIC', 2: 'PROCEDURAL'
    content TEXT NOT NULL,
    summary TEXT,
    
    -- Importance and retrieval
    importance REAL NOT NULL DEFAULT 0.5, -- 0.0 to 1.0, 0.5 is default, Importance of the memory.
    last_accessed_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    access_count INTEGER NOT NULL DEFAULT 0,
    
    -- Embeddings (store as JSON array for simplicity)
    embedding TEXT CHECK (embedding IS NULL OR json_valid(embedding)),
    
    -- Metadata
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),
    
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL,
    FOREIGN KEY (participant_id) REFERENCES participants(id) ON DELETE SET NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE SET NULL
);

CREATE INDEX idx_memories_participant_id ON memories(participant_id);
CREATE INDEX idx_memories_conversation_id ON memories(conversation_id);
CREATE INDEX idx_memories_importance ON memories(importance DESC);
CREATE INDEX idx_memories_type ON memories(memory_type);

-- Memory vectors are the vectors of the memory. They are the vectors of the memory.

CREATE TABLE memory_vectors (
    id BLOB PRIMARY KEY NOT NULL,
    memory_id BLOB NOT NULL,
    vector TEXT NOT NULL, -- JSON array of floats
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (memory_id) REFERENCES memories(id) ON DELETE CASCADE
);

CREATE INDEX idx_memory_vectors_memory_id ON memory_vectors(memory_id);

-- Sessions are the sessions of the memory. They are the sessions of the memory.

CREATE TABLE memory_sessions (
    id BLOB PRIMARY KEY NOT NULL,
    start_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    end_time TIMESTAMP,
    objective TEXT, -- The objective of the session
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)) -- JSON with session metadata, tags, etc.
);

-- Sources are the sources of the memory. They are the sources of the memory.

CREATE TABLE memory_sources (
    id BLOB PRIMARY KEY NOT NULL,
    source_type INTEGER NOT NULL DEFAULT 0, -- 0: 'USER', 1: 'DOCUMENT', 2: 'API', 3: 'AGENT', 4: 'SYSTEM', 5: 'OTHER'
    identifier TEXT NOT NULL,
    description TEXT,
    user_id BLOB, -- If the source was a user
    memories TEXT CHECK (memories IS NULL OR json_valid(memories)), -- JSON array of memory item IDs
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);

CREATE TABLE messages (
    id BLOB PRIMARY KEY NOT NULL,
    conversation_id BLOB NOT NULL,
    sender_id BLOB NOT NULL, -- participant_id of the sender
    parent_message_id BLOB, -- For threading
    content TEXT NOT NULL,     -- JSON object with raw and parsed content
    status INTEGER NOT NULL DEFAULT 0,      -- 0: 'pending', 1: 'sent', 2: 'delivered', 3: 'read', 4: 'failed'
    refs TEXT CHECK (metadata IS NULL OR json_valid(metadata)),  -- JSON array of referenced message IDs
    -- JSON containing files, reactions, etc. Simpler than a separate attachments table for V1.
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),  -- JSON with attachments, reactions, etc.
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    reply_to_id BLOB, -- For threading
    branch_conversation_id BLOB, -- For branching conversations
    parent_id BLOB, -- For threading
    workspace_id BLOB,      -- The workspace that the message belongs to
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
    FOREIGN KEY (sender_id) REFERENCES participants(id) ON DELETE CASCADE,
    FOREIGN KEY (reply_to_id) REFERENCES messages(id) ON DELETE CASCADE,
    FOREIGN KEY (branch_conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_message_id) REFERENCES messages(id) ON DELETE SET NULL
);

CREATE INDEX idx_messages_conversation_id ON messages(conversation_id) WHERE conversation_id IS NOT NULL;
CREATE INDEX idx_messages_workspace_id ON messages(workspace_id) WHERE workspace_id IS NOT NULL;
CREATE INDEX idx_messages_sender_id ON messages(sender_id) WHERE sender_id IS NOT NULL;

CREATE TABLE models (
    id BLOB PRIMARY KEY NOT NULL,
    provider TEXT NOT NULL, -- 'openai', 'anthropic', 'google', 'local'
    name TEXT NOT NULL,
    display_name TEXT,
    model_type INTEGER NOT NULL DEFAULT 0, -- 0: 'TEXT', 1: 'VISION', 2: 'EMBEDDING', 3: 'AUDIO'
    
    -- Capabilities
    context_size INTEGER NOT NULL DEFAULT 4096,
    max_tokens INTEGER NOT NULL DEFAULT 4096,
    supports_functions BOOLEAN NOT NULL DEFAULT 0,
    supports_vision BOOLEAN NOT NULL DEFAULT 0,
    supports_streaming BOOLEAN NOT NULL DEFAULT 1,
    
    -- Pricing (per 1M tokens)
    input_cost REAL,
    output_cost REAL,
    
    -- Configuration
    config TEXT CHECK (config IS NULL OR json_valid(config)),
    
    -- Status
    is_active BOOLEAN NOT NULL DEFAULT 1,
    is_deprecated BOOLEAN NOT NULL DEFAULT 0,

    registry_id BLOB,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (registry_id) REFERENCES registry(id) ON DELETE CASCADE
);

CREATE INDEX idx_models_registry_id ON models(registry_id);
CREATE INDEX idx_models_is_active ON models(is_active);

CREATE TABLE agent_models (
    agent_id BLOB NOT NULL,
    model_id BLOB NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE SET NULL,
    FOREIGN KEY (model_id) REFERENCES models(id) ON DELETE SET NULL,
    PRIMARY KEY (agent_id, model_id)
);

CREATE INDEX idx_agent_models_agent_id ON agent_models(agent_id);
CREATE INDEX idx_agent_models_model_id ON agent_models(model_id);

CREATE TABLE notes (
    id BLOB PRIMARY KEY NOT NULL,
    workspace_id BLOB,
    parent_note_id BLOB,
    task_id BLOB,
    event_id BLOB,
    type INTEGER NOT NULL DEFAULT 0, -- 'note', 'task', 'event', 'other'
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),  -- JSON with metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL,
    FOREIGN KEY (parent_note_id) REFERENCES notes(id) ON DELETE SET NULL,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE SET NULL,
    FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE SET NULL
);

CREATE INDEX idx_notes_workspace_id ON notes(workspace_id);
CREATE INDEX idx_notes_parent_note_id ON notes(parent_note_id);
CREATE INDEX idx_notes_task_id ON notes(task_id);
CREATE INDEX idx_notes_event_id ON notes(event_id);

CREATE TABLE notifications (
    id BLOB PRIMARY KEY NOT NULL,
    recipient_id BLOB NOT NULL,
    message TEXT NOT NULL,
    is_read BOOLEAN NOT NULL DEFAULT 0, -- 0: false, 1: true
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    task_id BLOB,
    event_id BLOB,
    agent_id BLOB,
    FOREIGN KEY (recipient_id) REFERENCES participants(id) ON DELETE SET NULL,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE SET NULL,
    FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE SET NULL
);

CREATE INDEX idx_notifications_recipient_id ON notifications(recipient_id);

-- Enhanced P2P Messaging Support

-- P2P Nodes: Track known P2P nodes in the network

CREATE TABLE p2p_nodes (
    participant_id BLOB NOT NULL,
    peer_id BLOB NOT NULL,
    node_type INTEGER NOT NULL DEFAULT 0, -- 0: 'AGENT_NODE', 1: 'GATEWAY_NODE', 2: 'RELAY_NODE'
    multiaddr TEXT NOT NULL, -- libp2p multiaddress
    public_key TEXT,
    capabilities TEXT CHECK (capabilities IS NULL OR json_valid(capabilities)), -- JSON array of node capabilities
    status INTEGER NOT NULL DEFAULT 0, -- 0: 'ONLINE', 1: 'OFFLINE', 2: 'UNREACHABLE'
    last_seen TIMESTAMP,
    connection_quality REAL, -- 0.0 to 1.0 connection quality score (nullable)
    latency_ms INTEGER, -- Changed to INTEGER to match i64 in Rust
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(participant_id, peer_id)
);

CREATE INDEX idx_p2p_nodes_peer_id ON p2p_nodes(peer_id);
CREATE INDEX idx_p2p_nodes_status ON p2p_nodes(status);
CREATE INDEX idx_p2p_nodes_last_seen ON p2p_nodes(last_seen);

-- P2P Message Queue: Queue for P2P messages

CREATE TABLE p2p_message_queue (
    id BLOB PRIMARY KEY NOT NULL,
    from_peer_id TEXT NOT NULL,
    to_peer_id TEXT NOT NULL,
    message_type INTEGER NOT NULL DEFAULT 0, -- 0: 'AGENT_MESSAGE', 1: 'SYSTEM_MESSAGE', 2: 'HEARTBEAT'
    priority INTEGER NOT NULL DEFAULT 0, -- 0: 'LOW', 1: 'NORMAL', 2: 'HIGH', 3: 'URGENT'
    payload TEXT NOT NULL, -- JSON message payload
    conversation_id BLOB,
    agent_chain_execution_id BLOB,
    status INTEGER NOT NULL DEFAULT 0, -- 0: 'PENDING', 1: 'SENT', 2: 'DELIVERED', 3: 'FAILED', 4: 'EXPIRED'
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    expires_at TIMESTAMP,
    sent_at TIMESTAMP,
    delivered_at TIMESTAMP,
    error_details TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE SET NULL,
    FOREIGN KEY (agent_chain_execution_id) REFERENCES agent_chain_executions(id) ON DELETE SET NULL
);

CREATE INDEX idx_p2p_message_queue_to_peer ON p2p_message_queue(to_peer_id);
CREATE INDEX idx_p2p_message_queue_status ON p2p_message_queue(status);
CREATE INDEX idx_p2p_message_queue_priority ON p2p_message_queue(priority DESC);
CREATE INDEX idx_p2p_message_queue_created_at ON p2p_message_queue(created_at);

CREATE TABLE participants ( -- Participant is a user, agent, contact (actor patterns)
    id BLOB PRIMARY KEY NOT NULL,
    user_id BLOB UNIQUE,
    agent_id BLOB UNIQUE,
    contact_id BLOB UNIQUE,
    display_name TEXT NOT NULL,
    avatar_url TEXT,
    participant_type INTEGER NOT NULL DEFAULT 0, -- 0: 'user', 1: 'agent', 2: 'contact'  
    status INTEGER NOT NULL DEFAULT 0,  -- 'active', 'inactive', 'busy', 'offline'
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),  -- JSON object with additional metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    workspace_id BLOB,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

CREATE INDEX idx_participants_workspace_id ON participants(workspace_id);
CREATE INDEX idx_participants_display_name ON participants(display_name);
CREATE INDEX idx_participants_user_id ON participants(user_id);
CREATE INDEX idx_participants_agent_id ON participants(agent_id);
CREATE INDEX idx_participants_contact_id ON participants(contact_id);

CREATE TABLE plans (
    id BLOB PRIMARY KEY NOT NULL,
    owner_participant_id BLOB NOT NULL,
    plan_type INTEGER NOT NULL, -- 'task', 'goal', 'other'
    plan_status INTEGER NOT NULL, -- 'pending', 'in_progress', 'completed', 'failed'
    plan_metadata TEXT CHECK (plan_metadata IS NULL OR json_valid(plan_metadata)), -- JSON with plan metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (owner_participant_id) REFERENCES participants(id) ON DELETE SET NULL
);

CREATE INDEX idx_plans_owner_participant_id ON plans(owner_participant_id);
CREATE INDEX idx_plans_plan_type ON plans(plan_type);
CREATE INDEX idx_plans_plan_status ON plans(plan_status);

CREATE TABLE procedures (
    id BLOB PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    execution_count INTEGER NOT NULL DEFAULT 0, -- Number of times the procedure has been executed
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_procedures_name ON procedures(name);
CREATE INDEX idx_procedures_description ON procedures(description);

CREATE TABLE procedures_steps (
    id BLOB PRIMARY KEY NOT NULL,
    procedure_id BLOB NOT NULL,
    step_number INTEGER NOT NULL,
    instruction TEXT NOT NULL,
    step_type TEXT NOT NULL,
    sub_procedure_id BLOB,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (procedure_id) REFERENCES procedures(id) ON DELETE SET NULL,
    FOREIGN KEY (sub_procedure_id) REFERENCES procedures(id) ON DELETE SET NULL
);

CREATE INDEX idx_procedures_steps_procedure_id ON procedures_steps(procedure_id);
CREATE INDEX idx_procedures_steps_step_number ON procedures_steps(step_number);

CREATE TABLE prompts (
    id BLOB PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    content TEXT NOT NULL,
    prompt_type INTEGER NOT NULL DEFAULT 0, -- 'system', 'user', 'agent', 'tool', 'other'
    status INTEGER NOT NULL DEFAULT 0, -- 'active', 'archived', 'deleted'
    template TEXT NOT NULL, -- JSON object with template
    variables TEXT, -- JSON array of variable definitions
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)), -- JSON object with additional metadata
    tags TEXT NOT NULL DEFAULT '', -- JSON array of tags
    created_by_id BLOB,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    workspace_id BLOB,
    FOREIGN KEY (created_by_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL
);

CREATE INDEX idx_prompts_workspace_id ON prompts(workspace_id);
CREATE INDEX idx_prompts_title ON prompts(title);
CREATE INDEX idx_prompts_created_by_id ON prompts(created_by_id);

CREATE TABLE registry (
    id BLOB PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    version TEXT NOT NULL DEFAULT '1.0.0',
    registry_type INTEGER NOT NULL DEFAULT 0, -- 0: 'MODEL', 1: 'TOOL', 2: 'AGENT', 3: 'OTHER'
    config TEXT CHECK (config IS NULL OR json_valid(config)),  -- JSON object with additional metadata
    is_public BOOLEAN NOT NULL DEFAULT 0, -- 0: false, 1: true
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    workspace_id BLOB,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL
);

CREATE INDEX idx_registry_workspace_id ON registry(workspace_id);
CREATE INDEX idx_registry_type ON registry(registry_type);

CREATE TABLE tasks (
    id BLOB PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status INTEGER NOT NULL DEFAULT 0, -- 0: TODO, 1: IN_PROGRESS, 2: COMPLETED, 3: FAILED
    start_time TIMESTAMP NOT NULL, -- RFC3339 string
    end_time TIMESTAMP, -- RFC3339 string
    due_date TIMESTAMP, -- RFC3339 string
    priority INTEGER NOT NULL DEFAULT 0, -- 0: LOW, 1: MEDIUM, 2: HIGH
    importance INTEGER NOT NULL DEFAULT 0, -- 0: LOW, 1: MEDIUM, 2: HIGH
    tags TEXT NOT NULL DEFAULT '', -- JSON array of tags
    url TEXT,
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)), -- JSON with task metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by_id BLOB,  
    assignee_participant_id BLOB,
    workspace_id BLOB,  
    conversation_id BLOB,
    memory_id BLOB,
    plan_id BLOB,
    document_id BLOB,
    file_id BLOB,
    FOREIGN KEY (plan_id) REFERENCES plans(id) ON DELETE SET NULL,
    FOREIGN KEY (created_by_id) REFERENCES participants(id) ON DELETE SET NULL,
    FOREIGN KEY (assignee_participant_id) REFERENCES participants(id) ON DELETE SET NULL
);

CREATE INDEX idx_tasks_plan_id ON tasks(plan_id);
CREATE INDEX idx_tasks_conversation_id ON tasks(conversation_id);
CREATE INDEX idx_tasks_memory_id ON tasks(memory_id);
CREATE INDEX idx_tasks_document_id ON tasks(document_id);

CREATE TABLE task_assignees (
    task_id BLOB NOT NULL,
    participant_id BLOB NOT NULL,
    role INTEGER NOT NULL DEFAULT 0, -- 0: 'PRIMARY', 1: 'SECONDARY', 2: 'OTHER'
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE SET NULL,
    FOREIGN KEY (participant_id) REFERENCES participants(id) ON DELETE SET NULL,
    PRIMARY KEY (task_id, participant_id)
);

CREATE INDEX idx_task_assignees_task_id ON task_assignees(task_id);
CREATE INDEX idx_task_assignees_participant_id ON task_assignees(participant_id);

CREATE TABLE tools (
    id BLOB PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    category INTEGER NOT NULL DEFAULT 0, -- 0: 'web', 1: 'file', 2: 'code', 3: 'data', 4: 'communication'
    
    -- Tool definition
    definition TEXT CHECK (definition IS NULL OR json_valid(definition)), -- OpenAPI-style schema
    
    -- Configuration
    config TEXT CHECK (config IS NULL OR json_valid(config)),
    auth_required BOOLEAN NOT NULL DEFAULT 0,
    
    -- Status
    is_active BOOLEAN NOT NULL DEFAULT 1,
    
    -- Relationships
    workspace_id BLOB,
    created_by_id BLOB,
    
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL,
    FOREIGN KEY (created_by_id) REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX idx_tools_created_by_id ON tools(created_by_id);
CREATE INDEX idx_tools_workspace_id ON tools(workspace_id);

CREATE TABLE agent_tools (
    agent_id BLOB NOT NULL,
    tool_id BLOB NOT NULL,
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE,
    FOREIGN KEY (tool_id) REFERENCES tools(id) ON DELETE CASCADE,
    PRIMARY KEY (agent_id, tool_id)
);

CREATE INDEX idx_agent_tools_agent_id ON agent_tools(agent_id);
CREATE INDEX idx_agent_tools_tool_id ON agent_tools(tool_id);

CREATE TABLE users (
    id BLOB PRIMARY KEY NOT NULL,
    contact_id BLOB,
    email TEXT UNIQUE,
    username TEXT UNIQUE,
    operator_agent_id BLOB UNIQUE,
    display_name TEXT NOT NULL,
    first_name TEXT,
    last_name TEXT,
    mobile_phone TEXT,
    avatar_url TEXT,
    bio TEXT,
    status INTEGER NOT NULL DEFAULT 0,  -- 'ACTIVE', 'INACTIVE', 'DELETED'
    email_verified BOOLEAN NOT NULL DEFAULT 0, -- 0: false, 1: true
    phone_verified BOOLEAN NOT NULL DEFAULT 0, -- 0: false, 1: true
    last_seen TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,        -- ISO 8601 DateTime string
    primary_role INTEGER NOT NULL DEFAULT 0, -- 0: 'user', 1: 'admin', 2: 'agent', 3: 'contact', 4: 'other' 
    roles TEXT NOT NULL,            -- JSON array of roles. Consider a separate roles table for complex scenarios.
    preferences TEXT CHECK (preferences IS NULL OR json_valid(preferences)),      -- JSON object with user preferences
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)), -- JSON object with additional metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    workspace_id BLOB,
    public_key BLOB NOT NULL,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL,
    FOREIGN KEY (operator_agent_id) REFERENCES agents(id) ON DELETE SET NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE SET NULL
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_contact_id ON users(contact_id);
CREATE INDEX idx_users_operator_agent_id ON users(operator_agent_id);

CREATE TABLE workflows (
    id BLOB PRIMARY KEY NOT NULL,
    workspace_id BLOB,
    name TEXT NOT NULL,
    description TEXT,
    workflow_type INTEGER NOT NULL DEFAULT 0, -- 0: AGENT_CHAIN, 1: DATA_PIPELINE, 2: LONG_RUNNING_PROCESS, 3: TRADITIONAL_WORKFLOW
    status INTEGER NOT NULL DEFAULT 0,        -- 0: DRAFT, 1: ACTIVE, 2: PAUSED, 3: COMPLETED, 4: FAILED
    config TEXT CHECK (config IS NULL OR json_valid(config)),
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
);

CREATE INDEX idx_workflows_workspace ON workflows(workspace_id);
CREATE INDEX idx_workflows_type_status ON workflows(workflow_type, status);

CREATE TABLE workflow_steps (
    id BLOB PRIMARY KEY NOT NULL,
    workflow_id BLOB NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    step_order INTEGER NOT NULL DEFAULT 0,
    step_type INTEGER NOT NULL DEFAULT 0, -- 0: TASK, 1: CONDITION, 2: LOOP
    participant_id BLOB NOT NULL,         -- references existing participant (agent/user/contact/system)
    participant_config TEXT CHECK (participant_config IS NULL OR json_valid(participant_config)), -- JSON specific to the participant's execution context
    input_schema TEXT CHECK (input_schema IS NULL OR json_valid(input_schema)),
    output_schema TEXT CHECK (output_schema IS NULL OR json_valid(output_schema)),
    retry_policy TEXT CHECK (retry_policy IS NULL OR json_valid(retry_policy)),
    timeout_seconds INTEGER DEFAULT 3600,
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workflow_id) REFERENCES workflows(id) ON DELETE CASCADE,
    FOREIGN KEY (participant_id) REFERENCES participants(id) ON DELETE SET NULL
);

CREATE INDEX idx_workflow_steps_workflow ON workflow_steps(workflow_id);
CREATE INDEX idx_workflow_steps_participant ON workflow_steps(participant_id);

CREATE TABLE workflow_executions (
    id BLOB PRIMARY KEY NOT NULL,
    workflow_id BLOB NOT NULL,
    initiated_by_participant_id BLOB, -- existing participant model (user/agent/system)
    status INTEGER NOT NULL DEFAULT 0, -- 0: PENDING, 1: RUNNING, 2: COMPLETED, 3: FAILED
    input_data TEXT CHECK (input_data IS NULL OR json_valid(input_data)),
    output_data TEXT CHECK (output_data IS NULL OR json_valid(output_data)),
    started_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP,
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workflow_id) REFERENCES workflows(id) ON DELETE CASCADE,
    FOREIGN KEY (initiated_by_participant_id) REFERENCES participants(id) ON DELETE SET NULL
);

CREATE INDEX idx_workflow_executions_workflow ON workflow_executions(workflow_id);
CREATE INDEX idx_workflow_executions_status ON workflow_executions(status);

CREATE TABLE workflow_step_executions (
    id BLOB PRIMARY KEY NOT NULL,
    workflow_execution_id BLOB NOT NULL,
    workflow_step_id BLOB NOT NULL,
    status INTEGER NOT NULL DEFAULT 0, -- PENDING, RUNNING, COMPLETED, FAILED
    attempt_number INTEGER DEFAULT 1,
    input_data TEXT CHECK (input_data IS NULL OR json_valid(input_data)),
    output_data TEXT CHECK (output_data IS NULL OR json_valid(output_data)),
    error_details TEXT,
    started_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP,
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workflow_execution_id) REFERENCES workflow_executions(id) ON DELETE CASCADE,
    FOREIGN KEY (workflow_step_id) REFERENCES workflow_steps(id) ON DELETE CASCADE
);

CREATE INDEX idx_workflow_step_executions_workflow_execution_id ON workflow_step_executions(workflow_execution_id);
CREATE INDEX idx_workflow_step_executions_workflow_step_id ON workflow_step_executions(workflow_step_id);
CREATE INDEX idx_workflow_step_executions_status ON workflow_step_executions(status);

CREATE TABLE workspaces (
    id BLOB PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    workspace_type INTEGER NOT NULL DEFAULT 0,  -- 'personal', 'group', 'organization', 'system'
    status INTEGER NOT NULL DEFAULT 0,  -- 'active', 'archived', 'deleted'
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),  -- JSON object with additional metadata
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_workspaces_type ON workspaces(workspace_type);
CREATE INDEX idx_workspaces_status ON workspaces(status);

CREATE TABLE workspace_members (
    workspace_id BLOB NOT NULL,
    participant_id BLOB NOT NULL,
    role INTEGER NOT NULL DEFAULT 0, -- 'MEMBER', 'ADMIN', 'OWNER'
    permissions TEXT CHECK (permissions IS NULL OR json_valid(permissions)),
    joined_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (workspace_id, participant_id),
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
    FOREIGN KEY (participant_id) REFERENCES participants(id) ON DELETE CASCADE
);

CREATE TABLE settings (
    id BLOB PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    value TEXT NOT NULL,
    settings_type INTEGER NOT NULL DEFAULT 0,  -- 'string', 'number', 'boolean', 'object'
    description TEXT,
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),  -- JSON object with additional metadata
    workspace_id BLOB,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL
);

CREATE INDEX idx_settings_name ON settings(name);

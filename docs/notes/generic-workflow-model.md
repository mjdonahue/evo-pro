Revised schema designed around a generic, durable actor-based workflow engine. This model unifies all your execution requirements into a single, extensible framework.

### Core Principles of this Actor-Based Workflow Schema

1.  **Definition vs. Execution:** The schema clearly separates the *template* of a process (`WorkflowDefinition`) from a *live instance* of that process (`WorkflowRun`).
2.  **Actors as the Unit of Work:** Every step in a workflow is executed by an "Actor." An actor is a self-contained unit with its own logic, state, and configuration. This could be an AI Agent, a data transformer, an API caller, or even a component that waits for human input.
3.  **Graph-Based Structure:** Workflows are defined as a Directed Acyclic Graph (DAG), stored in a flexible JSON format. This allows for simple sequences (agent chains), parallel execution (fan-out/fan-in), and conditional branching.
4.  **Durability:** The state of long-running actors and the overall workflow progress is persisted to the database. This allows the system to pause, resume, and recover from failures or restarts.
5.  **Extensibility:** Adding new capabilities (like a new type of agent or tool) doesn't require changing the schema. You simply define a new `ActorDefinition` and implement the corresponding code.

---

### The Generic Actor-Based Workflow Schema

This schema replaces `workflows`, `workflow_runs`, and `workflow_run_steps` from the previous suggestion with a more powerful and generic model.

#### 1. Actor & Workflow Definitions (The Blueprints)

These tables define *what* can be done.

```sql
-- The "class" definition for an actor. It describes the actor's type and the
-- configuration it requires. This is the bridge between your code and the database.
CREATE TABLE actor_definitions (
    id BLOB PRIMARY KEY NOT NULL,
    workspace_id BLOB NOT NULL,
    name TEXT NOT NULL,                     -- e.g., "GPT-4 Vision Agent", "Data Scraper", "Human Approval Task"
    description TEXT,
    -- The code identifier for the actor factory, e.g., "my_app.actors.GptAgent".
    -- The system uses this to know which actor class to instantiate.
    actor_type_identifier TEXT NOT NULL,
    -- A JSON Schema that defines the structure of the configuration this actor expects.
    -- This is great for validation and for auto-generating UIs.
    config_schema TEXT NOT NULL CHECK (json_valid(config_schema)),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
    UNIQUE(workspace_id, name)
);
CREATE INDEX idx_actor_definitions_workspace_id ON actor_definitions(workspace_id);

-- The definition of a workflow, composed as a graph of configured actor nodes.
-- This can represent an agent chain, an ETL pipeline, or a business process.
CREATE TABLE workflow_definitions (
    id BLOB PRIMARY KEY NOT NULL,
    workspace_id BLOB NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    -- The graph structure (nodes and edges) of the workflow.
    -- Nodes reference an `actor_definition_id` and provide the specific `config`.
    -- Edges define the flow of data and control.
    graph_definition TEXT NOT NULL CHECK (json_valid(graph_definition)),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
);
CREATE INDEX idx_workflow_definitions_workspace_id ON workflow_definitions(workspace_id);
```

**Example `graph_definition` JSON:**

```json
{
  "nodes": [
    {
      "id": "step_1_fetch_data",
      "actor_definition_id": "uuid-for-data-scraper-actor",
      "config": {
        "url": "https://example.com/data",
        "css_selector": ".item"
      }
    },
    {
      "id": "step_2_summarize",
      "actor_definition_id": "uuid-for-gpt4-agent-actor",
      "config": {
        "model": "gpt-4-turbo",
        "system_prompt": "You are an expert summarizer. Summarize the following content in three sentences.",
        "temperature": 0.5
      }
    }
  ],
  "edges": [
    {
      "from_node": "step_1_fetch_data",
      "to_node": "step_2_summarize",
      "from_output": "scraped_text",
      "to_input": "content_to_summarize"
    }
  ]
}
```

#### 2. Workflow Execution & State (The Live Instances)

These tables track the live, running, and completed processes.

```sql
-- A single, live instance of a workflow_definition. This is a "job" or "process".
CREATE TABLE workflow_runs (
    id BLOB PRIMARY KEY NOT NULL,
    definition_id BLOB NOT NULL,
    workspace_id BLOB NOT NULL,
    -- 0:PENDING, 1:RUNNING, 2:PAUSED, 3:COMPLETED, 4:FAILED, 5:CANCELLED
    status INTEGER NOT NULL DEFAULT 0,
    -- The initial data provided to the workflow.
    input_data TEXT CHECK (input_data IS NULL OR json_valid(input_data)),
    -- The final result of the entire workflow.
    output_data TEXT CHECK (output_data IS NULL OR json_valid(output_data)),
    error_details TEXT,
    triggered_by_participant_id BLOB,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (definition_id) REFERENCES workflow_definitions(id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
    FOREIGN KEY (triggered_by_participant_id) REFERENCES participants(id) ON DELETE SET NULL
);
CREATE INDEX idx_workflow_runs_status ON workflow_runs(status);

-- Tracks the execution of a single node within a specific workflow_run.
-- This is essential for observability, debugging, and retries.
CREATE TABLE node_runs (
    id BLOB PRIMARY KEY NOT NULL,
    workflow_run_id BLOB NOT NULL,
    -- The ID of the node from the workflow's graph_definition JSON (e.g., "step_1_fetch_data").
    node_id_in_graph TEXT NOT NULL,
    -- Status values are the same as workflow_runs.
    status INTEGER NOT NULL DEFAULT 0,
    input_data TEXT CHECK (input_data IS NULL OR json_valid(input_data)),
    output_data TEXT CHECK (output_data IS NULL OR json_valid(output_data)),
    error_details TEXT,
    retry_count INTEGER DEFAULT 0,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workflow_run_id) REFERENCES workflow_runs(id) ON DELETE CASCADE,
    UNIQUE(workflow_run_id, node_id_in_graph)
);
CREATE INDEX idx_node_runs_workflow_run_id ON node_runs(workflow_run_id);
CREATE INDEX idx_node_runs_status ON node_runs(status);

-- DURABILITY: Persists the state of a long-running or stateful Kameo actor.
-- The actor is responsible for serializing/deserializing its own state to/from JSON.
CREATE TABLE actor_states (
    -- The primary key IS the node_run_id. An actor's state is uniquely tied to its execution step.
    node_run_id BLOB PRIMARY KEY NOT NULL,
    -- The JSON representation of the actor's internal state.
    -- For a data pipeline, this might be a running counter. For an agent, it could be its memory buffer.
    state_data TEXT NOT NULL CHECK (json_valid(state_data)),
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (node_run_id) REFERENCES node_runs(id) ON DELETE CASCADE
);
```

### How This Model Solves Your Requirements:

1.  **Agent Chaining:** A `WorkflowDefinition` with a simple linear graph (`A -> B -> C`) where each node is an AI agent `ActorDefinition`.
2.  **Long-Running Processes:** `WorkflowRuns` can have a `PAUSED` or `RUNNING` status for days. If the system restarts, it can query for `RUNNING` workflows, inspect their `NodeRuns` and `ActorStates`, and re-hydrate the Kameo actors to continue exactly where they left off.
3.  **Data Ingestion Pipelines:** A `WorkflowDefinition` can be designed with nodes for "Extract" (e.g., `HttpActor`), "Transform" (e.g., `PandasTransformActor`), and "Load" (e.g., `DatabaseLoadActor`).
4.  **Traditional Workflow:** You can create a `HumanTaskActorDefinition`. When this actor runs, it creates an entry in a `human_tasks` table, sends a notification, and then enters a waiting state. Its state in the `actor_states` table might be `{"status": "waiting", "task_id": "..."}`. It will only complete when an external event (e.g., a user clicking "Approve" in a UI) sends a message back to the Kameo actor, which then updates its state and allows the workflow to proceed.
5.  **Kameo Actors and Configs:**
    *   `actor_definitions` maps directly to your Kameo actor classes.
    *   The `config` object within the `graph_definition` JSON provides the specific initialization parameters for each actor instance in the workflow.
    *   `actor_states` provides the persistence mechanism for making your Kameo actors durable and resilient, a crucial feature for any serious workflow engine.

    ---

    ## According to Claude

    I'll design a more generic, actor-based schema that can handle all these requirements while supporting Kameo actors. This approach treats everything as actors in a distributed system.

## 1. Core Actor System

```sql
-- Core actor registry - everything is an actor
CREATE TABLE actors (
    id BLOB PRIMARY KEY NOT NULL,
    actor_type INTEGER NOT NULL DEFAULT 0, -- 'USER', 'AGENT', 'PIPELINE', 'WORKFLOW', 'SERVICE', 'SYSTEM'
    name TEXT NOT NULL,
    description TEXT,
    
    -- Actor runtime configuration
    runtime_config TEXT CHECK (runtime_config IS NULL OR json_valid(runtime_config)), -- Kameo actor config
    actor_class TEXT NOT NULL, -- Rust struct name for Kameo spawning
    supervisor_id BLOB, -- Parent supervisor actor
    
    -- State management
    state_type INTEGER NOT NULL DEFAULT 0, -- 'STATELESS', 'STATEFUL', 'PERSISTENT'
    state_data TEXT CHECK (state_data IS NULL OR json_valid(state_data)),
    checkpoint_interval INTEGER, -- ms between state checkpoints
    
    -- Lifecycle
    status INTEGER NOT NULL DEFAULT 0, -- 'INITIALIZING', 'RUNNING', 'PAUSED', 'STOPPED', 'FAILED'
    restart_policy INTEGER NOT NULL DEFAULT 0, -- 'NEVER', 'ON_FAILURE', 'ALWAYS'
    max_restarts INTEGER DEFAULT 3,
    restart_count INTEGER DEFAULT 0,
    
    -- Resource limits
    max_memory_mb INTEGER,
    max_cpu_cores REAL,
    timeout_ms INTEGER,
    
    -- Networking (for distributed actors)
    node_id TEXT, -- Which node this actor is running on
    port INTEGER, -- If actor exposes network interface
    
    -- Metadata
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),
    
    -- Timestamps
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP,
    last_heartbeat TIMESTAMP,
    
    -- Relationships
    workspace_id BLOB,
    created_by_id BLOB,
    
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL,
    FOREIGN KEY (created_by_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (supervisor_id) REFERENCES actors(id) ON DELETE SET NULL
);

CREATE INDEX idx_actors_type_status ON actors(actor_type, status);
CREATE INDEX idx_actors_supervisor ON actors(supervisor_id);
CREATE INDEX idx_actors_workspace ON actors(workspace_id);
CREATE INDEX idx_actors_node ON actors(node_id);
```

## 2. Generic Process Definition System

```sql
-- Process definitions - templates for workflows, pipelines, chains
CREATE TABLE process_definitions (
    id BLOB PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    version TEXT NOT NULL DEFAULT '1.0.0',
    process_type INTEGER NOT NULL DEFAULT 0, -- 'WORKFLOW', 'PIPELINE', 'CHAIN', 'BATCH_JOB'
    
    -- Process specification (generic DAG definition)
    spec TEXT NOT NULL CHECK (json_valid(spec)), -- JSON DAG specification
    
    -- Runtime requirements
    required_capabilities TEXT CHECK (required_capabilities IS NULL OR json_valid(required_capabilities)),
    resource_requirements TEXT CHECK (resource_requirements IS NULL OR json_valid(resource_requirements)),
    
    -- Execution configuration
    execution_mode INTEGER NOT NULL DEFAULT 0, -- 'SEQUENTIAL', 'PARALLEL', 'HYBRID'
    retry_policy TEXT CHECK (retry_policy IS NULL OR json_valid(retry_policy)),
    timeout_config TEXT CHECK (timeout_config IS NULL OR json_valid(timeout_config)),
    
    -- Triggers and scheduling
    triggers TEXT CHECK (triggers IS NULL OR json_valid(triggers)), -- Event triggers, cron, etc.
    schedule_config TEXT CHECK (schedule_config IS NULL OR json_valid(schedule_config)),
    
    -- Status and versioning
    status INTEGER NOT NULL DEFAULT 0, -- 'DRAFT', 'ACTIVE', 'DEPRECATED', 'ARCHIVED'
    is_template BOOLEAN NOT NULL DEFAULT 0,
    
    -- Metadata
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),
    
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Relationships
    workspace_id BLOB,
    created_by_id BLOB,
    parent_definition_id BLOB, -- For versioning/inheritance
    
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL,
    FOREIGN KEY (created_by_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (parent_definition_id) REFERENCES process_definitions(id) ON DELETE SET NULL,
    
    UNIQUE(name, version, workspace_id)
);

CREATE INDEX idx_process_definitions_type ON process_definitions(process_type);
CREATE INDEX idx_process_definitions_workspace ON process_definitions(workspace_id);
CREATE INDEX idx_process_definitions_status ON process_definitions(status);
```

## 3. Process Execution Engine

```sql
-- Process instances - running processes
CREATE TABLE process_instances (
    id BLOB PRIMARY KEY NOT NULL,
    definition_id BLOB NOT NULL,
    
    -- Execution context
    execution_id TEXT NOT NULL, -- Unique execution identifier
    parent_instance_id BLOB, -- For sub-processes
    root_instance_id BLOB, -- Top-level process
    
    -- Runtime state
    status INTEGER NOT NULL DEFAULT 0, -- 'PENDING', 'RUNNING', 'PAUSED', 'COMPLETED', 'FAILED', 'CANCELLED'
    current_step TEXT, -- Current step ID in the process
    progress REAL DEFAULT 0.0, -- 0.0 to 1.0
    
    -- Data flow
    input_data TEXT CHECK (input_data IS NULL OR json_valid(input_data)),
    output_data TEXT CHECK (output_data IS NULL OR json_valid(output_data)),
    context_data TEXT CHECK (context_data IS NULL OR json_valid(context_data)), -- Shared context
    
    -- Error handling
    error_details TEXT,
    retry_count INTEGER DEFAULT 0,
    
    -- Timing
    started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP,
    paused_at TIMESTAMP,
    resumed_at TIMESTAMP,
    
    -- Resource tracking
    allocated_resources TEXT CHECK (allocated_resources IS NULL OR json_valid(allocated_resources)),
    resource_usage TEXT CHECK (resource_usage IS NULL OR json_valid(resource_usage)),
    
    -- Actor association
    coordinator_actor_id BLOB, -- Main coordinator actor
    
    -- Metadata
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),
    
    -- Relationships
    workspace_id BLOB,
    triggered_by_id BLOB,
    
    FOREIGN KEY (definition_id) REFERENCES process_definitions(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_instance_id) REFERENCES process_instances(id) ON DELETE SET NULL,
    FOREIGN KEY (root_instance_id) REFERENCES process_instances(id) ON DELETE SET NULL,
    FOREIGN KEY (coordinator_actor_id) REFERENCES actors(id) ON DELETE SET NULL,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL,
    FOREIGN KEY (triggered_by_id) REFERENCES actors(id) ON DELETE SET NULL
);

CREATE INDEX idx_process_instances_definition ON process_instances(definition_id);
CREATE INDEX idx_process_instances_status ON process_instances(status);
CREATE INDEX idx_process_instances_coordinator ON process_instances(coordinator_actor_id);
CREATE INDEX idx_process_instances_parent ON process_instances(parent_instance_id);
```

## 4. Step Execution Tracking

```sql
-- Individual step executions within processes
CREATE TABLE step_executions (
    id BLOB PRIMARY KEY NOT NULL,
    instance_id BLOB NOT NULL,
    
    -- Step identification
    step_id TEXT NOT NULL, -- Step ID from process definition
    step_name TEXT NOT NULL,
    step_type INTEGER NOT NULL DEFAULT 0, -- 'TASK', 'DECISION', 'PARALLEL', 'LOOP', 'WAIT'
    
    -- Execution details
    status INTEGER NOT NULL DEFAULT 0, -- 'PENDING', 'RUNNING', 'COMPLETED', 'FAILED', 'SKIPPED'
    attempt_number INTEGER DEFAULT 1,
    
    -- Data
    input_data TEXT CHECK (input_data IS NULL OR json_valid(input_data)),
    output_data TEXT CHECK (output_data IS NULL OR json_valid(output_data)),
    
    -- Actor execution
    executor_actor_id BLOB, -- Actor that executed this step
    
    -- Timing
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    duration_ms INTEGER,
    
    -- Error handling
    error_details TEXT,
    
    -- Dependencies
    depends_on TEXT CHECK (depends_on IS NULL OR json_valid(depends_on)), -- Array of step IDs
    
    -- Metadata
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),
    
    FOREIGN KEY (instance_id) REFERENCES process_instances(id) ON DELETE CASCADE,
    FOREIGN KEY (executor_actor_id) REFERENCES actors(id) ON DELETE SET NULL,
    
    UNIQUE(instance_id, step_id, attempt_number)
);

CREATE INDEX idx_step_executions_instance ON step_executions(instance_id);
CREATE INDEX idx_step_executions_status ON step_executions(status);
CREATE INDEX idx_step_executions_executor ON step_executions(executor_actor_id);
```

## 5. Actor Communication System

```sql
-- Actor message passing
CREATE TABLE actor_messages (
    id BLOB PRIMARY KEY NOT NULL,
    
    -- Message routing
    from_actor_id BLOB NOT NULL,
    to_actor_id BLOB NOT NULL,
    correlation_id TEXT, -- For request/response correlation
    
    -- Message details
    message_type INTEGER NOT NULL DEFAULT 0, -- 'COMMAND', 'QUERY', 'EVENT', 'RESPONSE'
    message_name TEXT NOT NULL, -- Message type name
    
    -- Payload
    payload TEXT NOT NULL CHECK (json_valid(payload)),
    
    -- Delivery
    status INTEGER NOT NULL DEFAULT 0, -- 'PENDING', 'SENT', 'DELIVERED', 'FAILED', 'EXPIRED'
    priority INTEGER NOT NULL DEFAULT 0, -- 0-9, higher = more priority
    
    -- Timing
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    sent_at TIMESTAMP,
    delivered_at TIMESTAMP,
    expires_at TIMESTAMP,
    
    -- Retry logic
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    
    -- Error handling
    error_details TEXT,
    
    -- Metadata
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),
    
    FOREIGN KEY (from_actor_id) REFERENCES actors(id) ON DELETE CASCADE,
    FOREIGN KEY (to_actor_id) REFERENCES actors(id) ON DELETE CASCADE
);

CREATE INDEX idx_actor_messages_to_status ON actor_messages(to_actor_id, status);
CREATE INDEX idx_actor_messages_correlation ON actor_messages(correlation_id);
CREATE INDEX idx_actor_messages_priority ON actor_messages(priority DESC, created_at);
```

## 6. Data Pipeline Support

```sql
-- Data sources and sinks
CREATE TABLE data_connectors (
    id BLOB PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    connector_type INTEGER NOT NULL DEFAULT 0, -- 'FILE', 'DATABASE', 'API', 'STREAM', 'QUEUE'
    
    -- Connection configuration
    connection_config TEXT NOT NULL CHECK (json_valid(connection_config)),
    credentials_id BLOB, -- Reference to encrypted credentials
    
    -- Schema and format
    schema_config TEXT CHECK (schema_config IS NULL OR json_valid(schema_config)),
    format_config TEXT CHECK (format_config IS NULL OR json_valid(format_config)),
    
    -- Capabilities
    supports_read BOOLEAN NOT NULL DEFAULT 1,
    supports_write BOOLEAN NOT NULL DEFAULT 1,
    supports_streaming BOOLEAN NOT NULL DEFAULT 0,
    
    -- Health monitoring
    status INTEGER NOT NULL DEFAULT 0, -- 'ACTIVE', 'INACTIVE', 'ERROR'
    last_health_check TIMESTAMP,
    
    -- Metadata
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),
    
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Relationships
    workspace_id BLOB,
    
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL,
    FOREIGN KEY (credentials_id) REFERENCES credentials(id) ON DELETE SET NULL
);

-- Data processing jobs
CREATE TABLE data_jobs (
    id BLOB PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    job_type INTEGER NOT NULL DEFAULT 0, -- 'INGESTION', 'TRANSFORMATION', 'SYNC', 'CLEANUP'
    
    -- Job configuration
    source_connector_id BLOB,
    target_connector_id BLOB,
    transformation_config TEXT CHECK (transformation_config IS NULL OR json_valid(transformation_config)),
    
    -- Processing settings
    batch_size INTEGER DEFAULT 1000,
    parallelism INTEGER DEFAULT 1,
    checkpoint_interval INTEGER DEFAULT 1000,
    
    -- Scheduling
    schedule_config TEXT CHECK (schedule_config IS NULL OR json_valid(schedule_config)),
    
    -- Associated process
    process_definition_id BLOB,
    
    -- Status
    status INTEGER NOT NULL DEFAULT 0, -- 'ACTIVE', 'PAUSED', 'DISABLED'
    
    -- Metadata
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),
    
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Relationships
    workspace_id BLOB,
    
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL,
    FOREIGN KEY (source_connector_id) REFERENCES data_connectors(id) ON DELETE SET NULL,
    FOREIGN KEY (target_connector_id) REFERENCES data_connectors(id) ON DELETE SET NULL,
    FOREIGN KEY (process_definition_id) REFERENCES process_definitions(id) ON DELETE SET NULL
);
```

## 7. Actor Configuration and Capabilities

```sql
-- Actor capabilities and configurations
CREATE TABLE actor_configs (
    id BLOB PRIMARY KEY NOT NULL,
    actor_id BLOB NOT NULL,
    
    -- Configuration type
    config_type INTEGER NOT NULL DEFAULT 0, -- 'RUNTIME', 'CAPABILITY', 'RESOURCE', 'NETWORK'
    config_name TEXT NOT NULL,
    
    -- Configuration data
    config_data TEXT NOT NULL CHECK (json_valid(config_data)),
    
    -- Versioning
    version TEXT NOT NULL DEFAULT '1.0.0',
    is_active BOOLEAN NOT NULL DEFAULT 1,
    
    -- Validation
    schema_version TEXT,
    validated_at TIMESTAMP,
    
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (actor_id) REFERENCES actors(id) ON DELETE CASCADE,
    UNIQUE(actor_id, config_type, config_name)
);

-- Actor relationships and dependencies
CREATE TABLE actor_relationships (
    id BLOB PRIMARY KEY NOT NULL,
    from_actor_id BLOB NOT NULL,
    to_actor_id BLOB NOT NULL,
    relationship_type INTEGER NOT NULL DEFAULT 0, -- 'SUPERVISES', 'DEPENDS_ON', 'COMMUNICATES_WITH', 'DELEGATES_TO'
    
    -- Relationship configuration
    config TEXT CHECK (config IS NULL OR json_valid(config)),
    
    -- Status
    is_active BOOLEAN NOT NULL DEFAULT 1,
    
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (from_actor_id) REFERENCES actors(id) ON DELETE CASCADE,
    FOREIGN KEY (to_actor_id) REFERENCES actors(id) ON DELETE CASCADE,
    UNIQUE(from_actor_id, to_actor_id, relationship_type)
);
```

## 8. Enhanced Monitoring and Metrics

```sql
-- Comprehensive metrics for all actors and processes
CREATE TABLE metrics (
    id BLOB PRIMARY KEY NOT NULL,
    
    -- Metric source
    actor_id BLOB,
    process_instance_id BLOB,
    step_execution_id BLOB,
    
    -- Metric details
    metric_type INTEGER NOT NULL DEFAULT 0, -- 'COUNTER', 'GAUGE', 'HISTOGRAM', 'TIMER'
    metric_name TEXT NOT NULL,
    metric_value REAL NOT NULL,
    
    -- Dimensions
    tags TEXT CHECK (tags IS NULL OR json_valid(tags)), -- JSON object with tags
    
    -- Timing
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Metadata
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),
    
    FOREIGN KEY (actor_id) REFERENCES actors(id) ON DELETE SET NULL,
    FOREIGN KEY (process_instance_id) REFERENCES process_instances(id) ON DELETE SET NULL,
    FOREIGN KEY (step_execution_id) REFERENCES step_executions(id) ON DELETE SET NULL
);

CREATE INDEX idx_metrics_actor_name_time ON metrics(actor_id, metric_name, timestamp);
CREATE INDEX idx_metrics_process_name_time ON metrics(process_instance_id, metric_name, timestamp);
```

## 9. Kameo Actor Integration Support

```sql
-- Kameo-specific actor state management
CREATE TABLE actor_state_snapshots (
    id BLOB PRIMARY KEY NOT NULL,
    actor_id BLOB NOT NULL,
    
    -- State snapshot
    state_data TEXT NOT NULL CHECK (json_valid(state_data)),
    sequence_number INTEGER NOT NULL,
    
    -- Snapshot metadata
    snapshot_type INTEGER NOT NULL DEFAULT 0, -- 'CHECKPOINT', 'MIGRATION', 'BACKUP'
    compression TEXT, -- 'NONE', 'GZIP', 'LZ4'
    
    -- Timing
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Metadata
    metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata)),
    
    FOREIGN KEY (actor_id) REFERENCES actors(id) ON DELETE CASCADE,
    UNIQUE(actor_id, sequence_number)
);

-- Actor mailbox persistence (for message recovery)
CREATE TABLE actor_mailboxes (
    id BLOB PRIMARY KEY NOT NULL,
    actor_id BLOB NOT NULL,
    
    -- Message queue state
    pending_messages TEXT NOT NULL CHECK (json_valid(pending_messages)), -- Array of message IDs
    processing_message_id BLOB,
    
    -- Mailbox configuration
    max_capacity INTEGER DEFAULT 1000,
    message_ordering INTEGER DEFAULT 0, -- 'FIFO', 'LIFO', 'PRIORITY'
    
    -- Status
    is_paused BOOLEAN NOT NULL DEFAULT 0,
    
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (actor_id) REFERENCES actors(id) ON DELETE CASCADE,
    FOREIGN KEY (processing_message_id) REFERENCES actor_messages(id) ON DELETE SET NULL,
    UNIQUE(actor_id)
);
```

## 10. Usage Examples

### Agent Chain Configuration
```json
{
  "process_type": "CHAIN",
  "spec": {
    "steps": [
      {
        "id": "analyze",
        "type": "TASK",
        "actor_class": "AnalyzerAgent",
        "config": {"model": "gpt-4"}
      },
      {
        "id": "transform",
        "type": "TASK", 
        "actor_class": "TransformAgent",
        "depends_on": ["analyze"]
      },
      {
        "id": "respond",
        "type": "TASK",
        "actor_class": "ResponseAgent", 
        "depends_on": ["transform"]
      }
    ]
  }
}
```

### Data Pipeline Configuration
```json
{
  "process_type": "PIPELINE",
  "spec": {
    "source": {
      "connector_id": "api-source",
      "batch_size": 1000
    },
    "transformations": [
      {
        "id": "clean",
        "actor_class": "DataCleanerActor"
      },
      {
        "id": "enrich", 
        "actor_class": "DataEnricherActor"
      }
    ],
    "sink": {
      "connector_id": "database-sink"
    }
  }
}
```

This schema provides:
- **Unified actor model** for all components
- **Generic process engine** for workflows, chains, and pipelines
- **Kameo integration** through actor configs and state management
- **Comprehensive monitoring** and metrics
- **Flexible data pipeline** support
- **Long-running process** management
- **Distributed execution** support

The design is extensible while maintaining simplicity for initial development phases.
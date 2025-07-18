import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type {
  ApiResponse,
  ListResponse,
  ApiError,
  Conversation,
  ConversationFilter,
  CreateConversationInput,
  Message,
  MessageFilter, 
  CreateMessageInput,
  Task,
  TaskFilter,
  CreateTaskInput,
  UpdateTaskStatusInput,
  TaskStats,
  Plan,
  PlanFilter,
  CreatePlanInput,
  UpdatePlanStatusInput,
  PlanStats,
  TaskAssignee,
  Agent,
  User,
  Uuid,
} from './types';
import { validateData } from './validation';

// Error handling
export class ApiClientError extends Error {
  constructor(
    public code: string,
    message: string,
    public details?: Record<string, any>
  ) {
    super(message);
    this.name = 'ApiClientError';
  }
}

// Event types for real-time updates
export interface ApiEvent<T = any> {
  event: string;
  payload: T;
}

export interface ConversationEvent {
  type: 'conversation_updated' | 'conversation_created' | 'conversation_deleted';
  conversation_id: Uuid;
  data?: Conversation;
}

export interface MessageEvent {
  type: 'message_created' | 'message_updated' | 'message_deleted';
  conversation_id: Uuid;
  message_id: Uuid;
  data?: Message;
}

export interface TaskEvent {
  type: 'task_created' | 'task_updated' | 'task_completed' | 'task_assigned';
  task_id: Uuid;
  data?: Task;
}

// Base API client with error handling and type safety
class BaseApiClient {
  private async invokeCommand<T>(
    command: string, 
    args?: Record<string, any>,
    schema?: any
  ): Promise<T> {
    try {
      const response = await invoke<ApiResponse<T>>(command, args);

      if (!response.success) {
        throw new ApiClientError(
          'api_error',
          response.error || 'Unknown API error',
          { command, args }
        );
      }

      // Validate response data against schema if provided
      if (schema) {
        return validateData<T>(response.data, schema);
      }

      return response.data;
    } catch (error) {
      if (error instanceof ApiClientError) {
        throw error;
      }

      // Handle Tauri/network errors
      throw new ApiClientError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown invoke error',
        { command, args, originalError: error }
      );
    }
  }

  protected async invoke<T = any>(command: string, args?: Record<string, any>, schema?: any): Promise<T> {
    return this.invokeCommand<T>(command, args, schema);
  }

  protected async invokeList<T>(command: string, args?: Record<string, any>, schema?: any): Promise<ListResponse<T>> {
    return this.invokeCommand<ListResponse<T>>(command, args, schema ? { items: [schema], total: Number } : undefined);
  }
}

// Conversation API
export class ConversationApi extends BaseApiClient {
  async list(filter?: ConversationFilter): Promise<ListResponse<Conversation>> {
    return this.invokeList<Conversation>('list_conversations', { filter });
  }

  async get(id: Uuid): Promise<Conversation | null> {
    return this.invoke<Conversation | null>('get_conversation', { id });
  }

  async create(input: CreateConversationInput): Promise<Conversation> {
    return this.invoke<Conversation>('create_conversation', { input });
  }

  async update(conversation: Conversation): Promise<Conversation> {
    return this.invoke<Conversation>('update_conversation', { conversation });
  }

  async delete(id: Uuid): Promise<void> {
    return this.invoke<void>('delete_conversation', { id });
  }

  async getParticipants(conversationId: Uuid): Promise<User[]> {
    return this.invoke<User[]>('get_conversation_participants', { conversation_id: conversationId });
  }

  async addParticipant(conversationId: Uuid, userId: Uuid): Promise<void> {
    return this.invoke<void>('add_conversation_participant', { 
      conversation_id: conversationId, 
      user_id: userId 
    });
  }
}

// Message API
export class MessageApi extends BaseApiClient {
  async list(filter?: MessageFilter): Promise<ListResponse<Message>> {
    return this.invokeList<Message>('list_messages', { filter });
  }

  async get(id: Uuid): Promise<Message | null> {
    return this.invoke<Message | null>('get_message', { id });
  }

  async create(input: CreateMessageInput): Promise<Message> {
    return this.invoke<Message>('create_message', { input });
  }

  async update(message: Message): Promise<Message> {
    return this.invoke<Message>('update_message', { message });
  }

  async delete(id: Uuid): Promise<void> {
    return this.invoke<void>('delete_message', { id });
  }

  async markAsRead(id: Uuid): Promise<void> {
    return this.invoke<void>('mark_message_read', { id });
  }
}

// Task API
export class TaskApi extends BaseApiClient {
  async list(filter?: TaskFilter): Promise<Task[]> {
    // Backend expects filter as direct parameter, not wrapped in object
    return this.invoke<Task[]>('list_tasks', filter ? { filter } : {});
  }

  async get(id: Uuid): Promise<Task | null> {
    return this.invoke<Task | null>('get_task', { id });
  }

  async create(input: CreateTaskInput): Promise<Task> {
    // Backend expects CreateTask, not wrapped in input object
    return this.invoke<Task>('create_task', { task: input });
  }

  async update(task: Task): Promise<Task> {
    return this.invoke<Task>('update_task', { task });
  }

  async delete(id: Uuid): Promise<void> {
    return this.invoke<void>('delete_tasks', { id });
  }

  async updateStatus(input: UpdateTaskStatusInput): Promise<void> {
    // Backend expects id and status as separate parameters
    return this.invoke<void>('update_task_status', { 
      id: input.id, 
      status: input.status 
    });
  }

  async start(id: Uuid): Promise<void> {
    return this.invoke<void>('start_task', { id });
  }

  async complete(id: Uuid): Promise<void> {
    return this.invoke<void>('complete_task', { id });
  }

  async fail(id: Uuid): Promise<void> {
    return this.invoke<void>('fail_task', { id });
  }

  async getStats(workspaceId?: Uuid): Promise<TaskStats> {
    return this.invoke<TaskStats>('get_task_stats', workspaceId ? { workspace_id: workspaceId } : {});
  }

  async getOverdue(): Promise<Task[]> {
    return this.invoke<Task[]>('get_overdue_tasks');
  }

  async getHighPriority(): Promise<Task[]> {
    return this.invoke<Task[]>('get_high_priority_tasks');
  }

  async getByPlan(planId: Uuid): Promise<Task[]> {
    return this.invoke<Task[]>('get_tasks_by_plan', { plan_id: planId });
  }
}

// Plan API
export class PlanApi extends BaseApiClient {
  async list(filter?: PlanFilter): Promise<ListResponse<Plan>> {
    return this.invokeList<Plan>('list_plans', { filter });
  }

  async get(id: Uuid): Promise<Plan | null> {
    return this.invoke<Plan | null>('get_plan', { id });
  }

  async create(input: CreatePlanInput): Promise<Plan> {
    return this.invoke<Plan>('create_plan', { input });
  }

  async update(plan: Plan): Promise<Plan> {
    return this.invoke<Plan>('update_plan', { plan });
  }

  async delete(id: Uuid): Promise<void> {
    return this.invoke<void>('delete_plan', { id });
  }

  async updateStatus(input: UpdatePlanStatusInput): Promise<void> {
    return this.invoke<void>('update_plan_status', { input });
  }

  async start(id: Uuid): Promise<void> {
    return this.invoke<void>('start_plan', { id });
  }

  async complete(id: Uuid): Promise<void> {
    return this.invoke<void>('complete_plan', { id });
  }

  async fail(id: Uuid): Promise<void> {
    return this.invoke<void>('fail_plan', { id });
  }

  async getStats(participantId?: Uuid): Promise<PlanStats> {
    return this.invoke<PlanStats>('get_plan_stats', { participant_id: participantId });
  }

  async getByParticipant(participantId: Uuid): Promise<Plan[]> {
    return this.invoke<Plan[]>('get_plans_by_participant', { participant_id: participantId });
  }

  async getActive(participantId: Uuid): Promise<Plan[]> {
    return this.invoke<Plan[]>('get_active_plans_by_participant', { participant_id: participantId });
  }
}

// Task Assignee API
export class TaskAssigneeApi extends BaseApiClient {
  async getByTask(taskId: Uuid): Promise<TaskAssignee[]> {
    return this.invoke<TaskAssignee[]>('get_task_assignees', { task_id: taskId });
  }

  async getByParticipant(participantId: Uuid): Promise<TaskAssignee[]> {
    return this.invoke<TaskAssignee[]>('get_assignee_tasks', { participant_id: participantId });
  }

  async addAssignee(taskId: Uuid, participantId: Uuid, role: string): Promise<TaskAssignee> {
    return this.invoke<TaskAssignee>('add_task_assignee', { 
      task_id: taskId, 
      participant_id: participantId, 
      role 
    });
  }

  async removeAssignee(taskId: Uuid, participantId: Uuid): Promise<void> {
    return this.invoke<void>('remove_task_assignee', { 
      task_id: taskId, 
      participant_id: participantId 
    });
  }

  async updateStatus(assigneeId: Uuid, status: string): Promise<void> {
    return this.invoke<void>('update_assignee_status', { 
      assignee_id: assigneeId, 
      status 
    });
  }

  async transferPrimary(taskId: Uuid, newPrimaryParticipantId: Uuid): Promise<void> {
    return this.invoke<void>('transfer_primary_assignee', { 
      task_id: taskId, 
      new_primary_participant_id: newPrimaryParticipantId 
    });
  }
}

// Agent API
export class AgentApi extends BaseApiClient {
  async list(): Promise<Agent[]> {
    return this.invoke<Agent[]>('list_agents');
  }

  async get(id: Uuid): Promise<Agent | null> {
    return this.invoke<Agent | null>('get_agent', { id });
  }

  async create(agent: Omit<Agent, 'id' | 'created_at' | 'updated_at'>): Promise<Agent> {
    return this.invoke<Agent>('create_agent', { agent });
  }

  async update(agent: Agent): Promise<Agent> {
    return this.invoke<Agent>('update_agent', { agent });
  }

  async delete(id: Uuid): Promise<void> {
    return this.invoke<void>('delete_agent', { id });
  }

  async invokeAgent(agentId: Uuid, input: string): Promise<string> {
    return this.invoke<string>('invoke_agent', { agent_id: agentId, input });
  }
}

// Event listener management
export class EventApi {
  private listeners: Map<string, UnlistenFn> = new Map();

  async onConversationEvents(callback: (event: ConversationEvent) => void): Promise<UnlistenFn> {
    const unlisten = await listen<ConversationEvent>('conversation-event', (event) => {
      callback(event.payload);
    });

    this.listeners.set('conversation-event', unlisten);
    return unlisten;
  }

  async onMessageEvents(callback: (event: MessageEvent) => void): Promise<UnlistenFn> {
    const unlisten = await listen<MessageEvent>('message-event', (event) => {
      callback(event.payload);
    });

    this.listeners.set('message-event', unlisten);
    return unlisten;
  }

  async onTaskEvents(callback: (event: TaskEvent) => void): Promise<UnlistenFn> {
    const unlisten = await listen<TaskEvent>('task-event', (event) => {
      callback(event.payload);
    });

    this.listeners.set('task-event', unlisten);
    return unlisten;
  }

  async removeAllListeners(): Promise<void> {
    for (const [_, unlisten] of this.listeners) {
      unlisten();
    }
    this.listeners.clear();
  }
}

// Main API client
export class ApiClient {
  public readonly conversations = new ConversationApi();
  public readonly messages = new MessageApi();
  public readonly tasks = new TaskApi();
  public readonly plans = new PlanApi();
  public readonly taskAssignees = new TaskAssigneeApi();
  public readonly agents = new AgentApi();
  public readonly events = new EventApi();

  // Utility methods
  async healthCheck(): Promise<boolean> {
    try {
      await invoke('health_check');
      return true;
    } catch {
      return false;
    }
  }

  async getVersion(): Promise<string> {
    return invoke<string>('get_version');
  }
}

// Singleton instance
export const apiClient = new ApiClient();

import { apiClient } from './client';
import {
  useBaseQuery,
  useBaseListQuery,
  useBaseMutation,
  UseBaseQueryOptions,
  UseBaseListQueryOptions,
  UseBaseMutationOptions,
  UseBaseQueryResult,
  UseBaseListQueryResult,
  UseBaseMutationResult
} from './baseHooks';
import type {
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
  Uuid,
} from './types';

// Enhanced Conversation Hooks

/**
 * Hook for fetching a paginated list of conversations
 * 
 * @param filter - Filter criteria for conversations
 * @param options - Query options
 * @returns Enhanced query result with pagination support
 */
export function useConversations(
  filter?: ConversationFilter, 
  options?: UseBaseListQueryOptions<Conversation>
): UseBaseListQueryResult<Conversation> {
  return useBaseListQuery(
    (offset, limit) => apiClient.conversations.list({ ...filter, offset, limit }),
    [filter],
    options
  );
}

/**
 * Hook for fetching a single conversation by ID
 * 
 * @param id - Conversation ID
 * @param options - Query options
 * @returns Enhanced query result
 */
export function useConversation(
  id: Uuid | null, 
  options?: UseBaseQueryOptions<Conversation | null>
): UseBaseQueryResult<Conversation | null> {
  return useBaseQuery(
    () => id ? apiClient.conversations.get(id) : Promise.resolve(null),
    [id],
    { 
      enabled: Boolean(id) && options?.enabled !== false,
      ...options
    }
  );
}

/**
 * Hook for creating a conversation
 * 
 * @param options - Mutation options
 * @returns Enhanced mutation result
 */
export function useCreateConversation(
  options?: UseBaseMutationOptions<Conversation, CreateConversationInput>
): UseBaseMutationResult<Conversation, CreateConversationInput> {
  return useBaseMutation(
    (input: CreateConversationInput) => apiClient.conversations.create(input),
    options
  );
}

/**
 * Hook for updating a conversation
 * 
 * @param options - Mutation options
 * @returns Enhanced mutation result
 */
export function useUpdateConversation(
  options?: UseBaseMutationOptions<Conversation, Conversation>
): UseBaseMutationResult<Conversation, Conversation> {
  return useBaseMutation(
    (conversation: Conversation) => apiClient.conversations.update(conversation),
    options
  );
}

/**
 * Hook for deleting a conversation
 * 
 * @param options - Mutation options
 * @returns Enhanced mutation result
 */
export function useDeleteConversation(
  options?: UseBaseMutationOptions<void, Uuid>
): UseBaseMutationResult<void, Uuid> {
  return useBaseMutation(
    (id: Uuid) => apiClient.conversations.delete(id),
    options
  );
}

// Enhanced Message Hooks

/**
 * Hook for fetching a paginated list of messages
 * 
 * @param filter - Filter criteria for messages
 * @param options - Query options
 * @returns Enhanced query result with pagination support
 */
export function useMessages(
  filter?: MessageFilter, 
  options?: UseBaseListQueryOptions<Message>
): UseBaseListQueryResult<Message> {
  return useBaseListQuery(
    (offset, limit) => apiClient.messages.list({ ...filter, offset, limit }),
    [filter],
    options
  );
}

/**
 * Hook for fetching a single message by ID
 * 
 * @param id - Message ID
 * @param options - Query options
 * @returns Enhanced query result
 */
export function useMessage(
  id: Uuid | null, 
  options?: UseBaseQueryOptions<Message | null>
): UseBaseQueryResult<Message | null> {
  return useBaseQuery(
    () => id ? apiClient.messages.get(id) : Promise.resolve(null),
    [id],
    { 
      enabled: Boolean(id) && options?.enabled !== false,
      ...options
    }
  );
}

/**
 * Hook for creating a message
 * 
 * @param options - Mutation options
 * @returns Enhanced mutation result
 */
export function useCreateMessage(
  options?: UseBaseMutationOptions<Message, CreateMessageInput>
): UseBaseMutationResult<Message, CreateMessageInput> {
  return useBaseMutation(
    (input: CreateMessageInput) => apiClient.messages.create(input),
    options
  );
}

/**
 * Hook for marking a message as read
 * 
 * @param options - Mutation options
 * @returns Enhanced mutation result
 */
export function useMarkMessageRead(
  options?: UseBaseMutationOptions<void, Uuid>
): UseBaseMutationResult<void, Uuid> {
  return useBaseMutation(
    (id: Uuid) => apiClient.messages.markAsRead(id),
    options
  );
}

// Enhanced Task Hooks

/**
 * Hook for fetching a list of tasks
 * 
 * @param filter - Filter criteria for tasks
 * @param options - Query options
 * @returns Enhanced query result
 */
export function useTasks(
  filter?: TaskFilter, 
  options?: UseBaseQueryOptions<Task[]>
): UseBaseQueryResult<Task[]> {
  return useBaseQuery(
    () => apiClient.tasks.list(filter),
    [filter],
    options
  );
}

/**
 * Hook for fetching a single task by ID
 * 
 * @param id - Task ID
 * @param options - Query options
 * @returns Enhanced query result
 */
export function useTask(
  id: Uuid | null, 
  options?: UseBaseQueryOptions<Task | null>
): UseBaseQueryResult<Task | null> {
  return useBaseQuery(
    () => id ? apiClient.tasks.get(id) : Promise.resolve(null),
    [id],
    { 
      enabled: Boolean(id) && options?.enabled !== false,
      ...options
    }
  );
}

/**
 * Hook for creating a task
 * 
 * @param options - Mutation options
 * @returns Enhanced mutation result
 */
export function useCreateTask(
  options?: UseBaseMutationOptions<Task, CreateTaskInput>
): UseBaseMutationResult<Task, CreateTaskInput> {
  return useBaseMutation(
    (input: CreateTaskInput) => apiClient.tasks.create(input),
    options
  );
}

/**
 * Hook for updating a task's status
 * 
 * @param options - Mutation options
 * @returns Enhanced mutation result
 */
export function useUpdateTaskStatus(
  options?: UseBaseMutationOptions<void, UpdateTaskStatusInput>
): UseBaseMutationResult<void, UpdateTaskStatusInput> {
  return useBaseMutation(
    (input: UpdateTaskStatusInput) => apiClient.tasks.updateStatus(input),
    options
  );
}

/**
 * Hook for fetching task statistics
 * 
 * @param workspaceId - Workspace ID
 * @param options - Query options
 * @returns Enhanced query result
 */
export function useTaskStats(
  workspaceId: Uuid | null, 
  options?: UseBaseQueryOptions<TaskStats>
): UseBaseQueryResult<TaskStats> {
  return useBaseQuery(
    () => workspaceId ? apiClient.tasks.getStats(workspaceId) : Promise.reject(new Error('Workspace ID is required')),
    [workspaceId],
    { 
      enabled: Boolean(workspaceId) && options?.enabled !== false,
      ...options
    }
  );
}

/**
 * Hook for fetching overdue tasks
 * 
 * @param options - Query options
 * @returns Enhanced query result
 */
export function useOverdueTasks(
  options?: UseBaseQueryOptions<Task[]>
): UseBaseQueryResult<Task[]> {
  return useBaseQuery(
    () => apiClient.tasks.getOverdue(),
    [],
    options
  );
}

/**
 * Hook for fetching high priority tasks
 * 
 * @param options - Query options
 * @returns Enhanced query result
 */
export function useHighPriorityTasks(
  options?: UseBaseQueryOptions<Task[]>
): UseBaseQueryResult<Task[]> {
  return useBaseQuery(
    () => apiClient.tasks.getHighPriority(),
    [],
    options
  );
}

// Enhanced Plan Hooks

/**
 * Hook for fetching a paginated list of plans
 * 
 * @param filter - Filter criteria for plans
 * @param options - Query options
 * @returns Enhanced query result with pagination support
 */
export function usePlans(
  filter?: PlanFilter, 
  options?: UseBaseListQueryOptions<Plan>
): UseBaseListQueryResult<Plan> {
  return useBaseListQuery(
    (offset, limit) => apiClient.plans.list({ ...filter, offset, limit }),
    [filter],
    options
  );
}

/**
 * Hook for fetching a single plan by ID
 * 
 * @param id - Plan ID
 * @param options - Query options
 * @returns Enhanced query result
 */
export function usePlan(
  id: Uuid | null, 
  options?: UseBaseQueryOptions<Plan | null>
): UseBaseQueryResult<Plan | null> {
  return useBaseQuery(
    () => id ? apiClient.plans.get(id) : Promise.resolve(null),
    [id],
    { 
      enabled: Boolean(id) && options?.enabled !== false,
      ...options
    }
  );
}

/**
 * Hook for creating a plan
 * 
 * @param options - Mutation options
 * @returns Enhanced mutation result
 */
export function useCreatePlan(
  options?: UseBaseMutationOptions<Plan, CreatePlanInput>
): UseBaseMutationResult<Plan, CreatePlanInput> {
  return useBaseMutation(
    (input: CreatePlanInput) => apiClient.plans.create(input),
    options
  );
}

/**
 * Hook for updating a plan's status
 * 
 * @param options - Mutation options
 * @returns Enhanced mutation result
 */
export function useUpdatePlanStatus(
  options?: UseBaseMutationOptions<void, UpdatePlanStatusInput>
): UseBaseMutationResult<void, UpdatePlanStatusInput> {
  return useBaseMutation(
    (input: UpdatePlanStatusInput) => apiClient.plans.updateStatus(input),
    options
  );
}

/**
 * Hook for fetching plan statistics
 * 
 * @param participantId - Participant ID
 * @param options - Query options
 * @returns Enhanced query result
 */
export function usePlanStats(
  participantId: Uuid | null, 
  options?: UseBaseQueryOptions<PlanStats>
): UseBaseQueryResult<PlanStats> {
  return useBaseQuery(
    () => participantId ? apiClient.plans.getStats(participantId) : Promise.reject(new Error('Participant ID is required')),
    [participantId],
    { 
      enabled: Boolean(participantId) && options?.enabled !== false,
      ...options
    }
  );
}

// Export all hooks from baseHooks for direct use
export * from './baseHooks';
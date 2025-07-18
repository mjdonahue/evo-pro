import { ipc_invoke, ipc_invoke_with_retry, createRetryableIpcInvoke } from './ipc';
import type {
  ApiResponse,
  ListResponse,
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
} from '../lib/api/types';
import { 
  cacheManager, 
  entityCacheKey, 
  listCacheKey, 
  CacheOptions 
} from '../lib/api/cache';
import { 
  ApiError, 
  ErrorCategory, 
  ErrorContext, 
  ErrorSeverity, 
  Errors, 
  ErrorUtils 
} from '../lib/api/errors';
import { RetryOptions, RetryManager } from '../lib/api/retry';
import { 
  offlineQueueManager, 
  QueuedOperation, 
  OfflineQueueOptions 
} from '../lib/api/offline';
import {
  syncManager,
  syncEvents,
  SyncEventType,
  SyncEventListener,
  SyncOptions,
  SyncProgress,
  SyncResult,
  SyncStatus,
  SyncOperation
} from '../lib/api/sync';

/**
 * Error class for Controller operations
 */
export class ControllerError extends ApiError {
  /**
   * Creates a new controller error
   * @param code - Error code
   * @param message - Error message
   * @param context - Error context
   */
  constructor(
    code: string,
    message: string,
    context?: ErrorContext
  ) {
    // Determine the error category based on the code
    let category = ErrorCategory.UNKNOWN;
    let severity = ErrorSeverity.ERROR;
    let retryable = false;

    // Map common error codes to categories
    if (code.includes('not_found') || code === 'get_error') {
      category = ErrorCategory.NOT_FOUND;
      severity = ErrorSeverity.WARNING;
    } else if (code.includes('validation')) {
      category = ErrorCategory.VALIDATION;
      severity = ErrorSeverity.WARNING;
    } else if (code.includes('auth')) {
      category = ErrorCategory.AUTHENTICATION;
    } else if (code.includes('permission')) {
      category = ErrorCategory.AUTHORIZATION;
    } else if (code === 'invoke_error') {
      category = ErrorCategory.NETWORK;
      retryable = true;
    } else if (code.includes('server')) {
      category = ErrorCategory.SERVER;
      retryable = true;
    } else if (code.includes('timeout')) {
      category = ErrorCategory.TIMEOUT;
      retryable = true;
    } else if (code.includes('create_error') || code.includes('update_error') || code.includes('delete_error')) {
      category = ErrorCategory.BUSINESS_LOGIC;
    }

    super(code, message, category, context, {
      severity,
      retryable
    });

    this.name = 'ControllerError';
  }
}

/**
 * Base controller class with common CRUD operations and error handling
 * @template T - The entity type
 * @template C - The creation input type
 * @template U - The update input type
 * @template F - The filter type for list operations
 */
export class BaseController<T, C, U, F = Record<string, any>> {
  /**
   * Retryable version of ipc_invoke for this controller
   * @private
   */
  private retryableInvoke: <R>(method: string, params?: object) => Promise<R>;

  /**
   * Creates a new controller for the specified entity type
   * @param entityName - The name of the entity type (used in API endpoint construction)
   * @param cacheOptions - Default cache options for this controller
   * @param retryOptions - Retry options for this controller
   */
  constructor(
    protected entityName: string,
    protected cacheOptions: CacheOptions = { ttl: 5 * 60 * 1000, tags: [] },
    protected retryOptions?: RetryOptions
  ) {
    // Add the entity name as a tag for cache invalidation
    if (!this.cacheOptions.tags) {
      this.cacheOptions.tags = [];
    }
    if (!this.cacheOptions.tags.includes(this.entityName)) {
      this.cacheOptions.tags.push(this.entityName);
    }

    // Create a retryable version of ipc_invoke with entity-specific retry options
    this.retryableInvoke = createRetryableIpcInvoke(this.retryOptions, this.entityName);
  }

  /**
   * Retrieves an entity by ID
   * @param id - The ID of the entity to retrieve
   * @param options - Cache options for this operation
   * @returns A promise that resolves to the entity
   * @throws {ControllerError} If the operation fails
   */
  async get(id: string | Uuid, options?: Partial<CacheOptions>): Promise<T> {
    const cacheKey = entityCacheKey(this.entityName, id.toString());
    const cacheOpts = { ...this.cacheOptions, ...options };
    const context: ErrorContext = {
      operation: 'get',
      entityType: this.entityName,
      entityId: id.toString(),
      request: { id }
    };

    try {
      // Try to get from cache first
      if (!cacheOpts.bypass) {
        const cachedData = cacheManager.get<T>(cacheKey, cacheOpts);
        if (cachedData) {
          return cachedData;
        }
      }

      // If not in cache or bypass is true, fetch from API with retry for transient failures
      const response = await this.retryableInvoke<ApiResponse<T>>(`get_${this.entityName}`, { id });

      if (!response.success) {
        // Use the Errors factory to create a not found error
        if (!response.data) {
          throw Errors.notFound(
            this.entityName,
            id.toString(),
            {
              ...context,
              details: { errorCode: response.errorCode, error: response.error }
            }
          );
        } else {
          // If we have an error but also data, it's a different kind of error
          throw new ControllerError(
            response.errorCode || 'get_error',
            response.error || `Failed to get ${this.entityName}`,
            context
          );
        }
      }

      // Cache the result if successful
      if (!cacheOpts.bypass) {
        cacheManager.set(cacheKey, response.data, cacheOpts);
      }

      return response.data;
    } catch (error) {
      if (error instanceof ApiError) {
        throw error;
      }

      // Convert unknown errors to ControllerError
      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : `Unknown error getting ${this.entityName}`,
        {
          ...context,
          originalError: error
        }
      );
    }
  }

  /**
   * Creates a new entity
   * @param data - The data for the new entity
   * @returns A promise that resolves to the created entity
   * @throws {ControllerError} If the operation fails
   */
  async create(data: C): Promise<T> {
    const context: ErrorContext = {
      operation: 'create',
      entityType: this.entityName,
      request: { data }
    };

    try {
      const response = await this.retryableInvoke<ApiResponse<T>>(`create_${this.entityName}`, { input: data });

      if (!response.success) {
        // Check for validation errors
        if (response.errorCode?.includes('validation')) {
          throw Errors.validation(
            response.error || `Invalid data for ${this.entityName}`,
            { inputData: data },
            {
              ...context,
              details: { errorCode: response.errorCode }
            }
          );
        } else {
          // Other business logic errors
          throw Errors.businessLogic(
            response.errorCode || 'create_error',
            response.error || `Failed to create ${this.entityName}`,
            {
              ...context,
              userMessage: `Could not create the ${this.entityName}.`,
              recoveryAction: 'Please check your input and try again.'
            }
          );
        }
      }

      // Invalidate list cache since a new entity was created
      cacheManager.clearByEntity(this.entityName);

      return response.data;
    } catch (error) {
      if (error instanceof ApiError) {
        throw error;
      }

      // Convert unknown errors to ControllerError
      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : `Unknown error creating ${this.entityName}`,
        {
          ...context,
          originalError: error
        }
      );
    }
  }

  /**
   * Updates an existing entity
   * @param id - The ID of the entity to update
   * @param data - The updated data for the entity
   * @returns A promise that resolves to the updated entity
   * @throws {ControllerError} If the operation fails
   */
  async update(id: string | Uuid, data: U): Promise<T> {
    const context: ErrorContext = {
      operation: 'update',
      entityType: this.entityName,
      entityId: id.toString(),
      request: { id, data }
    };

    try {
      const response = await this.retryableInvoke<ApiResponse<T>>(`update_${this.entityName}`, { id, data });

      if (!response.success) {
        // Check for not found errors
        if (!response.data && response.errorCode?.includes('not_found')) {
          throw Errors.notFound(
            this.entityName,
            id.toString(),
            {
              ...context,
              details: { errorCode: response.errorCode, error: response.error }
            }
          );
        }
        // Check for validation errors
        else if (response.errorCode?.includes('validation')) {
          throw Errors.validation(
            response.error || `Invalid data for ${this.entityName}`,
            { inputData: data },
            {
              ...context,
              details: { errorCode: response.errorCode }
            }
          );
        }
        // Other business logic errors
        else {
          throw Errors.businessLogic(
            response.errorCode || 'update_error',
            response.error || `Failed to update ${this.entityName}`,
            {
              ...context,
              userMessage: `Could not update the ${this.entityName}.`,
              recoveryAction: 'Please check your input and try again.'
            }
          );
        }
      }

      // Invalidate both the specific entity cache and list cache
      const cacheKey = entityCacheKey(this.entityName, id.toString());
      cacheManager.remove(cacheKey);
      cacheManager.clearByEntity(this.entityName);

      return response.data;
    } catch (error) {
      if (error instanceof ApiError) {
        throw error;
      }

      // Convert unknown errors to ControllerError
      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : `Unknown error updating ${this.entityName}`,
        {
          ...context,
          originalError: error
        }
      );
    }
  }

  /**
   * Deletes an entity by ID
   * @param id - The ID of the entity to delete
   * @returns A promise that resolves when the entity is deleted
   * @throws {ControllerError} If the operation fails
   */
  async delete(id: string | Uuid): Promise<void> {
    const context: ErrorContext = {
      operation: 'delete',
      entityType: this.entityName,
      entityId: id.toString(),
      request: { id }
    };

    try {
      const response = await this.retryableInvoke<ApiResponse<void>>(`delete_${this.entityName}`, { id });

      if (!response.success) {
        // Check for not found errors
        if (response.errorCode?.includes('not_found')) {
          throw Errors.notFound(
            this.entityName,
            id.toString(),
            {
              ...context,
              details: { errorCode: response.errorCode, error: response.error }
            }
          );
        }
        // Check for authorization errors
        else if (response.errorCode?.includes('permission') || response.errorCode?.includes('auth')) {
          throw Errors.authorization(
            response.error || `Not authorized to delete ${this.entityName}`,
            {
              ...context,
              details: { errorCode: response.errorCode }
            }
          );
        }
        // Other business logic errors
        else {
          throw Errors.businessLogic(
            response.errorCode || 'delete_error',
            response.error || `Failed to delete ${this.entityName}`,
            {
              ...context,
              userMessage: `Could not delete the ${this.entityName}.`,
              recoveryAction: 'Please try again later or contact support.'
            }
          );
        }
      }

      // Invalidate both the specific entity cache and list cache
      const cacheKey = entityCacheKey(this.entityName, id.toString());
      cacheManager.remove(cacheKey);
      cacheManager.clearByEntity(this.entityName);
    } catch (error) {
      if (error instanceof ApiError) {
        throw error;
      }

      // Convert unknown errors to ControllerError
      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : `Unknown error deleting ${this.entityName}`,
        {
          ...context,
          originalError: error
        }
      );
    }
  }

  /**
   * Retrieves a list of entities based on filter criteria
   * @param filter - The filter criteria
   * @param options - Cache options for this operation
   * @returns A promise that resolves to a list of entities
   * @throws {ControllerError} If the operation fails
   */
  async list(filter?: F, options?: Partial<CacheOptions>): Promise<ListResponse<T>> {
    const cacheKey = listCacheKey(this.entityName, filter);
    const cacheOpts = { ...this.cacheOptions, ...options };
    const context: ErrorContext = {
      operation: 'list',
      entityType: this.entityName,
      request: { filter }
    };

    try {
      // Try to get from cache first
      if (!cacheOpts.bypass) {
        const cachedData = cacheManager.get<ListResponse<T>>(cacheKey, cacheOpts);
        if (cachedData) {
          return cachedData;
        }
      }

      // If not in cache or bypass is true, fetch from API with retry for transient failures
      const response = await this.retryableInvoke<ApiResponse<ListResponse<T>>>(`list_${this.entityName}s`, { filter });

      if (!response.success) {
        // Check for validation errors in filter
        if (response.errorCode?.includes('validation')) {
          throw Errors.validation(
            response.error || `Invalid filter for ${this.entityName} list`,
            { filter },
            {
              ...context,
              details: { errorCode: response.errorCode }
            }
          );
        }
        // Check for authorization errors
        else if (response.errorCode?.includes('permission') || response.errorCode?.includes('auth')) {
          throw Errors.authorization(
            response.error || `Not authorized to list ${this.entityName}s`,
            {
              ...context,
              details: { errorCode: response.errorCode }
            }
          );
        }
        // Other errors
        else {
          throw Errors.businessLogic(
            response.errorCode || 'list_error',
            response.error || `Failed to list ${this.entityName}s`,
            {
              ...context,
              userMessage: `Could not retrieve the list of ${this.entityName}s.`,
              recoveryAction: 'Please try again later.'
            }
          );
        }
      }

      // Cache the result if successful
      if (!cacheOpts.bypass) {
        cacheManager.set(cacheKey, response.data, cacheOpts);
      }

      return response.data;
    } catch (error) {
      if (error instanceof ApiError) {
        throw error;
      }

      // Convert unknown errors to ControllerError
      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : `Unknown error listing ${this.entityName}s`,
        {
          ...context,
          originalError: error
        }
      );
    }
  }
}

/**
 * Controller for Conversation entities
 */
export class ConversationController extends BaseController<Conversation, CreateConversationInput, Conversation, ConversationFilter> {
  constructor() {
    super('conversation');
  }

  /**
   * Retrieves the participants of a conversation
   * @param conversationId - The ID of the conversation
   * @returns A promise that resolves to a list of users
   * @throws {ControllerError} If the operation fails
   */
  async getParticipants(conversationId: string | Uuid): Promise<User[]> {
    try {
      const response = await ipc_invoke<ApiResponse<User[]>>('get_conversation_participants', { conversation_id: conversationId });

      if (!response.success) {
        throw new ControllerError(
          'get_participants_error',
          response.error || 'Failed to get conversation participants',
          { conversationId }
        );
      }

      return response.data;
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error getting conversation participants',
        { conversationId, originalError: error }
      );
    }
  }

  /**
   * Adds a participant to a conversation
   * @param conversationId - The ID of the conversation
   * @param userId - The ID of the user to add
   * @returns A promise that resolves when the participant is added
   * @throws {ControllerError} If the operation fails
   */
  async addParticipant(conversationId: string | Uuid, userId: string | Uuid): Promise<void> {
    try {
      const response = await ipc_invoke<ApiResponse<void>>('add_conversation_participant', { 
        conversation_id: conversationId, 
        user_id: userId 
      });

      if (!response.success) {
        throw new ControllerError(
          'add_participant_error',
          response.error || 'Failed to add conversation participant',
          { conversationId, userId }
        );
      }
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error adding conversation participant',
        { conversationId, userId, originalError: error }
      );
    }
  }
}

/**
 * Controller for Message entities
 */
export class MessageController extends BaseController<Message, CreateMessageInput, Message, MessageFilter> {
  constructor() {
    super('message');
  }

  /**
   * Marks a message as read
   * @param id - The ID of the message to mark as read
   * @returns A promise that resolves when the message is marked as read
   * @throws {ControllerError} If the operation fails
   */
  async markAsRead(id: string | Uuid): Promise<void> {
    try {
      const response = await ipc_invoke<ApiResponse<void>>('mark_message_read', { id });

      if (!response.success) {
        throw new ControllerError(
          'mark_read_error',
          response.error || 'Failed to mark message as read',
          { id }
        );
      }
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error marking message as read',
        { id, originalError: error }
      );
    }
  }
}

/**
 * Controller for Task entities
 */
export class TaskController extends BaseController<Task, CreateTaskInput, Task, TaskFilter> {
  constructor() {
    super('task');
  }

  /**
   * Updates the status of a task
   * @param input - The input containing the task ID and new status
   * @returns A promise that resolves when the status is updated
   * @throws {ControllerError} If the operation fails
   */
  async updateStatus(input: UpdateTaskStatusInput): Promise<void> {
    try {
      const response = await ipc_invoke<ApiResponse<void>>('update_task_status', { 
        id: input.id, 
        status: input.status 
      });

      if (!response.success) {
        throw new ControllerError(
          'update_status_error',
          response.error || 'Failed to update task status',
          { input }
        );
      }
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error updating task status',
        { input, originalError: error }
      );
    }
  }

  /**
   * Starts a task
   * @param id - The ID of the task to start
   * @returns A promise that resolves when the task is started
   * @throws {ControllerError} If the operation fails
   */
  async start(id: string | Uuid): Promise<void> {
    try {
      const response = await ipc_invoke<ApiResponse<void>>('start_task', { id });

      if (!response.success) {
        throw new ControllerError(
          'start_task_error',
          response.error || 'Failed to start task',
          { id }
        );
      }
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error starting task',
        { id, originalError: error }
      );
    }
  }

  /**
   * Completes a task
   * @param id - The ID of the task to complete
   * @returns A promise that resolves when the task is completed
   * @throws {ControllerError} If the operation fails
   */
  async complete(id: string | Uuid): Promise<void> {
    try {
      const response = await ipc_invoke<ApiResponse<void>>('complete_task', { id });

      if (!response.success) {
        throw new ControllerError(
          'complete_task_error',
          response.error || 'Failed to complete task',
          { id }
        );
      }
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error completing task',
        { id, originalError: error }
      );
    }
  }

  /**
   * Fails a task
   * @param id - The ID of the task to fail
   * @returns A promise that resolves when the task is failed
   * @throws {ControllerError} If the operation fails
   */
  async fail(id: string | Uuid): Promise<void> {
    try {
      const response = await ipc_invoke<ApiResponse<void>>('fail_task', { id });

      if (!response.success) {
        throw new ControllerError(
          'fail_task_error',
          response.error || 'Failed to fail task',
          { id }
        );
      }
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error failing task',
        { id, originalError: error }
      );
    }
  }

  /**
   * Retrieves task statistics
   * @param workspaceId - Optional workspace ID to filter by
   * @returns A promise that resolves to task statistics
   * @throws {ControllerError} If the operation fails
   */
  async getStats(workspaceId?: string | Uuid): Promise<TaskStats> {
    try {
      const response = await ipc_invoke<ApiResponse<TaskStats>>('get_task_stats', workspaceId ? { workspace_id: workspaceId } : {});

      if (!response.success) {
        throw new ControllerError(
          'get_stats_error',
          response.error || 'Failed to get task statistics',
          { workspaceId }
        );
      }

      return response.data;
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error getting task statistics',
        { workspaceId, originalError: error }
      );
    }
  }

  /**
   * Retrieves overdue tasks
   * @returns A promise that resolves to a list of overdue tasks
   * @throws {ControllerError} If the operation fails
   */
  async getOverdue(): Promise<Task[]> {
    try {
      const response = await ipc_invoke<ApiResponse<Task[]>>('get_overdue_tasks');

      if (!response.success) {
        throw new ControllerError(
          'get_overdue_error',
          response.error || 'Failed to get overdue tasks',
          {}
        );
      }

      return response.data;
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error getting overdue tasks',
        { originalError: error }
      );
    }
  }

  /**
   * Retrieves high priority tasks
   * @returns A promise that resolves to a list of high priority tasks
   * @throws {ControllerError} If the operation fails
   */
  async getHighPriority(): Promise<Task[]> {
    try {
      const response = await ipc_invoke<ApiResponse<Task[]>>('get_high_priority_tasks');

      if (!response.success) {
        throw new ControllerError(
          'get_high_priority_error',
          response.error || 'Failed to get high priority tasks',
          {}
        );
      }

      return response.data;
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error getting high priority tasks',
        { originalError: error }
      );
    }
  }

  /**
   * Retrieves tasks by plan
   * @param planId - The ID of the plan
   * @returns A promise that resolves to a list of tasks
   * @throws {ControllerError} If the operation fails
   */
  async getByPlan(planId: string | Uuid): Promise<Task[]> {
    try {
      const response = await ipc_invoke<ApiResponse<Task[]>>('get_tasks_by_plan', { plan_id: planId });

      if (!response.success) {
        throw new ControllerError(
          'get_by_plan_error',
          response.error || 'Failed to get tasks by plan',
          { planId }
        );
      }

      return response.data;
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error getting tasks by plan',
        { planId, originalError: error }
      );
    }
  }
}

/**
 * Controller for Plan entities
 */
export class PlanController extends BaseController<Plan, CreatePlanInput, Plan, PlanFilter> {
  constructor() {
    super('plan');
  }

  /**
   * Updates the status of a plan
   * @param input - The input containing the plan ID and new status
   * @returns A promise that resolves when the status is updated
   * @throws {ControllerError} If the operation fails
   */
  async updateStatus(input: UpdatePlanStatusInput): Promise<void> {
    try {
      const response = await ipc_invoke<ApiResponse<void>>('update_plan_status', { input });

      if (!response.success) {
        throw new ControllerError(
          'update_status_error',
          response.error || 'Failed to update plan status',
          { input }
        );
      }
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error updating plan status',
        { input, originalError: error }
      );
    }
  }

  /**
   * Starts a plan
   * @param id - The ID of the plan to start
   * @returns A promise that resolves when the plan is started
   * @throws {ControllerError} If the operation fails
   */
  async start(id: string | Uuid): Promise<void> {
    try {
      const response = await ipc_invoke<ApiResponse<void>>('start_plan', { id });

      if (!response.success) {
        throw new ControllerError(
          'start_plan_error',
          response.error || 'Failed to start plan',
          { id }
        );
      }
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error starting plan',
        { id, originalError: error }
      );
    }
  }

  /**
   * Completes a plan
   * @param id - The ID of the plan to complete
   * @returns A promise that resolves when the plan is completed
   * @throws {ControllerError} If the operation fails
   */
  async complete(id: string | Uuid): Promise<void> {
    try {
      const response = await ipc_invoke<ApiResponse<void>>('complete_plan', { id });

      if (!response.success) {
        throw new ControllerError(
          'complete_plan_error',
          response.error || 'Failed to complete plan',
          { id }
        );
      }
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error completing plan',
        { id, originalError: error }
      );
    }
  }

  /**
   * Fails a plan
   * @param id - The ID of the plan to fail
   * @returns A promise that resolves when the plan is failed
   * @throws {ControllerError} If the operation fails
   */
  async fail(id: string | Uuid): Promise<void> {
    try {
      const response = await ipc_invoke<ApiResponse<void>>('fail_plan', { id });

      if (!response.success) {
        throw new ControllerError(
          'fail_plan_error',
          response.error || 'Failed to fail plan',
          { id }
        );
      }
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error failing plan',
        { id, originalError: error }
      );
    }
  }

  /**
   * Retrieves plan statistics
   * @param participantId - Optional participant ID to filter by
   * @returns A promise that resolves to plan statistics
   * @throws {ControllerError} If the operation fails
   */
  async getStats(participantId?: string | Uuid): Promise<PlanStats> {
    try {
      const response = await ipc_invoke<ApiResponse<PlanStats>>('get_plan_stats', participantId ? { participant_id: participantId } : {});

      if (!response.success) {
        throw new ControllerError(
          'get_stats_error',
          response.error || 'Failed to get plan statistics',
          { participantId }
        );
      }

      return response.data;
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error getting plan statistics',
        { participantId, originalError: error }
      );
    }
  }

  /**
   * Retrieves plans by participant
   * @param participantId - The ID of the participant
   * @returns A promise that resolves to a list of plans
   * @throws {ControllerError} If the operation fails
   */
  async getByParticipant(participantId: string | Uuid): Promise<Plan[]> {
    try {
      const response = await ipc_invoke<ApiResponse<Plan[]>>('get_plans_by_participant', { participant_id: participantId });

      if (!response.success) {
        throw new ControllerError(
          'get_by_participant_error',
          response.error || 'Failed to get plans by participant',
          { participantId }
        );
      }

      return response.data;
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error getting plans by participant',
        { participantId, originalError: error }
      );
    }
  }

  /**
   * Retrieves active plans by participant
   * @param participantId - The ID of the participant
   * @returns A promise that resolves to a list of active plans
   * @throws {ControllerError} If the operation fails
   */
  async getActive(participantId: string | Uuid): Promise<Plan[]> {
    try {
      const response = await ipc_invoke<ApiResponse<Plan[]>>('get_active_plans_by_participant', { participant_id: participantId });

      if (!response.success) {
        throw new ControllerError(
          'get_active_error',
          response.error || 'Failed to get active plans by participant',
          { participantId }
        );
      }

      return response.data;
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error getting active plans by participant',
        { participantId, originalError: error }
      );
    }
  }
}

/**
 * Controller for TaskAssignee entities
 */
export class TaskAssigneeController extends BaseController<TaskAssignee, any, any> {
  constructor() {
    super('task_assignee');
  }

  /**
   * Retrieves assignees by task
   * @param taskId - The ID of the task
   * @returns A promise that resolves to a list of task assignees
   * @throws {ControllerError} If the operation fails
   */
  async getByTask(taskId: string | Uuid): Promise<TaskAssignee[]> {
    try {
      const response = await ipc_invoke<ApiResponse<TaskAssignee[]>>('get_task_assignees', { task_id: taskId });

      if (!response.success) {
        throw new ControllerError(
          'get_by_task_error',
          response.error || 'Failed to get assignees by task',
          { taskId }
        );
      }

      return response.data;
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error getting assignees by task',
        { taskId, originalError: error }
      );
    }
  }

  /**
   * Retrieves tasks by participant
   * @param participantId - The ID of the participant
   * @returns A promise that resolves to a list of task assignees
   * @throws {ControllerError} If the operation fails
   */
  async getByParticipant(participantId: string | Uuid): Promise<TaskAssignee[]> {
    try {
      const response = await ipc_invoke<ApiResponse<TaskAssignee[]>>('get_assignee_tasks', { participant_id: participantId });

      if (!response.success) {
        throw new ControllerError(
          'get_by_participant_error',
          response.error || 'Failed to get tasks by participant',
          { participantId }
        );
      }

      return response.data;
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error getting tasks by participant',
        { participantId, originalError: error }
      );
    }
  }

  /**
   * Adds an assignee to a task
   * @param taskId - The ID of the task
   * @param participantId - The ID of the participant
   * @param role - The role of the assignee
   * @returns A promise that resolves to the created task assignee
   * @throws {ControllerError} If the operation fails
   */
  async addAssignee(taskId: string | Uuid, participantId: string | Uuid, role: string): Promise<TaskAssignee> {
    try {
      const response = await ipc_invoke<ApiResponse<TaskAssignee>>('add_task_assignee', { 
        task_id: taskId, 
        participant_id: participantId, 
        role 
      });

      if (!response.success) {
        throw new ControllerError(
          'add_assignee_error',
          response.error || 'Failed to add task assignee',
          { taskId, participantId, role }
        );
      }

      return response.data;
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error adding task assignee',
        { taskId, participantId, role, originalError: error }
      );
    }
  }

  /**
   * Removes an assignee from a task
   * @param taskId - The ID of the task
   * @param participantId - The ID of the participant
   * @returns A promise that resolves when the assignee is removed
   * @throws {ControllerError} If the operation fails
   */
  async removeAssignee(taskId: string | Uuid, participantId: string | Uuid): Promise<void> {
    try {
      const response = await ipc_invoke<ApiResponse<void>>('remove_task_assignee', { 
        task_id: taskId, 
        participant_id: participantId 
      });

      if (!response.success) {
        throw new ControllerError(
          'remove_assignee_error',
          response.error || 'Failed to remove task assignee',
          { taskId, participantId }
        );
      }
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error removing task assignee',
        { taskId, participantId, originalError: error }
      );
    }
  }

  /**
   * Updates the status of an assignee
   * @param assigneeId - The ID of the assignee
   * @param status - The new status
   * @returns A promise that resolves when the status is updated
   * @throws {ControllerError} If the operation fails
   */
  async updateStatus(assigneeId: string | Uuid, status: string): Promise<void> {
    try {
      const response = await ipc_invoke<ApiResponse<void>>('update_assignee_status', { 
        assignee_id: assigneeId, 
        status 
      });

      if (!response.success) {
        throw new ControllerError(
          'update_status_error',
          response.error || 'Failed to update assignee status',
          { assigneeId, status }
        );
      }
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error updating assignee status',
        { assigneeId, status, originalError: error }
      );
    }
  }

  /**
   * Transfers the primary assignee of a task
   * @param taskId - The ID of the task
   * @param newPrimaryParticipantId - The ID of the new primary participant
   * @returns A promise that resolves when the primary assignee is transferred
   * @throws {ControllerError} If the operation fails
   */
  async transferPrimary(taskId: string | Uuid, newPrimaryParticipantId: string | Uuid): Promise<void> {
    try {
      const response = await ipc_invoke<ApiResponse<void>>('transfer_primary_assignee', { 
        task_id: taskId, 
        new_primary_participant_id: newPrimaryParticipantId 
      });

      if (!response.success) {
        throw new ControllerError(
          'transfer_primary_error',
          response.error || 'Failed to transfer primary assignee',
          { taskId, newPrimaryParticipantId }
        );
      }
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error transferring primary assignee',
        { taskId, newPrimaryParticipantId, originalError: error }
      );
    }
  }
}

/**
 * Controller for Agent entities
 */
export class AgentController extends BaseController<Agent, Omit<Agent, 'id' | 'created_at' | 'updated_at'>, Agent> {
  constructor() {
    super('agent');
  }

  /**
   * Invokes an agent with input
   * @param agentId - The ID of the agent
   * @param input - The input for the agent
   * @returns A promise that resolves to the agent's response
   * @throws {ControllerError} If the operation fails
   */
  async invokeAgent(agentId: string | Uuid, input: string): Promise<string> {
    try {
      const response = await ipc_invoke<ApiResponse<string>>('invoke_agent', { agent_id: agentId, input });

      if (!response.success) {
        throw new ControllerError(
          'invoke_agent_error',
          response.error || 'Failed to invoke agent',
          { agentId, input }
        );
      }

      return response.data;
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error invoking agent',
        { agentId, input, originalError: error }
      );
    }
  }
}

/**
 * Controller for User entities
 */
export class UserController extends BaseController<User, any, User> {
  constructor() {
    super('user');
  }
}

/**
 * Main controller class that combines all specialized controllers
 */
export class Controllers {
  public readonly conversations = new ConversationController();
  public readonly messages = new MessageController();
  public readonly tasks = new TaskController();
  public readonly plans = new PlanController();
  public readonly taskAssignees = new TaskAssigneeController();
  public readonly agents = new AgentController();
  public readonly users = new UserController();

  /**
   * Checks if the API is healthy
   * @returns A promise that resolves to true if the API is healthy, false otherwise
   */
  async healthCheck(): Promise<boolean> {
    try {
      await ipc_invoke('health_check');
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Cache management methods
   */
  cache = {
    /**
     * Sets the caching strategy
     * @param strategy - The caching strategy to use
     */
    setStrategy: (strategy: CacheStrategy): void => {
      cacheManager.setStrategy(strategy);
    },

    /**
     * Sets default cache options
     * @param options - The default cache options
     */
    setDefaultOptions: (options: Partial<CacheOptions>): void => {
      cacheManager.setDefaultOptions(options);
    },

    /**
     * Clears the entire cache
     */
    clear: (): void => {
      cacheManager.clear();
    },

    /**
     * Clears cache entries with specific tags
     * @param tags - The tags to clear
     */
    clearByTags: (tags: string[]): void => {
      cacheManager.clearByTags(tags);
    },

    /**
     * Clears cache entries for a specific entity type
     * @param entityName - The entity name
     */
    clearByEntity: (entityName: string): void => {
      cacheManager.clearByEntity(entityName);
    }
  };

  /**
   * Retry mechanism configuration
   */
  retry = {
    /**
     * Sets the default retry options for all operations
     * @param options - The retry options to use
     */
    setDefaultOptions: (options: RetryOptions): void => {
      RetryManager.setDefaultOptions(options);
    },

    /**
     * Enables or disables retry globally
     * @param enabled - Whether retry should be enabled
     */
    setEnabled: (enabled: boolean): void => {
      RetryManager.setEnabled(enabled);
    },

    /**
     * Sets a custom logger for retry events
     * @param logger - The logger function
     */
    setLogger: (logger: (message: string, data?: any) => void): void => {
      RetryManager.setLogger(logger);
    },

    /**
     * Gets the current retry configuration
     * @returns The current retry configuration
     */
    getConfig: (): { defaultOptions: RetryOptions; enabled: boolean } => {
      return RetryManager.getConfig();
    }
  };

  /**
   * Offline queue management and synchronization
   */
  offline = {
    /**
     * Checks if the application is currently online
     * @returns Whether the application is online
     */
    isOnline: (): boolean => {
      return offlineQueueManager.isNetworkOnline();
    },

    /**
     * Gets all operations in the queue
     * @returns All queued operations
     */
    getQueuedOperations: (): QueuedOperation[] => {
      return offlineQueueManager.getQueuedOperations();
    },

    /**
     * Gets the number of operations in the queue
     * @returns The queue length
     */
    getQueueLength: (): number => {
      return offlineQueueManager.getQueueLength();
    },

    /**
     * Processes the queue of operations
     * @returns A promise that resolves when all operations have been processed
     * @deprecated Use synchronize() instead for better control and feedback
     */
    processQueue: async (): Promise<void> => {
      return offlineQueueManager.processQueue();
    },

    /**
     * Clears all operations from the queue
     */
    clearQueue: (): void => {
      offlineQueueManager.clearQueue();
    },

    /**
     * Removes a specific operation from the queue
     * @param id - The ID of the operation to remove
     */
    removeOperation: (id: string): void => {
      offlineQueueManager.removeOperation(id);
    },

    /**
     * Sets options for the offline queue
     * @param options - The options to set
     */
    setOptions: (options: OfflineQueueOptions): void => {
      // Create a new manager with the updated options
      const newManager = new OfflineQueueManager(undefined, options);

      // Copy over any existing operations
      const operations = offlineQueueManager.getQueuedOperations();
      operations.forEach(op => {
        if (!op.processing) {
          newManager.enqueue({
            type: op.type,
            method: op.method,
            params: op.params,
            entityType: op.entityType,
            entityId: op.entityId
          });
        }
      });

      // Clean up the old manager
      offlineQueueManager.destroy();

      // Replace the global instance
      (window as any).__OFFLINE_QUEUE_MANAGER__ = newManager;
    },

    /**
     * Synchronization methods
     */
    sync: {
      /**
       * Synchronizes offline operations with the server
       * @param options - Options for the synchronization process
       * @returns A promise that resolves to the synchronization result
       */
      synchronize: async (options?: SyncOptions): Promise<SyncResult> => {
        // Get the operations from the offline queue
        const operations = offlineQueueManager.getQueuedOperations();

        // Use the ipc_invoke function for actual API calls
        const result = await syncManager.synchronize(operations, ipc_invoke);

        // Process the synchronization result
        if (result.completed > 0) {
          // Remove all successfully synced operations
          operations.forEach(op => {
            // Find the corresponding sync operation
            const syncOp = result.failedOperations.find(failedOp => failedOp.id === op.id) ||
                          result.skippedOperations.find(skippedOp => skippedOp.id === op.id);

            // If the operation was not failed or skipped, it was successful
            if (!syncOp) {
              offlineQueueManager.removeOperation(op.id);
            }
          });

          // Clear the queue if all operations were successful
          if (result.failed === 0 && result.skipped === 0) {
            offlineQueueManager.clearQueue();
          }
        }

        return result;
      },

      /**
       * Aborts the current synchronization process
       */
      abort: (): void => {
        syncManager.abort();
      },

      /**
       * Gets the current synchronization progress
       * @returns The current progress
       */
      getProgress: (): SyncProgress => {
        return syncManager.getProgress();
      },

      /**
       * Checks if synchronization is in progress
       * @returns Whether synchronization is in progress
       */
      isSynchronizing: (): boolean => {
        return syncManager.isSynchronizing();
      },

      /**
       * Adds an event listener for synchronization events
       * @param type - The event type to listen for
       * @param listener - The listener function
       */
      addEventListener: (type: SyncEventType, listener: SyncEventListener): void => {
        syncEvents.addEventListener(type, listener);
      },

      /**
       * Removes an event listener for synchronization events
       * @param type - The event type to remove the listener from
       * @param listener - The listener function to remove
       */
      removeEventListener: (type: SyncEventType, listener: SyncEventListener): void => {
        syncEvents.removeEventListener(type, listener);
      },

      /**
       * Gets the current synchronization status
       * @returns The current status
       */
      getStatus: (): SyncStatus => {
        return syncManager.getProgress().status;
      }
    }
  };

  /**
   * Retrieves the API version
   * @returns A promise that resolves to the API version
   * @throws {ControllerError} If the operation fails
   */
  async getVersion(): Promise<string> {
    try {
      const response = await ipc_invoke<ApiResponse<string>>('get_version');

      if (!response.success) {
        throw new ControllerError(
          'get_version_error',
          response.error || 'Failed to get version',
          {}
        );
      }

      return response.data;
    } catch (error) {
      if (error instanceof ControllerError) {
        throw error;
      }

      throw new ControllerError(
        'invoke_error',
        error instanceof Error ? error.message : 'Unknown error getting version',
        { originalError: error }
      );
    }
  }
}

/**
 * Singleton instance of the Controllers class
 */
export const controllers = new Controllers();

/**
 * For backward compatibility with the old Controller class
 */
export class Controller<M, C, U> {
  suffix: string;

  constructor(suffix: string) {
    this.suffix = suffix;
  }

  async get(id: string): Promise<M> {
    return ipc_invoke<ApiResponse<M>>(`get_${this.suffix}`, { id }).then((res) => res.data);
  }

  async create(data: C): Promise<string> {
    return ipc_invoke<ApiResponse<string>>(`create_${this.suffix}`, { data }).then((res) => {
      return res.data;
    });
  }

  async update(id: string, data: U): Promise<string> {
    return ipc_invoke<ApiResponse<string>>(`update_${this.suffix}`, { id, data }).then((res) => {
      return res.data;
    });
  }

  async delete(id: string): Promise<string> {
    return ipc_invoke<ApiResponse<string>>(`delete_${this.suffix}`, { id }).then((res) => res.data);
  }
}

/**
 * For backward compatibility with the old ChatController
 */
export class ChatController extends Controller<any, any, any> {
  constructor() {
    super('chat');
  }

  async list(page: any): Promise<any[]> {
    return ipc_invoke<ApiResponse<any[]>>(`list_${this.suffix}s`, { page }).then((res) => res.data);
  }
}

/**
 * For backward compatibility with the old chatController
 */
export const chatController = new ChatController();

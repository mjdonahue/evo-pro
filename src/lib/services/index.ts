import { BaseService } from './baseService';
import { ConversationService, conversationService } from './conversationService';
import { TaskService, taskService } from './taskService';

/**
 * Service registry that provides access to all services in the application.
 * This is the main entry point for accessing services.
 */
export class ServiceRegistry {
  /**
   * Conversation service for managing conversations and messages
   */
  public readonly conversations: ConversationService;

  /**
   * Task service for managing tasks, plans, and task assignments
   */
  public readonly tasks: TaskService;

  /**
   * Creates a new service registry
   * @param options - Options for configuring the service registry
   */
  constructor(options: {
    conversationService?: ConversationService;
    taskService?: TaskService;
  } = {}) {
    this.conversations = options.conversationService || conversationService;
    this.tasks = options.taskService || taskService;
  }
}

/**
 * Singleton instance of the service registry
 */
export const services = new ServiceRegistry();

// Export all services and base classes
export { BaseService, ConversationService, TaskService };
export { conversationService, taskService };
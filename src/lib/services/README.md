# Service Layer

This directory contains the service layer for the application. The service layer provides higher-level abstractions over the API client, organizing functionality around business domains rather than entity types.

## Overview

The service layer is organized around business domains, with each service providing a cohesive API for a specific domain. Services use the API client under the hood but provide higher-level abstractions, combine multiple API calls, and add business logic.

## Service Registry

The `ServiceRegistry` class provides a single entry point for accessing all services in the application. It's available as a singleton instance `services` that can be imported from the services module:

```typescript
import { services } from '@/lib/services';

// Use the conversation service
const conversations = await services.conversations.getConversations();

// Use the task service
const tasks = await services.tasks.getTasks();
```

## Available Services

### BaseService

The `BaseService` class is the base class for all services. It provides common functionality like error handling and access to the API client.

### ConversationService

The `ConversationService` provides methods for working with conversations and messages:

```typescript
import { conversationService } from '@/lib/services';

// Get conversations
const conversations = await conversationService.getConversations();

// Get a specific conversation
const conversation = await conversationService.getConversation(id);

// Create a conversation
const newConversation = await conversationService.createConversation({
  title: 'New Conversation',
  type: 'Group'
});

// Send a message
const message = await conversationService.sendMessage(
  conversationId,
  { text: 'Hello, world!' }
);

// Start a new conversation with an initial message
const { conversation, message } = await conversationService.startConversation(
  'New Conversation',
  'Group',
  { text: 'Hello, world!' }
);
```

### TaskService

The `TaskService` provides methods for working with tasks, plans, and task assignments:

```typescript
import { taskService } from '@/lib/services';

// Get tasks
const tasks = await taskService.getTasks();

// Get a specific task
const task = await taskService.getTask(id);

// Create a task
const newTask = await taskService.createTask({
  plan_id: planId,
  participant_id: participantId,
  workspace_id: workspaceId,
  title: 'New Task',
  start_time: new Date().toISOString(),
  priority: 'Medium',
  urgency: 'Medium',
  importance: 'Medium',
  memory_type: 'Memory'
});

// Update task status
await taskService.updateTaskStatus(taskId, 'InProgress');

// Create a plan with an initial task
const { plan, task } = await taskService.createPlanWithTask(
  participantId,
  'Task',
  'New Task',
  {
    description: 'Task description',
    priority: 'High',
    dueDate: '2023-12-31T23:59:59Z'
  }
);
```

## Creating New Services

To create a new service:

1. Create a new file in the `services` directory
2. Extend the `BaseService` class
3. Implement methods for your service domain
4. Export a singleton instance of your service
5. Update the `ServiceRegistry` class in `index.ts` to include your service

Example:

```typescript
import { BaseService } from './baseService';
import type { User, Uuid } from '../api/types';

export class UserService extends BaseService {
  async getUsers(): Promise<User[]> {
    try {
      return await this.api.users.list();
    } catch (error) {
      this.handleError(error, { operation: 'getUsers' });
    }
  }

  async getUser(id: Uuid): Promise<User | null> {
    try {
      return await this.api.users.get(id);
    } catch (error) {
      this.handleError(error, { operation: 'getUser', id });
    }
  }
}

export const userService = new UserService();
```

Then update the `ServiceRegistry`:

```typescript
import { UserService, userService } from './userService';

export class ServiceRegistry {
  public readonly conversations: ConversationService;
  public readonly tasks: TaskService;
  public readonly users: UserService;

  constructor(options = {}) {
    this.conversations = options.conversationService || conversationService;
    this.tasks = options.taskService || taskService;
    this.users = options.userService || userService;
  }
}

export { UserService, userService };
```

## Testing Services

Services can be tested by mocking the API client. See the `__tests__` directory for examples.
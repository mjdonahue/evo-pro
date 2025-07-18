# API Controllers

This directory contains the API controllers for the Evo Design application. The controllers provide a type-safe interface for interacting with the backend API.

## Overview

The API controllers are organized into specialized controllers for each entity type, with a base controller that provides common CRUD operations. The controllers use the `ipc_invoke` function from `ipc.ts` for communication with the backend.

## Controllers

### BaseController

The `BaseController` class provides common CRUD operations and error handling for all entity types. It is a generic class that takes the following type parameters:

- `T`: The entity type
- `C`: The creation input type
- `U`: The update input type
- `F`: The filter type for list operations (defaults to `Record<string, any>`)

### Specialized Controllers

The following specialized controllers are available:

- `ConversationController`: For Conversation entities
- `MessageController`: For Message entities
- `TaskController`: For Task entities
- `PlanController`: For Plan entities
- `TaskAssigneeController`: For TaskAssignee entities
- `AgentController`: For Agent entities
- `UserController`: For User entities

### Main Controllers Class

The `Controllers` class combines all the specialized controllers into a single interface. It provides the following properties:

- `conversations`: ConversationController
- `messages`: MessageController
- `tasks`: TaskController
- `plans`: PlanController
- `taskAssignees`: TaskAssigneeController
- `agents`: AgentController
- `users`: UserController

It also provides utility methods:

- `healthCheck()`: Checks if the API is healthy
- `getVersion()`: Retrieves the API version

## Usage Examples

### Using the Controllers Singleton

The recommended way to use the controllers is through the `controllers` singleton:

```typescript
import { controllers } from './api/controllers';

// Get a conversation
const conversation = await controllers.conversations.get('conversation-id');

// Create a message
const message = await controllers.messages.create({
  conversation_id: 'conversation-id',
  content: { text: 'Hello, world!' }
});

// Update a task
const task = await controllers.tasks.update('task-id', {
  title: 'Updated task title',
  description: 'Updated task description'
});

// Delete a plan
await controllers.plans.delete('plan-id');

// List conversations with filtering
const conversations = await controllers.conversations.list({
  status: 'Active',
  limit: 10,
  offset: 0
});

// Check API health
const isHealthy = await controllers.healthCheck();
```

### Using Specialized Controllers Directly

You can also create instances of the specialized controllers directly:

```typescript
import { ConversationController, TaskController } from './api/controllers';

const conversationController = new ConversationController();
const taskController = new TaskController();

// Get a conversation
const conversation = await conversationController.get('conversation-id');

// Get task statistics
const taskStats = await taskController.getStats('workspace-id');
```

### Error Handling

The controllers provide comprehensive error handling through the `ControllerError` class:

```typescript
import { controllers, ControllerError } from './api/controllers';

try {
  const conversation = await controllers.conversations.get('non-existent-id');
} catch (error) {
  if (error instanceof ControllerError) {
    console.error(`Error code: ${error.code}`);
    console.error(`Error message: ${error.message}`);
    console.error(`Error details: ${JSON.stringify(error.details)}`);
  } else {
    console.error('Unknown error:', error);
  }
}
```

## Backward Compatibility

For backward compatibility with the old Controller class, the following classes are provided:

- `Controller<M, C, U>`: The original Controller class
- `ChatController`: The original ChatController class
- `chatController`: The original chatController singleton

These are provided for compatibility with existing code, but new code should use the new controllers.
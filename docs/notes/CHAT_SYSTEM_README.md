# Unified Chat System

A comprehensive chat framework built with React, TypeScript, Zustand, and Tailwind CSS that supports both user-to-user direct messaging and user-to-agent streaming conversations.

## Features

### ✨ Core Features
- **Unified Interface**: Single chat interface for both user-to-user and user-to-agent conversations
- **Real-time Messaging**: Optimistic updates with real-time synchronization
- **Agent Streaming**: Live streaming responses from AI agents
- **Typing Indicators**: Real-time typing status for participants
- **Message Status**: Sent, delivered, read status indicators
- **Participant Management**: Unified handling of users and agents
- **Responsive Design**: Mobile-friendly interface with Tailwind CSS

### 🏗️ Architecture

#### State Management (Zustand)
- **ParticipantStore**: Manages users and agents with unified participant interface
- **ConversationStore**: Handles conversation metadata, participants, and UI state
- **MessageStore**: Manages messages, optimistic updates, and streaming

#### Components
- **ChatInterface**: Main chat container with auto-scroll and real-time updates
- **MessageList**: Displays messages with grouping and timestamps
- **MessageInput**: Handles input with typing indicators and submission
- **MessageItem**: Individual message component with reactions and actions
- **ConversationHeader**: Shows conversation info and participant status
- **TypingIndicator**: Real-time typing status display
- **StreamingIndicator**: Live streaming response visualization

#### Hooks
- **useChatApi**: API operations for regular messaging
- **useAgentStreaming**: Streaming functionality for agent conversations

## Usage

### Basic Setup

```tsx
import { ChatInterface } from './components/chat/ChatInterface';
import { useParticipantStore, useConversationStore } from './stores';

function MyApp() {
  const currentUser = useParticipantStore(state => state.currentUser);
  const activeConversation = useConversationStore(state => state.activeConversationId);
  
  return (
    <div className="h-screen">
      {activeConversation && (
        <ChatInterface 
          conversationId={activeConversation}
          className="flex-1"
        />
      )}
    </div>
  );
}
```

### Creating Conversations

#### User-to-User Conversation
```tsx
const { createDirectConversation, addConversation } = useConversationStore();

// Create a direct conversation with another user
const conversation = createDirectConversation(otherUserId);
addConversation(conversation);
```

#### User-to-Agent Conversation
```tsx
const { createAgentConversation, addConversation } = useConversationStore();

// Create a conversation with an AI agent
const conversation = createAgentConversation(agentId);
addConversation(conversation);
```

### Managing Participants

```tsx
const { setCurrentUser, addUser, addAgent } = useParticipantStore();

// Set current user
setCurrentUser(user);

// Add other users
addUser(otherUser);

// Add AI agents
addAgent(agent);
```

### Sending Messages

The system automatically handles message sending through the `MessageInput` component:

- **User-to-User**: Regular API calls with optimistic updates
- **User-to-Agent**: Streaming API calls with real-time response generation

### Streaming Responses

Agent conversations automatically use streaming when available:

```tsx
const { streamMessage } = useAgentStreaming();

await streamMessage({
  conversationId,
  agentId,
  content: userMessage,
  onStart: (streamingMessage) => {
    // Streaming started
  },
  onChunk: (chunk) => {
    // New content chunk received
  },
  onComplete: (finalMessage) => {
    // Streaming completed
  },
  onError: (error) => {
    // Handle streaming error
  }
});
```

## Demo

Visit `/chat` to see the unified chat system in action with:
- Sample user-to-user conversation
- Sample user-to-agent conversation with streaming
- Interactive UI with conversation switching
- Real-time typing indicators
- Message status indicators

## API Integration

The system integrates with your Tauri backend through:

### Message API
- `createMessage`: Send new messages
- `updateMessage`: Update existing messages
- `deleteMessage`: Delete messages
- `markAsRead`: Mark messages as read

### Agent API
- `invokeAgent`: Invoke agent with streaming support
- `getAgent`: Get agent information

### Conversation API
- `createConversation`: Create new conversations
- `getConversationParticipants`: Get conversation participants
- `addParticipant`: Add participants to conversations

## Customization

### Styling
The system uses Tailwind CSS classes and can be customized by:
- Modifying component className props
- Updating the Tailwind configuration
- Creating custom CSS classes

### Message Types
Extend message types by:
- Adding new `MessageType` enums
- Updating message content parsing
- Creating custom message renderers

### Agent Integration
Customize agent behavior by:
- Implementing custom streaming protocols
- Adding agent-specific UI elements
- Extending agent metadata

## Performance

### Optimizations
- **Optimistic Updates**: Immediate UI feedback
- **Message Grouping**: Efficient rendering of consecutive messages
- **Auto-scroll Management**: Smart scroll behavior
- **Zustand Subscriptions**: Selective component re-rendering

### Best Practices
- Use `React.memo` for message components when needed
- Implement virtual scrolling for large message lists
- Debounce typing indicators
- Cache participant data

## Future Enhancements

- [ ] File attachments and media messages
- [ ] Message reactions and threading
- [ ] Voice messages and audio streaming
- [ ] End-to-end encryption
- [ ] Message search and filtering
- [ ] Conversation templates
- [ ] Agent personality customization
- [ ] Multi-language support

## Contributing

1. Follow the existing code patterns
2. Add proper TypeScript types
3. Include comprehensive tests
4. Update documentation
5. Follow the component architecture

## License

This chat system is part of the Evo Design project and follows the same licensing terms. 
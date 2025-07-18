import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import { immer } from 'zustand/middleware/immer';
import type { Message, Uuid } from '../lib/api/types';
import { MessageType, MessageStatus } from '../lib/api/types';

export interface StreamingMessage {
  id: string;
  conversationId: Uuid;
  senderId: Uuid;
  content: string;
  isStreaming: boolean;
  isComplete: boolean;
  timestamp: string;
  error?: string;
}

export interface MessageInfo extends Message {
  isOptimistic?: boolean; // For optimistic UI updates
  streamingContent?: string; // For streaming messages
  isStreaming?: boolean;
  reactions?: Record<string, Uuid[]>; // emoji -> user ids
  attachments?: any[]; // File attachments
  mentions?: Uuid[]; // Mentioned participants
}

export interface MessageState {
  // Core data
  messages: Map<Uuid, MessageInfo>;
  messagesByConversation: Map<Uuid, Uuid[]>; // conversation_id -> message_ids
  streamingMessages: Map<string, StreamingMessage>;
  
  // UI state
  isLoading: boolean;
  loadingConversations: Set<Uuid>;
  optimisticMessages: Map<string, MessageInfo>;
  
  // Actions
  setMessages: (conversationId: Uuid, messages: Message[]) => void;
  addMessage: (message: Message) => void;
  updateMessage: (id: Uuid, updates: Partial<MessageInfo>) => void;
  deleteMessage: (id: Uuid) => void;
  
  // Optimistic updates
  addOptimisticMessage: (tempId: string, message: Partial<MessageInfo>) => void;
  confirmOptimisticMessage: (tempId: string, confirmedMessage: Message) => void;
  removeOptimisticMessage: (tempId: string) => void;
  
  // Streaming support
  startStreaming: (streamingMessage: StreamingMessage) => void;
  updateStreamingContent: (id: string, content: string) => void;
  completeStreaming: (id: string, finalMessage?: Message) => void;
  stopStreaming: (id: string, error?: string) => void;
  
  // Reactions
  addReaction: (messageId: Uuid, emoji: string, userId: Uuid) => void;
  removeReaction: (messageId: Uuid, emoji: string, userId: Uuid) => void;
  
  // Status updates
  markAsRead: (messageId: Uuid) => void;
  markConversationAsRead: (conversationId: Uuid) => void;
  
  // Getters
  getMessage: (id: Uuid) => MessageInfo | null;
  getConversationMessages: (conversationId: Uuid) => MessageInfo[];
  getStreamingMessage: (id: string) => StreamingMessage | null;
  getUnreadCount: (conversationId: Uuid) => number;
  getLastMessage: (conversationId: Uuid) => MessageInfo | null;
  
  // Search and filtering
  searchMessages: (query: string, conversationId?: Uuid) => MessageInfo[];
  getMessagesByType: (type: MessageType, conversationId?: Uuid) => MessageInfo[];
  
  // Utilities
  isMessageFromCurrentUser: (messageId: Uuid, currentUserId: Uuid) => boolean;
  canDeleteMessage: (messageId: Uuid, currentUserId: Uuid) => boolean;
  canEditMessage: (messageId: Uuid, currentUserId: Uuid) => boolean;
}

export const useMessageStore = create<MessageState>()(
  subscribeWithSelector(
    immer((set, get) => ({
      // Initial state
      messages: new Map(),
      messagesByConversation: new Map(),
      streamingMessages: new Map(),
      isLoading: false,
      loadingConversations: new Set(),
      optimisticMessages: new Map(),
      
      // Actions
      setMessages: (conversationId, messages) => set((state) => {
        const messageIds: Uuid[] = [];
        
        messages.forEach(msg => {
          const messageInfo: MessageInfo = {
            ...msg,
            reactions: {},
            attachments: msg.metadata ? JSON.parse(msg.metadata).attachments || [] : [],
            mentions: msg.metadata ? JSON.parse(msg.metadata).mentions || [] : []
          };
          
          state.messages.set(msg.id, messageInfo);
          messageIds.push(msg.id);
        });
        
        // Sort messages by creation time
        messageIds.sort((a, b) => {
          const msgA = state.messages.get(a);
          const msgB = state.messages.get(b);
          if (!msgA || !msgB) return 0;
          return new Date(msgA.created_at).getTime() - new Date(msgB.created_at).getTime();
        });
        
        state.messagesByConversation.set(conversationId, messageIds);
      }),
      
      addMessage: (message) => set((state) => {
        const messageInfo: MessageInfo = {
          ...message,
          reactions: {},
          attachments: message.metadata ? JSON.parse(message.metadata).attachments || [] : [],
          mentions: message.metadata ? JSON.parse(message.metadata).mentions || [] : []
        };
        
        state.messages.set(message.id, messageInfo);
        
        // Add to conversation messages
        const conversationMessages = state.messagesByConversation.get(message.conversation_id) || [];
        const updatedMessages = [...conversationMessages, message.id];
        
        // Sort by creation time
        updatedMessages.sort((a, b) => {
          const msgA = state.messages.get(a);
          const msgB = state.messages.get(b);
          if (!msgA || !msgB) return 0;
          return new Date(msgA.created_at).getTime() - new Date(msgB.created_at).getTime();
        });
        
        state.messagesByConversation.set(message.conversation_id, updatedMessages);
      }),
      
      updateMessage: (id, updates) => set((state) => {
        const message = state.messages.get(id);
        if (message) {
          state.messages.set(id, { ...message, ...updates });
        }
      }),
      
      deleteMessage: (id) => set((state) => {
        const message = state.messages.get(id);
        if (message) {
          state.messages.delete(id);
          
          // Remove from conversation messages
          const conversationMessages = state.messagesByConversation.get(message.conversation_id) || [];
          const updatedMessages = conversationMessages.filter(msgId => msgId !== id);
          state.messagesByConversation.set(message.conversation_id, updatedMessages);
        }
      }),
      
      // Optimistic updates
      addOptimisticMessage: (tempId, message) => set((state) => {
        const messageInfo: MessageInfo = {
          id: tempId as unknown as Uuid,
          conversation_id: message.conversation_id!,
          workspace_id: message.workspace_id!,
          sender_id: message.sender_id!,
          type: message.type || MessageType.Text,
          content: message.content || '',
          status: MessageStatus.Pending,
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
          isOptimistic: true,
          reactions: {},
          attachments: [],
          mentions: [],
          ...message
        };
        
        state.optimisticMessages.set(tempId, messageInfo);
        
        // Add to conversation messages
        const conversationMessages = state.messagesByConversation.get(message.conversation_id!) || [];
        const updatedMessages = [...conversationMessages, tempId as unknown as Uuid];
        state.messagesByConversation.set(message.conversation_id!, updatedMessages);
      }),
      
      confirmOptimisticMessage: (tempId, confirmedMessage) => set((state) => {
        state.optimisticMessages.delete(tempId);
        
        // Remove temp message from conversation
        const conversationMessages = state.messagesByConversation.get(confirmedMessage.conversation_id) || [];
        const withoutTemp = conversationMessages.filter(id => id !== tempId as unknown as Uuid);
        
        // Add confirmed message
        const messageInfo: MessageInfo = {
          ...confirmedMessage,
          reactions: {},
          attachments: confirmedMessage.metadata ? JSON.parse(confirmedMessage.metadata).attachments || [] : [],
          mentions: confirmedMessage.metadata ? JSON.parse(confirmedMessage.metadata).mentions || [] : []
        };
        
        state.messages.set(confirmedMessage.id, messageInfo);
        
        const updatedMessages = [...withoutTemp, confirmedMessage.id];
        updatedMessages.sort((a, b) => {
          const msgA = state.messages.get(a) || state.optimisticMessages.get(a as string);
          const msgB = state.messages.get(b) || state.optimisticMessages.get(b as string);
          if (!msgA || !msgB) return 0;
          return new Date(msgA.created_at).getTime() - new Date(msgB.created_at).getTime();
        });
        
        state.messagesByConversation.set(confirmedMessage.conversation_id, updatedMessages);
      }),
      
      removeOptimisticMessage: (tempId) => set((state) => {
        const message = state.optimisticMessages.get(tempId);
        if (message) {
          state.optimisticMessages.delete(tempId);
          
          // Remove from conversation messages
          const conversationMessages = state.messagesByConversation.get(message.conversation_id) || [];
          const updatedMessages = conversationMessages.filter(id => id !== tempId as unknown as Uuid);
          state.messagesByConversation.set(message.conversation_id, updatedMessages);
        }
      }),
      
      // Streaming support
      startStreaming: (streamingMessage) => set((state) => {
        state.streamingMessages.set(streamingMessage.id, streamingMessage);
      }),
      
      updateStreamingContent: (id, content) => set((state) => {
        const streamingMessage = state.streamingMessages.get(id);
        if (streamingMessage) {
          state.streamingMessages.set(id, {
            ...streamingMessage,
            content
          });
        }
      }),
      
      completeStreaming: (id, finalMessage) => set((state) => {
        const streamingMessage = state.streamingMessages.get(id);
        if (streamingMessage) {
          state.streamingMessages.set(id, {
            ...streamingMessage,
            isStreaming: false,
            isComplete: true
          });
          
          // If we have a final message, add it to the regular messages
          if (finalMessage) {
            const messageInfo: MessageInfo = {
              ...finalMessage,
              reactions: {},
              attachments: finalMessage.metadata ? JSON.parse(finalMessage.metadata).attachments || [] : [],
              mentions: finalMessage.metadata ? JSON.parse(finalMessage.metadata).mentions || [] : []
            };
            
            state.messages.set(finalMessage.id, messageInfo);
            
            const conversationMessages = state.messagesByConversation.get(finalMessage.conversation_id) || [];
            const updatedMessages = [...conversationMessages, finalMessage.id];
            
            updatedMessages.sort((a, b) => {
              const msgA = state.messages.get(a);
              const msgB = state.messages.get(b);
              if (!msgA || !msgB) return 0;
              return new Date(msgA.created_at).getTime() - new Date(msgB.created_at).getTime();
            });
            
            state.messagesByConversation.set(finalMessage.conversation_id, updatedMessages);
          }
        }
      }),
      
      stopStreaming: (id, error) => set((state) => {
        const streamingMessage = state.streamingMessages.get(id);
        if (streamingMessage) {
          state.streamingMessages.set(id, {
            ...streamingMessage,
            isStreaming: false,
            isComplete: false,
            error
          });
        }
      }),
      
      // Reactions
      addReaction: (messageId, emoji, userId) => set((state) => {
        const message = state.messages.get(messageId);
        if (message) {
          const reactions = { ...message.reactions };
          if (!reactions[emoji]) {
            reactions[emoji] = [];
          }
          if (!reactions[emoji].includes(userId)) {
            reactions[emoji].push(userId);
          }
          state.messages.set(messageId, { ...message, reactions });
        }
      }),
      
      removeReaction: (messageId, emoji, userId) => set((state) => {
        const message = state.messages.get(messageId);
        if (message && message.reactions) {
          const reactions = { ...message.reactions };
          if (reactions[emoji]) {
            reactions[emoji] = reactions[emoji].filter(id => id !== userId);
            if (reactions[emoji].length === 0) {
              delete reactions[emoji];
            }
          }
          state.messages.set(messageId, { ...message, reactions });
        }
      }),
      
      // Status updates
      markAsRead: (messageId) => set((state) => {
        const message = state.messages.get(messageId);
        if (message) {
          state.messages.set(messageId, { ...message, status: MessageStatus.Read });
        }
      }),
      
      markConversationAsRead: (conversationId) => set((state) => {
        const messageIds = state.messagesByConversation.get(conversationId) || [];
        messageIds.forEach(id => {
          const message = state.messages.get(id);
          if (message && message.status !== MessageStatus.Read) {
            state.messages.set(id, { ...message, status: MessageStatus.Read });
          }
        });
      }),
      
      // Getters
      getMessage: (id) => {
        return get().messages.get(id) || null;
      },
      
      getConversationMessages: (conversationId) => {
        const state = get();
        const messageIds = state.messagesByConversation.get(conversationId) || [];
        
        return messageIds.map(id => {
          return state.messages.get(id) || state.optimisticMessages.get(id as string);
        }).filter(Boolean) as MessageInfo[];
      },
      
      getStreamingMessage: (id) => {
        return get().streamingMessages.get(id) || null;
      },
      
      getUnreadCount: (conversationId) => {
        const state = get();
        const messages = state.getConversationMessages(conversationId);
        return messages.filter(msg => msg.status !== MessageStatus.Read).length;
      },
      
      getLastMessage: (conversationId) => {
        const state = get();
        const messages = state.getConversationMessages(conversationId);
        return messages.length > 0 ? messages[messages.length - 1] : null;
      },
      
      // Search and filtering
      searchMessages: (query, conversationId) => {
        const state = get();
        
        let messagesToSearch: MessageInfo[];
        if (conversationId) {
          messagesToSearch = state.getConversationMessages(conversationId);
        } else {
          messagesToSearch = Array.from(state.messages.values());
        }
        
        return messagesToSearch.filter(msg => {
          const content = typeof msg.content === 'string' ? msg.content : JSON.stringify(msg.content);
          return content.toLowerCase().includes(query.toLowerCase());
        });
      },
      
      getMessagesByType: (type, conversationId) => {
        const state = get();
        
        let messagesToFilter: MessageInfo[];
        if (conversationId) {
          messagesToFilter = state.getConversationMessages(conversationId);
        } else {
          messagesToFilter = Array.from(state.messages.values());
        }
        
        return messagesToFilter.filter(msg => msg.type === type);
      },
      
      // Utilities
      isMessageFromCurrentUser: (messageId, currentUserId) => {
        const message = get().messages.get(messageId);
        return message ? message.sender_id === currentUserId : false;
      },
      
      canDeleteMessage: (messageId, currentUserId) => {
        const message = get().messages.get(messageId);
        if (!message) return false;
        
        // Users can delete their own messages
        if (message.sender_id === currentUserId) return true;
        
        // Add role-based permissions here if needed
        return false;
      },
      
      canEditMessage: (messageId, currentUserId) => {
        const message = get().messages.get(messageId);
        if (!message) return false;
        
        // Only text messages can be edited
        if (message.type !== MessageType.Text) return false;
        
        // Users can edit their own messages
        if (message.sender_id === currentUserId) return true;
        
        return false;
      }
    }))
  )
); 
import { BaseService } from './baseService';
import type {
  Conversation,
  ConversationFilter,
  CreateConversationInput,
  Message,
  MessageFilter,
  CreateMessageInput,
  User,
  Uuid,
  ListResponse
} from '../api/types';

/**
 * Service for managing conversations and messages.
 * Provides higher-level abstractions for conversation-related operations.
 */
export class ConversationService extends BaseService {
  /**
   * Retrieves a list of conversations based on filter criteria
   * @param filter - Filter criteria for conversations
   * @returns A promise that resolves to a list of conversations
   */
  async getConversations(filter?: ConversationFilter): Promise<ListResponse<Conversation>> {
    try {
      return await this.api.conversations.list(filter);
    } catch (error) {
      this.handleError(error, { operation: 'getConversations', filter });
    }
  }

  /**
   * Retrieves a single conversation by ID
   * @param id - The ID of the conversation to retrieve
   * @returns A promise that resolves to the conversation or null if not found
   */
  async getConversation(id: Uuid): Promise<Conversation | null> {
    try {
      return await this.api.conversations.get(id);
    } catch (error) {
      this.handleError(error, { operation: 'getConversation', id });
    }
  }

  /**
   * Creates a new conversation
   * @param input - The data for the new conversation
   * @returns A promise that resolves to the created conversation
   */
  async createConversation(input: CreateConversationInput): Promise<Conversation> {
    try {
      return await this.api.conversations.create(input);
    } catch (error) {
      this.handleError(error, { operation: 'createConversation', input });
    }
  }

  /**
   * Updates an existing conversation
   * @param conversation - The updated conversation data
   * @returns A promise that resolves to the updated conversation
   */
  async updateConversation(conversation: Conversation): Promise<Conversation> {
    try {
      return await this.api.conversations.update(conversation);
    } catch (error) {
      this.handleError(error, { operation: 'updateConversation', conversation });
    }
  }

  /**
   * Deletes a conversation
   * @param id - The ID of the conversation to delete
   * @returns A promise that resolves when the conversation is deleted
   */
  async deleteConversation(id: Uuid): Promise<void> {
    try {
      await this.api.conversations.delete(id);
    } catch (error) {
      this.handleError(error, { operation: 'deleteConversation', id });
    }
  }

  /**
   * Retrieves messages for a conversation
   * @param conversationId - The ID of the conversation
   * @param filter - Additional filter criteria for messages
   * @returns A promise that resolves to a list of messages
   */
  async getMessages(conversationId: Uuid, filter?: Omit<MessageFilter, 'conversation_id'>): Promise<ListResponse<Message>> {
    try {
      const messageFilter: MessageFilter = {
        ...filter,
        conversation_id: conversationId
      };
      return await this.api.messages.list(messageFilter);
    } catch (error) {
      this.handleError(error, { operation: 'getMessages', conversationId, filter });
    }
  }

  /**
   * Sends a message to a conversation
   * @param conversationId - The ID of the conversation
   * @param content - The content of the message
   * @param options - Additional options for the message
   * @returns A promise that resolves to the created message
   */
  async sendMessage(
    conversationId: Uuid,
    content: Record<string, any>,
    options?: {
      type?: string;
      parentId?: Uuid;
      replyToId?: Uuid;
      metadata?: Record<string, any>;
    }
  ): Promise<Message> {
    try {
      const input: CreateMessageInput = {
        conversation_id: conversationId,
        content,
        type: options?.type as any,
        parent_id: options?.parentId,
        reply_to_id: options?.replyToId,
        metadata: options?.metadata
      };
      return await this.api.messages.create(input);
    } catch (error) {
      this.handleError(error, { operation: 'sendMessage', conversationId, content, options });
    }
  }

  /**
   * Marks a message as read
   * @param messageId - The ID of the message to mark as read
   * @returns A promise that resolves when the message is marked as read
   */
  async markMessageAsRead(messageId: Uuid): Promise<void> {
    try {
      await this.api.messages.markAsRead(messageId);
    } catch (error) {
      this.handleError(error, { operation: 'markMessageAsRead', messageId });
    }
  }

  /**
   * Retrieves participants of a conversation
   * @param conversationId - The ID of the conversation
   * @returns A promise that resolves to a list of users
   */
  async getParticipants(conversationId: Uuid): Promise<User[]> {
    try {
      return await this.api.conversations.getParticipants(conversationId);
    } catch (error) {
      this.handleError(error, { operation: 'getParticipants', conversationId });
    }
  }

  /**
   * Adds a participant to a conversation
   * @param conversationId - The ID of the conversation
   * @param userId - The ID of the user to add
   * @returns A promise that resolves when the participant is added
   */
  async addParticipant(conversationId: Uuid, userId: Uuid): Promise<void> {
    try {
      await this.api.conversations.addParticipant(conversationId, userId);
    } catch (error) {
      this.handleError(error, { operation: 'addParticipant', conversationId, userId });
    }
  }

  /**
   * Creates a new conversation and sends an initial message
   * @param title - The title of the conversation
   * @param type - The type of conversation
   * @param initialMessage - The content of the initial message
   * @param metadata - Additional metadata for the conversation
   * @returns A promise that resolves to an object containing the created conversation and message
   */
  async startConversation(
    title: string,
    type: string,
    initialMessage: Record<string, any>,
    metadata?: Record<string, any>
  ): Promise<{ conversation: Conversation; message: Message }> {
    try {
      // Create the conversation
      const conversation = await this.api.conversations.create({
        title,
        type: type as any,
        metadata
      });

      // Send the initial message
      const message = await this.api.messages.create({
        conversation_id: conversation.id,
        content: initialMessage
      });

      return { conversation, message };
    } catch (error) {
      this.handleError(error, { operation: 'startConversation', title, type, initialMessage, metadata });
    }
  }
}

// Singleton instance
export const conversationService = new ConversationService();
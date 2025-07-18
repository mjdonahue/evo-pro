import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { conversationService } from '../conversationService';
import { apiClient } from '../../api/client';
import type { Conversation, Message } from '../../api/types';

// Mock the API client
vi.mock('../../api/client', () => ({
  apiClient: {
    conversations: {
      list: vi.fn(),
      get: vi.fn(),
      create: vi.fn(),
      update: vi.fn(),
      delete: vi.fn(),
      getParticipants: vi.fn(),
      addParticipant: vi.fn(),
    },
    messages: {
      list: vi.fn(),
      get: vi.fn(),
      create: vi.fn(),
      update: vi.fn(),
      delete: vi.fn(),
      markAsRead: vi.fn(),
    },
  },
  ApiClientError: class ApiClientError extends Error {
    constructor(public code: string, message: string, public details?: Record<string, any>) {
      super(message);
      this.name = 'ApiClientError';
    }
  },
}));

describe('ConversationService', () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('getConversations', () => {
    it('should return conversations from the API', async () => {
      // Mock data
      const mockConversations = {
        items: [
          { id: '1', title: 'Conversation 1' },
          { id: '2', title: 'Conversation 2' },
        ],
        total: 2,
      };

      // Mock the API response
      vi.mocked(apiClient.conversations.list).mockResolvedValueOnce(mockConversations);

      // Call the service method
      const result = await conversationService.getConversations({ limit: 10 });

      // Verify the API was called with the correct parameters
      expect(apiClient.conversations.list).toHaveBeenCalledWith({ limit: 10 });

      // Verify the result
      expect(result).toEqual(mockConversations);
    });

    it('should handle errors', async () => {
      // Mock the API to throw an error
      vi.mocked(apiClient.conversations.list).mockRejectedValueOnce(new Error('API error'));

      // Call the service method and expect it to throw
      await expect(conversationService.getConversations()).rejects.toThrow('API error');

      // Verify the API was called
      expect(apiClient.conversations.list).toHaveBeenCalled();
    });
  });

  describe('startConversation', () => {
    it('should create a conversation and send an initial message', async () => {
      // Mock data
      const mockConversation = { id: '1', title: 'New Conversation' } as Conversation;
      const mockMessage = { id: '1', conversation_id: '1', content: { text: 'Hello' } } as Message;

      // Mock the API responses
      vi.mocked(apiClient.conversations.create).mockResolvedValueOnce(mockConversation);
      vi.mocked(apiClient.messages.create).mockResolvedValueOnce(mockMessage);

      // Call the service method
      const result = await conversationService.startConversation(
        'New Conversation',
        'Group',
        { text: 'Hello' }
      );

      // Verify the APIs were called with the correct parameters
      expect(apiClient.conversations.create).toHaveBeenCalledWith({
        title: 'New Conversation',
        type: 'Group',
        metadata: undefined,
      });

      expect(apiClient.messages.create).toHaveBeenCalledWith({
        conversation_id: '1',
        content: { text: 'Hello' },
      });

      // Verify the result
      expect(result).toEqual({
        conversation: mockConversation,
        message: mockMessage,
      });
    });

    it('should handle errors when creating the conversation', async () => {
      // Mock the API to throw an error
      vi.mocked(apiClient.conversations.create).mockRejectedValueOnce(new Error('API error'));

      // Call the service method and expect it to throw
      await expect(
        conversationService.startConversation('New Conversation', 'Group', { text: 'Hello' })
      ).rejects.toThrow('API error');

      // Verify the API was called
      expect(apiClient.conversations.create).toHaveBeenCalled();
      expect(apiClient.messages.create).not.toHaveBeenCalled();
    });
  });

  // Add more tests for other methods as needed
});
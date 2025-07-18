import { useState, useCallback } from 'react';
import { ApiClient } from '../lib/api/client';
import { MessageType, type CreateMessageInput, type Message, type Uuid } from '../lib/api/types';

const apiClient = new ApiClient();

interface SendMessageParams {
  conversationId: Uuid;
  content: string;
  type?: MessageType;
  parentId?: Uuid;
  replyToId?: Uuid;
  metadata?: Record<string, any>;
}

export const useChatApi = () => {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  const sendMessage = useCallback(async (params: SendMessageParams): Promise<Message> => {
    setIsLoading(true);
    setError(null);
    
    try {
      const messageInput: CreateMessageInput = {
        conversation_id: params.conversationId,
        content: { text: params.content },
        type: params.type || MessageType.Text,
        parent_id: params.parentId,
        reply_to_id: params.replyToId,
        metadata: params.metadata
      };
      
      const message = await apiClient.messages.create(messageInput);
      return message;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to send message';
      setError(errorMessage);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);
  
  const updateMessage = useCallback(async (message: Message): Promise<Message> => {
    setIsLoading(true);
    setError(null);
    
    try {
      const updatedMessage = await apiClient.messages.update(message);
      return updatedMessage;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to update message';
      setError(errorMessage);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);
  
  const deleteMessage = useCallback(async (messageId: Uuid): Promise<void> => {
    setIsLoading(true);
    setError(null);
    
    try {
      await apiClient.messages.delete(messageId);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to delete message';
      setError(errorMessage);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);
  
  const markAsRead = useCallback(async (messageId: Uuid): Promise<void> => {
    try {
      await apiClient.messages.markAsRead(messageId);
    } catch (err) {
      console.error('Failed to mark message as read:', err);
    }
  }, []);
  
  return {
    sendMessage,
    updateMessage,
    deleteMessage,
    markAsRead,
    isLoading,
    error
  };
}; 
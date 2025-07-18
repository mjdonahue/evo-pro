import { useState, useCallback } from 'react';
import { useMessageStore, type StreamingMessage } from '../stores/messageStore';
import { ApiClient } from '../lib/api/client';
import { MessageType, type Message, type Uuid } from '../lib/api/types';

const apiClient = new ApiClient();

interface StreamMessageParams {
  conversationId: Uuid;
  agentId: Uuid;
  content: string;
  onStart?: (streamingMessage: StreamingMessage) => void;
  onChunk?: (chunk: string) => void;
  onComplete?: (finalMessage: Message) => void;
  onError?: (error: string) => void;
}

interface StreamMessageResponse {
  userMessage?: Message;
  streamingId: string;
}

export const useAgentStreaming = () => {
  const [isStreaming, setIsStreaming] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  const startStreaming = useMessageStore(state => state.startStreaming);
  const updateStreamingContent = useMessageStore(state => state.updateStreamingContent);
  const completeStreaming = useMessageStore(state => state.completeStreaming);
  const stopStreaming = useMessageStore(state => state.stopStreaming);
  
  const streamMessage = useCallback(async (params: StreamMessageParams): Promise<StreamMessageResponse> => {
    setIsStreaming(true);
    setError(null);
    
    const streamingId = `stream_${Date.now()}_${Math.random()}`;
    
    try {
      // First, send the user's message
      const userMessage = await apiClient.messages.create({
        conversation_id: params.conversationId,
        content: { text: params.content },
        type: MessageType.Text
      });
      
      // Create streaming message for agent response
      const streamingMessage: StreamingMessage = {
        id: streamingId,
        conversationId: params.conversationId,
        senderId: params.agentId,
        content: '',
        isStreaming: true,
        isComplete: false,
        timestamp: new Date().toISOString()
      };
      
      startStreaming(streamingMessage);
      params.onStart?.(streamingMessage);
      
      // Invoke agent with streaming
      const response = await apiClient.agents.invokeAgent(params.agentId, params.content);
      
      // Simulate streaming by chunking the response
      // In a real implementation, this would be actual streaming from the backend
      const chunks = response.match(/.{1,10}/g) || [response];
      let accumulatedContent = '';
      
      for (const chunk of chunks) {
        accumulatedContent += chunk;
        updateStreamingContent(streamingId, accumulatedContent);
        params.onChunk?.(chunk);
        
        // Add delay to simulate streaming
        await new Promise(resolve => setTimeout(resolve, 50));
      }
      
      // Create final message
      const finalMessage = await apiClient.messages.create({
        conversation_id: params.conversationId,
        content: { text: accumulatedContent },
        type: MessageType.Text,
        metadata: {
          agentId: params.agentId,
          streamingId: streamingId
        }
      });
      
      completeStreaming(streamingId, finalMessage);
      params.onComplete?.(finalMessage);
      
      return {
        userMessage,
        streamingId
      };
      
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Streaming failed';
      setError(errorMessage);
      stopStreaming(streamingId, errorMessage);
      params.onError?.(errorMessage);
      throw err;
    } finally {
      setIsStreaming(false);
    }
  }, [startStreaming, updateStreamingContent, completeStreaming, stopStreaming]);
  
  const stopCurrentStream = useCallback((streamingId: string) => {
    stopStreaming(streamingId, 'Stream stopped by user');
    setIsStreaming(false);
  }, [stopStreaming]);
  
  return {
    streamMessage,
    stopCurrentStream,
    isStreaming,
    error
  };
}; 
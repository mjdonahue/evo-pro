import React, { useState, useRef, useCallback, useEffect } from 'react';
import { useMessageStore } from '../../stores/messageStore';
import { useConversationStore } from '../../stores/conversationStore';
import { useParticipantStore } from '../../stores/participantStore';
import { useAgentStreaming } from '../../hooks/useAgentStreaming';
import { useChatApi } from '../../hooks/useChatApi';
import { MessageType } from '../../lib/api/types';
import type { Uuid } from '../../lib/api/types';

interface MessageInputProps {
  conversationId: Uuid;
  currentUserId: Uuid;
  isAgentConversation: boolean;
  agentId?: Uuid;
}

export const MessageInput: React.FC<MessageInputProps> = ({
  conversationId,
  currentUserId,
  isAgentConversation,
  agentId
}) => {
  const [message, setMessage] = useState('');
  const [isTyping, setIsTyping] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const typingTimeoutRef = useRef<NodeJS.Timeout>();
  
  // Store hooks
  const addOptimisticMessage = useMessageStore(state => state.addOptimisticMessage);
  const confirmOptimisticMessage = useMessageStore(state => state.confirmOptimisticMessage);
  const removeOptimisticMessage = useMessageStore(state => state.removeOptimisticMessage);
  
  const setTyping = useConversationStore(state => state.setTyping);
  const currentUser = useParticipantStore(state => state.currentUser);
  
  // API hooks
  const { sendMessage, isLoading } = useChatApi();
  const { streamMessage, isStreaming } = useAgentStreaming();
  
  // Auto-resize textarea
  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = `${textareaRef.current.scrollHeight}px`;
    }
  }, [message]);
  
  // Handle typing indicators
  const handleTypingStart = useCallback(() => {
    if (!isTyping) {
      setIsTyping(true);
      setTyping(conversationId, currentUserId, true);
    }
    
    // Clear existing timeout
    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current);
    }
    
    // Set new timeout to stop typing indicator
    typingTimeoutRef.current = setTimeout(() => {
      setIsTyping(false);
      setTyping(conversationId, currentUserId, false);
    }, 2000);
  }, [isTyping, conversationId, currentUserId]); // Remove setTyping - it's stable
  
  const handleTypingStop = useCallback(() => {
    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current);
    }
    setIsTyping(false);
    setTyping(conversationId, currentUserId, false);
  }, [conversationId, currentUserId]); // Remove setTyping - it's stable
  
  // Handle input changes
  const handleInputChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setMessage(e.target.value);
    
    if (e.target.value.trim()) {
      handleTypingStart();
    } else {
      handleTypingStop();
    }
  };
  
  // Handle message submission
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!message.trim() || isLoading || isStreaming) return;
    
    const messageContent = message.trim();
    setMessage('');
    handleTypingStop();
    
    // Generate temporary ID for optimistic update
    const tempId = `temp_${Date.now()}_${Math.random()}`;
    
    try {
      // Add optimistic message
      addOptimisticMessage(tempId, {
        conversation_id: conversationId,
        workspace_id: currentUser?.workspace_id,
        sender_id: currentUserId,
        type: MessageType.Text,
        content: JSON.stringify({ text: messageContent }),
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      });
      
      if (isAgentConversation && agentId) {
        // Stream message to agent
        const response = await streamMessage({
          conversationId,
          agentId,
          content: messageContent,
          onStart: (streamingMessage) => {
            // Streaming started
          },
          onChunk: (chunk) => {
            // Handle streaming chunk
          },
          onComplete: (finalMessage) => {
            // Confirm optimistic message and add agent response
            confirmOptimisticMessage(tempId, finalMessage);
          },
          onError: (error) => {
            // Remove optimistic message on error
            removeOptimisticMessage(tempId);
            console.error('Streaming error:', error);
          }
        });
        
        // Confirm the user's message
        if (response.userMessage) {
          confirmOptimisticMessage(tempId, response.userMessage);
        }
      } else {
        // Regular user-to-user message
        const sentMessage = await sendMessage({
          conversationId,
          content: messageContent,
          type: MessageType.Text
        });
        
        // Confirm optimistic message
        confirmOptimisticMessage(tempId, sentMessage);
      }
    } catch (error) {
      // Remove optimistic message on error
      removeOptimisticMessage(tempId);
      console.error('Failed to send message:', error);
    }
  };
  
  // Handle keyboard shortcuts
  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e);
    }
  };
  
  const isDisabled = isLoading || isStreaming;
  
  return (
    <form onSubmit={handleSubmit} className="flex items-end space-x-2">
      <div className="flex-1">
        <textarea
          ref={textareaRef}
          value={message}
          onChange={handleInputChange}
          onKeyDown={handleKeyDown}
          placeholder={
            isAgentConversation 
              ? "Ask the agent anything..." 
              : "Type a message..."
          }
          className="w-full px-4 py-2 border border-gray-300 rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent max-h-32 min-h-[40px]"
          rows={1}
          disabled={isDisabled}
        />
        
        {/* Character count or typing indicator */}
        <div className="flex justify-between items-center mt-1 text-xs text-gray-500">
          <div>
            {isTyping && "Typing..."}
          </div>
          <div>
            {message.length > 0 && `${message.length} characters`}
          </div>
        </div>
      </div>
      
      <button
        type="submit"
        disabled={!message.trim() || isDisabled}
        className={`px-4 py-2 rounded-lg font-medium transition-colors ${
          !message.trim() || isDisabled
            ? 'bg-gray-300 text-gray-500 cursor-not-allowed'
            : isAgentConversation
            ? 'bg-purple-600 text-white hover:bg-purple-700'
            : 'bg-blue-600 text-white hover:bg-blue-700'
        }`}
      >
        {isLoading || isStreaming ? (
          <div className="flex items-center space-x-2">
            <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
            <span>{isStreaming ? 'Streaming...' : 'Sending...'}</span>
          </div>
        ) : (
          <div className="flex items-center space-x-2">
            <span>Send</span>
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
            </svg>
          </div>
        )}
      </button>
    </form>
  );
}; 
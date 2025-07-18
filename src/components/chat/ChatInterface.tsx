import React, { useEffect, useRef, useState, useCallback, useMemo } from 'react';
import { useParticipantStore, ParticipantType } from '../../stores/participantStore';
import { useConversationStore } from '../../stores/conversationStore';
import { useMessageStore } from '../../stores/messageStore';
import { MessageList } from './MessageList';
import { MessageInput } from './MessageInput';
import { ConversationHeader } from './ConversationHeader';
import { TypingIndicator } from './TypingIndicator';
import { StreamingIndicator } from './StreamingIndicator';
import { LoadingSpinner } from '../ui/LoadingSpinner';
import type { Uuid } from '../../lib/api/types';

interface ChatInterfaceProps {
  conversationId: Uuid;
  className?: string;
}

export const ChatInterface: React.FC<ChatInterfaceProps> = ({ 
  conversationId, 
  className = 'h-full w-full'
}) => {
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const [isAutoScrollEnabled, setIsAutoScrollEnabled] = useState(true);
  const hasMarkedAsReadRef = useRef(false);
  
  // Store hooks with memoized selectors
  const currentUser = useParticipantStore(useCallback(state => state.currentUser, []));
  const getParticipant = useParticipantStore(useCallback(state => state.getParticipant, []));
  const isAgent = useParticipantStore(useCallback(state => state.isAgent, []));
  
  // Memoize selectors to prevent infinite re-renders
  const conversation = useConversationStore(
    useCallback(state => state.getConversation(conversationId), [conversationId])
  );
  const setActiveConversation = useConversationStore(useCallback(state => state.setActiveConversation, []));
  const resetActiveConversationUnreadCount = useConversationStore(useCallback(state => state.resetActiveConversationUnreadCount, []));
  
  const messages = useMessageStore(
    useCallback(state => state.getConversationMessages(conversationId), [conversationId])
  );
  const streamingMessages = useMessageStore(useCallback(state => state.streamingMessages, []));
  const markConversationAsRead = useMessageStore(useCallback(state => state.markConversationAsRead, []));
  
  // Set active conversation on mount
  useEffect(() => {
    setActiveConversation(conversationId);
    resetActiveConversationUnreadCount();
    hasMarkedAsReadRef.current = false; // Reset the flag for new conversation
    
    return () => {
      setActiveConversation(null);
    };
  }, [conversationId, setActiveConversation, resetActiveConversationUnreadCount]);
  
  // Auto-scroll to bottom and mark messages as read (only once)
  useEffect(() => {
    if (isAutoScrollEnabled && messagesEndRef.current) {
      messagesEndRef.current.scrollIntoView({ behavior: 'smooth' });
      
      // Mark messages as read only once when first viewing the conversation
      if (messages.length > 0 && !hasMarkedAsReadRef.current) {
        markConversationAsRead(conversationId);
        hasMarkedAsReadRef.current = true;
      }
    }
  }, [messages, streamingMessages, isAutoScrollEnabled, conversationId]); // markConversationAsRead is stable
  
  // Handle scroll to detect if user is at bottom
  const handleScroll = (e: React.UIEvent<HTMLDivElement>) => {
    const { scrollTop, scrollHeight, clientHeight } = e.currentTarget;
    const isAtBottom = scrollHeight - scrollTop <= clientHeight + 100; // 100px threshold
    setIsAutoScrollEnabled(isAtBottom);
  };
  
  if (!conversation || !currentUser) {
    return (
      <div className={`flex items-center justify-center h-full ${className}`}>
        <LoadingSpinner />
      </div>
    );
  }
  
  // Memoize derived values to prevent unnecessary re-renders
  const isAgentConversation = useMemo(() => 
    conversation?.participants.some(p => p.type === ParticipantType.AGENT) ?? false, 
    [conversation?.participants]
  );
  
  const agentParticipant = useMemo(() => 
    conversation?.participants.find(p => p.type === ParticipantType.AGENT), 
    [conversation?.participants]
  );
  
  // Memoize streaming messages to prevent unnecessary re-renders
  const filteredStreamingMessages = useMemo(() => 
    Array.from(streamingMessages.values())
      .filter(msg => msg.conversationId === conversationId)
      .map(streamingMsg => (
        <StreamingIndicator 
          key={streamingMsg.id}
          streamingMessage={streamingMsg}
          participant={getParticipant(streamingMsg.senderId)}
        />
      )), 
    [streamingMessages, conversationId, getParticipant]
  );
  
  return (
    <div className={`flex flex-col h-full bg-white ${className}`}>
      {/* Header */}
      <ConversationHeader 
        conversation={conversation}
        isAgentConversation={isAgentConversation}
        agentParticipant={agentParticipant}
      />
      
      {/* Messages */}
      <div 
        className="flex-1 overflow-y-auto px-4 py-2"
        onScroll={handleScroll}
      >
        <MessageList 
          messages={messages}
          currentUserId={currentUser.id}
          conversationId={conversationId}
          isAgentConversation={isAgentConversation}
        />
        
        {/* Streaming messages */}
        {filteredStreamingMessages}
        
        {/* Typing indicator */}
        {conversation.isTyping && (
          <TypingIndicator 
            typingUsers={conversation.typingUsers}
            currentUserId={currentUser.id}
          />
        )}
        
        {/* Auto-scroll anchor */}
        <div ref={messagesEndRef} />
      </div>
      
      {/* Message Input */}
      <div className="border-t border-gray-200 p-4">
        <MessageInput 
          conversationId={conversationId}
          currentUserId={currentUser.id}
          isAgentConversation={isAgentConversation}
          agentId={agentParticipant?.id}
        />
      </div>
      
      {/* Scroll to bottom button */}
      {!isAutoScrollEnabled && (
        <button
          onClick={() => {
            setIsAutoScrollEnabled(true);
            messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
          }}
          className="absolute bottom-20 right-8 bg-blue-500 text-white p-3 rounded-full shadow-lg hover:bg-blue-600 transition-colors"
          title="Scroll to bottom"
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 14l-7 7m0 0l-7-7m7 7V3" />
          </svg>
        </button>
      )}
    </div>
  );
}; 
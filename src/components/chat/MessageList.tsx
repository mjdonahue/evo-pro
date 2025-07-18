import React from 'react';
import { MessageItem } from './MessageItem';
import { useParticipantStore } from '../../stores/participantStore';
import type { MessageInfo } from '../../stores/messageStore';
import type { Uuid } from '../../lib/api/types';

interface MessageListProps {
  messages: MessageInfo[];
  currentUserId: Uuid;
  conversationId: Uuid;
  isAgentConversation: boolean;
}

export const MessageList: React.FC<MessageListProps> = ({
  messages,
  currentUserId,
  conversationId,
  isAgentConversation
}) => {
  const getParticipant = useParticipantStore(state => state.getParticipant);
  
  if (messages.length === 0) {
    return (
      <div className="flex items-center justify-center h-full text-gray-500">
        <div className="text-center">
          <div className="text-lg mb-2">No messages yet</div>
          <div className="text-sm">
            {isAgentConversation 
              ? "Start a conversation with the agent" 
              : "Send a message to start the conversation"
            }
          </div>
        </div>
      </div>
    );
  }
  
  return (
    <div className="space-y-4">
      {messages.map((message, index) => {
        const participant = getParticipant(message.sender_id);
        const isCurrentUser = message.sender_id === currentUserId;
        const previousMessage = index > 0 ? messages[index - 1] : null;
        const nextMessage = index < messages.length - 1 ? messages[index + 1] : null;
        
        // Group consecutive messages from the same sender
        const isFirstInGroup = !previousMessage || previousMessage.sender_id !== message.sender_id;
        const isLastInGroup = !nextMessage || nextMessage.sender_id !== message.sender_id;
        
        // Time grouping - show timestamp if more than 5 minutes apart
        const showTimestamp = !previousMessage || 
          (new Date(message.created_at).getTime() - new Date(previousMessage.created_at).getTime()) > 5 * 60 * 1000;
        
        return (
          <MessageItem
            key={message.id.toString()}
            message={message}
            participant={participant}
            isCurrentUser={isCurrentUser}
            isFirstInGroup={isFirstInGroup}
            isLastInGroup={isLastInGroup}
            showTimestamp={showTimestamp}
            isAgentConversation={isAgentConversation}
            conversationId={conversationId}
          />
        );
      })}
    </div>
  );
}; 
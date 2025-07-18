import React, { useState } from 'react';
import { format } from 'date-fns';
import { MessageType, MessageStatus } from '../../lib/api/types';
import { ParticipantType, type ParticipantInfo } from '../../stores/participantStore';
import { type MessageInfo } from '../../stores/messageStore';
import type { Uuid } from '../../lib/api/types';

interface MessageItemProps {
  message: MessageInfo;
  participant: ParticipantInfo | null;
  isCurrentUser: boolean;
  isFirstInGroup: boolean;
  isLastInGroup: boolean;
  showTimestamp: boolean;
  isAgentConversation: boolean;
  conversationId: Uuid;
}

export const MessageItem: React.FC<MessageItemProps> = ({
  message,
  participant,
  isCurrentUser,
  isFirstInGroup,
  isLastInGroup,
  showTimestamp,
  isAgentConversation
}) => {
  const [showActions, setShowActions] = useState(false);
  
  // Parse message content
  const getMessageContent = () => {
    if (typeof message.content === 'string') {
      try {
        const parsed = JSON.parse(message.content);
        return parsed.text || message.content;
      } catch {
        return message.content;
      }
    }
    if (typeof message.content === 'object' && message.content !== null) {
      return (message.content as any).text || 'Message content unavailable';
    }
    return 'Message content unavailable';
  };
  
  const getStatusIcon = () => {
    if (!isCurrentUser) return null;
    
    switch (message.status) {
      case MessageStatus.Pending:
        return <div className="w-4 h-4 border-2 border-gray-400 border-t-transparent rounded-full animate-spin" />;
      case MessageStatus.Sent:
        return <div className="w-4 h-4 text-gray-400">✓</div>;
      case MessageStatus.Delivered:
        return <div className="w-4 h-4 text-blue-400">✓✓</div>;
      case MessageStatus.Read:
        return <div className="w-4 h-4 text-green-400">✓✓</div>;
      case MessageStatus.Failed:
        return <div className="w-4 h-4 text-red-400">✗</div>;
      default:
        return null;
    }
  };
  
  const isAgent = participant?.type === ParticipantType.AGENT;
  const messageContent = getMessageContent();
  
  return (
    <div className={`flex ${isCurrentUser ? 'justify-end' : 'justify-start'}`}>
      <div className={`max-w-xs lg:max-w-md ${isCurrentUser ? 'order-2' : 'order-1'}`}>
        {/* Timestamp */}
        {showTimestamp && (
          <div className="text-xs text-gray-500 text-center mb-2">
            {format(new Date(message.created_at), 'MMM d, yyyy h:mm a')}
          </div>
        )}
        
        {/* Message bubble */}
        <div
          className={`relative px-4 py-2 rounded-lg ${
            isCurrentUser
              ? 'bg-blue-600 text-white'
              : isAgent
              ? 'bg-purple-100 text-purple-900 border border-purple-200'
              : 'bg-gray-100 text-gray-900'
          } ${
            isFirstInGroup && isLastInGroup
              ? 'rounded-lg'
              : isFirstInGroup
              ? isCurrentUser
                ? 'rounded-br-sm'
                : 'rounded-bl-sm'
              : isLastInGroup
              ? isCurrentUser
                ? 'rounded-tr-sm'
                : 'rounded-tl-sm'
              : isCurrentUser
              ? 'rounded-r-sm'
              : 'rounded-l-sm'
          }`}
          onMouseEnter={() => setShowActions(true)}
          onMouseLeave={() => setShowActions(false)}
        >
          {/* Sender name (for non-current users and first in group) */}
          {!isCurrentUser && isFirstInGroup && (
            <div className={`text-xs font-medium mb-1 ${
              isAgent ? 'text-purple-600' : 'text-gray-600'
            }`}>
              {participant?.name || 'Unknown'}
              {isAgent && ' (Agent)'}
            </div>
          )}
          
          {/* Message content */}
          <div className="break-words">
            {messageContent}
          </div>
          
          {/* Optimistic message indicator */}
          {message.isOptimistic && (
            <div className="text-xs opacity-75 mt-1">
              Sending...
            </div>
          )}
          
          {/* Message actions */}
          {showActions && (
            <div className={`absolute ${
              isCurrentUser ? 'left-0 -translate-x-full' : 'right-0 translate-x-full'
            } top-0 flex items-center space-x-1 bg-white shadow-lg rounded-lg p-1 z-10`}>
              <button
                className="p-1 hover:bg-gray-100 rounded text-gray-500 hover:text-gray-700"
                title="React"
              >
                😊
              </button>
              <button
                className="p-1 hover:bg-gray-100 rounded text-gray-500 hover:text-gray-700"
                title="Reply"
              >
                ↩️
              </button>
              {isCurrentUser && (
                <button
                  className="p-1 hover:bg-gray-100 rounded text-gray-500 hover:text-gray-700"
                  title="Delete"
                >
                  🗑️
                </button>
              )}
            </div>
          )}
        </div>
        
        {/* Status indicator */}
        {isCurrentUser && isLastInGroup && (
          <div className="flex justify-end mt-1">
            {getStatusIcon()}
          </div>
        )}
        
        {/* Reactions */}
        {message.reactions && Object.keys(message.reactions).length > 0 && (
          <div className="flex flex-wrap gap-1 mt-1">
            {Object.entries(message.reactions).map(([emoji, userIds]) => (
              <button
                key={emoji}
                className="text-xs bg-gray-100 hover:bg-gray-200 rounded-full px-2 py-1 flex items-center space-x-1"
                title={`${userIds.length} reaction${userIds.length > 1 ? 's' : ''}`}
              >
                <span>{emoji}</span>
                <span className="text-gray-600">{userIds.length}</span>
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}; 
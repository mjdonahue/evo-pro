import React from 'react';
import { useParticipantStore } from '../../stores/participantStore';
import type { Uuid } from '../../lib/api/types';

interface TypingIndicatorProps {
  typingUsers: Uuid[];
  currentUserId: Uuid;
}

export const TypingIndicator: React.FC<TypingIndicatorProps> = ({
  typingUsers,
  currentUserId
}) => {
  const getDisplayName = useParticipantStore(state => state.getDisplayName);
  
  // Filter out current user from typing users
  const otherTypingUsers = typingUsers.filter(id => id !== currentUserId);
  
  if (otherTypingUsers.length === 0) {
    return null;
  }
  
  const getTypingText = () => {
    if (otherTypingUsers.length === 1) {
      return `${getDisplayName(otherTypingUsers[0])} is typing...`;
    } else if (otherTypingUsers.length === 2) {
      return `${getDisplayName(otherTypingUsers[0])} and ${getDisplayName(otherTypingUsers[1])} are typing...`;
    } else {
      return `${getDisplayName(otherTypingUsers[0])} and ${otherTypingUsers.length - 1} others are typing...`;
    }
  };
  
  return (
    <div className="flex justify-start mb-4">
      <div className="bg-gray-100 rounded-lg px-4 py-2 max-w-xs">
        <div className="flex items-center space-x-2">
          <div className="flex space-x-1">
            <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '0ms' }} />
            <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '150ms' }} />
            <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '300ms' }} />
          </div>
          <span className="text-sm text-gray-600">
            {getTypingText()}
          </span>
        </div>
      </div>
    </div>
  );
}; 
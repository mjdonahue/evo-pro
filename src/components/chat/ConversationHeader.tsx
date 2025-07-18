import React from 'react';
import { useParticipantStore, ParticipantType } from '../../stores/participantStore';
import type { ConversationInfo, ConversationParticipant } from '../../stores/conversationStore';

interface ConversationHeaderProps {
  conversation: ConversationInfo;
  isAgentConversation: boolean;
  agentParticipant?: ConversationParticipant;
}

export const ConversationHeader: React.FC<ConversationHeaderProps> = ({
  conversation,
  isAgentConversation,
  agentParticipant
}) => {
  const getParticipant = useParticipantStore(state => state.getParticipant);
  const currentUser = useParticipantStore(state => state.currentUser);
  
  const getConversationTitle = () => {
    if (conversation.title) {
      return conversation.title;
    }
    
    if (isAgentConversation && agentParticipant) {
      const agent = getParticipant(agentParticipant.id);
      return agent?.name || 'Agent';
    }
    
    // For direct conversations, show the other participant's name
    const otherParticipant = conversation.participants.find(p => p.id !== currentUser?.id);
    if (otherParticipant) {
      const participant = getParticipant(otherParticipant.id);
      return participant?.name || 'Unknown';
    }
    
    return 'Conversation';
  };
  
  const getStatusText = () => {
    if (isAgentConversation) {
      return 'AI Assistant';
    }
    
    const otherParticipant = conversation.participants.find(p => p.id !== currentUser?.id);
    if (otherParticipant) {
      const participant = getParticipant(otherParticipant.id);
      return participant?.status === 'online' ? 'Online' : 'Offline';
    }
    
    return `${conversation.participants.length} participants`;
  };
  
  return (
    <div className="flex items-center justify-between p-4 border-b border-gray-200 bg-white">
      <div className="flex items-center space-x-3">
        {/* Avatar */}
        <div className={`w-10 h-10 rounded-full flex items-center justify-center text-white font-medium ${
          isAgentConversation ? 'bg-purple-500' : 'bg-blue-500'
        }`}>
          {isAgentConversation ? '🤖' : getConversationTitle().charAt(0).toUpperCase()}
        </div>
        
        {/* Title and status */}
        <div>
          <h2 className="font-semibold text-gray-900">
            {getConversationTitle()}
          </h2>
          <p className={`text-sm ${
            isAgentConversation ? 'text-purple-600' : 'text-gray-500'
          }`}>
            {getStatusText()}
          </p>
        </div>
      </div>
      
      {/* Actions */}
      <div className="flex items-center space-x-2">
        {isAgentConversation && (
          <button
            className="p-2 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-lg"
            title="Agent settings"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </button>
        )}
        
        <button
          className="p-2 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-lg"
          title="More options"
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z" />
          </svg>
        </button>
      </div>
    </div>
  );
}; 
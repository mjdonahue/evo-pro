import React from 'react';
import { ParticipantType, type ParticipantInfo } from '../../stores/participantStore';
import type { StreamingMessage } from '../../stores/messageStore';

interface StreamingIndicatorProps {
  streamingMessage: StreamingMessage;
  participant: ParticipantInfo | null;
}

export const StreamingIndicator: React.FC<StreamingIndicatorProps> = ({
  streamingMessage,
  participant
}) => {
  const isAgent = participant?.type === ParticipantType.AGENT;
  
  return (
    <div className="flex justify-start mb-4">
      <div className="max-w-xs lg:max-w-md">
        {/* Streaming message bubble */}
        <div className={`relative px-4 py-2 rounded-lg ${
          isAgent 
            ? 'bg-purple-100 text-purple-900 border border-purple-200' 
            : 'bg-gray-100 text-gray-900'
        }`}>
          {/* Sender name */}
          <div className={`text-xs font-medium mb-1 ${
            isAgent ? 'text-purple-600' : 'text-gray-600'
          }`}>
            {participant?.name || 'Unknown'}
            {isAgent && ' (Agent)'}
          </div>
          
          {/* Streaming content */}
          <div className="break-words">
            {streamingMessage.content}
            {streamingMessage.isStreaming && (
              <span className="inline-block w-2 h-4 bg-current animate-pulse ml-1" />
            )}
          </div>
          
          {/* Streaming status */}
          {streamingMessage.isStreaming && (
            <div className="flex items-center space-x-2 mt-2 text-xs text-gray-500">
              <div className="flex space-x-1">
                <div className="w-1 h-1 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '0ms' }} />
                <div className="w-1 h-1 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '150ms' }} />
                <div className="w-1 h-1 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '300ms' }} />
              </div>
              <span>Generating response...</span>
            </div>
          )}
          
          {/* Error state */}
          {streamingMessage.error && (
            <div className="text-xs text-red-500 mt-2">
              Error: {streamingMessage.error}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}; 
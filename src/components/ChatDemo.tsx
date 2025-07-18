/** @jsxImportSource react */
import React, { useEffect, useState } from 'react';
import { ChatInterface } from './chat/ChatInterface';
import { useParticipantStore, ParticipantType } from '../stores/participantStore';
import { useConversationStore } from '../stores/conversationStore';
import { useMessageStore } from '../stores/messageStore';
import { ConversationType, ConversationStatus, UserStatus, AgentStatus, MessageType, MessageStatus } from '../lib/api/types';
import type { User, Agent, Conversation, Message, Uuid } from '../lib/api/types';
import { faker } from '@faker-js/faker';

export const ChatDemo: React.FC = () => {
  const [activeConversationId, setActiveConversationId] = useState<Uuid | null>(null);
  const [conversationType, setConversationType] = useState<'user' | 'agent'>('user');
  
  // Store hooks - no need for useCallback as store functions are stable
  const setCurrentUser = useParticipantStore(state => state.setCurrentUser);
  const addUser = useParticipantStore(state => state.addUser);
  const addAgent = useParticipantStore(state => state.addAgent);
  const currentUser = useParticipantStore(state => state.currentUser);
  const participants = useParticipantStore(state => state.participants);
  
  const addConversation = useConversationStore(state => state.addConversation);
  const addParticipant = useConversationStore(state => state.addParticipant);
  const conversations = useConversationStore(state => state.conversations);
  
  const addMessage = useMessageStore(state => state.addMessage);
  
  // Initialize demo data
  useEffect(() => {
    // Create current user
    const user: User = {
      id: faker.string.uuid() as unknown as Uuid,
      participant_id: faker.string.uuid() as unknown as Uuid,
      email: 'demo@example.com',
      username: 'demo_user',
      display_name: 'Demo User',
      first_name: 'Demo',
      last_name: 'User',
      workspace_id: faker.string.uuid() as unknown as Uuid,
      status: UserStatus.Active,
      email_verified: true,
      phone_verified: false,
      last_seen: new Date().toISOString(),
      roles: JSON.stringify(['user']),
      preferences: JSON.stringify({}),
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    };
    
    setCurrentUser(user);
    
    // Create another user for user-to-user chat
    const otherUser: User = {
      id: faker.string.uuid() as unknown as Uuid,
      participant_id: faker.string.uuid() as unknown as Uuid,
      email: 'other@example.com',
      username: 'other_user',
      display_name: 'Other User',
      first_name: 'Other',
      last_name: 'User',
      workspace_id: user.workspace_id,
      status: UserStatus.Active,
      email_verified: true,
      phone_verified: false,
      last_seen: new Date().toISOString(),
      roles: JSON.stringify(['user']),
      preferences: JSON.stringify({}),
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    };
    
    addUser(otherUser);
    
    // Create an agent
    const agent: Agent = {
      id: faker.string.uuid() as unknown as Uuid,
      workspace_id: user.workspace_id,
      name: 'Assistant AI',
      description: 'A helpful AI assistant',
      agent_type: 'Assistant' as any,
      status: AgentStatus.Active,
      capabilities: JSON.stringify(['text_generation', 'question_answering']),
      tools: JSON.stringify([]),
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    };
    
    addAgent(agent);
    
    // Create user-to-user conversation
    const userConversation: Conversation = {
      id: faker.string.uuid() as unknown as Uuid,
      title: 'Chat with Other User',
      type: ConversationType.Private,
      status: ConversationStatus.Active,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    };
    
    addConversation(userConversation);
    addParticipant(userConversation.id, {
      id: user.id,
      type: ParticipantType.USER,
      role: 'member',
      joinedAt: new Date().toISOString(),
      isActive: true
    });
    addParticipant(userConversation.id, {
      id: otherUser.id,
      type: ParticipantType.USER,
      role: 'member',
      joinedAt: new Date().toISOString(),
      isActive: true
    });
    
    // Create user-to-agent conversation
    const agentConversation: Conversation = {
      id: faker.string.uuid() as unknown as Uuid,
      title: 'Chat with Assistant AI',
      type: ConversationType.Private,
      status: ConversationStatus.Active,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    };
    
    addConversation(agentConversation);
    addParticipant(agentConversation.id, {
      id: user.id,
      type: ParticipantType.USER,
      role: 'member',
      joinedAt: new Date().toISOString(),
      isActive: true
    });
    addParticipant(agentConversation.id, {
      id: agent.id,
      type: ParticipantType.AGENT,
      role: 'member',
      joinedAt: new Date().toISOString(),
      isActive: true
    });
    
    // Add some sample messages
    const sampleMessage: Message = {
      id: faker.string.uuid() as unknown as Uuid,
      conversation_id: userConversation.id,
      workspace_id: user.workspace_id,
      sender_id: otherUser.id,
      type: MessageType.Text,
      content: JSON.stringify({ text: 'Hello! How are you doing today?' }),
      status: MessageStatus.Read,
      created_at: new Date(Date.now() - 60000).toISOString(),
      updated_at: new Date(Date.now() - 60000).toISOString()
    };
    
    addMessage(sampleMessage);
    
    // Set default active conversation
    setActiveConversationId(userConversation.id);
  }, [setCurrentUser, addUser, addAgent, addConversation, addParticipant, addMessage]);
  
  const getConversationsByType = (type: 'user' | 'agent') => {
    return Array.from(conversations.values()).filter(conv => {
      const hasAgent = conv.participants.some(p => p.type === ParticipantType.AGENT);
      return type === 'agent' ? hasAgent : !hasAgent;
    });
  };
  
  const switchConversationType = (type: 'user' | 'agent') => {
    setConversationType(type);
    const conversations = getConversationsByType(type);
    if (conversations.length > 0) {
      setActiveConversationId(conversations[0].id);
    }
  };
  
  const userConversations = getConversationsByType('user');
  const agentConversations = getConversationsByType('agent');
  
  return (
    <div className="h-screen flex bg-gray-100">
      {/* Sidebar */}
      <div className="w-80 bg-white border-r border-gray-200 flex flex-col">
        {/* Header */}
        <div className="p-4 border-b border-gray-200">
          <h1 className="text-xl font-semibold text-gray-900">Chat Demo</h1>
          <p className="text-sm text-gray-600">Unified messaging system</p>
        </div>
        
        {/* Conversation Type Selector */}
        <div className="p-4 border-b border-gray-200">
          <div className="flex bg-gray-100 rounded-lg p-1">
            <button
              onClick={() => switchConversationType('user')}
              className={`flex-1 py-2 px-3 rounded-md text-sm font-medium transition-colors ${
                conversationType === 'user'
                  ? 'bg-white text-blue-600 shadow-sm'
                  : 'text-gray-600 hover:text-gray-900'
              }`}
            >
              👥 Users
            </button>
            <button
              onClick={() => switchConversationType('agent')}
              className={`flex-1 py-2 px-3 rounded-md text-sm font-medium transition-colors ${
                conversationType === 'agent'
                  ? 'bg-white text-purple-600 shadow-sm'
                  : 'text-gray-600 hover:text-gray-900'
              }`}
            >
              🤖 Agents
            </button>
          </div>
        </div>
        
        {/* Conversations List */}
        <div className="flex-1 overflow-y-auto">
          {conversationType === 'user' && (
            <div className="p-4">
              <h3 className="text-sm font-medium text-gray-500 mb-3">User Conversations</h3>
              {userConversations.map(conv => (
                <button
                  key={conv.id.toString()}
                  onClick={() => setActiveConversationId(conv.id)}
                  className={`w-full text-left p-3 rounded-lg mb-2 transition-colors ${
                    activeConversationId === conv.id
                      ? 'bg-blue-50 border border-blue-200'
                      : 'hover:bg-gray-50'
                  }`}
                >
                  <div className="flex items-center space-x-3">
                    <div className="w-8 h-8 bg-blue-500 rounded-full flex items-center justify-center text-white text-sm">
                      👥
                    </div>
                    <div className="flex-1">
                      <div className="font-medium text-gray-900">
                        {conv.title || 'User Chat'}
                      </div>
                      <div className="text-sm text-gray-500">
                        {conv.participants.length} participants
                      </div>
                    </div>
                  </div>
                </button>
              ))}
            </div>
          )}
          
          {conversationType === 'agent' && (
            <div className="p-4">
              <h3 className="text-sm font-medium text-gray-500 mb-3">Agent Conversations</h3>
              {agentConversations.map(conv => (
                <button
                  key={conv.id.toString()}
                  onClick={() => setActiveConversationId(conv.id)}
                  className={`w-full text-left p-3 rounded-lg mb-2 transition-colors ${
                    activeConversationId === conv.id
                      ? 'bg-purple-50 border border-purple-200'
                      : 'hover:bg-gray-50'
                  }`}
                >
                  <div className="flex items-center space-x-3">
                    <div className="w-8 h-8 bg-purple-500 rounded-full flex items-center justify-center text-white text-sm">
                      🤖
                    </div>
                    <div className="flex-1">
                      <div className="font-medium text-gray-900">
                        {conv.title || 'Agent Chat'}
                      </div>
                      <div className="text-sm text-purple-600">
                        AI Assistant
                      </div>
                    </div>
                  </div>
                </button>
              ))}
            </div>
          )}
        </div>
        
        {/* User Info */}
        {currentUser && (
          <div className="p-4 border-t border-gray-200">
            <div className="flex items-center space-x-3">
              <div className="w-8 h-8 bg-green-500 rounded-full flex items-center justify-center text-white text-sm">
                {currentUser.display_name.charAt(0)}
              </div>
              <div>
                <div className="font-medium text-gray-900">{currentUser.display_name}</div>
                <div className="text-sm text-green-600">Online</div>
              </div>
            </div>
          </div>
        )}
      </div>
      
      {/* Chat Interface */}
      <div className="flex-1 flex flex-col">
        {activeConversationId ? (
          <ChatInterface 
            conversationId={activeConversationId}
            className="flex-1"
          />
        ) : (
          <div className="flex-1 flex items-center justify-center text-gray-500">
            <div className="text-center">
              <div className="text-lg mb-2">Select a conversation</div>
              <div className="text-sm">
                Choose a conversation from the sidebar to start chatting
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}; 
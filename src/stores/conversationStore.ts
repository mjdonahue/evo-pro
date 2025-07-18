import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import { immer } from 'zustand/middleware/immer';
import { enableMapSet } from 'immer';

enableMapSet();
import type { Conversation, Uuid } from '../lib/api/types';
import { ConversationType, ConversationStatus } from '../lib/api/types';
import { ParticipantType } from './participantStore';

export interface ConversationParticipant {
  id: Uuid;
  type: ParticipantType;
  role: 'owner' | 'admin' | 'member' | 'observer';
  joinedAt: string;
  isActive: boolean;
}

export interface ConversationInfo extends Conversation {
  participants: ConversationParticipant[];
  unreadCount: number;
  lastMessageAt?: string;
  lastMessagePreview?: string;
  isTyping: boolean;
  typingUsers: Uuid[];
}

export interface ConversationState {
  // Core data
  conversations: Map<Uuid, ConversationInfo>;
  activeConversationId: Uuid | null;
  
  // UI state
  isLoading: boolean;
  searchQuery: string;
  selectedType: ConversationType | 'all';
  
  // Actions
  setConversations: (conversations: Conversation[]) => void;
  addConversation: (conversation: Conversation) => void;
  updateConversation: (id: Uuid, updates: Partial<ConversationInfo>) => void;
  deleteConversation: (id: Uuid) => void;
  setActiveConversation: (id: Uuid | null) => void;
  
  // Participants
  addParticipant: (conversationId: Uuid, participant: ConversationParticipant) => void;
  removeParticipant: (conversationId: Uuid, participantId: Uuid) => void;
  updateParticipant: (conversationId: Uuid, participantId: Uuid, updates: Partial<ConversationParticipant>) => void;
  
  // Typing indicators
  setTyping: (conversationId: Uuid, participantId: Uuid, isTyping: boolean) => void;
  
  // Unread counts
  incrementUnreadCount: (conversationId: Uuid) => void;
  resetUnreadCount: (conversationId: Uuid) => void;
  resetActiveConversationUnreadCount: () => void;
  
  // Search and filtering
  setSearchQuery: (query: string) => void;
  setSelectedType: (type: ConversationType | 'all') => void;
  
  // Getters
  getConversation: (id: Uuid) => ConversationInfo | null;
  getActiveConversation: () => ConversationInfo | null;
  getFilteredConversations: () => ConversationInfo[];
  getTotalUnreadCount: () => number;
  
  // Conversation creation helpers
  createDirectConversation: (participantId: Uuid) => Partial<ConversationInfo>;
  createGroupConversation: (title: string, participantIds: Uuid[]) => Partial<ConversationInfo>;
  createAgentConversation: (agentId: Uuid) => Partial<ConversationInfo>;
}

export const useConversationStore = create<ConversationState>()(
  subscribeWithSelector(
    immer((set, get) => ({
      // Initial state
      conversations: new Map(),
      activeConversationId: null,
      isLoading: false,
      searchQuery: '',
      selectedType: 'all',
      
      // Actions
      setConversations: (conversations) => set((state) => {
        state.conversations.clear();
        conversations.forEach(conv => {
          const conversationInfo: ConversationInfo = {
            ...conv,
            participants: [],
            unreadCount: 0,
            isTyping: false,
            typingUsers: []
          };
          state.conversations.set(conv.id, conversationInfo);
        });
      }),
      
      addConversation: (conversation) => set((state) => {
        const conversationInfo: ConversationInfo = {
          ...conversation,
          participants: [],
          unreadCount: 0,
          isTyping: false,
          typingUsers: []
        };
        state.conversations.set(conversation.id, conversationInfo);
      }),
      
      updateConversation: (id, updates) => set((state) => {
        const conversation = state.conversations.get(id);
        if (conversation) {
          state.conversations.set(id, { ...conversation, ...updates });
        }
      }),
      
      deleteConversation: (id) => set((state) => {
        state.conversations.delete(id);
        if (state.activeConversationId === id) {
          state.activeConversationId = null;
        }
      }),
      
      setActiveConversation: (id) => set((state) => {
        state.activeConversationId = id;
      }),
      
      // Participants
      addParticipant: (conversationId, participant) => set((state) => {
        const conversation = state.conversations.get(conversationId);
        if (conversation) {
          const updatedParticipants = [...conversation.participants, participant];
          state.conversations.set(conversationId, { 
            ...conversation, 
            participants: updatedParticipants 
          });
        }
      }),
      
      removeParticipant: (conversationId, participantId) => set((state) => {
        const conversation = state.conversations.get(conversationId);
        if (conversation) {
          const updatedParticipants = conversation.participants.filter(p => p.id !== participantId);
          state.conversations.set(conversationId, { 
            ...conversation, 
            participants: updatedParticipants 
          });
        }
      }),
      
      updateParticipant: (conversationId, participantId, updates) => set((state) => {
        const conversation = state.conversations.get(conversationId);
        if (conversation) {
          const updatedParticipants = conversation.participants.map(p => 
            p.id === participantId ? { ...p, ...updates } : p
          );
          state.conversations.set(conversationId, { 
            ...conversation, 
            participants: updatedParticipants 
          });
        }
      }),
      
      // Typing indicators
      setTyping: (conversationId, participantId, isTyping) => set((state) => {
        const conversation = state.conversations.get(conversationId);
        if (conversation) {
          let typingUsers = [...conversation.typingUsers];
          
          if (isTyping && !typingUsers.includes(participantId)) {
            typingUsers.push(participantId);
          } else if (!isTyping) {
            typingUsers = typingUsers.filter(id => id !== participantId);
          }
          
          state.conversations.set(conversationId, {
            ...conversation,
            typingUsers,
            isTyping: typingUsers.length > 0
          });
        }
      }),
      
      // Unread counts
      incrementUnreadCount: (conversationId) => set((state) => {
        const conversation = state.conversations.get(conversationId);
        if (conversation && state.activeConversationId !== conversationId) {
          state.conversations.set(conversationId, {
            ...conversation,
            unreadCount: conversation.unreadCount + 1
          });
        }
      }),
      
      resetUnreadCount: (conversationId) => set((state) => {
        const conversation = state.conversations.get(conversationId);
        if (conversation) {
          state.conversations.set(conversationId, {
            ...conversation,
            unreadCount: 0
          });
        }
      }),
      
      resetActiveConversationUnreadCount: () => set((state) => {
        if (state.activeConversationId) {
          const conversation = state.conversations.get(state.activeConversationId);
          if (conversation) {
            state.conversations.set(state.activeConversationId, {
              ...conversation,
              unreadCount: 0
            });
          }
        }
      }),
      
      // Search and filtering
      setSearchQuery: (query) => set((state) => {
        state.searchQuery = query;
      }),
      
      setSelectedType: (type) => set((state) => {
        state.selectedType = type;
      }),
      
      // Getters
      getConversation: (id) => {
        return get().conversations.get(id) || null;
      },
      
      getActiveConversation: () => {
        const { activeConversationId, conversations } = get();
        return activeConversationId ? conversations.get(activeConversationId) || null : null;
      },
      
      getFilteredConversations: () => {
        const { conversations, searchQuery, selectedType } = get();
        let filtered = Array.from(conversations.values());
        
        // Filter by type
        if (selectedType !== 'all') {
          filtered = filtered.filter(conv => conv.type === selectedType);
        }
        
        // Filter by search query
        if (searchQuery.trim()) {
          filtered = filtered.filter(conv => 
            conv.title?.toLowerCase().includes(searchQuery.toLowerCase())
          );
        }
        
        // Sort by last message time
        return filtered.sort((a, b) => {
          const aTime = a.lastMessageAt || a.updated_at;
          const bTime = b.lastMessageAt || b.updated_at;
          return new Date(bTime).getTime() - new Date(aTime).getTime();
        });
      },
      
      getTotalUnreadCount: () => {
        return Array.from(get().conversations.values())
          .reduce((total, conv) => total + conv.unreadCount, 0);
      },
      
      // Conversation creation helpers
      createDirectConversation: (participantId) => ({
        type: ConversationType.Private,
        status: ConversationStatus.Active,
        participants: [
          {
            id: participantId,
            type: ParticipantType.USER, // Will be updated based on actual participant
            role: 'member' as const,
            joinedAt: new Date().toISOString(),
            isActive: true
          }
        ],
        unreadCount: 0,
        isTyping: false,
        typingUsers: []
      }),
      
      createGroupConversation: (title, participantIds) => ({
        title,
        type: ConversationType.Group,
        status: ConversationStatus.Active,
        participants: participantIds.map(id => ({
          id,
          type: ParticipantType.USER,
          role: 'member' as const,
          joinedAt: new Date().toISOString(),
          isActive: true
        })),
        unreadCount: 0,
        isTyping: false,
        typingUsers: []
      }),
      
      createAgentConversation: (agentId) => ({
        type: ConversationType.Private,
        status: ConversationStatus.Active,
        participants: [
          {
            id: agentId,
            type: ParticipantType.AGENT,
            role: 'member' as const,
            joinedAt: new Date().toISOString(),
            isActive: true
          }
        ],
        unreadCount: 0,
        isTyping: false,
        typingUsers: []
      })
    }))
  )
); 
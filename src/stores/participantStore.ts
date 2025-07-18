import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import { immer } from 'zustand/middleware/immer';
import type { User, Agent, Uuid } from '../lib/api/types';

export enum ParticipantType {
  USER = 'user',
  AGENT = 'agent'
}

export interface ParticipantInfo {
  id: Uuid;
  type: ParticipantType;
  name: string;
  avatar?: string;
  status: 'online' | 'offline' | 'busy' | 'away';
  isTyping?: boolean;
  lastSeen?: string;
  metadata?: Record<string, any>;
}

export interface ParticipantState {
  // Core data
  users: Map<Uuid, User>;
  agents: Map<Uuid, Agent>;
  participants: Map<Uuid, ParticipantInfo>;
  currentUser: User | null;
  
  // UI state
  selectedParticipant: Uuid | null;
  typingIndicators: Map<Uuid, boolean>;
  
  // Actions
  setCurrentUser: (user: User) => void;
  addUser: (user: User) => void;
  addAgent: (agent: Agent) => void;
  updateParticipant: (id: Uuid, updates: Partial<ParticipantInfo>) => void;
  setSelectedParticipant: (id: Uuid | null) => void;
  setTyping: (participantId: Uuid, isTyping: boolean) => void;
  getParticipant: (id: Uuid) => ParticipantInfo | null;
  getParticipantsByType: (type: ParticipantType) => ParticipantInfo[];
  searchParticipants: (query: string) => ParticipantInfo[];
  
  // Utilities
  isAgent: (id: Uuid) => boolean;
  isUser: (id: Uuid) => boolean;
  getDisplayName: (id: Uuid) => string;
  getAvatar: (id: Uuid) => string | undefined;
}

export const useParticipantStore = create<ParticipantState>()(
  subscribeWithSelector(
    immer((set, get) => ({
      // Initial state
      users: new Map(),
      agents: new Map(),
      participants: new Map(),
      currentUser: null,
      selectedParticipant: null,
      typingIndicators: new Map(),
      
      // Actions
      setCurrentUser: (user) => set((state) => {
        state.currentUser = user;
        state.users.set(user.id, user);
        
        // Create participant info
        const participantInfo: ParticipantInfo = {
          id: user.id,
          type: ParticipantType.USER,
          name: user.display_name,
          avatar: user.avatar,
          status: user.status === 'Active' ? 'online' : 'offline',
          lastSeen: user.last_seen,
          metadata: user.metadata ? JSON.parse(user.metadata) : undefined
        };
        
        state.participants.set(user.id, participantInfo);
      }),
      
      addUser: (user) => set((state) => {
        state.users.set(user.id, user);
        
        const participantInfo: ParticipantInfo = {
          id: user.id,
          type: ParticipantType.USER,
          name: user.display_name,
          avatar: user.avatar,
          status: user.status === 'Active' ? 'online' : 'offline',
          lastSeen: user.last_seen,
          metadata: user.metadata ? JSON.parse(user.metadata) : undefined
        };
        
        state.participants.set(user.id, participantInfo);
      }),
      
      addAgent: (agent) => set((state) => {
        state.agents.set(agent.id, agent);
        
        const participantInfo: ParticipantInfo = {
          id: agent.id,
          type: ParticipantType.AGENT,
          name: agent.name,
          avatar: undefined, // Agents might not have avatars
          status: agent.status === 'Active' ? 'online' : 'offline',
          metadata: agent.metadata ? JSON.parse(agent.metadata) : undefined
        };
        
        state.participants.set(agent.id, participantInfo);
      }),
      
      updateParticipant: (id, updates) => set((state) => {
        const participant = state.participants.get(id);
        if (participant) {
          state.participants.set(id, { ...participant, ...updates });
        }
      }),
      
      setSelectedParticipant: (id) => set((state) => {
        state.selectedParticipant = id;
      }),
      
      setTyping: (participantId, isTyping) => set((state) => {
        state.typingIndicators.set(participantId, isTyping);
        
        // Update participant info
        const participant = state.participants.get(participantId);
        if (participant) {
          state.participants.set(participantId, { ...participant, isTyping });
        }
      }),
      
      // Getters
      getParticipant: (id) => {
        return get().participants.get(id) || null;
      },
      
      getParticipantsByType: (type) => {
        return Array.from(get().participants.values()).filter(p => p.type === type);
      },
      
      searchParticipants: (query) => {
        const participants = Array.from(get().participants.values());
        return participants.filter(p => 
          p.name.toLowerCase().includes(query.toLowerCase())
        );
      },
      
      // Utilities
      isAgent: (id) => {
        return get().agents.has(id);
      },
      
      isUser: (id) => {
        return get().users.has(id);
      },
      
      getDisplayName: (id) => {
        const participant = get().participants.get(id);
        return participant?.name || 'Unknown';
      },
      
      getAvatar: (id) => {
        const participant = get().participants.get(id);
        return participant?.avatar;
      }
    }))
  )
); 
import React, { useState } from 'react';
import { Sidebar } from '../navigation/Sidebar';
import { ConversationPanel } from '../navigation/ConversationPanel';
import { MainContent } from '../content/MainContent';
export function MainLayout() {
  const [isConversationPanelOpen, setIsConversationPanelOpen] = useState(true);
  const [activeSection, setActiveSection] = useState('home');
  const toggleConversationPanel = () => {
    setIsConversationPanelOpen(!isConversationPanelOpen);
  };
  return <div className="flex h-full">
      <Sidebar activeSection={activeSection} setActiveSection={setActiveSection} toggleConversationPanel={toggleConversationPanel} />
      {isConversationPanelOpen && activeSection !== 'home' ? <ConversationPanel isOpen={isConversationPanelOpen} onClose={() => setIsConversationPanelOpen(false)} activeSection={activeSection} setActiveSection={setActiveSection}>
          <MainContent activeSection={activeSection} setActiveSection={setActiveSection} />
        </ConversationPanel> : <MainContent activeSection={activeSection} setActiveSection={setActiveSection} />}
    </div>;
}
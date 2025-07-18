import React, { useState } from 'react'
import { Routes, Route, useLocation, Navigate } from 'react-router-dom'
import { Sidebar } from '../navigation/Sidebar'
import { ConversationPanel } from '../navigation/ConversationPanel'
import { MainContent } from '../content/MainContent'

export function MainLayout() {
  // State for the slide-out panel content
  const [activePanelSection, setActivePanelSection] = useState('communications')
  // State for slide-out panel visibility
  const [isConversationPanelOpen, setIsConversationPanelOpen] = useState(true)
  // Get current route
  const location = useLocation()
  const currentPath = location.pathname === '/' ? '/dashboard' : location.pathname
  const activeSection = currentPath.substring(1) // Remove leading slash
  
  const toggleConversationPanel = () => {
    setIsConversationPanelOpen(!isConversationPanelOpen)
  }

  const renderRoutes = () => (
    <Routes>
      <Route path="/" element={<Navigate to="/dashboard" replace />} />
      <Route path="/dashboard" element={<MainContent activeSection="dashboard" />} />
      <Route path="/communications" element={<MainContent activeSection="communications" />} />
      <Route path="/tasks" element={<MainContent activeSection="tasks" />} />
      <Route path="/calendar" element={<MainContent activeSection="calendar" />} />
      <Route path="/documents" element={<MainContent activeSection="documents" />} />
      <Route path="/settings" element={<MainContent activeSection="settings" />} />
      <Route path="*" element={<Navigate to="/dashboard" replace />} />
    </Routes>
  )

  return (
    <div className="flex h-full">
      <Sidebar
        activeSection={activeSection}
        toggleConversationPanel={toggleConversationPanel}
        isPanelOpen={isConversationPanelOpen}
      />
      {isConversationPanelOpen ? (
        <ConversationPanel
          isOpen={isConversationPanelOpen}
          onClose={() => setIsConversationPanelOpen(false)}
          activePanelSection={activePanelSection}
          setActivePanelSection={setActivePanelSection}
          activeMainSection={activeSection}
        >
          {renderRoutes()}
        </ConversationPanel>
      ) : (
        renderRoutes()
      )}
    </div>
  )
}

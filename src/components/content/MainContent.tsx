import React from 'react'
import {
  FileText,
  CheckSquare,
  Calendar,
  Bot,
  Users,
  Folder,
} from 'lucide-react'
import {
  ResizablePanel,
  ResizableHandle,
  ResizablePanelGroup,
} from '../ui/resizable'
import { CommunicationsContent } from './CommunicationsContent'
import { TasksContent } from './TasksContent'
import { CalendarContent } from './CalendarContent'
import { DocumentsContent } from './DocumentsContent'
import { SettingsContent } from './SettingsContent'
import { ChatInput } from './ChatInput'
import { Dashboard } from '../Dashboard'

import { TopNavigation } from '../navigation/TopNavigation'
import { Breadcrumbs } from '../navigation/Breadcrumbs'
import { TaskDashboard } from '../TaskDashboard'
interface MainContentProps {
  activeSection: string
}
export function MainContent({ activeSection }: MainContentProps) {
  return (
    <div className="flex-1 h-full flex flex-col">
      {/* Top Navigation */}
      <TopNavigation />
      {/* Main content area with resizable panels */}
      <div className="flex-1">
        <ResizablePanelGroup direction="vertical" className="h-full">
          <ResizablePanel defaultSize={75} className="overflow-y-auto">
            <div className="p-6">
              <Breadcrumbs />
              {activeSection === 'communications' && <CommunicationsContent />}
              {activeSection === 'tasks' && <TaskDashboard />}
              {activeSection === 'calendar' && <CalendarContent />}
              {activeSection === 'documents' && <DocumentsContent />}
              {(activeSection === 'home' || activeSection === 'dashboard') && (
                <Dashboard />
              )}
              {activeSection === 'settings' && <SettingsContent />}
              {activeSection !== 'communications' &&
                activeSection !== 'home' &&
                activeSection !== 'dashboard' &&
                activeSection !== 'tasks' &&
                activeSection !== 'calendar' &&
                activeSection !== 'documents' &&
                activeSection !== 'settings' && (
                  <div className="flex items-center justify-center h-full text-muted-foreground">
                    {activeSection.charAt(0).toUpperCase() +
                      activeSection.slice(1)}{' '}
                    content will appear here
                  </div>
                )}
            </div>
          </ResizablePanel>
          <ResizableHandle />
          <ResizablePanel defaultSize={25} minSize={15} maxSize={50}>
            <ChatInput />
          </ResizablePanel>
        </ResizablePanelGroup>
      </div>
    </div>
  )
}

import React from 'react'
import { Link, useNavigate } from 'react-router-dom'
import {
  FileText,
  Bot,
  GitBranch,
  Settings,
  Sun,
  Moon,
  Home,
  MessageSquare,
  CheckSquare,
  Calendar,
  LayoutDashboard,
  ChevronRight,
  ChevronLeft,
  FolderOpen,
} from 'lucide-react'
import { useTheme } from '../theme/ThemeProvider'
import { OfflineIndicator } from '../ui/OfflineIndicator'
interface SidebarProps {
  activeSection: string
  toggleConversationPanel: () => void
  isPanelOpen: boolean
}
export function Sidebar({
  activeSection,
  toggleConversationPanel,
  isPanelOpen,
}: SidebarProps) {
  const { theme, setTheme } = useTheme()
  const navigate = useNavigate()
  const menuItems = [
    {
      id: 'dashboard',
      icon: <LayoutDashboard size={20} />,
      label: 'Dashboard',
      path: '/dashboard',
    },
    {
      id: 'communications',
      icon: <MessageSquare size={20} />,
      label: 'Communications',
      path: '/communications',
    },
    {
      id: 'tasks',
      icon: <CheckSquare size={20} />,
      label: 'Tasks',
      path: '/tasks',
    },
    {
      id: 'calendar',
      icon: <Calendar size={20} />,
      label: 'Calendar',
      path: '/calendar',
    },
    {
      id: 'documents',
      icon: <FolderOpen size={20} />,
      label: 'Documents',
      path: '/documents',
    },
    {
      id: 'chat',
      icon: <MessageSquare size={20} />,
      label: 'Chat',
      path: '/chat',
    },
  ]
  return (
    <div className="h-full w-16 bg-secondary flex flex-col items-center py-4 border-r border-border">
      {/* Home icon */}
      <Link
        to="/dashboard"
        className="relative w-10 h-10 rounded-md flex items-center justify-center cursor-pointer text-primary hover:bg-accent/50 transition-colors group mb-6"
      >
        <Home size={20} />
        <div className="absolute left-full ml-2 px-2 py-1 bg-popover text-popover-foreground text-sm rounded-md invisible opacity-0 group-hover:visible group-hover:opacity-100 transition-opacity whitespace-nowrap">
          Home
        </div>
      </Link>
      {/* Navigation items */}
      <div className="flex-1 flex flex-col items-center space-y-6">
        {menuItems.map((item) => (
          <Link
            key={item.id}
            to={item.path}
            className={`relative w-10 h-10 rounded-md flex items-center justify-center cursor-pointer transition-colors group ${activeSection === item.id ? 'bg-primary/10 text-primary' : 'text-muted-foreground hover:text-foreground hover:bg-accent/50'}`}
          >
            {item.icon}
            <div className="absolute left-full ml-2 px-2 py-1 bg-popover text-popover-foreground text-sm rounded-md invisible opacity-0 group-hover:visible group-hover:opacity-100 transition-opacity whitespace-nowrap">
              {item.label}
            </div>
          </Link>
        ))}
      </div>
      {/* Theme toggle and settings */}
      <div className="mt-auto mb-4 flex flex-col gap-4">
        {/* Offline status indicator */}
        <OfflineIndicator />

        {/* Chevron to toggle conversation panel */}
        <div
          className="relative w-10 h-10 rounded-md flex items-center justify-center cursor-pointer text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors group"
          onClick={toggleConversationPanel}
        >
          {isPanelOpen ? <ChevronLeft size={20} /> : <ChevronRight size={20} />}
          <div className="absolute left-full ml-2 px-2 py-1 bg-popover text-popover-foreground text-sm rounded-md invisible opacity-0 group-hover:visible group-hover:opacity-100 transition-opacity whitespace-nowrap">
            {isPanelOpen ? 'Close Workspace' : 'Open Workspace'}
          </div>
        </div>
        <div
          className="relative w-10 h-10 rounded-md flex items-center justify-center cursor-pointer text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors group"
          onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
        >
          {theme === 'dark' ? <Sun size={20} /> : <Moon size={20} />}
          <div className="absolute left-full ml-2 px-2 py-1 bg-popover text-popover-foreground text-sm rounded-md invisible opacity-0 group-hover:visible group-hover:opacity-100 transition-opacity whitespace-nowrap">
            {theme === 'dark' ? 'Light mode' : 'Dark mode'}
          </div>
        </div>
        <Link
          to="/settings"
          className={`relative w-10 h-10 rounded-md flex items-center justify-center cursor-pointer transition-colors group ${activeSection === 'settings' ? 'bg-primary/10 text-primary' : 'text-muted-foreground hover:text-foreground hover:bg-accent/50'}`}
        >
          <Settings size={20} />
          <div className="absolute left-full ml-2 px-2 py-1 bg-popover text-popover-foreground text-sm rounded-md invisible opacity-0 group-hover:visible group-hover:opacity-100 transition-opacity whitespace-nowrap">
            Settings
          </div>
        </Link>
      </div>
      {/* User profile 
      <div className="relative w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center cursor-pointer hover:bg-primary/20 transition-colors group">
        <img
          src="https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80"
          alt="User profile"
          className="w-8 h-8 rounded-full object-cover"
        />
        <div className="absolute left-full ml-2 px-2 py-1 bg-popover text-popover-foreground text-sm rounded-md invisible opacity-0 group-hover:visible group-hover:opacity-100 transition-opacity whitespace-nowrap">
          Your Profile
        </div>
      </div> */}
    </div>
  )
}

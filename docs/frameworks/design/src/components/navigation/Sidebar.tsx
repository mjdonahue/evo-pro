import React from 'react';
import { FileText, Bot, GitBranch, Settings, Sun, Moon, Home, MessageSquare, CheckSquare, Calendar, LayoutDashboard } from 'lucide-react';
import { useTheme } from '../theme/ThemeProvider';
interface SidebarProps {
  activeSection: string;
  setActiveSection: (section: string) => void;
  toggleConversationPanel: () => void;
}
export function Sidebar({
  activeSection,
  setActiveSection,
  toggleConversationPanel
}: SidebarProps) {
  const {
    theme,
    setTheme
  } = useTheme();
  const menuItems = [{
    id: 'home',
    icon: <LayoutDashboard size={20} />,
    label: 'Dashboard'
  }, {
    id: 'communications',
    icon: <MessageSquare size={20} />,
    label: 'Communications'
  }, {
    id: 'tasks',
    icon: <CheckSquare size={20} />,
    label: 'Tasks'
  }, {
    id: 'calendar',
    icon: <Calendar size={20} />,
    label: 'Calendar'
  }];
  return <div className="h-full w-16 bg-secondary flex flex-col items-center py-4 border-r border-border">
      {/* Home icon */}
      <div className="relative w-10 h-10 rounded-md flex items-center justify-center cursor-pointer text-primary hover:bg-accent/50 transition-colors group mb-6" onClick={toggleConversationPanel}>
        <Home size={20} />
        <div className="absolute left-full ml-2 px-2 py-1 bg-popover text-popover-foreground text-sm rounded-md invisible opacity-0 group-hover:visible group-hover:opacity-100 transition-opacity whitespace-nowrap">
          Open Workspace
        </div>
      </div>
      {/* Navigation items */}
      <div className="flex-1 flex flex-col items-center space-y-6">
        {menuItems.map(item => <div key={item.id} className={`relative w-10 h-10 rounded-md flex items-center justify-center cursor-pointer transition-colors group ${activeSection === item.id ? 'bg-primary/10 text-primary' : 'text-muted-foreground hover:text-foreground hover:bg-accent/50'}`} onClick={() => setActiveSection(item.id)}>
            {item.icon}
            <div className="absolute left-full ml-2 px-2 py-1 bg-popover text-popover-foreground text-sm rounded-md invisible opacity-0 group-hover:visible group-hover:opacity-100 transition-opacity whitespace-nowrap">
              {item.label}
            </div>
          </div>)}
      </div>
      {/* Theme toggle and settings */}
      <div className="mt-auto mb-4 flex flex-col gap-4">
        <div className="relative w-10 h-10 rounded-md flex items-center justify-center cursor-pointer text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors group" onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}>
          {theme === 'dark' ? <Sun size={20} /> : <Moon size={20} />}
          <div className="absolute left-full ml-2 px-2 py-1 bg-popover text-popover-foreground text-sm rounded-md invisible opacity-0 group-hover:visible group-hover:opacity-100 transition-opacity whitespace-nowrap">
            {theme === 'dark' ? 'Light mode' : 'Dark mode'}
          </div>
        </div>
        <div className="relative w-10 h-10 rounded-md flex items-center justify-center cursor-pointer text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors group" onClick={() => setActiveSection('settings')}>
          <Settings size={20} />
          <div className="absolute left-full ml-2 px-2 py-1 bg-popover text-popover-foreground text-sm rounded-md invisible opacity-0 group-hover:visible group-hover:opacity-100 transition-opacity whitespace-nowrap">
            Settings
          </div>
        </div>
      </div>
      {/* User profile */}
      <div className="relative w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center cursor-pointer hover:bg-primary/20 transition-colors group">
        <img src="https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80" alt="User profile" className="w-8 h-8 rounded-full object-cover" />
        <div className="absolute left-full ml-2 px-2 py-1 bg-popover text-popover-foreground text-sm rounded-md invisible opacity-0 group-hover:visible group-hover:opacity-100 transition-opacity whitespace-nowrap">
          Your Profile
        </div>
      </div>
    </div>;
}
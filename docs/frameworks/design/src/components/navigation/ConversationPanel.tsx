import React, { useEffect, useState } from 'react';
import { X, Search, MessageSquare, CheckSquare, Calendar, FileText, Bot, Users, Folder } from 'lucide-react';
import { ResizablePanel, ResizableHandle, ResizablePanelGroup } from '../ui/resizable';
interface ConversationPanelProps {
  isOpen: boolean;
  onClose: () => void;
  activeSection: string;
  setActiveSection: (section: string) => void;
  children: React.ReactNode;
}
export function ConversationPanel({
  isOpen,
  onClose,
  activeSection,
  setActiveSection,
  children
}: ConversationPanelProps) {
  const [width, setWidth] = useState(288);
  const [maxWidth, setMaxWidth] = useState(window.innerWidth * 0.5);
  // Mock data for different sections
  const communicationsData = [{
    id: 1,
    name: 'EVO Assistant Chat',
    time: 'Just now',
    excerpt: 'I can help you with that task...',
    type: 'ai',
    unread: true
  }, {
    id: 2,
    name: 'Project Team',
    time: '2h ago',
    excerpt: 'New updates on the roadmap...',
    type: 'group',
    unread: true
  }, {
    id: 3,
    name: 'Sarah Miller',
    time: '3h ago',
    excerpt: 'Thanks for the feedback on...',
    type: 'direct',
    unread: false
  }];
  const tasksData = [{
    id: 1,
    title: 'Review Design System',
    due: 'Today',
    priority: 'High',
    status: 'In Progress'
  }, {
    id: 2,
    title: 'Update Documentation',
    due: 'Tomorrow',
    priority: 'Medium',
    status: 'To Do'
  }, {
    id: 3,
    title: 'Team Planning',
    due: 'Next Week',
    priority: 'Low',
    status: 'To Do'
  }];
  const calendarData = [{
    id: 1,
    title: 'Team Standup',
    time: '10:00 AM',
    duration: '30m',
    participants: 5
  }, {
    id: 2,
    title: 'Product Review',
    time: '2:00 PM',
    duration: '1h',
    participants: 8
  }, {
    id: 3,
    title: 'Design Sync',
    time: '4:00 PM',
    duration: '45m',
    participants: 4
  }];
  const documentsData = [{
    id: 1,
    name: 'Project Documentation',
    type: 'folder',
    items: 5
  }, {
    id: 2,
    name: 'Meeting Notes.md',
    type: 'file',
    modified: '2h ago'
  }, {
    id: 3,
    name: 'Design Assets',
    type: 'folder',
    items: 12
  }];
  const navigationItems = [{
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
  }, {
    id: 'documents',
    icon: <FileText size={20} />,
    label: 'Documents'
  }];
  const getSearchPlaceholder = () => {
    switch (activeSection) {
      case 'communications':
        return 'Search conversations...';
      case 'tasks':
        return 'Search tasks...';
      case 'calendar':
        return 'Search events...';
      case 'documents':
        return 'Search files and folders...';
      default:
        return 'Search...';
    }
  };
  const renderContent = () => {
    switch (activeSection) {
      case 'communications':
        return communicationsData.map(item => <div key={item.id} className="p-3 hover:bg-accent/50 cursor-pointer border-b border-border last:border-b-0">
            <div className="flex justify-between items-start">
              <div className="flex items-center gap-2">
                {item.type === 'ai' && <Bot size={16} className="text-primary" />}
                {item.type === 'group' && <Users size={16} className="text-primary" />}
                {item.type === 'direct' && <MessageSquare size={16} className="text-primary" />}
                <span className={`font-medium ${item.unread ? 'text-foreground' : 'text-muted-foreground'}`}>
                  {item.name}
                </span>
              </div>
              <span className="text-xs text-muted-foreground">{item.time}</span>
            </div>
            <p className="text-sm truncate text-muted-foreground mt-1">
              {item.excerpt}
            </p>
            {item.unread && <div className="mt-1 flex justify-end">
                <div className="w-2 h-2 rounded-full bg-primary"></div>
              </div>}
          </div>);
      case 'tasks':
        return tasksData.map(task => <div key={task.id} className="p-3 hover:bg-accent/50 cursor-pointer border-b border-border last:border-b-0">
            <div className="flex justify-between items-start">
              <span className="font-medium text-foreground">{task.title}</span>
              <span className={`text-xs px-2 py-1 rounded-full ${task.priority === 'High' ? 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200' : task.priority === 'Medium' ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200' : 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'}`}>
                {task.priority}
              </span>
            </div>
            <div className="flex justify-between mt-2 text-sm text-muted-foreground">
              <span>Due: {task.due}</span>
              <span>{task.status}</span>
            </div>
          </div>);
      case 'calendar':
        return calendarData.map(event => <div key={event.id} className="p-3 hover:bg-accent/50 cursor-pointer border-b border-border last:border-b-0">
            <div className="flex justify-between items-start">
              <span className="font-medium text-foreground">{event.title}</span>
              <span className="text-xs text-muted-foreground">
                {event.duration}
              </span>
            </div>
            <div className="flex justify-between mt-2 text-sm">
              <span className="text-primary">{event.time}</span>
              <span className="text-muted-foreground">
                {event.participants} participants
              </span>
            </div>
          </div>);
      case 'documents':
        return documentsData.map(item => <div key={item.id} className="p-3 hover:bg-accent/50 cursor-pointer border-b border-border last:border-b-0">
            <div className="flex items-center gap-2">
              {item.type === 'folder' ? <Folder size={16} className="text-primary" /> : <FileText size={16} className="text-primary" />}
              <span className="font-medium text-foreground">{item.name}</span>
            </div>
            <div className="mt-1 text-sm text-muted-foreground">
              {item.type === 'folder' ? `${item.items} items` : `Modified ${item.modified}`}
            </div>
          </div>);
      default:
        return null;
    }
  };
  if (!isOpen) return null;
  return <ResizablePanelGroup direction="horizontal" className="flex-1">
      <ResizablePanel defaultSize={25} minSize={20} maxSize={40} className="h-full bg-background border-r border-border flex flex-col">
        <div className="p-4 flex items-center justify-between border-b border-border">
          <h2 className="font-semibold capitalize">{activeSection}</h2>
          <button onClick={onClose} className="w-8 h-8 rounded-md flex items-center justify-center hover:bg-accent/50 transition-colors">
            <X size={18} />
          </button>
        </div>
        <div className="border-b border-border">
          <div className="flex items-center px-2 py-2 gap-1">
            {navigationItems.map(item => <div key={item.id} className="relative flex-1">
                <button onClick={() => setActiveSection(item.id)} className={`w-full p-2 rounded-md flex items-center justify-center transition-colors ${activeSection === item.id ? 'bg-primary/10 text-primary' : 'text-muted-foreground hover:text-foreground hover:bg-accent/50'}`}>
                  {item.icon}
                </button>
                <div className="absolute left-1/2 -translate-x-1/2 top-full mt-1 px-2 py-1 bg-popover text-popover-foreground text-xs rounded-md invisible opacity-0 group-hover:visible group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none">
                  {item.label}
                </div>
              </div>)}
          </div>
        </div>
        <div className="p-3">
          <div className="relative">
            <Search size={16} className="absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground" />
            <input type="text" placeholder={getSearchPlaceholder()} className="w-full rounded-md py-2 pl-9 pr-3 bg-accent/50 border-0 focus:ring-1 focus:ring-primary text-sm" />
          </div>
        </div>
        <div className="flex-1 overflow-y-auto">{renderContent()}</div>
      </ResizablePanel>
      <ResizableHandle />
      <ResizablePanel defaultSize={75}>{children}</ResizablePanel>
    </ResizablePanelGroup>;
}
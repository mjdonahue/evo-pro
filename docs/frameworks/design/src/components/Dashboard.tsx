import React, { Component } from 'react';
import { MessageSquare, CheckSquare, Calendar, FileText, Bot, Users, Folder, ArrowRight, Plus, Clock } from 'lucide-react';
export interface DashboardProps {
  setActiveSection: (section: string) => void;
}
export function Dashboard({
  setActiveSection
}: DashboardProps) {
  // Expanded mock data with 10 items for each section
  const recentConversations = [{
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
    excerpt: 'Thanks for the feedback on the design proposal!',
    type: 'direct',
    unread: false
  }, {
    id: 4,
    name: 'Code Review Discussion',
    time: 'Yesterday',
    excerpt: "I, ve, addressed, all, the, comments, from, the, PR, ...'",
    type: 'group',
    unread: false
  }, {
    id: 5,
    name: 'EVO Assistant Help',
    time: 'Yesterday',
    excerpt: 'Here are some resources about React hooks...',
    type: 'ai',
    unread: false
  }, {
    id: 6,
    name: 'Alex Johnson',
    time: '2d ago',
    excerpt: 'When is the next team meeting scheduled?',
    type: 'direct',
    unread: false
  }, {
    id: 7,
    name: 'Product Planning',
    time: '2d ago',
    excerpt: 'We need to finalize the feature set for Q3...',
    type: 'group',
    unread: false
  }, {
    id: 8,
    name: 'EVO Code Assistant',
    time: '3d ago',
    excerpt: "Here, s, the, refactored, component, you, requested, ...'",
    type: 'ai',
    unread: false
  }, {
    id: 9,
    name: 'Design Team',
    time: '4d ago',
    excerpt: 'The new design system components are ready...',
    type: 'group',
    unread: false
  }, {
    id: 10,
    name: 'Michael Chang',
    time: '5d ago',
    excerpt: 'Can we discuss the client feedback tomorrow?',
    type: 'direct',
    unread: false
  }];
  const recentTasks = [{
    id: 1,
    title: 'Review Design System',
    due: 'Today',
    priority: 'High',
    status: 'In Progress'
  }, {
    id: 2,
    title: 'Update Documentation',
    due: 'Today',
    priority: 'Medium',
    status: 'To Do'
  }, {
    id: 3,
    title: 'Fix Navigation Bug',
    due: 'Today',
    priority: 'High',
    status: 'To Do'
  }, {
    id: 4,
    title: 'Prepare Demo for Client',
    due: 'Tomorrow',
    priority: 'High',
    status: 'Not Started'
  }, {
    id: 5,
    title: 'Code Review PR #234',
    due: 'Today',
    priority: 'Medium',
    status: 'In Progress'
  }, {
    id: 6,
    title: 'Update User Flows',
    due: '2 days',
    priority: 'Low',
    status: 'Not Started'
  }, {
    id: 7,
    title: 'Research API Integration',
    due: '3 days',
    priority: 'Medium',
    status: 'Not Started'
  }, {
    id: 8,
    title: 'Create Test Cases',
    due: 'This Week',
    priority: 'Medium',
    status: 'Not Started'
  }, {
    id: 9,
    title: 'Refactor Authentication',
    due: 'This Week',
    priority: 'Low',
    status: 'Not Started'
  }, {
    id: 10,
    title: 'Update Dependencies',
    due: 'Next Week',
    priority: 'Low',
    status: 'Not Started'
  }];
  // Today's calendar agenda
  const todaysAgenda = [{
    id: 1,
    title: 'Team Standup',
    time: '09:00 AM',
    duration: '30m',
    participants: 5
  }, {
    id: 2,
    title: 'Design Review',
    time: '10:30 AM',
    duration: '1h',
    participants: 4
  }, {
    id: 3,
    title: 'Lunch Break',
    time: '12:00 PM',
    duration: '1h',
    participants: 0
  }, {
    id: 4,
    title: 'Product Planning',
    time: '1:30 PM',
    duration: '1h 30m',
    participants: 6
  }, {
    id: 5,
    title: 'Client Meeting',
    time: '3:00 PM',
    duration: '45m',
    participants: 3
  }, {
    id: 6,
    title: 'Code Review',
    time: '4:00 PM',
    duration: '1h',
    participants: 2
  }, {
    id: 7,
    title: 'Weekly Wrap-up',
    time: '5:00 PM',
    duration: '30m',
    participants: 8
  }];
  const recentDocuments = [{
    id: 1,
    name: 'Project Documentation',
    type: 'folder',
    items: 5,
    modified: '2h ago'
  }, {
    id: 2,
    name: 'Meeting Notes.md',
    type: 'file',
    modified: '3h ago'
  }, {
    id: 3,
    name: 'Design Assets',
    type: 'folder',
    items: 12,
    modified: 'Yesterday'
  }, {
    id: 4,
    name: 'API Specification.md',
    type: 'file',
    modified: 'Yesterday'
  }, {
    id: 5,
    name: 'Quarterly Report.pdf',
    type: 'file',
    modified: '2d ago'
  }, {
    id: 6,
    name: 'User Research',
    type: 'folder',
    items: 8,
    modified: '3d ago'
  }, {
    id: 7,
    name: 'Component Library.sketch',
    type: 'file',
    modified: '4d ago'
  }, {
    id: 8,
    name: 'Product Roadmap.md',
    type: 'file',
    modified: '5d ago'
  }, {
    id: 9,
    name: 'Marketing Assets',
    type: 'folder',
    items: 15,
    modified: '1w ago'
  }, {
    id: 10,
    name: 'Budget Forecast.xlsx',
    type: 'file',
    modified: '1w ago'
  }];
  // Get current date for the agenda
  const today = new Date();
  const dateOptions = {
    weekday: 'long',
    month: 'long',
    day: 'numeric'
  } as const;
  const formattedDate = today.toLocaleDateString('en-US', dateOptions);
  return <div className="flex flex-col h-full">
      <div className="p-4 flex justify-between items-center border-b border-border">
        <h2 className="text-2xl font-semibold">Dashboard</h2>
        <button onClick={() => setActiveSection('communications')} className="flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 transition-colors">
          <Plus size={18} />
          New Conversation
        </button>
      </div>
      <div className="grid grid-cols-2 gap-4 p-4 flex-1 overflow-auto">
        {/* Communications Card */}
        <div className="bg-card rounded-lg border shadow-sm">
          <div className="p-4 border-b flex justify-between items-center">
            <div className="flex items-center gap-2">
              <MessageSquare className="w-5 h-5 text-primary" />
              <h3 className="font-semibold">Recent Conversations</h3>
            </div>
            <button onClick={() => setActiveSection('communications')} className="text-sm text-muted-foreground hover:text-primary flex items-center gap-1">
              View all <ArrowRight className="w-4 h-4" />
            </button>
          </div>
          <div className="p-4 space-y-2 overflow-auto max-h-[calc(50vh-8rem)]">
            {recentConversations.map(conv => <div key={conv.id} className="p-3 hover:bg-accent/50 rounded-md cursor-pointer">
                <div className="flex justify-between items-start">
                  <div className="flex items-center gap-2">
                    {conv.type === 'ai' && <Bot size={16} className="text-primary" />}
                    {conv.type === 'group' && <Users size={16} className="text-primary" />}
                    {conv.type === 'direct' && <MessageSquare size={16} className="text-primary" />}
                    <span className="font-medium">{conv.name}</span>
                  </div>
                  <span className="text-xs text-muted-foreground">
                    {conv.time}
                  </span>
                </div>
                <p className="text-sm text-muted-foreground mt-1 truncate">
                  {conv.excerpt}
                </p>
                {conv.unread && <div className="mt-1 flex justify-end">
                    <div className="w-2 h-2 rounded-full bg-primary"></div>
                  </div>}
              </div>)}
          </div>
        </div>
        {/* Tasks Card */}
        <div className="bg-card rounded-lg border shadow-sm">
          <div className="p-4 border-b flex justify-between items-center">
            <div className="flex items-center gap-2">
              <CheckSquare className="w-5 h-5 text-primary" />
              <h3 className="font-semibold">Tasks</h3>
            </div>
            <button onClick={() => setActiveSection('tasks')} className="text-sm text-muted-foreground hover:text-primary flex items-center gap-1">
              View all <ArrowRight className="w-4 h-4" />
            </button>
          </div>
          <div className="p-4 space-y-2 overflow-auto max-h-[calc(50vh-8rem)]">
            {recentTasks.map(task => <div key={task.id} className="p-3 hover:bg-accent/50 rounded-md cursor-pointer">
                <div className="flex justify-between items-start">
                  <span className="font-medium">{task.title}</span>
                  <span className={`text-xs px-2 py-1 rounded-full ${task.priority === 'High' ? 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200' : task.priority === 'Medium' ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200' : 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'}`}>
                    {task.priority}
                  </span>
                </div>
                <div className="flex justify-between mt-2 text-sm text-muted-foreground">
                  <span>Due: {task.due}</span>
                  <span>{task.status}</span>
                </div>
              </div>)}
          </div>
        </div>
        {/* Calendar Card - Today's Agenda */}
        <div className="bg-card rounded-lg border shadow-sm">
          <div className="p-4 border-b flex justify-between items-center">
            <div className="flex items-center gap-2">
              <Calendar className="w-5 h-5 text-primary" />
              <h3 className="font-semibold">Today's Agenda</h3>
            </div>
            <button onClick={() => setActiveSection('calendar')} className="text-sm text-muted-foreground hover:text-primary flex items-center gap-1">
              Full Calendar <ArrowRight className="w-4 h-4" />
            </button>
          </div>
          <div className="p-4 space-y-2 overflow-auto max-h-[calc(50vh-8rem)]">
            <div className="mb-3 text-sm font-medium text-muted-foreground flex items-center gap-2">
              <Clock size={14} />
              {formattedDate}
            </div>
            {todaysAgenda.length > 0 ? todaysAgenda.map(event => <div key={event.id} className="p-3 hover:bg-accent/50 rounded-md cursor-pointer border-l-2 border-primary">
                  <div className="flex justify-between items-start">
                    <span className="font-medium">{event.title}</span>
                    <span className="text-xs text-muted-foreground">
                      {event.duration}
                    </span>
                  </div>
                  <div className="flex justify-between mt-2 text-sm">
                    <span className="text-primary">{event.time}</span>
                    {event.participants > 0 && <span className="text-muted-foreground flex items-center gap-1">
                        <Users size={14} />
                        {event.participants}
                      </span>}
                  </div>
                </div>) : <div className="p-4 text-center text-muted-foreground">
                No events scheduled for today
              </div>}
          </div>
        </div>
        {/* Documents Card */}
        <div className="bg-card rounded-lg border shadow-sm">
          <div className="p-4 border-b flex justify-between items-center">
            <div className="flex items-center gap-2">
              <FileText className="w-5 h-5 text-primary" />
              <h3 className="font-semibold">Recent Documents</h3>
            </div>
            <button onClick={() => setActiveSection('documents')} className="text-sm text-muted-foreground hover:text-primary flex items-center gap-1">
              View all <ArrowRight className="w-4 h-4" />
            </button>
          </div>
          <div className="p-4 space-y-2 overflow-auto max-h-[calc(50vh-8rem)]">
            {recentDocuments.map(doc => <div key={doc.id} className="p-3 hover:bg-accent/50 rounded-md cursor-pointer">
                <div className="flex items-center gap-2">
                  {doc.type === 'folder' ? <Folder size={16} className="text-primary" /> : <FileText size={16} className="text-primary" />}
                  <span className="font-medium">{doc.name}</span>
                </div>
                <div className="mt-1 text-sm text-muted-foreground">
                  {doc.type === 'folder' ? `${doc.items} items • Modified ${doc.modified}` : `Modified ${doc.modified}`}
                </div>
              </div>)}
          </div>
        </div>
      </div>
    </div>;
}
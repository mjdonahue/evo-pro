import { FileText } from 'lucide-react'

export function CommunicationsContent() {
    const messages = [
      {
        id: 1,
        sender: 'AI Assistant',
        content: 'Hello! How can I help you today?',
        time: '10:30 AM',
        isUser: false,
      },
      {
        id: 2,
        sender: 'You',
        content: 'I need help setting up a new project.',
        time: '10:31 AM',
        isUser: true,
      },
      {
        id: 3,
        sender: 'AI Assistant',
        content:
          'Sure, I can help with that. What kind of project are you working on?',
        time: '10:31 AM',
        isUser: false,
      },
      {
        id: 4,
        sender: 'You',
        content: "I'm building a desktop app using Tauri and React.",
        time: '10:32 AM',
        isUser: true,
      },
      {
        id: 5,
        sender: 'AI Assistant',
        content:
          "Great choice! Tauri is excellent for building lightweight cross-platform desktop apps. Let me guide you through the setup process. First, you'll need to install the Tauri CLI.",
        time: '10:33 AM',
        isUser: false,
      },
    ]
    return (
      <div className="space-y-4">
        {messages.map((message) => (
          <div
            key={message.id}
            className={`flex ${message.isUser ? 'justify-end' : 'justify-start'}`}
          >
            <div
              className={`max-w-[70%] rounded-lg p-4 ${message.isUser ? 'bg-primary text-primary-foreground' : 'bg-accent'}`}
            >
              <div className="flex justify-between items-center mb-1">
                <span
                  className={`font-medium text-sm ${message.isUser ? 'text-primary-foreground' : 'text-foreground'}`}
                >
                  {message.sender}
                </span>
                <span
                  className={`text-xs ${message.isUser ? 'text-primary-foreground/70' : 'text-muted-foreground'}`}
                >
                  {message.time}
                </span>
              </div>
              <p
                className={`text-sm ${message.isUser ? 'text-primary-foreground' : 'text-foreground'}`}
              >
                {message.content}
              </p>
            </div>
          </div>
        ))}
        {/* Example attachment */}
        <div className="flex justify-start">
          <div className="max-w-[70%] rounded-lg p-4 bg-accent">
            <div className="flex justify-between items-center mb-1">
              <span className="font-medium text-sm text-foreground">
                AI Assistant
              </span>
              <span className="text-xs text-muted-foreground">10:35 AM</span>
            </div>
            <p className="text-sm text-foreground mb-3">
              Here's a document that might help you get started:
            </p>
            <div className="bg-background rounded-md p-3 flex items-center border border-border">
              <FileText className="text-primary mr-3" size={24} />
              <div>
                <p className="text-sm font-medium">
                  Tauri Getting Started Guide.pdf
                </p>
                <p className="text-xs text-muted-foreground">2.4 MB • PDF</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    )
  }
  
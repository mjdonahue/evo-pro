import React from 'react';
import { Paperclip, Mic, Send, Image, Video } from 'lucide-react';
import { ResizablePanel, ResizablePanelGroup, ResizableHandle } from '../ui/resizable';
export function ChatInput() {
  return <ResizablePanelGroup direction="vertical">
      <ResizablePanel defaultSize={100}>
        <div className="flex-1" />
      </ResizablePanel>
      <ResizableHandle />
      <ResizablePanel defaultSize={25} minSize={15} maxSize={50}>
        <div className="h-full bg-accent/50 p-3 border-t border-border">
          <div className="flex items-center space-x-2 mb-3">
            <button className="p-2 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors">
              <Paperclip size={18} />
            </button>
            <button className="p-2 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors">
              <Image size={18} />
            </button>
            <button className="p-2 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors">
              <Video size={18} />
            </button>
            <button className="p-2 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors">
              <Mic size={18} />
            </button>
          </div>
          <div className="flex">
            <input type="text" placeholder="Type your message..." className="flex-1 bg-transparent border-0 focus:ring-0 text-foreground" />
            <button className="ml-2 p-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 transition-colors">
              <Send size={18} />
            </button>
          </div>
        </div>
      </ResizablePanel>
    </ResizablePanelGroup>;
}
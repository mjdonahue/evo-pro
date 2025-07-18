import React, { useState } from 'react'
import {
  Paperclip,
  Mic,
  Send,
  Image,
  Video,
  Smile,
  XCircle,
} from 'lucide-react'
import {
  ResizablePanel,
  ResizablePanelGroup,
  ResizableHandle,
} from '../ui/resizable'
export function ChatInput() {
  const [message, setMessage] = useState('')
  const [isRecording, setIsRecording] = useState(false)
  const [attachments, setAttachments] = useState<string[]>([])
  const handleSendMessage = () => {
    if (message.trim() || attachments.length > 0) {
      console.log('Sending message:', message)
      console.log('Attachments:', attachments)
      setMessage('')
      setAttachments([])
    }
  }
  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSendMessage()
    }
  }
  const toggleRecording = () => {
    setIsRecording(!isRecording)
  }
  const addAttachment = (type: string) => {
    // Simulate adding an attachment
    const newAttachment = `${type}-${Date.now()}`
    setAttachments([...attachments, newAttachment])
  }
  const removeAttachment = (attachment: string) => {
    setAttachments(attachments.filter((a) => a !== attachment))
  }
  return (
    <ResizablePanelGroup direction="vertical">
      <ResizableHandle />
      <ResizablePanel defaultSize={25} minSize={15} maxSize={50}>
        <div className="h-full bg-accent/50 p-3 border-t border-border">
          {attachments.length > 0 && (
            <div className="flex flex-wrap gap-2 mb-3">
              {attachments.map((attachment) => (
                <div
                  key={attachment}
                  className="bg-background rounded-md px-3 py-1 text-sm flex items-center gap-1 border border-border"
                >
                  {attachment.startsWith('file') && <Paperclip size={14} />}
                  {attachment.startsWith('image') && <Image size={14} />}
                  {attachment.startsWith('video') && <Video size={14} />}
                  <span className="truncate max-w-[150px]">
                    {attachment.split('-')[0]} attachment
                  </span>
                  <button
                    onClick={() => removeAttachment(attachment)}
                    className="text-muted-foreground hover:text-foreground"
                  >
                    <XCircle size={14} />
                  </button>
                </div>
              ))}
            </div>
          )}
          <div className="flex items-center space-x-2 mb-3">
            <button
              className="p-2 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
              onClick={() => addAttachment('file')}
            >
              <Paperclip size={18} />
            </button>
            <button
              className="p-2 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
              onClick={() => addAttachment('image')}
            >
              <Image size={18} />
            </button>
            <button
              className="p-2 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
              onClick={() => addAttachment('video')}
            >
              <Video size={18} />
            </button>
            <button
              className={`p-2 rounded ${isRecording ? 'bg-red-500 text-white' : 'hover:bg-accent text-muted-foreground hover:text-foreground'} transition-colors`}
              onClick={toggleRecording}
            >
              <Mic size={18} />
            </button>
            <button className="p-2 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors">
              <Smile size={18} />
            </button>
          </div>
          <div className="flex">
            <textarea
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              onKeyDown={handleKeyPress}
              placeholder="Type your message..."
              className="flex-1 bg-transparent border-0 focus:ring-0 text-foreground resize-none min-h-[40px] max-h-[120px] py-2"
              rows={1}
            />
            <button
              className={`ml-2 p-2 ${message.trim() || attachments.length > 0 ? 'bg-primary text-primary-foreground' : 'bg-primary/50 text-primary-foreground/50 cursor-not-allowed'} rounded-md hover:bg-primary/90 transition-colors`}
              onClick={handleSendMessage}
              disabled={!message.trim() && attachments.length === 0}
            >
              <Send size={18} />
            </button>
          </div>
          {isRecording && (
            <div className="mt-2 p-2 bg-background border border-border rounded-md flex items-center justify-between">
              <div className="flex items-center gap-2">
                <div className="w-2 h-2 rounded-full bg-red-500 animate-pulse"></div>
                <span className="text-sm">Recording audio...</span>
              </div>
              <button
                className="text-sm text-red-500 hover:text-red-600"
                onClick={toggleRecording}
              >
                Cancel
              </button>
            </div>
          )}
        </div>
      </ResizablePanel>
    </ResizablePanelGroup>
  )
}

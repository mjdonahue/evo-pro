import React, { useEffect, useState, useRef } from 'react'
import {
  Bell,
  MessageSquare,
  Calendar,
  CheckSquare,
  FileText,
  X,
} from 'lucide-react'
interface Notification {
  id: number
  title: string
  message: string
  time: string
  type: 'message' | 'calendar' | 'task' | 'document'
  read: boolean
}
export function NotificationsMenu() {
  const [isOpen, setIsOpen] = useState(false)
  const [notifications, setNotifications] = useState<Notification[]>([
    {
      id: 1,
      title: 'New message',
      message: 'Sarah Miller sent you a message',
      time: '10 min ago',
      type: 'message',
      read: false,
    },
    {
      id: 2,
      title: 'Upcoming meeting',
      message: 'Design Review in 30 minutes',
      time: '30 min ago',
      type: 'calendar',
      read: false,
    },
    {
      id: 3,
      title: 'Task assigned',
      message: 'Fix Navigation Bug was assigned to you',
      time: '1 hour ago',
      type: 'task',
      read: false,
    },
    {
      id: 4,
      title: 'Document updated',
      message: 'API Specification.md was updated',
      time: '2 hours ago',
      type: 'document',
      read: true,
    },
    {
      id: 5,
      title: 'New message',
      message: 'Alex Johnson sent you a message',
      time: 'Yesterday',
      type: 'message',
      read: true,
    },
  ])
  const menuRef = useRef<HTMLDivElement>(null)
  const unreadCount = notifications.filter((n) => !n.read).length
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setIsOpen(false)
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
    }
  }, [])
  const markAsRead = (id: number) => {
    setNotifications(
      notifications.map((notification) =>
        notification.id === id
          ? {
              ...notification,
              read: true,
            }
          : notification,
      ),
    )
  }
  const markAllAsRead = () => {
    setNotifications(
      notifications.map((notification) => ({
        ...notification,
        read: true,
      })),
    )
  }
  const removeNotification = (id: number) => {
    setNotifications(
      notifications.filter((notification) => notification.id !== id),
    )
  }
  const getIcon = (type: string) => {
    switch (type) {
      case 'message':
        return <MessageSquare size={16} className="text-blue-500" />
      case 'calendar':
        return <Calendar size={16} className="text-green-500" />
      case 'task':
        return <CheckSquare size={16} className="text-purple-500" />
      case 'document':
        return <FileText size={16} className="text-orange-500" />
      default:
        return <Bell size={16} className="text-primary" />
    }
  }
  return (
    <div className="relative" ref={menuRef}>
      <button
        className="relative w-10 h-10 rounded-md flex items-center justify-center text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors"
        onClick={() => setIsOpen(!isOpen)}
      >
        <Bell size={20} />
        {unreadCount > 0 && (
          <span className="absolute top-2 right-2 w-4 h-4 bg-primary text-primary-foreground text-xs flex items-center justify-center rounded-full">
            {unreadCount}
          </span>
        )}
      </button>
      {isOpen && (
        <div className="absolute right-0 mt-2 w-80 bg-background border border-border rounded-md shadow-lg z-50">
          <div className="flex items-center justify-between p-4 border-b border-border">
            <h3 className="font-semibold">Notifications</h3>
            {unreadCount > 0 && (
              <button
                className="text-xs text-primary hover:underline"
                onClick={markAllAsRead}
              >
                Mark all as read
              </button>
            )}
          </div>
          <div className="max-h-96 overflow-y-auto">
            {notifications.length > 0 ? (
              notifications.map((notification) => (
                <div
                  key={notification.id}
                  className={`p-4 border-b border-border last:border-b-0 ${notification.read ? 'opacity-70' : ''}`}
                >
                  <div className="flex justify-between">
                    <div className="flex items-start">
                      <div className="mt-0.5 mr-3">
                        {getIcon(notification.type)}
                      </div>
                      <div>
                        <p className="text-sm font-medium">
                          {notification.title}
                        </p>
                        <p className="text-sm text-muted-foreground">
                          {notification.message}
                        </p>
                        <p className="text-xs text-muted-foreground mt-1">
                          {notification.time}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-start space-x-1">
                      {!notification.read && (
                        <button
                          className="p-1 hover:bg-accent/50 rounded-md"
                          onClick={() => markAsRead(notification.id)}
                        >
                          <div className="w-2 h-2 bg-primary rounded-full"></div>
                        </button>
                      )}
                      <button
                        className="p-1 hover:bg-accent/50 rounded-md"
                        onClick={() => removeNotification(notification.id)}
                      >
                        <X size={14} className="text-muted-foreground" />
                      </button>
                    </div>
                  </div>
                </div>
              ))
            ) : (
              <div className="p-4 text-center text-muted-foreground">
                No notifications
              </div>
            )}
          </div>
          <div className="p-2 border-t border-border">
            <button className="w-full py-2 text-sm text-center text-primary hover:bg-accent/50 rounded-md">
              View all notifications
            </button>
          </div>
        </div>
      )}
    </div>
  )
}

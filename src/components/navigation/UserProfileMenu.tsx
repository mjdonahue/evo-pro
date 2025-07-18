import React, { useEffect, useState, useRef } from 'react'
import { Link } from 'react-router-dom'
import {
  User,
  Settings,
  HelpCircle,
  LogOut,
  Bell,
  Mail,
  CheckCircle,
} from 'lucide-react'
export function UserProfileMenu() {
  const [isOpen, setIsOpen] = useState(false)
  const menuRef = useRef<HTMLDivElement>(null)
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
  return (
    <div className="relative" ref={menuRef}>
      <div
        className="w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center cursor-pointer hover:bg-primary/20 transition-colors"
        onClick={() => setIsOpen(!isOpen)}
      >
        <img
          src="https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80"
          alt="User profile"
          className="w-8 h-8 rounded-full object-cover"
        />
      </div>
      {isOpen && (
        <div className="absolute right-0 mt-2 w-64 bg-background border border-border rounded-md shadow-lg z-50">
          <div className="p-4 border-b border-border">
            <div className="flex items-center">
              <img
                src="https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80"
                alt="User profile"
                className="w-10 h-10 rounded-full object-cover mr-3"
              />
              <div>
                <p className="font-medium">John Doe</p>
                <p className="text-sm text-muted-foreground">
                  john.doe@example.com
                </p>
              </div>
            </div>
          </div>
          <div className="py-1">
            <Link
              to="/profile"
              className="flex items-center px-4 py-2 hover:bg-accent/50 text-sm"
              onClick={() => setIsOpen(false)}
            >
              <User size={16} className="mr-3 text-muted-foreground" />
              Your Profile
            </Link>
            <Link
              to="/settings"
              className="flex items-center px-4 py-2 hover:bg-accent/50 text-sm"
              onClick={() => setIsOpen(false)}
            >
              <Settings size={16} className="mr-3 text-muted-foreground" />
              Settings
            </Link>
            <Link
              to="/help"
              className="flex items-center px-4 py-2 hover:bg-accent/50 text-sm"
              onClick={() => setIsOpen(false)}
            >
              <HelpCircle size={16} className="mr-3 text-muted-foreground" />
              Help & Support
            </Link>
          </div>
          <div className="py-1 border-t border-border">
            <button
              className="w-full flex items-center px-4 py-2 hover:bg-accent/50 text-sm"
              onClick={() => setIsOpen(false)}
            >
              <LogOut size={16} className="mr-3 text-muted-foreground" />
              Sign out
            </button>
          </div>
        </div>
      )}
    </div>
  )
}

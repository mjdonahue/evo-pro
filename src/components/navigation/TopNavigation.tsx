import React from 'react'
import { Search, HelpCircle } from 'lucide-react'
import { NotificationsMenu } from './NotificationsMenu'
import { UserProfileMenu } from './UserProfileMenu'
export function TopNavigation() {
  return (
    <div className="h-14 border-b border-border flex items-center justify-between px-6">
      <div className="flex-1 max-w-xl">
        <div className="relative">
          <Search
            size={18}
            className="absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground"
          />
          <input
            type="text"
            placeholder="Search anything..."
            className="w-full pl-10 pr-4 py-2 bg-accent/50 border-0 rounded-md focus:ring-1 focus:ring-primary text-sm"
          />
        </div>
      </div>
      <div className="flex items-center space-x-2">
        <button className="w-10 h-10 rounded-md flex items-center justify-center text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors">
          <HelpCircle size={20} />
        </button>
        <NotificationsMenu />
        <UserProfileMenu />
      </div>
    </div>
  )
}

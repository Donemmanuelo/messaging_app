'use client'

import { useState } from 'react'
import { useAuthStore } from '@/stores/authStore'
import { useChatStore } from '@/stores/chatStore'
import { MagnifyingGlassIcon, EllipsisVerticalIcon } from '@heroicons/react/24/outline'
import ChatList from './ChatList'
import UserProfile from './UserProfile'

export default function Sidebar() {
  const [searchQuery, setSearchQuery] = useState('')
  const [showProfile, setShowProfile] = useState(false)
  const { user, logout } = useAuthStore()
  const { chats } = useChatStore()

  const filteredChats = chats.filter(chat => 
    chat.name?.toLowerCase().includes(searchQuery.toLowerCase()) ||
    chat.participants.some(p => 
      p.username.toLowerCase().includes(searchQuery.toLowerCase()) ||
      p.display_name?.toLowerCase().includes(searchQuery.toLowerCase())
    )
  )

  return (
    <div className="flex flex-col h-full bg-white">
      {/* Header */}
      <div className="flex items-center justify-between p-4 bg-whatsapp-gray-light border-b">
        <div className="flex items-center space-x-3">
          <img
            src={user?.avatar_url || '/default-avatar.png'}
            alt="Profile"
            className="w-10 h-10 rounded-full cursor-pointer"
            onClick={() => setShowProfile(true)}
          />
          <span className="font-medium">{user?.display_name || user?.username}</span>
        </div>
        <div className="relative">
          <button className="p-2 rounded-full hover:bg-gray-200">
            <EllipsisVerticalIcon className="w-5 h-5" />
          </button>
          {/* Dropdown menu would go here */}
        </div>
      </div>

      {/* Search */}
      <div className="p-3 border-b">
        <div className="relative">
          <MagnifyingGlassIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-400" />
          <input
            type="text"
            placeholder="Search chats..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 bg-gray-100 rounded-lg focus:outline-none focus:ring-2 focus:ring-whatsapp-green"
          />
        </div>
      </div>

      {/* Chat List */}
      <div className="flex-1 overflow-y-auto custom-scrollbar">
        <ChatList chats={filteredChats} />
      </div>

      {/* User Profile Modal */}
      {showProfile && (
        <UserProfile onClose={() => setShowProfile(false)} />
      )}
    </div>
  )
}
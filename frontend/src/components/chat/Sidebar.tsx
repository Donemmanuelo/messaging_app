'use client';

import { useState } from 'react';
import { PlusIcon, UserGroupIcon } from '@heroicons/react/24/outline';
import type { Chat, User } from '@/types/chat';

interface SidebarProps {
  chats: Chat[];
  currentUser: User;
  onChatSelect: (chatId: string) => void;
  onCreateChat: () => void;
  onCreateGroup: () => void;
}

export default function Sidebar({
  chats,
  currentUser,
  onChatSelect,
  onCreateChat,
  onCreateGroup,
}: SidebarProps) {
  const [searchQuery, setSearchQuery] = useState('');

  const filteredChats = chats.filter((chat) =>
    chat.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="w-64 bg-white border-r border-gray-200 flex flex-col">
      <div className="p-4 border-b border-gray-200">
        <div className="flex items-center space-x-2">
          <img
            src={currentUser.avatar || '/default-avatar.png'}
            alt={currentUser.displayName || currentUser.username}
            className="w-10 h-10 rounded-full"
          />
          <div>
            <h2 className="font-semibold">
              {currentUser.displayName || currentUser.username}
            </h2>
            <p className="text-sm text-gray-500">
              {currentUser.status === 'online' ? 'Online' : 'Offline'}
            </p>
          </div>
        </div>
      </div>

      <div className="p-4">
        <div className="flex space-x-2">
          <button
            onClick={onCreateChat}
            className="flex-1 bg-indigo-600 text-white px-4 py-2 rounded-lg hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500"
          >
            <PlusIcon className="w-5 h-5 inline-block mr-1" />
            New Chat
          </button>
          <button
            onClick={onCreateGroup}
            className="flex-1 bg-gray-100 text-gray-700 px-4 py-2 rounded-lg hover:bg-gray-200 focus:outline-none focus:ring-2 focus:ring-gray-500"
          >
            <UserGroupIcon className="w-5 h-5 inline-block mr-1" />
            New Group
          </button>
        </div>
      </div>

      <div className="p-4">
        <input
          type="text"
          placeholder="Search chats..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500"
        />
      </div>

      <div className="flex-1 overflow-y-auto">
        {filteredChats.map((chat) => (
          <div
            key={chat.id}
            onClick={() => onChatSelect(chat.id)}
            className="p-4 hover:bg-gray-50 cursor-pointer border-b border-gray-200"
          >
            <div className="flex items-center space-x-3">
              <div className="relative">
                <img
                  src={chat.isGroup ? '/group-avatar.png' : chat.participants[0].avatar || '/default-avatar.png'}
                  alt={chat.name}
                  className="w-12 h-12 rounded-full"
                />
                {!chat.isGroup && chat.participants[0].status === 'online' && (
                  <div className="absolute bottom-0 right-0 w-3 h-3 bg-green-500 rounded-full border-2 border-white" />
                )}
              </div>
              <div className="flex-1 min-w-0">
                <div className="flex items-center justify-between">
                  <h3 className="font-medium truncate">{chat.name}</h3>
                  {chat.lastMessage && (
                    <span className="text-xs text-gray-500">
                      {new Date(chat.lastMessage.timestamp).toLocaleTimeString([], {
                        hour: '2-digit',
                        minute: '2-digit',
                      })}
                    </span>
                  )}
                </div>
                <p className="text-sm text-gray-500 truncate">
                  {chat.lastMessage
                    ? chat.lastMessage.type === 'text'
                      ? chat.lastMessage.content
                      : `Shared ${chat.lastMessage.type}`
                    : 'No messages yet'}
                </p>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
} 
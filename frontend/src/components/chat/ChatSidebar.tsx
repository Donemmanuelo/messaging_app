'use client';

import { useState } from 'react';
import { UserGroupIcon, UserCircleIcon } from '@heroicons/react/24/outline';
import type { Chat, User } from '@/types/chat';

interface ChatSidebarProps {
  chat: Chat;
  currentUser: User;
  onClose: () => void;
}

export default function ChatSidebar({ chat, currentUser, onClose }: ChatSidebarProps) {
  const [activeTab, setActiveTab] = useState<'info' | 'members'>('info');

  const getParticipantName = (participant: User) => {
    return participant.displayName || participant.username;
  };

  return (
    <div className="w-80 bg-white border-l border-gray-200 flex flex-col">
      <div className="p-4 border-b border-gray-200">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold">Chat Info</h2>
          <button
            onClick={onClose}
            className="text-gray-500 hover:text-gray-700 focus:outline-none"
          >
            ×
          </button>
        </div>
      </div>

      <div className="p-4 border-b border-gray-200">
        <div className="flex items-center space-x-3">
          <div className="relative">
            <img
              src={chat.isGroup ? '/group-avatar.png' : chat.participants[0].avatar || '/default-avatar.png'}
              alt={chat.name}
              className="w-16 h-16 rounded-full"
            />
            {!chat.isGroup && chat.participants[0].status === 'online' && (
              <div className="absolute bottom-0 right-0 w-3 h-3 bg-green-500 rounded-full border-2 border-white" />
            )}
          </div>
          <div>
            <h3 className="text-lg font-semibold">{chat.name}</h3>
            {chat.isGroup ? (
              <p className="text-sm text-gray-500">{chat.participants.length} members</p>
            ) : (
              <p className="text-sm text-gray-500">
                {chat.participants[0].status === 'online' ? 'Online' : 'Offline'}
              </p>
            )}
          </div>
        </div>
      </div>

      <div className="flex border-b border-gray-200">
        <button
          onClick={() => setActiveTab('info')}
          className={`flex-1 py-3 text-sm font-medium ${
            activeTab === 'info'
              ? 'text-indigo-600 border-b-2 border-indigo-600'
              : 'text-gray-500 hover:text-gray-700'
          }`}
        >
          Info
        </button>
        <button
          onClick={() => setActiveTab('members')}
          className={`flex-1 py-3 text-sm font-medium ${
            activeTab === 'members'
              ? 'text-indigo-600 border-b-2 border-indigo-600'
              : 'text-gray-500 hover:text-gray-700'
          }`}
        >
          Members
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-4">
        {activeTab === 'info' ? (
          <div className="space-y-4">
            {chat.isGroup && (
              <>
                <div>
                  <h4 className="text-sm font-medium text-gray-500">Description</h4>
                  <p className="mt-1 text-gray-900">
                    {(chat as any).description || 'No description'}
                  </p>
                </div>
                <div>
                  <h4 className="text-sm font-medium text-gray-500">Created</h4>
                  <p className="mt-1 text-gray-900">
                    {new Date(chat.createdAt).toLocaleDateString()}
                  </p>
                </div>
              </>
            )}
            <div>
              <h4 className="text-sm font-medium text-gray-500">Media</h4>
              <div className="mt-2 grid grid-cols-3 gap-2">
                {/* Media grid would go here */}
              </div>
            </div>
          </div>
        ) : (
          <div className="space-y-4">
            {chat.participants.map((participant) => (
              <div key={participant.id} className="flex items-center space-x-3">
                <div className="relative">
                  <img
                    src={participant.avatar || '/default-avatar.png'}
                    alt={getParticipantName(participant)}
                    className="w-10 h-10 rounded-full"
                  />
                  {participant.status === 'online' && (
                    <div className="absolute bottom-0 right-0 w-2.5 h-2.5 bg-green-500 rounded-full border-2 border-white" />
                  )}
                </div>
                <div>
                  <p className="font-medium">{getParticipantName(participant)}</p>
                  <p className="text-sm text-gray-500">
                    {participant.id === currentUser.id ? 'You' : participant.status}
                  </p>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {chat.isGroup && (
        <div className="p-4 border-t border-gray-200">
          <button className="w-full bg-red-600 text-white px-4 py-2 rounded-lg hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-500">
            Leave Group
          </button>
        </div>
      )}
    </div>
  );
} 
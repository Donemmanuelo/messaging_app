'use client';

import { useState } from 'react';
import { EllipsisVerticalIcon, PhoneIcon, VideoCameraIcon } from '@heroicons/react/24/outline';
import type { Chat, User } from '@/types/chat';

interface ChatHeaderProps {
  chat: Chat;
  currentUser: User;
  onStartCall: (type: 'audio' | 'video') => void;
  onViewProfile: () => void;
}

export default function ChatHeader({
  chat,
  currentUser,
  onStartCall,
  onViewProfile,
}: ChatHeaderProps) {
  const [showMenu, setShowMenu] = useState(false);

  const getParticipantName = () => {
    if (chat.isGroup) {
      return chat.name;
    }
    const participant = chat.participants.find((p) => p.id !== currentUser.id);
    return participant?.displayName || participant?.username || 'Unknown User';
  };

  const getParticipantStatus = () => {
    if (chat.isGroup) {
      return `${chat.participants.length} members`;
    }
    const participant = chat.participants.find((p) => p.id !== currentUser.id);
    return participant?.status === 'online' ? 'Online' : 'Offline';
  };

  return (
    <div className="bg-white border-b border-gray-200 p-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-3">
          <div className="relative">
            <img
              src={chat.isGroup ? '/group-avatar.png' : chat.participants[0].avatar || '/default-avatar.png'}
              alt={getParticipantName()}
              className="w-12 h-12 rounded-full"
            />
            {!chat.isGroup && chat.participants[0].status === 'online' && (
              <div className="absolute bottom-0 right-0 w-3 h-3 bg-green-500 rounded-full border-2 border-white" />
            )}
          </div>
          <div>
            <h2 className="text-lg font-semibold">{getParticipantName()}</h2>
            <p className="text-sm text-gray-500">{getParticipantStatus()}</p>
          </div>
        </div>

        <div className="flex items-center space-x-2">
          {!chat.isGroup && (
            <>
              <button
                onClick={() => onStartCall('audio')}
                className="p-2 text-gray-500 hover:text-gray-700 focus:outline-none"
              >
                <PhoneIcon className="w-6 h-6" />
              </button>
              <button
                onClick={() => onStartCall('video')}
                className="p-2 text-gray-500 hover:text-gray-700 focus:outline-none"
              >
                <VideoCameraIcon className="w-6 h-6" />
              </button>
            </>
          )}
          <div className="relative">
            <button
              onClick={() => setShowMenu(!showMenu)}
              className="p-2 text-gray-500 hover:text-gray-700 focus:outline-none"
            >
              <EllipsisVerticalIcon className="w-6 h-6" />
            </button>
            {showMenu && (
              <div className="absolute right-0 mt-2 w-48 bg-white rounded-md shadow-lg py-1 z-10">
                <button
                  onClick={() => {
                    onViewProfile();
                    setShowMenu(false);
                  }}
                  className="block w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                >
                  View Profile
                </button>
                {chat.isGroup && (
                  <>
                    <button className="block w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100">
                      Group Settings
                    </button>
                    <button className="block w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100">
                      Add Members
                    </button>
                  </>
                )}
                <button className="block w-full text-left px-4 py-2 text-sm text-red-600 hover:bg-gray-100">
                  Leave Chat
                </button>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
} 
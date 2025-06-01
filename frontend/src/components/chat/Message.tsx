'use client';

import { useState } from 'react';
import { format } from 'date-fns';
import { EllipsisHorizontalIcon, ArrowUturnLeftIcon } from '@heroicons/react/24/outline';
import type { Message as MessageType } from '@/types/chat';

interface MessageProps {
  message: MessageType;
  isOwnMessage: boolean;
  onReply: (message: MessageType) => void;
}

export default function Message({ message, isOwnMessage, onReply }: MessageProps) {
  const [showMenu, setShowMenu] = useState(false);

  const renderMessageContent = () => {
    switch (message.type) {
      case 'image':
        return (
          <img
            src={message.mediaUrl}
            alt="Shared image"
            className="max-w-sm rounded-lg"
          />
        );
      case 'video':
        return (
          <video
            src={message.mediaUrl}
            controls
            className="max-w-sm rounded-lg"
          />
        );
      case 'audio':
        return (
          <audio
            src={message.mediaUrl}
            controls
            className="w-full"
          />
        );
      default:
        return <p className="text-gray-900">{message.content}</p>;
    }
  };

  return (
    <div
      className={`flex ${isOwnMessage ? 'justify-end' : 'justify-start'}`}
      onMouseEnter={() => setShowMenu(true)}
      onMouseLeave={() => setShowMenu(false)}
    >
      <div className={`relative max-w-[70%] ${isOwnMessage ? 'order-2' : 'order-1'}`}>
        {message.replyTo && (
          <div className="mb-1 text-sm text-gray-500">
            Replying to: {message.replyTo.content}
          </div>
        )}
        <div
          className={`rounded-lg p-3 ${
            isOwnMessage
              ? 'bg-indigo-600 text-white'
              : 'bg-gray-100 text-gray-900'
          }`}
        >
          {renderMessageContent()}
          <div
            className={`text-xs mt-1 ${
              isOwnMessage ? 'text-indigo-200' : 'text-gray-500'
            }`}
          >
            {format(new Date(message.timestamp), 'HH:mm')}
          </div>
        </div>
        {showMenu && (
          <div
            className={`absolute top-0 ${
              isOwnMessage ? 'left-0 -translate-x-full' : 'right-0 translate-x-full'
            } flex items-center space-x-1`}
          >
            <button
              onClick={() => onReply(message)}
              className="p-1 text-gray-500 hover:text-gray-700 focus:outline-none"
              aria-label="Reply"
            >
              <ArrowUturnLeftIcon className="w-4 h-4" />
            </button>
            <button
              onClick={() => setShowMenu(false)}
              className="p-1 text-gray-500 hover:text-gray-700 focus:outline-none"
              aria-label="More options"
            >
              <EllipsisHorizontalIcon className="w-4 h-4" />
            </button>
          </div>
        )}
      </div>
    </div>
  );
} 
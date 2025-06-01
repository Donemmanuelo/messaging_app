import React from 'react';
import { MessageReactions } from './MessageReactions';

interface MessageProps {
  message: {
    id: string;
    content: string;
    created_at: string;
    sender: {
      id: string;
      username: string;
    };
    reply_to?: {
      id: string;
      content: string;
      sender: {
        id: string;
        username: string;
      };
    };
    reactions: Array<{
      emoji: string;
      count: number;
      users: Array<{
        id: string;
        username: string;
      }>;
    }>;
    read_by: Array<{
      id: string;
      username: string;
    }>;
  };
  currentUserId: string;
  onReply: (messageId: string) => void;
  onForward: (messageId: string) => void;
  onDelete: (messageId: string) => void;
}

export const Message: React.FC<MessageProps> = ({
  message,
  currentUserId,
  onReply,
  onForward,
  onDelete
}) => {
  const isOwnMessage = message.sender.id === currentUserId;

  return (
    <div
      className={`flex ${
        isOwnMessage ? 'justify-end' : 'justify-start'
      } mb-4`}
    >
      <div
        className={`max-w-[70%] ${
          isOwnMessage
            ? 'bg-blue-500 text-white'
            : 'bg-gray-100 text-gray-900'
        } rounded-lg px-4 py-2`}
      >
        {message.reply_to && (
          <div
            className={`text-sm mb-1 ${
              isOwnMessage ? 'text-blue-100' : 'text-gray-500'
            }`}
          >
            Replying to {message.reply_to.sender.username}:{' '}
            {message.reply_to.content}
          </div>
        )}

        <div className="flex items-start gap-2">
          {!isOwnMessage && (
            <span className="font-medium">
              {message.sender.username}
            </span>
          )}
          <p className="break-words">{message.content}</p>
        </div>

        <div className="flex items-center justify-between mt-1">
          <span
            className={`text-xs ${
              isOwnMessage ? 'text-blue-100' : 'text-gray-500'
            }`}
          >
            {new Date(message.created_at).toLocaleTimeString()}
          </span>

          <div className="flex items-center gap-2">
            <button
              onClick={() => onReply(message.id)}
              className={`p-1 rounded-full hover:bg-opacity-20 ${
                isOwnMessage
                  ? 'hover:bg-white'
                  : 'hover:bg-gray-900'
              }`}
            >
              <svg
                className="w-4 h-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M3 10h10a8 8 0 018 8v2M3 10l6 6m-6-6l6-6"
                />
              </svg>
            </button>

            <button
              onClick={() => onForward(message.id)}
              className={`p-1 rounded-full hover:bg-opacity-20 ${
                isOwnMessage
                  ? 'hover:bg-white'
                  : 'hover:bg-gray-900'
              }`}
            >
              <svg
                className="w-4 h-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"
                />
              </svg>
            </button>

            {isOwnMessage && (
              <button
                onClick={() => onDelete(message.id)}
                className="p-1 rounded-full hover:bg-white hover:bg-opacity-20"
              >
                <svg
                  className="w-4 h-4"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                  />
                </svg>
              </button>
            )}
          </div>
        </div>

        <MessageReactions
          messageId={message.id}
          reactions={message.reactions}
          currentUserId={currentUserId}
        />

        {isOwnMessage && message.read_by.length > 0 && (
          <div className="mt-1 text-xs text-blue-100">
            Read by:{' '}
            {message.read_by
              .map((user) => user.username)
              .join(', ')}
          </div>
        )}
      </div>
    </div>
  );
}; 
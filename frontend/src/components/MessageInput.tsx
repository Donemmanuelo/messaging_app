import React, { useState, useRef } from 'react';
import { MediaUpload } from './MediaUpload';

interface MessageInputProps {
  onSendMessage: (message: {
    content: string;
    mediaUrl?: string;
    mediaType?: string;
    replyToId?: string;
  }) => void;
  replyingTo?: {
    id: string;
    content: string;
    sender: {
      id: string;
      username: string;
    };
  };
  onCancelReply: () => void;
}

export const MessageInput: React.FC<MessageInputProps> = ({
  onSendMessage,
  replyingTo,
  onCancelReply,
}) => {
  const [message, setMessage] = useState('');
  const [showMediaUpload, setShowMediaUpload] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (message.trim() || showMediaUpload) {
      onSendMessage({
        content: message.trim(),
        replyToId: replyingTo?.id,
      });
      setMessage('');
      if (textareaRef.current) {
        textareaRef.current.style.height = 'auto';
      }
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e);
    }
  };

  const handleTextareaChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setMessage(e.target.value);
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = `${Math.min(
        textareaRef.current.scrollHeight,
        200
      )}px`;
    }
  };

  return (
    <div className="border-t p-4">
      {replyingTo && (
        <div className="mb-2 p-2 bg-gray-100 rounded-lg">
          <div className="flex items-center justify-between">
            <div className="text-sm">
              <span className="font-medium">
                Replying to {replyingTo.sender.username}
              </span>
              <p className="text-gray-600 truncate">
                {replyingTo.content}
              </p>
            </div>
            <button
              onClick={onCancelReply}
              className="text-gray-500 hover:text-gray-700"
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
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          </div>
        </div>
      )}

      {showMediaUpload && (
        <MediaUpload
          onUploadComplete={(url, type) => {
            onSendMessage({
              content: message.trim(),
              mediaUrl: url,
              mediaType: type,
              replyToId: replyingTo?.id,
            });
            setMessage('');
            setShowMediaUpload(false);
          }}
          onCancel={() => setShowMediaUpload(false)}
        />
      )}

      <form onSubmit={handleSubmit} className="flex items-end gap-2">
        <button
          type="button"
          onClick={() => setShowMediaUpload(true)}
          className="p-2 text-gray-500 hover:text-gray-700"
        >
          <svg
            className="w-6 h-6"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13"
            />
          </svg>
        </button>

        <div className="flex-1">
          <textarea
            ref={textareaRef}
            value={message}
            onChange={handleTextareaChange}
            onKeyDown={handleKeyDown}
            placeholder="Type a message..."
            className="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none"
            style={{ minHeight: '40px', maxHeight: '200px' }}
          />
        </div>

        <button
          type="submit"
          disabled={!message.trim() && !showMediaUpload}
          className="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Send
        </button>
      </form>
    </div>
  );
}; 
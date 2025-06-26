'use client';

import { useState, useRef, useEffect } from 'react';
import { PaperAirplaneIcon, PaperClipIcon, FaceSmileIcon } from '@heroicons/react/24/outline';
import type { MessageInputProps } from '@/types/chat';
import { encryptMessage, importPublicKey } from '@/lib/e2ee';

export default function MessageInput({ onSendMessage, onTyping, recipientId }: MessageInputProps) {
  const [message, setMessage] = useState('');
  const [isEmojiPickerOpen, setIsEmojiPickerOpen] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const publicKeyCache = useRef<{ [id: string]: string }>({});

  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = `${textareaRef.current.scrollHeight}px`;
    }
  }, [message]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (message.trim()) {
      const recipientPublicKeyBase64 = await getRecipientPublicKey(recipientId);
      const recipientPublicKey = await importPublicKey(recipientPublicKeyBase64);
      const encrypted = await encryptMessage(message.trim(), recipientPublicKey);
      onSendMessage(encrypted, 'text');
      setMessage('');
      if (textareaRef.current) {
        textareaRef.current.style.height = 'auto';
      }
    }
  };

  async function getRecipientPublicKey(userId: string) {
    if (publicKeyCache.current[userId]) {
      return publicKeyCache.current[userId];
    }
    const res = await fetch(`/api/users/${userId}/public_key`);
    if (!res.ok) throw new Error('Failed to fetch recipient public key');
    const data = await res.json();
    publicKeyCache.current[userId] = data.public_key;
    return data.public_key;
  }

  const handleFileSelect = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const formData = new FormData();
    formData.append('file', file);

    try {
      const response = await fetch('/api/media/upload', {
        method: 'POST',
        body: formData,
      });

      if (response.ok) {
        const data = await response.json();
        const type = file.type.startsWith('image/')
          ? 'image'
          : file.type.startsWith('video/')
          ? 'video'
          : 'audio';
        onSendMessage('', type, data.url);
      }
    } catch (error) {
      console.error('Failed to upload file:', error);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="flex items-end space-x-2">
      <button
        type="button"
        aria-label="Attach file"
        onClick={() => fileInputRef.current?.click()}
        className="p-2 text-gray-500 hover:text-gray-700 focus:outline-none"
      >
        <PaperClipIcon className="w-6 h-6" />
        <span className="sr-only">Attach file</span>
      </button>
      <input
        type="file"
        ref={fileInputRef}
        onChange={handleFileSelect}
        className="hidden"
        accept="image/*,video/*,audio/*"
        aria-label="Attach file"
      />

      <div className="flex-1 relative">
        <textarea
          ref={textareaRef}
          value={message}
          onChange={(e) => {
            setMessage(e.target.value);
            onTyping(e.target.value.length > 0);
          }}
          onKeyDown={handleKeyDown}
          placeholder="Type a message..."
          className="w-full resize-none rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent p-3 pr-10"
          rows={1}
        />
        <button
          type="button"
          aria-label="Emoji picker"
          onClick={() => setIsEmojiPickerOpen(!isEmojiPickerOpen)}
          className="absolute right-2 bottom-2 text-gray-500 hover:text-gray-700 focus:outline-none"
        >
          <FaceSmileIcon className="w-6 h-6" />
          <span className="sr-only">Emoji</span>
        </button>
      </div>

      <button
        type="submit"
        aria-label="Send"
        disabled={!message.trim()}
        className="p-2 text-indigo-600 hover:text-indigo-700 focus:outline-none disabled:opacity-50"
      >
        <PaperAirplaneIcon className="w-6 h-6" />
        <span className="sr-only">Send</span>
      </button>
    </form>
  );
} 
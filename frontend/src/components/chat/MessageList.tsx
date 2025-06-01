'use client';

import { useEffect, useRef } from 'react';
import { format } from 'date-fns';
import type { Message as MessageType } from '@/types/chat';
import Message from './Message';

interface MessageListProps {
  messages: MessageType[];
  currentUserId: string;
  onReply: (message: MessageType) => void;
}

export default function MessageList({ messages, currentUserId, onReply }: MessageListProps) {
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const groupMessagesByDate = (messages: MessageType[]) => {
    const groups: { [key: string]: MessageType[] } = {};
    
    messages.forEach((message) => {
      const date = format(new Date(message.timestamp), 'yyyy-MM-dd');
      if (!groups[date]) {
        groups[date] = [];
      }
      groups[date].push(message);
    });

    return groups;
  };

  const messageGroups = groupMessagesByDate(messages);

  return (
    <div className="space-y-4">
      {Object.entries(messageGroups).map(([date, messages]) => (
        <div key={date} className="space-y-4">
          <div className="flex justify-center">
            <span className="px-4 py-1 text-xs font-medium text-gray-500 bg-gray-100 rounded-full">
              {format(new Date(date), 'MMMM d, yyyy')}
            </span>
          </div>
          {messages.map((message, index) => {
            const isLastInGroup =
              index === messages.length - 1 ||
              messages[index + 1].senderId !== message.senderId;

            return (
              <Message
                key={message.id}
                message={message}
                isOwnMessage={message.senderId === currentUserId}
                showAvatar={isLastInGroup}
                onReply={() => onReply(message)}
              />
            );
          })}
        </div>
      ))}
      <div ref={messagesEndRef} />
    </div>
  );
} 
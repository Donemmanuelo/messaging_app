'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useAuthStore } from '@/stores/auth';
import type { Message, WebSocketMessage, Chat } from '@/types/chat';
import ChatHeader from '@/components/chat/ChatHeader';
import MessageList from '@/components/chat/MessageList';
import MessageInput from '@/components/chat/MessageInput';
import { useWebSocket } from '@/hooks/useWebSocket';

export default function ChatPage({ params }: { params: { chatId: string } }) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [chat, setChat] = useState<Chat | null>(null);
  const router = useRouter();
  const { user } = useAuthStore();

  const handleWebSocketMessage = (data: WebSocketMessage) => {
    switch (data.type) {
      case 'message':
        if (data.chatId === params.chatId && data.message) {
          setMessages((prev) => [...prev, data.message as Message]);
        }
        break;
      case 'typing':
        if (data.chatId === params.chatId) {
          // setIsTyping(data.isTyping || false);
        }
        break;
      case 'message_status':
        if (data.messageId && data.status) {
          setMessages((prev) =>
            prev.map((msg) =>
              msg.id === data.messageId
                ? { ...msg, status: data.status as Message['status'] }
                : msg
            )
          );
        }
        break;
    }
  };

  const { sendMessage } = useWebSocket(
    process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:8080',
    handleWebSocketMessage
  );

  useEffect(() => {
    if (!user) {
      router.push('/auth/login');
      return;
    }

    fetchMessages(params.chatId);
    fetchChatDetails(params.chatId);
  }, [user, router, params.chatId]);

  const fetchMessages = async (chatId: string) => {
    try {
      const response = await fetch(`/api/chats/${chatId}/messages`);
      if (response.ok) {
        const data = await response.json();
        setMessages(data);
      }
    } catch (error) {
      console.error('Failed to fetch messages:', error);
    }
  };

  const fetchChatDetails = async (chatId: string) => {
    try {
      const response = await fetch(`/api/chats/${chatId}`);
      if (response.ok) {
        const data = await response.json();
        setChat(data);
      }
    } catch (error) {
      console.error('Failed to fetch chat details:', error);
    }
  };

  const handleSendMessage = async (content: string, type: 'text' | 'image' | 'video' | 'audio', mediaUrl?: string) => {
    sendMessage({
      type: 'message',
      chatId: params.chatId,
      message: {
        id: Date.now().toString(),
        content,
        senderId: user?.id || '',
        timestamp: new Date().toISOString(),
        status: 'sent',
        type,
        mediaUrl
      }
    });
  };

  const handleTyping = (isTyping: boolean) => {
    sendMessage({
      type: 'typing',
      chatId: params.chatId,
      isTyping
    });
  };

  return (
    <div className="flex flex-col h-screen bg-gray-50">
      {chat && user && <ChatHeader chat={chat} currentUser={user} onStartCall={() => {}} onViewProfile={() => {}} />}
      <div className="flex-1 overflow-y-auto p-4">
        <MessageList
          messages={messages}
          currentUserId={user?.id || ''}
          onReply={(messageId) => {
            // Handle reply logic
          }}
        />
      </div>
      <div className="p-4 border-t border-gray-200 bg-white">
        <MessageInput
          onSendMessage={handleSendMessage}
          onTyping={handleTyping}
        />
      </div>
    </div>
  );
} 
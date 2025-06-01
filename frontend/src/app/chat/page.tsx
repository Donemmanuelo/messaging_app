'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useAuthStore } from '@/store/authStore';
import type { Message, WebSocketMessage } from '@/types/chat';
import ChatSidebar from '@/components/chat/ChatSidebar';
import ChatHeader from '@/components/chat/ChatHeader';
import MessageList from '@/components/chat/MessageList';
import MessageInput from '@/components/chat/MessageInput';

export default function ChatPage() {
  const [activeChatId, setActiveChatId] = useState<string>();
  const [messages, setMessages] = useState<Message[]>([]);
  const [isTyping, setIsTyping] = useState(false);
  const [ws, setWs] = useState<WebSocket | null>(null);
  const router = useRouter();
  const { user } = useAuthStore();

  useEffect(() => {
    if (!user) {
      router.push('/auth/login');
      return;
    }

    // Initialize WebSocket connection
    const websocket = new WebSocket(process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:8080');
    
    websocket.onopen = () => {
      console.log('WebSocket connected');
      websocket.send(JSON.stringify({
        type: 'auth',
        token: localStorage.getItem('token')
      }));
    };

    websocket.onmessage = (event) => {
      const data = JSON.parse(event.data);
      handleWebSocketMessage(data);
    };

    websocket.onclose = () => {
      console.log('WebSocket disconnected');
    };

    setWs(websocket);

    return () => {
      websocket.close();
    };
  }, [user, router]);

  useEffect(() => {
    if (activeChatId) {
      fetchMessages(activeChatId);
    }
  }, [activeChatId]);

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

  const handleWebSocketMessage = (data: WebSocketMessage) => {
    switch (data.type) {
      case 'message':
        if (data.chatId === activeChatId && data.message) {
          setMessages((prev) => [...prev, data.message as Message]);
        }
        break;
      case 'typing':
        if (data.chatId === activeChatId) {
          setIsTyping(data.isTyping || false);
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

  const handleSendMessage = async (content: string, type: 'text' | 'image' | 'video' | 'audio', mediaUrl?: string) => {
    if (!activeChatId || !ws) return;

    const message = {
      type: 'message',
      chatId: activeChatId,
      content,
      messageType: type,
      mediaUrl,
    };

    ws.send(JSON.stringify(message));
  };

  const handleTyping = (isTyping: boolean) => {
    if (!activeChatId || !ws) return;

    ws.send(JSON.stringify({
      type: 'typing',
      chatId: activeChatId,
      isTyping,
    }));
  };

  return (
    <div className="flex h-screen bg-gray-50">
      <ChatSidebar
        onChatSelect={setActiveChatId}
        activeChatId={activeChatId}
      />
      
      <div className="flex-1 flex flex-col">
        {activeChatId ? (
          <>
            <ChatHeader chatId={activeChatId} />
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
          </>
        ) : (
          <div className="flex-1 flex items-center justify-center">
            <div className="text-center">
              <h2 className="text-2xl font-semibold text-gray-900">
                Select a chat to start messaging
              </h2>
              <p className="mt-2 text-gray-500">
                Choose from your existing conversations or start a new one
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
} 
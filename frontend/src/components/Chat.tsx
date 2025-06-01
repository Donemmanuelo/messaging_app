import React, { useState, useEffect, useRef } from 'react';
import { Message } from './Message';
import { MessageInput } from './MessageInput';
import { ForwardMessage } from './ForwardMessage';
import { useWebSocket } from '../hooks/useWebSocket';
import type { WebSocketMessage } from '../types/chat';
import { MessageSearch } from './MessageSearch';

interface ChatMessage {
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
}

interface ChatProps {
  chatId: string;
  isGroup: boolean;
  currentUserId: string;
}

export const Chat: React.FC<ChatProps> = ({
  chatId,
  isGroup,
  currentUserId,
}) => {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [replyingTo, setReplyingTo] = useState<{
    id: string;
    content: string;
    sender: {
      id: string;
      username: string;
    };
  } | null>(null);
  const [forwardingMessages, setForwardingMessages] = useState<string[]>([]);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const { sendMessage: sendWebSocketMessage } = useWebSocket(chatId);

  useEffect(() => {
    fetchMessages(chatId);
  }, [chatId]);

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const fetchMessages = async (id: string) => {
    try {
      const response = await fetch(`/api/messages/${id}`);
      if (!response.ok) throw new Error('Failed to fetch messages');
      const data = await response.json();
      setMessages(data);
    } catch (error) {
      console.error('Error fetching messages:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  const handleSendMessage = async (content: string, mediaUrl?: string) => {
    const message = {
      chatId,
      content,
      mediaUrl,
      mediaType: mediaUrl ? getMediaType(mediaUrl) : undefined,
      replyToId: replyingTo?.id,
    };

    try {
      const response = await fetch('/api/messages', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(message),
      });

      if (!response.ok) throw new Error('Failed to send message');
      
      const newMessage = await response.json();
      setMessages((prev) => [...prev, newMessage]);
      setReplyingTo(null);

      const wsMessage: WebSocketMessage = {
        type: 'message',
        chatId,
        message: {
          id: newMessage.id,
          content: newMessage.content,
          senderId: newMessage.sender.id,
          timestamp: newMessage.timestamp.toISOString(),
          status: 'sent',
          type: newMessage.mediaType || 'text',
          mediaUrl: newMessage.mediaUrl,
        },
      };
      sendWebSocketMessage(wsMessage);
    } catch (error) {
      console.error('Error sending message:', error);
    }
  };

  const handleReply = (messageId: string) => {
    const message = messages.find((m) => m.id === messageId);
    if (message) {
      setReplyingTo({
        id: message.id,
        content: message.content,
        sender: message.sender,
      });
    }
  };

  const handleForward = async (messageId: string) => {
    setForwardingMessages([messageId]);
  };

  const handleForwardComplete = async (targetChatId: string) => {
    try {
      const response = await fetch('/api/messages/forward', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          messageIds: forwardingMessages,
          targetChatId,
        }),
      });

      if (!response.ok) throw new Error('Failed to forward messages');
      setForwardingMessages([]);
    } catch (error) {
      console.error('Error forwarding messages:', error);
    }
  };

  const handleDelete = (messageId: string) => {
    setMessages((prev) => prev.filter((msg) => msg.id !== messageId));
  };

  const getMediaType = (url: string): string => {
    const extension = url.split('.').pop()?.toLowerCase();
    if (!extension) return 'document';

    const imageExtensions = ['jpg', 'jpeg', 'png', 'gif', 'webp'];
    const videoExtensions = ['mp4', 'webm', 'mov'];
    const audioExtensions = ['mp3', 'wav', 'ogg'];

    if (imageExtensions.includes(extension)) return 'image';
    if (videoExtensions.includes(extension)) return 'video';
    if (audioExtensions.includes(extension)) return 'audio';
    return 'document';
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <div className="p-4 border-b">
        <MessageSearch chatId={chatId} />
      </div>

      <div className="flex-1 overflow-y-auto p-4">
        {messages.map((message) => (
          <Message
            key={message.id}
            message={message}
            currentUserId={currentUserId}
            onReply={handleReply}
            onForward={handleForward}
            onDelete={handleDelete}
            chatId={chatId}
          />
        ))}
        <div ref={messagesEndRef} />
      </div>

      {forwardingMessages.length > 0 && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <ForwardMessage
            messageIds={forwardingMessages}
            onForward={handleForwardComplete}
            onCancel={() => setForwardingMessages([])}
          />
        </div>
      )}

      <MessageInput
        onSendMessage={handleSendMessage}
        replyingTo={replyingTo || undefined}
        onCancelReply={() => setReplyingTo(null)}
      />
    </div>
  );
}; 
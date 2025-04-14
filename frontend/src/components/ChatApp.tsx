import React, { useEffect, useState, useCallback } from 'react';
import { useAuth } from '../contexts/AuthContext';
import ChatList from './ChatList';
import ChatWindow from './ChatWindow';
import UserList from './UserList';
import { WebSocketMessage, Chat, User, Message } from '../types';

const ChatApp: React.FC = () => {
  const { user, token } = useAuth();
  const [ws, setWs] = useState<WebSocket | null>(null);
  const [chats, setChats] = useState<Chat[]>([]);
  const [activeChat, setActiveChat] = useState<Chat | null>(null);
  const [users, setUsers] = useState<User[]>([]);
  const [messages, setMessages] = useState<Message[]>([]);
  const [error, setError] = useState<string | null>(null);

  // WebSocket connection management
  useEffect(() => {
    if (!user || !token) return;

    const wsUrl = `${process.env.REACT_APP_WS_URL}?token=${token}`;
    const socket = new WebSocket(wsUrl);

    socket.onopen = () => {
      console.log('WebSocket connected');
      setWs(socket);
    };

    socket.onclose = () => {
      console.log('WebSocket disconnected');
      setWs(null);
      // Attempt to reconnect after 5 seconds
      setTimeout(() => {
        setWs(new WebSocket(wsUrl));
      }, 5000);
    };

    socket.onerror = (error) => {
      console.error('WebSocket error:', error);
      setError('Connection error. Please try again.');
    };

    socket.onmessage = (event) => {
      const message: WebSocketMessage = JSON.parse(event.data);
      handleWebSocketMessage(message);
    };

    return () => {
      socket.close();
    };
  }, [user, token]);

  const handleWebSocketMessage = useCallback((message: WebSocketMessage) => {
    switch (message.type) {
      case 'message':
        setMessages(prev => [...prev, message.data]);
        break;
      case 'status':
        setUsers(prev => prev.map(u => 
          u.id === message.data.userId ? { ...u, status: message.data.status } : u
        ));
        break;
      case 'typing':
        // Handle typing indicator
        break;
      case 'read':
        // Handle read receipts
        break;
      default:
        console.warn('Unknown message type:', message.type);
    }
  }, []);

  const sendMessage = useCallback((content: string) => {
    if (!ws || !activeChat) return;

    const message = {
      type: 'message',
      data: {
        chatId: activeChat.id,
        content,
        senderId: user?.id,
      }
    };

    ws.send(JSON.stringify(message));
  }, [ws, activeChat, user]);

  const updateStatus = useCallback((status: string) => {
    if (!ws) return;

    const message = {
      type: 'status',
      data: { status }
    };

    ws.send(JSON.stringify(message));
  }, [ws]);

  if (error) {
    return <div className="error">{error}</div>;
  }

  return (
    <div className="chat-app">
      <div className="sidebar">
        <UserList users={users} />
        <ChatList 
          chats={chats} 
          activeChat={activeChat} 
          onSelectChat={setActiveChat} 
        />
      </div>
      <div className="main-content">
        {activeChat ? (
          <ChatWindow
            chat={activeChat}
            messages={messages}
            onSendMessage={sendMessage}
          />
        ) : (
          <div className="no-chat-selected">
            Select a chat to start messaging
          </div>
        )}
      </div>
    </div>
  );
};

export default ChatApp; 
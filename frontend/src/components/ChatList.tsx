import React from 'react';
import { Chat } from '../types';

interface ChatListProps {
  chats: Chat[];
  activeChat: Chat | null;
  onSelectChat: (chat: Chat) => void;
}

const ChatList: React.FC<ChatListProps> = ({ chats, activeChat, onSelectChat }) => {
  return (
    <div className="chat-list">
      <h2>Chats</h2>
      <div className="chat-items">
        {chats.map((chat) => (
          <div
            key={chat.id}
            className={`chat-item ${activeChat?.id === chat.id ? 'active' : ''}`}
            onClick={() => onSelectChat(chat)}
          >
            <div className="chat-name">{chat.name}</div>
            {chat.lastMessage && (
              <div className="last-message">
                <span className="sender">
                  {chat.lastMessage.senderId === chat.participants[0].id
                    ? chat.participants[0].username
                    : chat.participants[1].username}
                  :
                </span>
                <span className="content">{chat.lastMessage.content}</span>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};

export default ChatList; 
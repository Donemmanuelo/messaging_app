export interface User {
  id: string;
  username: string;
  displayName?: string;
  avatar?: string;
  status: 'online' | 'offline' | 'away';
  lastSeen?: string;
}

export interface Message {
  id: string;
  content: string;
  senderId: string;
  timestamp: string;
  status: 'sent' | 'delivered' | 'read';
  type: 'text' | 'image' | 'video' | 'audio';
  mediaUrl?: string;
  replyTo?: {
    id: string;
    content: string;
  };
}

export interface Chat {
  id: string;
  name: string;
  isGroup: boolean;
  participants: User[];
  lastMessage?: Message;
  unreadCount: number;
  createdAt: string;
  updatedAt: string;
}

export interface ChatParticipant {
  user: User;
  role: 'admin' | 'member';
  joinedAt: string;
}

export interface GroupChat extends Omit<Chat, 'participants'> {
  description?: string;
  avatar?: string;
  participants: ChatParticipant[];
}

export interface DirectChat extends Chat {
  participant: User;
}

export interface MessageInputProps {
  onSendMessage: (content: string, type: 'text' | 'image' | 'video' | 'audio', mediaUrl?: string) => void;
  onTyping: (isTyping: boolean) => void;
}

export interface MessageListProps {
  messages: Message[];
  currentUserId: string;
  onReply: (messageId: string) => void;
}

export interface MessageProps {
  message: Message;
  isOwnMessage: boolean;
  showAvatar: boolean;
  onReply: (messageId: string) => void;
}

export interface ChatHeaderProps {
  chatId: string;
}

export interface ChatSidebarProps {
  onChatSelect: (chatId: string) => void;
  activeChatId?: string;
}

export interface WebSocketMessage {
  type: 'message' | 'typing' | 'message_status' | 'user_status';
  chatId?: string;
  message?: Message;
  isTyping?: boolean;
  messageId?: string;
  status?: 'sent' | 'delivered' | 'read';
  userId?: string;
  userStatus?: 'online' | 'offline' | 'away';
} 
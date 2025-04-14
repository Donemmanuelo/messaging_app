export interface User {
  id: string;
  username: string;
  email: string;
  status: string;
  lastSeen?: Date;
}

export interface Chat {
  id: string;
  name: string;
  isGroup: boolean;
  participants: User[];
  lastMessage?: Message;
  createdAt: Date;
  updatedAt: Date;
}

export interface Message {
  id: string;
  chatId: string;
  senderId: string;
  content: string;
  createdAt: Date;
  readBy: string[];
}

export interface WebSocketMessage {
  type: 'message' | 'status' | 'typing' | 'read';
  data: any;
}

export interface AuthContextType {
  user: User | null;
  token: string | null;
  login: (email: string, password: string) => Promise<void>;
  register: (username: string, email: string, password: string) => Promise<void>;
  logout: () => void;
  refreshToken: () => Promise<void>;
} 
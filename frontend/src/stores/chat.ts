import { create } from 'zustand';
import type { Chat, Message, User } from '@/types/chat';

interface ChatState {
  chats: Chat[];
  activeChat: Chat | null;
  messages: Message[];
  typingUsers: Set<string>;
  setChats: (chats: Chat[]) => void;
  setActiveChat: (chat: Chat | null) => void;
  addMessage: (message: Message, chatId: string) => void;
  updateMessage: (messageId: string, updates: Partial<Message>) => void;
  setTypingUser: (userId: string, isTyping: boolean) => void;
  updateUserStatus: (userId: string, status: User['status']) => void;
}

export const useChatStore = create<ChatState>((set) => ({
  chats: [],
  activeChat: null,
  messages: [],
  typingUsers: new Set(),

  setChats: (chats) => set({ chats }),
  setActiveChat: (chat) => set({ activeChat: chat }),

  addMessage: (message, chatId) =>
    set((state) => ({
      messages: [...state.messages, message],
      chats: state.chats.map((chat) =>
        chat.id === chatId
          ? {
              ...chat,
              lastMessage: message,
              unreadCount: chat.id === state.activeChat?.id ? 0 : chat.unreadCount + 1,
            }
          : chat
      ),
    })),

  updateMessage: (messageId, updates) =>
    set((state) => ({
      messages: state.messages.map((message) =>
        message.id === messageId ? { ...message, ...updates } : message
      ),
    })),

  setTypingUser: (userId, isTyping) =>
    set((state) => {
      const newTypingUsers = new Set(state.typingUsers);
      if (isTyping) {
        newTypingUsers.add(userId);
      } else {
        newTypingUsers.delete(userId);
      }
      return { typingUsers: newTypingUsers };
    }),

  updateUserStatus: (userId, status) =>
    set((state) => ({
      chats: state.chats.map((chat) => ({
        ...chat,
        participants: chat.participants.map((participant) =>
          participant.id === userId ? { ...participant, status } : participant
        ),
      })),
    })),
})); 
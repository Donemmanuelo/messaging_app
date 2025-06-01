import { create } from 'zustand';

interface ChatState {
  messages: any[];
  typing: Record<string, Record<string, boolean>>; // chatId -> senderId -> isTyping
  addMessage: (message: any) => void;
  setTyping: (chatId: string, senderId: string, isTyping: boolean) => void;
}

export const useChatStore = create<ChatState>((set) => ({
  messages: [],
  typing: {},
  addMessage: (message) => set((state) => ({ messages: [...state.messages, message] })),
  setTyping: (chatId, senderId, isTyping) => set((state) => ({
    typing: {
      ...state.typing,
      [chatId]: {
        ...(state.typing[chatId] || {}),
        [senderId]: isTyping,
      },
    },
  })),
})); 
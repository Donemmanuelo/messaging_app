import { create } from 'zustand';
import type { WebSocketMessage } from '@/types/chat';

interface WebSocketState {
  socket: WebSocket | null;
  isConnected: boolean;
  connect: () => void;
  disconnect: () => void;
  sendMessage: (message: WebSocketMessage) => void;
}

export const useWebSocketStore = create<WebSocketState>((set, get) => ({
  socket: null,
  isConnected: false,

  connect: () => {
    const socket = new WebSocket(process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:3001');

    socket.onopen = () => {
      set({ isConnected: true });
    };

    socket.onclose = () => {
      set({ isConnected: false, socket: null });
    };

    socket.onerror = (error) => {
      console.error('WebSocket error:', error);
      set({ isConnected: false, socket: null });
    };

    set({ socket });
  },

  disconnect: () => {
    const { socket } = get();
    if (socket) {
      socket.close();
      set({ socket: null, isConnected: false });
    }
  },

  sendMessage: (message: WebSocketMessage) => {
    const { socket, isConnected } = get();
    if (socket && isConnected) {
      socket.send(JSON.stringify(message));
    }
  },
})); 
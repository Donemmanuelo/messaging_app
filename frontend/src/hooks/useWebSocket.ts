import { useState, useEffect, useCallback } from 'react';
import type { WebSocketMessage } from '@/types/chat';

export function useWebSocket(url: string, onMessage?: (data: WebSocketMessage) => void) {
  const [ws, setWs] = useState<WebSocket | null>(null);
  const [isConnected, setIsConnected] = useState(false);

  useEffect(() => {
    const websocket = new WebSocket(url);
    
    websocket.onopen = () => {
      console.log('WebSocket connected');
      setIsConnected(true);
      websocket.send(JSON.stringify({
        type: 'auth',
        token: localStorage.getItem('token')
      }));
    };

    websocket.onmessage = (event) => {
      const data = JSON.parse(event.data) as WebSocketMessage;
      onMessage?.(data);
    };

    websocket.onclose = () => {
      console.log('WebSocket disconnected');
      setIsConnected(false);
    };

    setWs(websocket);

    return () => {
      websocket.close();
    };
  }, [url, onMessage]);

  const sendMessage = useCallback((message: WebSocketMessage) => {
    if (ws && isConnected) {
      ws.send(JSON.stringify(message));
    }
  }, [ws, isConnected]);

  return {
    ws,
    isConnected,
    sendMessage
  };
} 
import { useEffect, useRef, useState, useCallback } from "react";
import type { WebSocketMessage } from '@/types/chat';

export function useWebSocket(url: string) {
  const ws = useRef<WebSocket | null>(null);
  const [incomingCall, setIncomingCall] = useState<null | { caller: any; callType: "audio" | "video"; offer: any }>(null);

  useEffect(() => {
    ws.current = new WebSocket(url);
    ws.current.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        if (msg.type === "call-offer") {
          setIncomingCall({
            caller: msg.caller,
            callType: msg.callType,
            offer: msg.offer,
          });
        }
        // ...handle other message types...
      } catch {}
    };
    return () => {
      ws.current?.close();
    };
  }, [url]);

  const send = useCallback((data: any) => {
    ws.current?.send(JSON.stringify(data));
  }, []);

  return { ws: ws.current, send, incomingCall, setIncomingCall };
} 
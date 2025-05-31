import { useEffect, useRef } from 'react'
import { useAuthStore } from '@/stores/authStore'
import { useChatStore } from '@/stores/chatStore'

interface WebSocketMessage {
  message_type: string
  chat_id?: string
  sender_id: string
  content?: string
  timestamp: string
}

export const useWebSocket = () => {
  const ws = useRef<WebSocket | null>(null)
  const { user } = useAuthStore()
  const { addMessage, setTyping } = useChatStore()

  useEffect(() => {
    if (!user) return

    const WS_URL = process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:8000'
    ws.current = new WebSocket(`${WS_URL}/ws`)

    ws.current.onopen = () => {
      console.log('WebSocket connected')
    }

    ws.current.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data)
        
        switch (data.message_type) {
          case 'message':
            addMessage(data)
            break
          case 'typing':
            setTyping(data.chat_id, data.sender_id, data.is_typing)
            break
          default:
            console.log('Unknown message type:', data.message_type)
        }
      } catch (error) {
        console.error('Error parsing WebSocket message:', error)
      }
    }

    ws.current.onclose = () => {
      console.log('WebSocket disconnected')
    }

    ws.current.onerror = (error) => {
      console.error('WebSocket error:', error)
    }

    return () => {
      if (ws.current) {
        ws.current.close()
      }
    }
  }, [user, addMessage, setTyping])

  const sendMessage = (message: WebSocketMessage) => {
    if (ws.current && ws.current.readyState === WebSocket.OPEN) {
      ws.current.send(JSON.stringify(message))
    }
  }

  return { sendMessage }
}
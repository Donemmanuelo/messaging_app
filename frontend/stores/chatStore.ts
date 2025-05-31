import { create } from 'zustand'
import axios from 'axios'
import toast from 'react-hot-toast'

interface Message {
  id: string
  chat_id: string
  sender: {
    id: string
    username: string
    display_name?: string
    avatar_url?: string
  }
  content?: string
  message_type: 'text' | 'image' | 'video' | 'audio' | 'document'
  media_url?: string
  reply_to?: Message
  status: 'sent' | 'delivered' | 'read'
  created_at: string
}

interface Chat {
  id: string
  chat_type: 'direct' | 'group'
  name?: string
  avatar_url?: string
  participants: Array<{
    id: string
    username: string
    display_name?: string
    avatar_url?: string
    is_online: boolean
  }>
  last_message?: Message
  unread_count: number
  updated_at: string
}

interface ChatState {
  chats: Chat[]
  activeChat: Chat | null
  messages: { [chatId: string]: Message[] }
  isLoading: boolean
  typingUsers: { [chatId: string]: string[] }
  
  // Actions
  fetchChats: () => Promise<void>
  fetchMessages: (chatId: string) => Promise<void>
  sendMessage: (chatId: string, content: string, messageType?: string) => Promise<void>
  setActiveChat: (chat: Chat | null) => void
  addMessage: (message: Message) => void
  updateMessageStatus: (messageId: string, status: string) => void
  setTyping: (chatId: string, userId: string, isTyping: boolean) => void
}

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8000'

export const useChatStore = create<ChatState>((set, get) => ({
  chats: [],
  activeChat: null,
  messages: {},
  isLoading: false,
  typingUsers: {},

  fetchChats: async () => {
    set({ isLoading: true })
    try {
      const response = await axios.get(`${API_URL}/api/chats`)
      set({ chats: response.data, isLoading: false })
    } catch (error: any) {
      set({ isLoading: false })
      toast.error('Failed to fetch chats')
    }
  },

  fetchMessages: async (chatId: string) => {
    try {
      const response = await axios.get(`${API_URL}/api/chats/${chatId}/messages`)
      set((state) => ({
        messages: {
          ...state.messages,
          [chatId]: response.data,
        },
      }))
    } catch (error: any) {
      toast.error('Failed to fetch messages')
    }
  },

  sendMessage: async (chatId: string, content: string, messageType = 'text') => {
    try {
      const response = await axios.post(`${API_URL}/api/chats/${chatId}/messages`, {
        content,
        message_type: messageType,
      })
      
      // Message will be added via WebSocket
    } catch (error: any) {
      toast.error('Failed to send message')
    }
  },

  setActiveChat: (chat: Chat | null) => {
    set({ activeChat: chat })
  },

  addMessage: (message: Message) => {
    set((state) => ({
      messages: {
        ...state.messages,
        [message.chat_id]: [
          ...(state.messages[message.chat_id] || []),
          message,
        ],
      },
    }))
  },

  updateMessageStatus: (messageId: string, status: string) => {
    set((state) => {
      const newMessages = { ...state.messages }
      Object.keys(newMessages).forEach((chatId) => {
        newMessages[chatId] = newMessages[chatId].map((msg) =>
          msg.id === messageId ? { ...msg, status: status as any } : msg
        )
      })
      return { messages: newMessages }
    })
  },

  setTyping: (chatId: string, userId: string, isTyping: boolean) => {
    set((state) => {
      const currentTyping = state.typingUsers[chatId] || []
      let newTyping: string[]
      
      if (isTyping) {
        newTyping = currentTyping.includes(userId) 
          ? currentTyping 
          : [...currentTyping, userId]
      } else {
        newTyping = currentTyping.filter(id => id !== userId)
      }
      
      return {
        typingUsers: {
          ...state.typingUsers,
          [chatId]: newTyping,
        },
      }
    })
  },
}))
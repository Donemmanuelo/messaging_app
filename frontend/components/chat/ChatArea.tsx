'use client'

import { useEffect, useRef } from 'react'
import { useChatStore } from '@/stores/chatStore'
import { useAuthStore } from '@/stores/authStore'
import ChatHeader from './ChatHeader'
import MessageList from './MessageList'
import MessageInput from './MessageInput'

export default function ChatArea() {
  const { activeChat, messages } = useChatStore()
  const { user } = useAuthStore()
  const messagesEndRef = useRef<HTMLDivElement>(null)

  const chatMessages = activeChat ? messages[activeChat.id] || [] : []

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [chatMessages])

  if (!activeChat) {
    return null
  }

  return (
    <div className="flex flex-col h-full">
      <ChatHeader chat={activeChat} />
      
      <div className="flex-1 overflow-y-auto bg-chat-pattern bg-whatsapp-gray-light">
        <MessageList messages={chatMessages} currentUserId={user?.id || ''} />
        <div ref={messagesEndRef} />
      </div>
      
      <MessageInput chatId={activeChat.id} />
    </div>
  )
}
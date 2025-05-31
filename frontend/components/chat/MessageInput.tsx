'use client'

import { useState, useRef } from 'react'
import { PaperAirplaneIcon, PaperClipIcon, FaceSmileIcon } from '@heroicons/react/24/outline'
import { useChatStore } from '@/stores/chatStore'
import { useWebSocket } from '@/hooks/useWebSocket'
import EmojiPicker from 'emoji-picker-react'

interface MessageInputProps {
  chatId: string
}

export default function MessageInput({ chatId }: MessageInputProps) {
  const [message, setMessage] = useState('')
  const [showEmojiPicker, setShowEmojiPicker] = useState(false)
  const [isTyping, setIsTyping] = useState(false)
  const { sendMessage: sendMessageToStore } = useChatStore()
  const { sendMessage: sendWebSocketMessage } = useWebSocket()
  const fileInputRef = useRef<HTMLInputElement>(null)
  const typingTimeoutRef = useRef<NodeJS.Timeout>()

  const handleSendMessage = async () => {
    if (!message.trim()) return

    try {
      await sendMessageToStore(chatId, message.trim())
      setMessage('')
      setIsTyping(false)
    } catch (error) {
      console.error('Failed to send message:', error)
    }
  }

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSendMessage()
    }
  }

  const handleInputChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setMessage(e.target.value)
    
    // Handle typing indicator
    if (!isTyping) {
      setIsTyping(true)
      sendWebSocketMessage({
        message_type: 'typing',
        chat_id: chatId,
        sender_id: '',
        timestamp: new Date().toISOString()
      })
    }

    // Clear existing timeout
    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current)
    }

    // Set new timeout to stop typing indicator
    typingTimeoutRef.current = setTimeout(() => {
      setIsTyping(false)
      sendWebSocketMessage({
        message_type: 'typing',
        chat_id: chatId,
        sender_id: '',
        timestamp: new Date().toISOString()
      })
    }, 1000)
  }

  const handleEmojiClick = (emojiData: any) => {
    setMessage(prev => prev + emojiData.emoji)
    setShowEmojiPicker(false)
  }

  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (file) {
      // Handle file upload logic here
      console.log('File selected:', file)
    }
  }

  return (
    <div className="p-4 bg-white border-t">
      <div className="flex items-end space-x-2">
        <button
          onClick={() => fileInputRef.current?.click()}
          className="p-2 rounded-full hover:bg-gray-100"
        >
          <PaperClipIcon className="w-6 h-6 text-gray-600" />
        </button>
        
        <div className="flex-1 relative">
          <textarea
            value={message}
            onChange={handleInputChange}
            onKeyPress={handleKeyPress}
            placeholder="Type a message..."
            className="w-full px-4 py-2 pr-12 border border-gray-300 rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-whatsapp-green focus:border-transparent"
            rows={1}
            style={{ minHeight: '40px', maxHeight: '120px' }}
          />
          
          <button
            onClick={() => setShowEmojiPicker(!showEmojiPicker)}
            className="absolute right-2 top-1/2 transform -translate-y-1/2 p-1 rounded-full hover:bg-gray-100"
          >
            <FaceSmileIcon className="w-6 h-6 text-gray-600" />
          </button>
          
          {showEmojiPicker && (
            <div className="absolute bottom-full right-0 mb-2 z-10">
              <EmojiPicker onEmojiClick={handleEmojiClick} />
            </div>
          )}
        </div>
        
        <button
          onClick={handleSendMessage}
          disabled={!message.trim()}
          className="p-2 rounded-full bg-whatsapp-green text-white hover:bg-whatsapp-green-dark disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <PaperAirplaneIcon className="w-6 h-6" />
        </button>
      </div>
      
      <input
        ref={fileInputRef}
        type="file"
        onChange={handleFileUpload}
        className="hidden"
        accept="image/*,video/*,audio/*,.pdf,.doc,.docx"
      />
    </div>
  )
}
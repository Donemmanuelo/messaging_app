'use client'

import { format, isToday, isYesterday } from 'date-fns'
import { CheckIcon, CheckCheckIcon } from '@heroicons/react/24/outline'

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

interface MessageListProps {
  messages: Message[]
  currentUserId: string
}

export default function MessageList({ messages, currentUserId }: MessageListProps) {
  const formatMessageTime = (timestamp: string) => {
    const date = new Date(timestamp)
    if (isToday(date)) {
      return format(date, 'HH:mm')
    } else if (isYesterday(date)) {
      return `Yesterday ${format(date, 'HH:mm')}`
    } else {
      return format(date, 'dd/MM/yyyy HH:mm')
    }
  }

  const renderMessageStatus = (status: string) => {
    switch (status) {
      case 'sent':
        return <CheckIcon className="w-4 h-4 text-gray-400" />
      case 'delivered':
        return <CheckCheckIcon className="w-4 h-4 text-gray-400" />
      case 'read':
        return <CheckCheckIcon className="w-4 h-4 text-blue-500" />
      default:
        return null
    }
  }

  const groupMessagesByDate = (messages: Message[]) => {
    const groups: { [key: string]: Message[] } = {}
    
    messages.forEach(message => {
      const date = new Date(message.created_at)
      let dateKey: string
      
      if (isToday(date)) {
        dateKey = 'Today'
      } else if (isYesterday(date)) {
        dateKey = 'Yesterday'
      } else {
        dateKey = format(date, 'dd/MM/yyyy')
      }
      
      if (!groups[dateKey]) {
        groups[dateKey] = []
      }
      groups[dateKey].push(message)
    })
    
    return groups
  }

  const messageGroups = groupMessagesByDate(messages)

  return (
    <div className="p-4 space-y-4">
      {Object.entries(messageGroups).map(([date, dateMessages]) => (
        <div key={date}>
          {/* Date separator */}
          <div className="flex justify-center mb-4">
            <span className="bg-white px-3 py-1 rounded-lg text-sm text-gray-600 shadow-sm">
              {date}
            </span>
          </div>
          
          {/* Messages for this date */}
          {dateMessages.map((message) => {
            const isOwnMessage = message.sender.id === currentUserId
            
            return (
              <div
                key={message.id}
                className={`flex ${isOwnMessage ? 'justify-end' : 'justify-start'} mb-2`}
              >
                <div
                  className={`max-w-xs lg:max-w-md px-4 py-2 rounded-lg message-bubble ${
                    isOwnMessage
                      ? 'bg-whatsapp-green text-white'
                      : 'bg-white text-gray-900 shadow-sm'
                  }`}
                >
                  {message.reply_to && (
                    <div className="border-l-4 border-gray-300 pl-2 mb-2 text-sm opacity-75">
                      <p className="font-medium">{message.reply_to.sender.username}</p>
                      <p className="truncate">{message.reply_to.content}</p>
                    </div>
                  )}
                  
                  {message.message_type === 'text' && (
                    <p className="whitespace-pre-wrap break-words">{message.content}</p>
                  )}
                  
                  {message.message_type === 'image' && (
                    <div>
                      <img
                        src={message.media_url}
                        alt="Shared image"
                        className="rounded-lg max-w-full h-auto"
                      />
                      {message.content && (
                        <p className="mt-2 whitespace-pre-wrap break-words">{message.content}</p>
                      )}
                    </div>
                  )}
                  
                  <div className={`flex items-center justify-end mt-1 space-x-1 text-xs ${
                    isOwnMessage ? 'text-green-100' : 'text-gray-500'
                  }`}>
                    <span>{formatMessageTime(message.created_at)}</span>
                    {isOwnMessage && renderMessageStatus(message.status)}
                  </div>
                </div>
              </div>
            )
          })}
        </div>
      ))}
    </div>
  )
}
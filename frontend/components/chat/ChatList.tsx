'use client'

import { useChatStore } from '@/stores/chatStore'
import { formatDistanceToNow } from 'date-fns'

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
  last_message?: {
    content?: string
    created_at: string
    sender: {
      username: string
    }
  }
  unread_count: number
  updated_at: string
}

interface ChatListProps {
  chats: Chat[]
}

export default function ChatList({ chats }: ChatListProps) {
  const { setActiveChat, activeChat, fetchMessages } = useChatStore()

  const handleChatClick = (chat: Chat) => {
    setActiveChat(chat)
    fetchMessages(chat.id)
  }

  const getChatName = (chat: Chat) => {
    if (chat.chat_type === 'group') {
      return chat.name || 'Group Chat'
    }
    const otherParticipant = chat.participants[0]
    return otherParticipant?.display_name || otherParticipant?.username || 'Unknown'
  }

  const getChatAvatar = (chat: Chat) => {
    if (chat.avatar_url) return chat.avatar_url
    if (chat.chat_type === 'group') return '/group-avatar.png'
    return chat.participants[0]?.avatar_url || '/default-avatar.png'
  }

  if (chats.length === 0) {
    return (
      <div className="flex items-center justify-center h-32 text-gray-500">
        No chats found
      </div>
    )
  }

  return (
    <div>
      {chats.map((chat) => (
        <div
          key={chat.id}
          onClick={() => handleChatClick(chat)}
          className={`flex items-center p-3 hover:bg-gray-50 cursor-pointer border-b border-gray-100 ${
            activeChat?.id === chat.id ? 'bg-whatsapp-green-light' : ''
          }`}
        >
          <div className="relative">
            <img
              src={getChatAvatar(chat)}
              alt={getChatName(chat)}
              className="w-12 h-12 rounded-full"
            />
            {chat.chat_type === 'direct' && chat.participants[0]?.is_online && (
              <div className="absolute bottom-0 right-0 w-3 h-3 bg-green-500 rounded-full border-2 border-white"></div>
            )}
          </div>
          
          <div className="ml-3 flex-1 min-w-0">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-medium text-gray-900 truncate">
                {getChatName(chat)}
              </h3>
              {chat.last_message && (
                <span className="text-xs text-gray-500">
                  {formatDistanceToNow(new Date(chat.last_message.created_at), { addSuffix: true })}
                </span>
              )}
            </div>
            
            <div className="flex items-center justify-between mt-1">
              <p className="text-sm text-gray-600 truncate">
                {chat.last_message ? (
                  <>
                    {chat.chat_type === 'group' && (
                      <span className="font-medium">
                        {chat.last_message.sender.username}:{' '}
                      </span>
                    )}
                    {chat.last_message.content || 'Media'}
                  </>
                ) : (
                  'No messages yet'
                )}
              </p>
              
              {chat.unread_count > 0 && (
                <span className="bg-whatsapp-green text-white text-xs rounded-full px-2 py-1 min-w-[20px] text-center">
                  {chat.unread_count}
                </span>
              )}
            </div>
          </div>
        </div>
      ))}
    </div>
  )
}
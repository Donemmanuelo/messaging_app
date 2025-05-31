'use client'

import { PhoneIcon, VideoCameraIcon, EllipsisVerticalIcon } from '@heroicons/react/24/outline'

interface ChatHeaderProps {
  chat: {
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
  }
}

export default function ChatHeader({ chat }: ChatHeaderProps) {
  const getChatName = () => {
    if (chat.chat_type === 'group') {
      return chat.name || 'Group Chat'
    }
    const otherParticipant = chat.participants[0]
    return otherParticipant?.display_name || otherParticipant?.username || 'Unknown'
  }

  const getChatAvatar = () => {
    if (chat.avatar_url) return chat.avatar_url
    if (chat.chat_type === 'group') return '/group-avatar.png'
    return chat.participants[0]?.avatar_url || '/default-avatar.png'
  }

  const getOnlineStatus = () => {
    if (chat.chat_type === 'group') {
      const onlineCount = chat.participants.filter(p => p.is_online).length
      return `${onlineCount} online`
    }
    return chat.participants[0]?.is_online ? 'Online' : 'Offline'
  }

  return (
    <div className="flex items-center justify-between p-4 bg-whatsapp-gray-light border-b">
      <div className="flex items-center space-x-3">
        <img
          src={getChatAvatar()}
          alt={getChatName()}
          className="w-10 h-10 rounded-full"
        />
        <div>
          <h2 className="font-medium text-gray-900">{getChatName()}</h2>
          <p className="text-sm text-gray-600">{getOnlineStatus()}</p>
        </div>
      </div>
      
      <div className="flex items-center space-x-2">
        <button className="p-2 rounded-full hover:bg-gray-200">
          <PhoneIcon className="w-5 h-5 text-gray-600" />
        </button>
        <button className="p-2 rounded-full hover:bg-gray-200">
          <VideoCameraIcon className="w-5 h-5 text-gray-600" />
        </button>
        <button className="p-2 rounded-full hover:bg-gray-200">
          <EllipsisVerticalIcon className="w-5 h-5 text-gray-600" />
        </button>
      </div>
    </div>
  )
}
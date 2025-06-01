import React, { useState, useEffect } from 'react';

interface ForwardMessageProps {
  messageIds: string[];
  onForward: (targetChatId: string) => void;
  onCancel: () => void;
}

interface Chat {
  id: string;
  name: string;
  isGroup: boolean;
  participants: {
    id: string;
    username: string;
  }[];
}

export const ForwardMessage: React.FC<ForwardMessageProps> = ({
  messageIds,
  onForward,
  onCancel,
}) => {
  const [chats, setChats] = useState<Chat[]>([]);
  const [selectedChatId, setSelectedChatId] = useState<string>('');
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    fetchChats();
  }, []);

  const fetchChats = async () => {
    try {
      const response = await fetch('/api/chats');
      if (!response.ok) throw new Error('Failed to fetch chats');
      const data = await response.json();
      setChats(data);
    } catch (error) {
      console.error('Error fetching chats:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleForward = () => {
    if (selectedChatId) {
      onForward(selectedChatId);
    }
  };

  if (isLoading) {
    return (
      <div className="p-4">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  return (
    <div className="p-4 bg-white rounded-lg shadow-lg">
      <h3 className="text-lg font-semibold mb-4">Forward Message</h3>
      
      <div className="mb-4">
        <label className="block text-sm font-medium text-gray-700 mb-1">
          Select Chat
        </label>
        <select
          value={selectedChatId}
          onChange={(e) => setSelectedChatId(e.target.value)}
          className="w-full px-3 py-2 border rounded-md"
        >
          <option value="">Select a chat...</option>
          {chats.map((chat) => (
            <option key={chat.id} value={chat.id}>
              {chat.isGroup ? chat.name : chat.participants[0].username}
            </option>
          ))}
        </select>
      </div>

      <div className="flex justify-end space-x-2">
        <button
          onClick={onCancel}
          className="px-4 py-2 border rounded-md hover:bg-gray-50"
        >
          Cancel
        </button>
        <button
          onClick={handleForward}
          disabled={!selectedChatId}
          className="px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Forward
        </button>
      </div>
    </div>
  );
}; 
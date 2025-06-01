import React, { useState } from 'react';
import { useRouter } from 'next/navigation';

export default function CreateGroupPage() {
  const router = useRouter();
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<any[]>([]);
  const [selectedUsers, setSelectedUsers] = useState<string[]>([]);

  const handleSearch = async (query: string) => {
    if (!query) {
      setSearchResults([]);
      return;
    }

    try {
      const response = await fetch(`/api/users/search?q=${encodeURIComponent(query)}`);
      const data = await response.json();
      setSearchResults(data.filter((user: any) => !selectedUsers.includes(user.id)));
    } catch (error) {
      console.error('Error searching users:', error);
    }
  };

  const handleCreateGroup = async () => {
    if (!name || selectedUsers.length === 0) {
      return;
    }

    try {
      const response = await fetch('/api/groups', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          name,
          description,
          memberIds: selectedUsers,
        }),
      });

      const data = await response.json();
      router.push(`/chat/group/${data.id}`);
    } catch (error) {
      console.error('Error creating group:', error);
    }
  };

  return (
    <div className="max-w-2xl mx-auto p-4">
      <h1 className="text-2xl font-bold mb-6">Create New Group</h1>

      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Group Name
          </label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="w-full px-3 py-2 border rounded-md"
            placeholder="Enter group name"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Description (Optional)
          </label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            className="w-full px-3 py-2 border rounded-md"
            placeholder="Enter group description"
            rows={3}
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Add Members
          </label>
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => {
              setSearchQuery(e.target.value);
              handleSearch(e.target.value);
            }}
            className="w-full px-3 py-2 border rounded-md mb-2"
            placeholder="Search users..."
          />

          {searchResults.length > 0 && (
            <div className="border rounded-md max-h-48 overflow-y-auto">
              {searchResults.map((user) => (
                <div
                  key={user.id}
                  className="flex items-center space-x-2 p-2 hover:bg-gray-50 cursor-pointer"
                  onClick={() => {
                    if (selectedUsers.includes(user.id)) {
                      setSelectedUsers(selectedUsers.filter(id => id !== user.id));
                    } else {
                      setSelectedUsers([...selectedUsers, user.id]);
                    }
                  }}
                >
                  <input
                    type="checkbox"
                    checked={selectedUsers.includes(user.id)}
                    onChange={() => {}}
                    className="mr-2"
                  />
                  <div className="w-8 h-8 rounded-full bg-gray-200 overflow-hidden">
                    {user.avatarUrl ? (
                      <img
                        src={user.avatarUrl}
                        alt={user.displayName || user.username}
                        className="w-full h-full object-cover"
                      />
                    ) : (
                      <div className="w-full h-full flex items-center justify-center text-gray-500">
                        {(user.displayName || user.username).charAt(0).toUpperCase()}
                      </div>
                    )}
                  </div>
                  <div>
                    <div className="font-medium">{user.displayName || user.username}</div>
                    <div className="text-sm text-gray-500">@{user.username}</div>
                  </div>
                </div>
              ))}
            </div>
          )}

          {selectedUsers.length > 0 && (
            <div className="mt-4">
              <h3 className="text-sm font-medium text-gray-700 mb-2">Selected Members</h3>
              <div className="space-y-2">
                {selectedUsers.map((userId) => {
                  const user = searchResults.find(u => u.id === userId);
                  if (!user) return null;
                  return (
                    <div key={userId} className="flex items-center justify-between p-2 bg-gray-50 rounded-md">
                      <div className="flex items-center space-x-2">
                        <div className="w-8 h-8 rounded-full bg-gray-200 overflow-hidden">
                          {user.avatarUrl ? (
                            <img
                              src={user.avatarUrl}
                              alt={user.displayName || user.username}
                              className="w-full h-full object-cover"
                            />
                          ) : (
                            <div className="w-full h-full flex items-center justify-center text-gray-500">
                              {(user.displayName || user.username).charAt(0).toUpperCase()}
                            </div>
                          )}
                        </div>
                        <div>
                          <div className="font-medium">{user.displayName || user.username}</div>
                          <div className="text-sm text-gray-500">@{user.username}</div>
                        </div>
                      </div>
                      <button
                        onClick={() => setSelectedUsers(selectedUsers.filter(id => id !== userId))}
                        className="text-red-500 hover:text-red-600"
                      >
                        Remove
                      </button>
                    </div>
                  );
                })}
              </div>
            </div>
          )}
        </div>

        <div className="flex justify-end space-x-2">
          <button
            onClick={() => router.back()}
            className="px-4 py-2 border rounded-md hover:bg-gray-50"
          >
            Cancel
          </button>
          <button
            onClick={handleCreateGroup}
            disabled={!name || selectedUsers.length === 0}
            className="px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Create Group
          </button>
        </div>
      </div>
    </div>
  );
} 
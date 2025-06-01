import React, { useState } from 'react';

interface GroupMembersProps {
  members: Array<{
    id: string;
    userId: string;
    role: string;
    user: {
      id: string;
      username: string;
      displayName?: string;
      avatarUrl?: string;
    };
  }>;
  currentUser: {
    id: string;
    role: string;
  };
  onAddMembers: (memberIds: string[]) => void;
  onRemoveMember: (memberId: string) => void;
  onUpdateRole: (memberId: string, role: string) => void;
}

export const GroupMembers: React.FC<GroupMembersProps> = ({
  members,
  currentUser,
  onAddMembers,
  onRemoveMember,
  onUpdateRole,
}) => {
  const [isAddingMembers, setIsAddingMembers] = useState(false);
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
      setSearchResults(data.filter((user: any) => !members.some(m => m.userId === user.id)));
    } catch (error) {
      console.error('Error searching users:', error);
    }
  };

  const handleAddSelectedUsers = () => {
    onAddMembers(selectedUsers);
    setSelectedUsers([]);
    setIsAddingMembers(false);
    setSearchQuery('');
    setSearchResults([]);
  };

  const isAdmin = currentUser.role === 'admin';

  return (
    <div className="p-4">
      <div className="flex justify-between items-center mb-4">
        <h3 className="text-lg font-semibold">Group Members</h3>
        {isAdmin && (
          <button
            onClick={() => setIsAddingMembers(true)}
            className="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600"
          >
            Add Members
          </button>
        )}
      </div>

      {isAddingMembers && (
        <div className="mb-4">
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => {
              setSearchQuery(e.target.value);
              handleSearch(e.target.value);
            }}
            className="w-full px-2 py-1 border rounded mb-2"
            placeholder="Search users..."
          />
          {searchResults.length > 0 && (
            <div className="border rounded max-h-48 overflow-y-auto">
              {searchResults.map((user) => (
                <div
                  key={user.id}
                  className="flex items-center space-x-2 p-2 hover:bg-gray-100 cursor-pointer"
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
            <div className="mt-2 flex justify-end space-x-2">
              <button
                onClick={() => {
                  setSelectedUsers([]);
                  setIsAddingMembers(false);
                  setSearchQuery('');
                  setSearchResults([]);
                }}
                className="px-3 py-1 bg-gray-200 rounded hover:bg-gray-300"
              >
                Cancel
              </button>
              <button
                onClick={handleAddSelectedUsers}
                className="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600"
              >
                Add Selected
              </button>
            </div>
          )}
        </div>
      )}

      <div className="space-y-2">
        {members.map((member) => (
          <div key={member.id} className="flex items-center justify-between p-2 hover:bg-gray-50">
            <div className="flex items-center space-x-2">
              <div className="w-8 h-8 rounded-full bg-gray-200 overflow-hidden">
                {member.user.avatarUrl ? (
                  <img
                    src={member.user.avatarUrl}
                    alt={member.user.displayName || member.user.username}
                    className="w-full h-full object-cover"
                  />
                ) : (
                  <div className="w-full h-full flex items-center justify-center text-gray-500">
                    {(member.user.displayName || member.user.username).charAt(0).toUpperCase()}
                  </div>
                )}
              </div>
              <div>
                <div className="font-medium">{member.user.displayName || member.user.username}</div>
                <div className="text-sm text-gray-500">
                  {member.role.charAt(0).toUpperCase() + member.role.slice(1)}
                </div>
              </div>
            </div>
            {isAdmin && member.userId !== currentUser.id && (
              <div className="flex items-center space-x-2">
                <select
                  value={member.role}
                  onChange={(e) => onUpdateRole(member.userId, e.target.value)}
                  className="px-2 py-1 border rounded text-sm"
                >
                  <option value="member">Member</option>
                  <option value="admin">Admin</option>
                </select>
                <button
                  onClick={() => onRemoveMember(member.userId)}
                  className="text-red-500 hover:text-red-600"
                >
                  Remove
                </button>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}; 
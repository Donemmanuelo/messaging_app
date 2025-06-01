import React, { useState } from 'react';

interface GroupInfoProps {
  group: {
    id: string;
    name: string;
    description?: string;
    avatarUrl?: string;
    createdBy: string;
  };
  onUpdate: () => void;
}

export const GroupInfo: React.FC<GroupInfoProps> = ({ group, onUpdate }) => {
  const [isEditing, setIsEditing] = useState(false);
  const [name, setName] = useState(group.name);
  const [description, setDescription] = useState(group.description || '');

  const handleUpdate = async () => {
    try {
      await fetch(`/api/groups/${group.id}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          name,
          description,
        }),
      });
      setIsEditing(false);
      onUpdate();
    } catch (error) {
      console.error('Error updating group:', error);
    }
  };

  return (
    <div className="p-4 border-b border-gray-200">
      <div className="flex items-center space-x-4">
        <div className="w-12 h-12 rounded-full bg-gray-200 overflow-hidden">
          {group.avatarUrl ? (
            <img
              src={group.avatarUrl}
              alt={group.name}
              className="w-full h-full object-cover"
            />
          ) : (
            <div className="w-full h-full flex items-center justify-center text-gray-500">
              {group.name.charAt(0).toUpperCase()}
            </div>
          )}
        </div>
        <div className="flex-1">
          {isEditing ? (
            <div className="space-y-2">
              <input
                type="text"
                value={name}
                onChange={(e) => setName(e.target.value)}
                className="w-full px-2 py-1 border rounded"
                placeholder="Group name"
              />
              <textarea
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                className="w-full px-2 py-1 border rounded"
                placeholder="Group description"
                rows={2}
              />
              <div className="flex space-x-2">
                <button
                  onClick={handleUpdate}
                  className="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600"
                >
                  Save
                </button>
                <button
                  onClick={() => setIsEditing(false)}
                  className="px-3 py-1 bg-gray-200 rounded hover:bg-gray-300"
                >
                  Cancel
                </button>
              </div>
            </div>
          ) : (
            <div>
              <h2 className="text-lg font-semibold">{group.name}</h2>
              {group.description && (
                <p className="text-sm text-gray-500">{group.description}</p>
              )}
              <button
                onClick={() => setIsEditing(true)}
                className="mt-2 text-sm text-blue-500 hover:text-blue-600"
              >
                Edit group info
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}; 
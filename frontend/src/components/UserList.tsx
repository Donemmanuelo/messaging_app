import React from 'react';
import { User } from '../types';

interface UserListProps {
  users: User[];
}

const UserList: React.FC<UserListProps> = ({ users }) => {
  return (
    <div className="user-list">
      <h2>Online Users</h2>
      <div className="user-items">
        {users.map((user) => (
          <div key={user.id} className="user-item">
            <div className="user-info">
              <span className="username">{user.username}</span>
              <span className={`status ${user.status.toLowerCase()}`}>
                {user.status}
              </span>
            </div>
            {user.lastSeen && (
              <div className="last-seen">
                Last seen: {new Date(user.lastSeen).toLocaleString()}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};

export default UserList; 
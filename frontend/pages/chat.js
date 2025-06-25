import { useEffect, useState, useRef } from 'react';
import { api, getUserFromToken } from '../utils/api';
import { connectWebSocket, sendWebSocketMessage } from '../socket';

const REACTIONS = ['👍', '❤️', '😂'];

export default function Chat() {
  const [groups, setGroups] = useState([]);
  const [selectedGroup, setSelectedGroup] = useState(null);
  const [messages, setMessages] = useState([]);
  const [newMessage, setNewMessage] = useState('');
  const [newGroup, setNewGroup] = useState('');
  const [users, setUsers] = useState([]);
  const [reactions, setReactions] = useState({});
  const [reads, setReads] = useState({});
  const [notification, setNotification] = useState(null);
  const user = getUserFromToken();
  const wsConnected = useRef(false);

  const fetchGroups = () => {
    const token = localStorage.getItem('token');
    if (!token) return;
    api.get('/api/groups', { headers: { Authorization: `Bearer ${token}` } })
      .then(res => setGroups(res.data))
      .catch(() => setGroups([]));
  };

  const fetchUsers = () => {
    const token = localStorage.getItem('token');
    if (!token) return;
    api.get('/api/users', { headers: { Authorization: `Bearer ${token}` } })
      .then(res => setUsers(res.data))
      .catch(() => setUsers([]));
  };

  const fetchReactions = async (messageId) => {
    const token = localStorage.getItem('token');
    if (!token) return;
    const res = await api.get(`/api/message_reactions/${messageId}`, { headers: { Authorization: `Bearer ${token}` } });
    setReactions(prev => ({ ...prev, [messageId]: res.data }));
  };

  const fetchReads = async (messageId) => {
    const token = localStorage.getItem('token');
    if (!token) return;
    const res = await api.get(`/api/message_reads/${messageId}`, { headers: { Authorization: `Bearer ${token}` } });
    setReads(prev => ({ ...prev, [messageId]: res.data }));
  };

  useEffect(() => {
    fetchGroups();
    fetchUsers();
  }, []);

  useEffect(() => {
    if (!selectedGroup) return;
    const token = localStorage.getItem('token');
    api.get(`/api/messages/${selectedGroup.id}`, { headers: { Authorization: `Bearer ${token}` } })
      .then(res => {
        setMessages(res.data);
        res.data.forEach(m => {
          fetchReactions(m.id);
          fetchReads(m.id);
        });
      })
      .catch(() => setMessages([]));
  }, [selectedGroup]);

  useEffect(() => {
    if (!wsConnected.current) {
      connectWebSocket((msg) => {
        try {
          const data = JSON.parse(msg);
          if (data.chat_id === selectedGroup?.id) {
            setMessages((prev) => [...prev, data]);
            fetchReactions(data.id);
            fetchReads(data.id);
          } else if (data.chat_id) {
            const group = groups.find(g => g.id === data.chat_id);
            setNotification({
              groupName: group ? group.name : `Group ${data.chat_id}`,
              content: data.content,
            });
            setTimeout(() => setNotification(null), 3000);
          }
        } catch {}
      });
      wsConnected.current = true;
    }
    // eslint-disable-next-line
  }, [selectedGroup, groups]);

  useEffect(() => {
    if (!user || !messages.length) return;
    const token = localStorage.getItem('token');
    messages.forEach(m => {
      if (!reads[m.id] || !reads[m.id].some(r => r.user_id === user.id)) {
        api.post('/api/message_reads', { message_id: m.id, user_id: user.id }, { headers: { Authorization: `Bearer ${token}` } })
          .then(() => fetchReads(m.id));
      }
    });
    // eslint-disable-next-line
  }, [messages, user]);

  const handleSend = async (e) => {
    e.preventDefault();
    if (!newMessage || !selectedGroup || !user) return;
    sendWebSocketMessage(JSON.stringify({
      chat_id: selectedGroup.id,
      sender_id: user.id,
      content: newMessage,
    }));
    setNewMessage('');
  };

  const handleCreateGroup = async (e) => {
    e.preventDefault();
    if (!newGroup || !user) return;
    const token = localStorage.getItem('token');
    await api.post('/api/groups', {
      name: newGroup,
      created_by: user.id,
    }, { headers: { Authorization: `Bearer ${token}` } });
    setNewGroup('');
    fetchGroups();
  };

  const handleReact = async (messageId, reaction) => {
    const token = localStorage.getItem('token');
    if (!user || !token) return;
    const alreadyReacted = (reactions[messageId] || []).some(r => r.user_id === user.id && r.reaction === reaction);
    if (alreadyReacted) {
      await api.delete('/api/message_reactions', {
        data: { message_id: messageId, user_id: user.id, reaction },
        headers: { Authorization: `Bearer ${token}` },
      });
    } else {
      await api.post('/api/message_reactions', {
        message_id: messageId, user_id: user.id, reaction
      }, { headers: { Authorization: `Bearer ${token}` } });
    }
    fetchReactions(messageId);
  };

  const handleGroupAvatarChange = async (e) => {
    if (!selectedGroup || !user) return;
    const file = e.target.files[0];
    if (!file) return;
    const formData = new FormData();
    formData.append('file', file);
    const token = localStorage.getItem('token');
    await api.post(`/api/groups/${selectedGroup.id}/avatar`, formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
        Authorization: `Bearer ${token}`,
      },
    });
    fetchGroups();
  };

  return (
    <div className="flex h-screen bg-gray-100">
      {notification && (
        <div className="fixed top-4 left-1/2 transform -translate-x-1/2 bg-blue-600 text-white px-6 py-3 rounded shadow-lg z-50 cursor-pointer animate-fadeIn" onClick={() => setNotification(null)}>
          <b>New message in {notification.groupName}:</b> {notification.content}
        </div>
      )}
      {/* Sidebar */}
      <div className="sm:w-full md:w-64 border-r border-gray-200 bg-white p-4 flex flex-col">
        <form onSubmit={handleCreateGroup} className="mb-4 flex">
          <input
            type="text"
            value={newGroup}
            onChange={e => setNewGroup(e.target.value)}
            placeholder="New group name"
            className="flex-1 border rounded-l px-2 py-1 text-sm focus:outline-none focus:ring-2 focus:ring-blue-400"
            aria-label="New group name"
          />
          <button type="submit" className="bg-blue-500 text-white px-3 py-1 rounded-r text-sm transition-colors hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-400" aria-label="Create group">+</button>
        </form>
        <h4 className="font-semibold mb-2">Groups</h4>
        <ul className="mb-4">
          {groups.map(g => (
            <li
              key={g.id}
              className={`mb-2 cursor-pointer px-2 py-1 rounded flex items-center transition-colors ${selectedGroup?.id === g.id ? 'bg-blue-100 font-bold' : 'hover:bg-gray-100'}`}
              onClick={() => setSelectedGroup(g)}
              tabIndex={0}
              aria-label={`Select group ${g.name}`}
              onKeyDown={e => { if (e.key === 'Enter') setSelectedGroup(g); }}
            >
              {g.avatar_url ? (
                <img src={g.avatar_url} alt={g.name + ' avatar'} className="w-7 h-7 rounded-full mr-2" />
              ) : (
                <span className="w-7 h-7 rounded-full bg-gray-200 inline-block mr-2" aria-label="No avatar" />
              )}
              <span>{g.name}</span>
            </li>
          ))}
        </ul>
        <h4 className="font-semibold mb-2">Users</h4>
        <ul className="overflow-y-auto flex-1">
          {users.map(u => (
            <li key={u.id} className="mb-2 flex items-center">
              {u.avatar_url ? (
                <img src={u.avatar_url} alt={u.username + ' avatar'} className="w-6 h-6 rounded-full mr-2" />
              ) : (
                <span className="w-6 h-6 rounded-full bg-gray-200 inline-block mr-2" aria-label="No avatar" />
              )}
              <span>{u.username}</span>
              <span className={`ml-2 text-xs ${u.online ? 'text-green-500' : 'text-gray-400'}`}>{u.online ? '●' : '○'}</span>
            </li>
          ))}
        </ul>
      </div>
      {/* Chat Window */}
      <div className="flex-1 flex flex-col p-6 overflow-x-auto">
        {selectedGroup && (
          <div className="flex items-center mb-4">
            {selectedGroup.avatar_url ? (
              <img src={selectedGroup.avatar_url} alt={selectedGroup.name + ' avatar'} className="w-10 h-10 rounded-full mr-3" />
            ) : (
              <span className="w-10 h-10 rounded-full bg-gray-200 inline-block mr-3" aria-label="No avatar" />
            )}
            <span className="font-bold text-lg mr-4">{selectedGroup.name}</span>
            {user && selectedGroup.created_by === user.id && (
              <label className="ml-2 cursor-pointer text-xs text-blue-600 hover:underline">
                Change Avatar
                <input type="file" accept="image/*" onChange={handleGroupAvatarChange} className="hidden" aria-label="Upload group avatar" />
              </label>
            )}
          </div>
        )}
        <div className="flex-1 overflow-y-auto mb-4">
          {messages.map(m => (
            <div key={m.id || m.content + m.sender_id} className={`mb-4 flex ${m.sender_id === user?.id ? 'justify-end' : 'justify-start'}`}>
              <div className={`max-w-lg px-4 py-2 rounded-lg shadow ${m.sender_id === user?.id ? 'bg-blue-500 text-white' : 'bg-white border'}`} tabIndex={0} aria-label={`Message from ${m.sender_id === user?.id ? 'You' : m.sender_id}`}> 
                <div className="font-semibold text-xs mb-1">{m.sender_id === user?.id ? 'You' : m.sender_id}</div>
                <div>{m.content}</div>
                <div className="mt-2 flex items-center space-x-1">
                  {REACTIONS.map(r => {
                    const count = (reactions[m.id] || []).filter(x => x.reaction === r).length;
                    const reacted = (reactions[m.id] || []).some(x => x.reaction === r && x.user_id === user?.id);
                    return (
                      <button
                        key={r}
                        onClick={() => handleReact(m.id, r)}
                        className={`px-2 py-0.5 rounded-full border text-xs flex items-center transition-colors focus:outline-none focus:ring-2 focus:ring-blue-400 ${reacted ? 'bg-blue-100 border-blue-400' : 'bg-gray-100 border-gray-300'} mr-1`}
                        aria-label={`React with ${r}`}
                      >
                        {r} {count > 0 && <span className="ml-0.5">{count}</span>}
                      </button>
                    );
                  })}
                </div>
                <div className="mt-1 text-xs text-gray-500 flex items-center flex-wrap">
                  <span className="mr-1">Read by:</span>
                  {(reads[m.id] || []).length === 0 && <span>None</span>}
                  {(reads[m.id] || []).map(r => {
                    const u = users.find(u => u.id === r.user_id);
                    return u ? (
                      <span key={u.id} className="flex items-center mr-2">
                        {u.avatar_url ? (
                          <img src={u.avatar_url} alt={u.username + ' avatar'} className="w-4 h-4 rounded-full mr-1 inline" />
                        ) : (
                          <span className="w-4 h-4 rounded-full bg-gray-200 inline-block mr-1" aria-label="No avatar" />
                        )}
                        {u.username}
                      </span>
                    ) : null;
                  })}
                </div>
              </div>
            </div>
          ))}
        </div>
        {selectedGroup && (
          <form onSubmit={handleSend} className="flex mt-auto">
            <input
              type="text"
              value={newMessage}
              onChange={e => setNewMessage(e.target.value)}
              placeholder="Type a message..."
              className="flex-1 border rounded-l px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-400"
              aria-label="Type a message"
            />
            <button type="submit" className="bg-blue-500 text-white px-4 py-2 rounded-r text-sm transition-colors hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-400" aria-label="Send message">Send</button>
          </form>
        )}
      </div>
    </div>
  );
} 
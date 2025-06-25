import { useEffect, useState } from 'react';
import { api, getUserFromToken } from '../utils/api';

export default function Profile() {
  const user = getUserFromToken();
  const [profile, setProfile] = useState({ username: '', email: '', avatar_url: '' });
  const [username, setUsername] = useState('');
  const [email, setEmail] = useState('');
  const [avatarUrl, setAvatarUrl] = useState('');
  const [success, setSuccess] = useState('');
  const [error, setError] = useState('');

  useEffect(() => {
    if (!user) return;
    api.get('/api/users', { headers: { Authorization: `Bearer ${localStorage.getItem('token')}` } })
      .then(res => {
        const me = res.data.find(u => u.id === user.id);
        if (me) {
          setProfile(me);
          setUsername(me.username);
          setEmail(me.email);
          setAvatarUrl(me.avatar_url || '');
        }
      });
  }, [user]);

  const handleProfileUpdate = async (e) => {
    e.preventDefault();
    setSuccess(''); setError('');
    try {
      await api.patch(`/api/users/${user.id}`, { username, email }, { headers: { Authorization: `Bearer ${localStorage.getItem('token')}` } });
      setSuccess('Profile updated!');
    } catch {
      setError('Update failed');
    }
  };

  const handleAvatarChange = async (e) => {
    const file = e.target.files[0];
    if (!file) return;
    const formData = new FormData();
    formData.append('file', file);
    try {
      const res = await api.post(`/api/users/${user.id}/avatar`, formData, {
        headers: {
          'Content-Type': 'multipart/form-data',
          Authorization: `Bearer ${localStorage.getItem('token')}`,
        },
      });
      setAvatarUrl(res.data.avatar_url);
      setSuccess('Avatar updated!');
    } catch {
      setError('Avatar upload failed');
    }
  };

  if (!user) return <div className="p-8">Please log in.</div>;

  return (
    <div className="max-w-md mx-auto mt-10 bg-white p-8 rounded shadow w-full">
      <h2 className="text-2xl font-bold mb-6">Profile Settings</h2>
      <form onSubmit={handleProfileUpdate} className="space-y-4">
        <div className="flex items-center mb-4">
          {avatarUrl ? (
            <img src={avatarUrl} alt="Your avatar" className="w-16 h-16 rounded-full mr-4" />
          ) : (
            <span className="w-16 h-16 rounded-full bg-gray-200 inline-block mr-4" aria-label="No avatar" />
          )}
          <label className="cursor-pointer text-blue-600 hover:underline text-sm">
            Change Avatar
            <input type="file" accept="image/*" onChange={handleAvatarChange} className="hidden" aria-label="Upload avatar" />
          </label>
        </div>
        <div>
          <label className="block text-sm font-medium">Username</label>
          <input
            type="text"
            value={username}
            onChange={e => setUsername(e.target.value)}
            className="w-full border rounded px-3 py-2 mt-1 focus:outline-none focus:ring-2 focus:ring-blue-400"
            aria-label="Username"
          />
        </div>
        <div>
          <label className="block text-sm font-medium">Email</label>
          <input
            type="email"
            value={email}
            onChange={e => setEmail(e.target.value)}
            className="w-full border rounded px-3 py-2 mt-1 focus:outline-none focus:ring-2 focus:ring-blue-400"
            aria-label="Email"
          />
        </div>
        <button type="submit" className="bg-blue-500 text-white px-4 py-2 rounded transition-colors hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-400">Update Profile</button>
        {success && <div className="text-green-600 mt-2">{success}</div>}
        {error && <div className="text-red-600 mt-2">{error}</div>}
      </form>
    </div>
  );
} 
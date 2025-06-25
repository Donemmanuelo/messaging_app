import { useState } from 'react';
import { api } from '../utils/api';

export default function FileUpload() {
  const [file, setFile] = useState(null);
  const [url, setUrl] = useState('');
  const [error, setError] = useState('');

  const handleChange = (e) => {
    setFile(e.target.files[0]);
    setUrl('');
    setError('');
  };

  const handleUpload = async (e) => {
    e.preventDefault();
    if (!file) return;
    const formData = new FormData();
    formData.append('file', file);
    try {
      const token = localStorage.getItem('token');
      const res = await api.post('/api/media/upload', formData, {
        headers: {
          'Content-Type': 'multipart/form-data',
          Authorization: `Bearer ${token}`,
        },
      });
      setUrl(res.data.url);
    } catch (err) {
      setError('Upload failed');
    }
  };

  return (
    <div style={{ margin: '2rem 0' }}>
      <form onSubmit={handleUpload}>
        <input type="file" onChange={handleChange} />
        <button type="submit">Upload</button>
      </form>
      {url && <div>Uploaded: <a href={url} target="_blank" rel="noopener noreferrer">{url}</a></div>}
      {error && <div style={{ color: 'red' }}>{error}</div>}
    </div>
  );
} 
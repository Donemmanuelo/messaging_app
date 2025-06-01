import React, { useState, useRef } from 'react';

interface MediaUploadProps {
  onUploadComplete: (url: string, type: string) => void;
  onCancel: () => void;
}

export const MediaUpload: React.FC<MediaUploadProps> = ({
  onUploadComplete,
  onCancel,
}) => {
  const [isUploading, setIsUploading] = useState(false);
  const [preview, setPreview] = useState<string | null>(null);
  const [mediaType, setMediaType] = useState<string>('image');
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleFileSelect = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    // Set media type based on file type
    if (file.type.startsWith('image/')) {
      setMediaType('image');
    } else if (file.type.startsWith('video/')) {
      setMediaType('video');
    } else if (file.type.startsWith('audio/')) {
      setMediaType('audio');
    } else {
      setMediaType('document');
    }

    // Create preview for images and videos
    if (file.type.startsWith('image/') || file.type.startsWith('video/')) {
      const reader = new FileReader();
      reader.onload = (e) => {
        setPreview(e.target?.result as string);
      };
      reader.readAsDataURL(file);
    } else {
      setPreview(null);
    }
  };

  const handleUpload = async () => {
    const file = fileInputRef.current?.files?.[0];
    if (!file) return;

    setIsUploading(true);

    try {
      const formData = new FormData();
      formData.append('file', file);
      formData.append('type', mediaType);

      const response = await fetch('/api/media/upload', {
        method: 'POST',
        body: formData,
      });

      if (!response.ok) {
        throw new Error('Upload failed');
      }

      const data = await response.json();
      onUploadComplete(data.url, data.type);
    } catch (error) {
      console.error('Error uploading file:', error);
      // You might want to show an error message to the user here
    } finally {
      setIsUploading(false);
    }
  };

  return (
    <div className="p-4 border rounded-lg bg-white shadow-lg">
      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Select Media Type
          </label>
          <select
            value={mediaType}
            onChange={(e) => setMediaType(e.target.value)}
            className="w-full px-3 py-2 border rounded-md"
          >
            <option value="image">Image</option>
            <option value="video">Video</option>
            <option value="audio">Audio</option>
            <option value="document">Document</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Choose File
          </label>
          <input
            ref={fileInputRef}
            type="file"
            onChange={handleFileSelect}
            accept={
              mediaType === 'image'
                ? 'image/*'
                : mediaType === 'video'
                ? 'video/*'
                : mediaType === 'audio'
                ? 'audio/*'
                : '*/*'
            }
            className="w-full"
          />
        </div>

        {preview && (
          <div className="mt-4">
            {mediaType === 'image' ? (
              <img
                src={preview}
                alt="Preview"
                className="max-h-48 rounded-lg"
              />
            ) : mediaType === 'video' ? (
              <video
                src={preview}
                controls
                className="max-h-48 rounded-lg"
              />
            ) : null}
          </div>
        )}

        <div className="flex justify-end space-x-2">
          <button
            onClick={onCancel}
            className="px-4 py-2 border rounded-md hover:bg-gray-50"
            disabled={isUploading}
          >
            Cancel
          </button>
          <button
            onClick={handleUpload}
            disabled={isUploading || !fileInputRef.current?.files?.length}
            className="px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isUploading ? 'Uploading...' : 'Upload'}
          </button>
        </div>
      </div>
    </div>
  );
}; 
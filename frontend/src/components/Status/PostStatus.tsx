import React from "react";

export default function PostStatus() {
  // Mock user
  const user = { name: "You", avatar: "/user.svg" };
  return (
    <div className="flex items-center gap-3 w-full max-w-md mb-4">
      <img src={user.avatar} alt={user.name} className="w-10 h-10 rounded-full border" />
      <input
        type="text"
        placeholder="Share a status..."
        className="flex-1 px-4 py-2 rounded-full border border-gray-300 focus:outline-none focus:ring-2 focus:ring-blue-400"
      />
      <button className="ml-2 px-4 py-2 rounded-full bg-blue-500 text-white font-semibold hover:bg-blue-600 transition">Post</button>
    </div>
  );
} 
import React from 'react';

interface MessageReactionsProps {
  messageId: string;
  reactions: Array<{
    emoji: string;
    count: number;
    users: Array<{
      id: string;
      username: string;
    }>;
  }>;
  currentUserId: string;
}

export const MessageReactions: React.FC<MessageReactionsProps> = ({ reactions }) => {
  if (!reactions.length) return null;
  return (
    <div className="flex flex-wrap gap-1 mt-1">
      {reactions.map((reaction, index) => (
        <span key={index} className="text-xs">
          {reaction.emoji} {reaction.count}
        </span>
      ))}
    </div>
  );
}; 
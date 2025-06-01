import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { useWebSocket } from '../hooks/useWebSocket';
import { useAuth } from '../hooks/useAuth';
import { MessageList } from './MessageList';
import { MessageInput } from './MessageInput';
import { GroupInfo } from './GroupInfo';
import { GroupMembers } from './GroupMembers';

interface GroupChatProps {
  groupId: string;
}

export const GroupChat: React.FC<GroupChatProps> = ({ groupId }) => {
  const [group, setGroup] = useState<any>(null);
  const [members, setMembers] = useState<any[]>([]);
  const [messages, setMessages] = useState<any[]>([]);
  const [isTyping, setIsTyping] = useState<boolean>(false);
  const { user } = useAuth();
  const { sendMessage, lastMessage } = useWebSocket();

  useEffect(() => {
    // Fetch group info
    fetchGroupInfo();
    // Fetch group members
    fetchGroupMembers();
    // Fetch messages
    fetchMessages();
  }, [groupId]);

  useEffect(() => {
    if (lastMessage) {
      if (lastMessage.type === 'message') {
        setMessages(prev => [...prev, lastMessage.data]);
      } else if (lastMessage.type === 'typing') {
        setIsTyping(lastMessage.data.isTyping);
      }
    }
  }, [lastMessage]);

  const fetchGroupInfo = async () => {
    try {
      const response = await fetch(`/api/groups/${groupId}`);
      const data = await response.json();
      setGroup(data);
    } catch (error) {
      console.error('Error fetching group info:', error);
    }
  };

  const fetchGroupMembers = async () => {
    try {
      const response = await fetch(`/api/groups/${groupId}/members`);
      const data = await response.json();
      setMembers(data);
    } catch (error) {
      console.error('Error fetching group members:', error);
    }
  };

  const fetchMessages = async () => {
    try {
      const response = await fetch(`/api/chats/${groupId}/messages`);
      const data = await response.json();
      setMessages(data);
    } catch (error) {
      console.error('Error fetching messages:', error);
    }
  };

  const handleSendMessage = async (content: string, mediaUrl?: string) => {
    try {
      const message = {
        chatId: groupId,
        content,
        mediaUrl,
        type: mediaUrl ? 'media' : 'text',
      };
      sendMessage(JSON.stringify(message));
    } catch (error) {
      console.error('Error sending message:', error);
    }
  };

  const handleAddMembers = async (memberIds: string[]) => {
    try {
      await fetch(`/api/groups/${groupId}/members`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ memberIds }),
      });
      fetchGroupMembers();
    } catch (error) {
      console.error('Error adding members:', error);
    }
  };

  const handleRemoveMember = async (memberId: string) => {
    try {
      await fetch(`/api/groups/${groupId}/members`, {
        method: 'DELETE',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ memberIds: [memberId] }),
      });
      fetchGroupMembers();
    } catch (error) {
      console.error('Error removing member:', error);
    }
  };

  const handleUpdateMemberRole = async (memberId: string, role: string) => {
    try {
      await fetch(`/api/groups/${groupId}/members/role`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ userId: memberId, role }),
      });
      fetchGroupMembers();
    } catch (error) {
      console.error('Error updating member role:', error);
    }
  };

  if (!group) {
    return <div>Loading...</div>;
  }

  return (
    <div className="flex h-screen">
      <div className="flex-1 flex flex-col">
        <GroupInfo group={group} onUpdate={fetchGroupInfo} />
        <MessageList
          messages={messages}
          currentUser={user}
          isTyping={isTyping}
        />
        <MessageInput onSendMessage={handleSendMessage} />
      </div>
      <div className="w-80 border-l border-gray-200">
        <GroupMembers
          members={members}
          currentUser={user}
          onAddMembers={handleAddMembers}
          onRemoveMember={handleRemoveMember}
          onUpdateRole={handleUpdateMemberRole}
        />
      </div>
    </div>
  );
}; 
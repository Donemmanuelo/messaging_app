import { GroupChat } from '@/components/GroupChat';

interface GroupChatPageProps {
  params: {
    id: string;
  };
}

export default function GroupChatPage({ params }: GroupChatPageProps) {
  return <GroupChat groupId={params.id} />;
} 
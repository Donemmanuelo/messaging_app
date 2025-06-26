import React, { useEffect, useState } from "react";

interface Status {
  id: string;
  user: { id: string; name: string; avatar?: string };
  createdAt: string;
  seen: boolean;
  content: string;
}

export default function StatusList() {
  const [statuses, setStatuses] = useState<Status[]>([]);
  const [loading, setLoading] = useState(true);
  const currentUserId = "00000000-0000-0000-0000-000000000001";
  const [seenStatusIds, setSeenStatusIds] = useState<string[]>([]);

  useEffect(() => {
    Promise.all([
      fetch('/api/status').then(res => res.json()),
      fetch(`/api/status_views/${currentUserId}`).then(res => res.json())
    ]).then(([statusList, seenIds]) => {
      setSeenStatusIds(seenIds);
      setStatuses(statusList.map((status: any) => ({
        id: status.id,
        user: status.user,
        createdAt: status.created_at,
        seen: seenIds.includes(status.id),
        content: status.content,
      })));
    }).finally(() => setLoading(false));
  }, []);

  const markSeen = (statusId: string) => {
    fetch('/api/status_views', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ status_id: statusId, user_id: currentUserId })
    }).then(() => {
      setSeenStatusIds(ids => [...ids, statusId]);
      setStatuses(sts => sts.map(s => s.id === statusId ? { ...s, seen: true } : s));
    });
  };

  if (loading) return <div>Loading statuses...</div>;

  return (
    <div className="flex flex-col gap-3 w-full max-w-md">
      {statuses.map((status) => (
        <div
          key={status.id}
          className={`flex items-center gap-3 p-2 rounded-lg border ${
            status.seen ? "border-gray-200" : "border-blue-500 bg-blue-50"
          } cursor-pointer`}
          onClick={() => !status.seen && markSeen(status.id)}
        >
          <div className="relative">
            <img
              src={status.user.avatar || "/user.svg"}
              alt={status.user.name}
              className="w-10 h-10 rounded-full border-2"
              style={{ borderColor: status.seen ? "#e5e7eb" : "#3b82f6" }}
            />
            {!status.seen && (
              <span className="absolute top-0 right-0 w-3 h-3 bg-blue-500 rounded-full border-2 border-white"></span>
            )}
          </div>
          <div className="flex-1">
            <div className="font-semibold">{status.user.name}</div>
            <div className="text-xs text-gray-500">
              {new Date(status.createdAt).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
            </div>
          </div>
        </div>
      ))}
    </div>
  );
} 
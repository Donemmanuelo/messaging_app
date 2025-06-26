import React, { useEffect, useState } from "react";

interface Call {
  id: string;
  user: { name: string; avatar?: string };
  type: "audio" | "video";
  time: string;
  status: "missed" | "answered";
}

export default function CallHistory() {
  const [calls, setCalls] = useState<Call[]>([]);
  const [loading, setLoading] = useState(true);
  // Mock current user
  const currentUserId = "00000000-0000-0000-0000-000000000001";

  useEffect(() => {
    fetch(`/api/call_logs/${currentUserId}`)
      .then(res => res.json())
      .then(data => {
        setCalls(data.map((log: any) => {
          const isCaller = log.caller_id === currentUserId;
          return {
            id: log.id,
            user: {
              name: isCaller ? log.callee_name || log.callee_id : log.caller_name || log.caller_id,
              avatar: isCaller ? log.callee_avatar : log.caller_avatar
            },
            type: log.call_type,
            time: log.started_at,
            status: log.status,
          };
        }));
      })
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <div>Loading call history...</div>;

  return (
    <div className="w-full max-w-md flex flex-col gap-3">
      <h3 className="font-semibold mb-2">Call History</h3>
      {calls.map((call) => (
        <div key={call.id} className="flex items-center gap-3 p-2 rounded-lg border border-gray-200">
          <img src={call.user.avatar || "/user.svg"} alt={call.user.name} className="w-10 h-10 rounded-full border" />
          <div className="flex-1">
            <div className="font-semibold">{call.user.name}</div>
            <div className="text-xs text-gray-500">
              {new Date(call.time).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
            </div>
          </div>
          <div className={`text-xs font-semibold ${call.status === "missed" ? "text-red-500" : "text-green-500"}`}>{call.status}</div>
          <div className="text-xs text-gray-400">{call.type === "video" ? "📹" : "📞"}</div>
        </div>
      ))}
    </div>
  );
} 
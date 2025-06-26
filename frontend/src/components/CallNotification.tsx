import React from "react";

interface CallNotificationProps {
  caller: { name: string; avatar?: string };
  callType: "audio" | "video";
  onAccept: () => void;
  onDecline: () => void;
}

const CallNotification: React.FC<CallNotificationProps> = ({ caller, callType, onAccept, onDecline }) => {
  return (
    <div className="fixed top-4 left-1/2 transform -translate-x-1/2 bg-white shadow-lg rounded-xl flex items-center gap-4 px-6 py-4 z-50 border border-blue-500">
      <img src={caller.avatar || "/user.svg"} alt={caller.name} className="w-12 h-12 rounded-full border" />
      <div>
        <div className="font-semibold">Incoming {callType === "video" ? "Video" : "Audio"} Call</div>
        <div className="text-gray-600">from {caller.name}</div>
      </div>
      <button onClick={onAccept} className="ml-4 px-4 py-2 rounded-full bg-green-500 text-white font-semibold hover:bg-green-600 transition">Accept</button>
      <button onClick={onDecline} className="ml-2 px-4 py-2 rounded-full bg-red-500 text-white font-semibold hover:bg-red-600 transition">Decline</button>
    </div>
  );
};

export default CallNotification; 
import React, { useEffect, useRef } from "react";
import { useGroupWebRTCCall, CallType } from "../hooks/useGroupWebRTCCall";

interface GroupCallUIProps {
  webSocket: WebSocket;
  groupMemberIds: string[];
  myId: string;
  callType: CallType;
  onEnd: () => void;
}

export const GroupCallUI: React.FC<GroupCallUIProps> = ({
  webSocket,
  groupMemberIds,
  myId,
  callType,
  onEnd,
}) => {
  const {
    inCall,
    localStream,
    remoteStreams,
    startCall,
    handleSignal,
    endCall,
  } = useGroupWebRTCCall(webSocket, groupMemberIds, myId);

  const localVideoRef = useRef<HTMLVideoElement>(null);
  useEffect(() => {
    if (localVideoRef.current && localStream) {
      localVideoRef.current.srcObject = localStream;
    }
  }, [localStream]);

  useEffect(() => {
    // Listen for signaling messages
    const handler = (event: MessageEvent) => {
      const msg = JSON.parse(event.data);
      if (msg.type && msg.type.startsWith("group-")) {
        handleSignal(msg);
      }
    };
    webSocket.addEventListener("message", handler);
    return () => webSocket.removeEventListener("message", handler);
  }, [webSocket, handleSignal]);

  useEffect(() => {
    startCall(callType);
    // eslint-disable-next-line
  }, []);

  const handleEnd = () => {
    endCall();
    onEnd();
  };

  return (
    <div className="group-call-ui">
      <div className="video-grid" style={{ display: "flex", flexWrap: "wrap", gap: 8 }}>
        <div>
          <video ref={localVideoRef} autoPlay muted playsInline style={{ width: 200, height: 150, background: "#222" }} />
          <div style={{ textAlign: "center" }}>You</div>
        </div>
        {remoteStreams.map((stream, idx) => (
          <div key={idx}>
            <video
              autoPlay
              playsInline
              ref={el => {
                if (el) el.srcObject = stream;
              }}
              style={{ width: 200, height: 150, background: "#222" }}
            />
            <div style={{ textAlign: "center" }}>Participant {idx + 1}</div>
          </div>
        ))}
      </div>
      <div style={{ marginTop: 16 }}>
        <button onClick={handleEnd} style={{ background: "#e53e3e", color: "white", padding: 8, borderRadius: 4 }}>End Call</button>
      </div>
    </div>
  );
};

export default GroupCallUI; 
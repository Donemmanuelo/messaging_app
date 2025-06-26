import React, { useEffect } from "react";
import { useWebRTCCall } from "@/hooks/useWebRTCCall";

export function CallUI({ webSocket, remoteUserId }: { webSocket: WebSocket; remoteUserId: string }) {
  const { inCall, localStream, remoteStream, startCall, handleSignal, endCall } =
    useWebRTCCall(webSocket, remoteUserId);

  // Listen for signaling messages
  useEffect(() => {
    function onMessage(event: MessageEvent) {
      const msg = JSON.parse(event.data);
      if (msg.from === remoteUserId) {
        handleSignal(msg);
      }
    }
    webSocket.addEventListener("message", onMessage);
    return () => webSocket.removeEventListener("message", onMessage);
  }, [webSocket, remoteUserId, handleSignal]);

  return (
    <div>
      {!inCall && (
        <div>
          <button onClick={() => startCall("audio")}>Start Audio Call</button>
          <button onClick={() => startCall("video")}>Start Video Call</button>
        </div>
      )}
      {inCall && (
        <div>
          <div>
            <video
              autoPlay
              playsInline
              muted
              ref={(el) => {
                if (el && localStream) el.srcObject = localStream;
              }}
              style={{ width: 200 }}
            />
            <video
              autoPlay
              playsInline
              ref={(el) => {
                if (el && remoteStream) el.srcObject = remoteStream;
              }}
              style={{ width: 200 }}
            />
          </div>
          <button onClick={endCall}>End Call</button>
        </div>
      )}
    </div>
  );
} 
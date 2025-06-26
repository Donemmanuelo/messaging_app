import { useRef, useState, useCallback } from "react";

export type CallType = "audio" | "video";

export function useWebRTCCall(webSocket: WebSocket, remoteUserId: string) {
  const [inCall, setInCall] = useState(false);
  const [localStream, setLocalStream] = useState<MediaStream | null>(null);
  const [remoteStream, setRemoteStream] = useState<MediaStream | null>(null);
  const peerRef = useRef<RTCPeerConnection | null>(null);

  // Start a call (offer)
  const startCall = useCallback(async (type: CallType) => {
    const media = await navigator.mediaDevices.getUserMedia({
      audio: true,
      video: type === "video",
    });
    setLocalStream(media);

    const peer = new RTCPeerConnection();
    peerRef.current = peer;

    media.getTracks().forEach((track) => peer.addTrack(track, media));

    peer.ontrack = (event) => {
      setRemoteStream(event.streams[0]);
    };

    peer.onicecandidate = (event) => {
      if (event.candidate) {
        webSocket.send(
          JSON.stringify({
            type: "ice-candidate",
            candidate: event.candidate,
            to: remoteUserId,
          })
        );
      }
    };

    const offer = await peer.createOffer();
    await peer.setLocalDescription(offer);

    webSocket.send(
      JSON.stringify({
        type: "call-offer",
        offer,
        to: remoteUserId,
      })
    );
    setInCall(true);
  }, [webSocket, remoteUserId]);

  // Handle incoming signaling messages
  const handleSignal = useCallback(
    async (msg: any) => {
      const peer = peerRef.current;
      if (!peer) return;

      if (msg.type === "call-offer") {
        const media = await navigator.mediaDevices.getUserMedia({
          audio: true,
          video: !!msg.offer.sdp.includes("video"),
        });
        setLocalStream(media);

        media.getTracks().forEach((track) => peer.addTrack(track, media));
        peer.ontrack = (event) => setRemoteStream(event.streams[0]);
        peer.onicecandidate = (event) => {
          if (event.candidate) {
            webSocket.send(
              JSON.stringify({
                type: "ice-candidate",
                candidate: event.candidate,
                to: remoteUserId,
              })
            );
          }
        };

        await peer.setRemoteDescription(new RTCSessionDescription(msg.offer));
        const answer = await peer.createAnswer();
        await peer.setLocalDescription(answer);

        webSocket.send(
          JSON.stringify({
            type: "call-answer",
            answer,
            to: remoteUserId,
          })
        );
        setInCall(true);
      } else if (msg.type === "call-answer") {
        await peer.setRemoteDescription(new RTCSessionDescription(msg.answer));
      } else if (msg.type === "ice-candidate" && msg.candidate) {
        await peer.addIceCandidate(new RTCIceCandidate(msg.candidate));
      }
    },
    [webSocket, remoteUserId]
  );

  // End call
  const endCall = useCallback(() => {
    peerRef.current?.close();
    setInCall(false);
    setLocalStream(null);
    setRemoteStream(null);
  }, []);

  return {
    inCall,
    localStream,
    remoteStream,
    startCall,
    handleSignal,
    endCall,
  };
} 
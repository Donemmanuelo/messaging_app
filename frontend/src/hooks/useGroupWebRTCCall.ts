import { useRef, useState, useCallback } from "react";

export type CallType = "audio" | "video";

export function useGroupWebRTCCall(webSocket: WebSocket, groupMemberIds: string[], myId: string) {
  const [inCall, setInCall] = useState(false);
  const [localStream, setLocalStream] = useState<MediaStream | null>(null);
  const [remoteStreams, setRemoteStreams] = useState<{ [id: string]: MediaStream }>({});
  const peersRef = useRef<{ [id: string]: RTCPeerConnection }>({});

  // Start a group call (offer)
  const startCall = useCallback(async (type: CallType) => {
    const media = await navigator.mediaDevices.getUserMedia({
      audio: true,
      video: type === "video",
    });
    setLocalStream(media);
    setInCall(true);

    for (const memberId of groupMemberIds) {
      if (memberId === myId) continue;
      const peer = new RTCPeerConnection();
      peersRef.current[memberId] = peer;
      media.getTracks().forEach((track) => peer.addTrack(track, media));
      peer.ontrack = (event) => {
        setRemoteStreams((prev) => ({ ...prev, [memberId]: event.streams[0] }));
      };
      peer.onicecandidate = (event) => {
        if (event.candidate) {
          webSocket.send(
            JSON.stringify({
              type: "group-ice-candidate",
              candidate: event.candidate,
              to: memberId,
              from: myId,
            })
          );
        }
      };
      const offer = await peer.createOffer();
      await peer.setLocalDescription(offer);
      webSocket.send(
        JSON.stringify({
          type: "group-call-offer",
          offer,
          to: memberId,
          from: myId,
        })
      );
    }
  }, [webSocket, groupMemberIds, myId]);

  // Handle incoming signaling messages
  const handleSignal = useCallback(
    async (msg: any) => {
      const { from } = msg;
      if (!from || from === myId) return;
      let peer = peersRef.current[from];
      if (!peer) {
        peer = new RTCPeerConnection();
        peersRef.current[from] = peer;
        if (localStream) {
          localStream.getTracks().forEach((track) => peer.addTrack(track, localStream));
        }
        peer.ontrack = (event) => {
          setRemoteStreams((prev) => ({ ...prev, [from]: event.streams[0] }));
        };
        peer.onicecandidate = (event) => {
          if (event.candidate) {
            webSocket.send(
              JSON.stringify({
                type: "group-ice-candidate",
                candidate: event.candidate,
                to: from,
                from: myId,
              })
            );
          }
        };
      }
      if (msg.type === "group-call-offer") {
        await peer.setRemoteDescription(new RTCSessionDescription(msg.offer));
        const answer = await peer.createAnswer();
        await peer.setLocalDescription(answer);
        webSocket.send(
          JSON.stringify({
            type: "group-call-answer",
            answer,
            to: from,
            from: myId,
          })
        );
      } else if (msg.type === "group-call-answer") {
        await peer.setRemoteDescription(new RTCSessionDescription(msg.answer));
      } else if (msg.type === "group-ice-candidate" && msg.candidate) {
        await peer.addIceCandidate(new RTCIceCandidate(msg.candidate));
      }
    },
    [webSocket, myId, localStream]
  );

  // End call
  const endCall = useCallback(() => {
    Object.values(peersRef.current).forEach((peer) => peer.close());
    peersRef.current = {};
    setInCall(false);
    setLocalStream(null);
    setRemoteStreams({});
  }, []);

  return {
    inCall,
    localStream,
    remoteStreams: Object.values(remoteStreams),
    startCall,
    handleSignal,
    endCall,
  };
} 
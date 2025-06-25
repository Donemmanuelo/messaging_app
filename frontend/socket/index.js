let socket = null;

export function connectWebSocket(onMessage) {
  socket = new WebSocket('ws://localhost:8080/ws');
  socket.onopen = () => console.log('WebSocket connected');
  socket.onmessage = (event) => {
    if (onMessage) onMessage(event.data);
  };
  socket.onclose = () => console.log('WebSocket disconnected');
}

export function sendWebSocketMessage(msg) {
  if (socket && socket.readyState === WebSocket.OPEN) {
    socket.send(msg);
  }
} 
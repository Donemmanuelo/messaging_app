export interface WebSocketMessage {
  type: string;
  content?: string;
  [key: string]: any;
}

export class WebSocketClient {
  private ws: WebSocket | null = null;
  private url: string;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectTimeout = 1000;
  private messageHandlers: ((message: WebSocketMessage) => void)[] = [];
  private errorHandlers: ((error: Event) => void)[] = [];
  private closeHandlers: ((event: CloseEvent) => void)[] = [];
  private openHandlers: ((event: Event) => void)[] = [];

  constructor(url: string) {
    this.url = url;
    this.connect();
  }

  private connect() {
    this.ws = new WebSocket(this.url);

    this.ws.addEventListener('open', (event) => {
      this.reconnectAttempts = 0;
      this.openHandlers.forEach(handler => handler(event));
      this.send({ type: 'ping' }); // Send initial heartbeat
    });

    this.ws.addEventListener('message', (event) => {
      try {
        const message = JSON.parse(event.data) as WebSocketMessage;
        this.messageHandlers.forEach(handler => handler(message));
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    });

    this.ws.addEventListener('error', (event) => {
      this.errorHandlers.forEach(handler => handler(event));
    });

    this.ws.addEventListener('close', (event) => {
      this.closeHandlers.forEach(handler => handler(event));
      this.attemptReconnect();
    });
  }

  private attemptReconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      setTimeout(() => {
        this.connect();
      }, this.reconnectTimeout * this.reconnectAttempts);
    }
  }

  public send(message: WebSocketMessage) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    } else {
      console.error('WebSocket is not connected');
    }
  }

  public onMessage(handler: (message: WebSocketMessage) => void) {
    this.messageHandlers.push(handler);
  }

  public onError(handler: (error: Event) => void) {
    this.errorHandlers.push(handler);
  }

  public onClose(handler: (event: CloseEvent) => void) {
    this.closeHandlers.push(handler);
  }

  public onOpen(handler: (event: Event) => void) {
    this.openHandlers.push(handler);
  }

  public close() {
    if (this.ws) {
      this.ws.removeEventListener('open', this.handleOpen);
      this.ws.removeEventListener('message', this.handleMessage);
      this.ws.removeEventListener('error', this.handleError);
      this.ws.removeEventListener('close', this.handleClose);
      this.ws.close();
      this.ws = null;
    }
  }

  private handleOpen = (event: Event) => {
    this.openHandlers.forEach(handler => handler(event));
  };

  private handleMessage = (event: MessageEvent) => {
    try {
      const message = JSON.parse(event.data) as WebSocketMessage;
      this.messageHandlers.forEach(handler => handler(message));
    } catch (error) {
      console.error('Failed to parse WebSocket message:', error);
    }
  };

  private handleError = (event: Event) => {
    this.errorHandlers.forEach(handler => handler(event));
  };

  private handleClose = (event: CloseEvent) => {
    this.closeHandlers.forEach(handler => handler(event));
  };
} 
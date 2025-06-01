import { describe, it, expect, beforeEach, vi } from 'vitest';
import '@testing-library/jest-dom';
import { WebSocketClient } from '../websocket';

// Mock the WebSocket
const mockWebSocket = {
  send: vi.fn(),
  close: vi.fn(),
  addEventListener: vi.fn(),
  removeEventListener: vi.fn(),
};

global.WebSocket = vi.fn(() => mockWebSocket);

describe('WebSocket Client', () => {
  let wsClient: WebSocketClient;

  beforeEach(() => {
    vi.clearAllMocks();
    wsClient = new WebSocketClient('ws://localhost:8080');
  });

  it('connects to WebSocket server', () => {
    expect(global.WebSocket).toHaveBeenCalledWith('ws://localhost:8080');
    expect(mockWebSocket.addEventListener).toHaveBeenCalledWith('open', expect.any(Function));
    expect(mockWebSocket.addEventListener).toHaveBeenCalledWith('message', expect.any(Function));
    expect(mockWebSocket.addEventListener).toHaveBeenCalledWith('error', expect.any(Function));
    expect(mockWebSocket.addEventListener).toHaveBeenCalledWith('close', expect.any(Function));
  });

  it('sends message to server', () => {
    const message = { type: 'text', content: 'Hello' };
    wsClient.send(message);

    expect(mockWebSocket.send).toHaveBeenCalledWith(JSON.stringify(message));
  });

  it('handles incoming messages', () => {
    const messageHandler = vi.fn();
    wsClient.onMessage(messageHandler);

    const message = { type: 'text', content: 'Hello' };
    const messageEvent = new MessageEvent('message', {
      data: JSON.stringify(message),
    });

    const messageCallback = mockWebSocket.addEventListener.mock.calls.find(
      (call) => call[0] === 'message'
    )[1];

    messageCallback(messageEvent);

    expect(messageHandler).toHaveBeenCalledWith(message);
  });

  it('handles connection errors', () => {
    const errorHandler = vi.fn();
    wsClient.onError(errorHandler);

    const errorEvent = new Event('error');
    const errorCallback = mockWebSocket.addEventListener.mock.calls.find(
      (call) => call[0] === 'error'
    )[1];

    errorCallback(errorEvent);

    expect(errorHandler).toHaveBeenCalled();
  });

  it('handles connection close', () => {
    const closeHandler = vi.fn();
    wsClient.onClose(closeHandler);

    const closeEvent = new CloseEvent('close');
    const closeCallback = mockWebSocket.addEventListener.mock.calls.find(
      (call) => call[0] === 'close'
    )[1];

    closeCallback(closeEvent);

    expect(closeHandler).toHaveBeenCalled();
  });

  it('reconnects on connection close', () => {
    const closeEvent = new CloseEvent('close');
    const closeCallback = mockWebSocket.addEventListener.mock.calls.find(
      (call) => call[0] === 'close'
    )[1];

    closeCallback(closeEvent);

    expect(global.WebSocket).toHaveBeenCalledTimes(2);
  });

  it('closes connection', () => {
    wsClient.close();

    expect(mockWebSocket.close).toHaveBeenCalled();
  });

  it('removes event listeners on close', () => {
    wsClient.close();

    expect(mockWebSocket.removeEventListener).toHaveBeenCalledWith('open', expect.any(Function));
    expect(mockWebSocket.removeEventListener).toHaveBeenCalledWith('message', expect.any(Function));
    expect(mockWebSocket.removeEventListener).toHaveBeenCalledWith('error', expect.any(Function));
    expect(mockWebSocket.removeEventListener).toHaveBeenCalledWith('close', expect.any(Function));
  });

  it('handles connection open', () => {
    const openHandler = vi.fn();
    wsClient.onOpen(openHandler);

    const openEvent = new Event('open');
    const openCallback = mockWebSocket.addEventListener.mock.calls.find(
      (call) => call[0] === 'open'
    )[1];

    openCallback(openEvent);

    expect(openHandler).toHaveBeenCalled();
  });

  it('sends heartbeat on connection open', () => {
    const openEvent = new Event('open');
    const openCallback = mockWebSocket.addEventListener.mock.calls.find(
      (call) => call[0] === 'open'
    )[1];

    openCallback(openEvent);

    expect(mockWebSocket.send).toHaveBeenCalledWith(JSON.stringify({ type: 'ping' }));
  });
}); 
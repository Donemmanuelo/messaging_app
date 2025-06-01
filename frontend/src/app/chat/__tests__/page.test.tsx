import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import ChatPage from '../page';

// Mock the WebSocket
const mockWebSocket = {
  send: vi.fn(),
  close: vi.fn(),
  addEventListener: vi.fn(),
  removeEventListener: vi.fn(),
};

global.WebSocket = vi.fn(() => mockWebSocket);

// Mock the fetch function
global.fetch = vi.fn();

describe('ChatPage Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (global.fetch as jest.Mock).mockResolvedValue({
      ok: true,
      json: () => Promise.resolve([]),
    });
  });

  it('renders chat interface', () => {
    render(<ChatPage />);

    expect(screen.getByRole('textbox')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /send/i })).toBeInTheDocument();
  });

  it('loads chat list on mount', async () => {
    const mockChats = [
      {
        id: '1',
        name: 'John Doe',
        lastMessage: 'Hello',
        lastMessageTime: '2024-02-20T12:00:00Z',
        unreadCount: 2,
      },
    ];

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve(mockChats),
    });

    render(<ChatPage />);

    await waitFor(() => {
      expect(screen.getByText('John Doe')).toBeInTheDocument();
    });
  });

  it('handles WebSocket connection', () => {
    render(<ChatPage />);

    expect(global.WebSocket).toHaveBeenCalled();
    expect(mockWebSocket.addEventListener).toHaveBeenCalledWith(
      'message',
      expect.any(Function)
    );
  });

  it('sends message through WebSocket', async () => {
    render(<ChatPage />);

    const input = screen.getByRole('textbox');
    await userEvent.type(input, 'Hello, world!');

    const form = input.closest('form');
    fireEvent.submit(form!);

    expect(mockWebSocket.send).toHaveBeenCalledWith(
      expect.stringContaining('Hello, world!')
    );
  });

  it('handles file upload', async () => {
    render(<ChatPage />);

    const file = new File(['test'], 'test.png', { type: 'image/png' });
    const input = screen.getByLabelText(/attach file/i);

    await userEvent.upload(input, file);

    expect(mockWebSocket.send).toHaveBeenCalledWith(
      expect.stringContaining('test.png')
    );
  });

  it('shows typing indicator', async () => {
    render(<ChatPage />);

    const input = screen.getByRole('textbox');
    await userEvent.type(input, 'Hello');

    expect(mockWebSocket.send).toHaveBeenCalledWith(
      expect.stringContaining('typing')
    );
  });

  it('handles WebSocket errors', () => {
    render(<ChatPage />);

    const errorCallback = mockWebSocket.addEventListener.mock.calls.find(
      (call) => call[0] === 'error'
    )[1];

    errorCallback(new Event('error'));

    expect(screen.getByText(/connection error/i)).toBeInTheDocument();
  });

  it('reconnects WebSocket on close', () => {
    render(<ChatPage />);

    const closeCallback = mockWebSocket.addEventListener.mock.calls.find(
      (call) => call[0] === 'close'
    )[1];

    closeCallback(new Event('close'));

    expect(global.WebSocket).toHaveBeenCalledTimes(2);
  });
}); 
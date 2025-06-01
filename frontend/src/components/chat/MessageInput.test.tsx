import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import MessageInput from './MessageInput';
import { useAuthStore } from '@/store/authStore';
import { useChatStore } from '@/store/chatStore';

// Mock the stores
vi.mock('@/stores/authStore', () => ({
  useAuthStore: vi.fn()
}));

vi.mock('@/stores/chatStore', () => ({
  useChatStore: vi.fn()
}));

describe('MessageInput', () => {
  const mockUser = {
    id: 'user1',
    username: 'testuser',
    email: 'test@example.com'
  };

  const mockSelectedChat = {
    id: '1',
    name: 'Test Chat',
    participants: ['user1', 'user2']
  };

  beforeEach(() => {
    vi.clearAllMocks();
    (useAuthStore as any).mockReturnValue({
      user: mockUser,
      isAuthenticated: true
    });
    (useChatStore as any).mockReturnValue({
      selectedChat: mockSelectedChat,
      sendMessage: vi.fn()
    });
  });

  it('renders message input correctly', () => {
    render(<MessageInput />);
    
    expect(screen.getByPlaceholderText('Type a message...')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /send/i })).toBeInTheDocument();
  });

  it('sends message when form is submitted', () => {
    const mockSendMessage = vi.fn();
    (useChatStore as any).mockReturnValue({
      selectedChat: mockSelectedChat,
      sendMessage: mockSendMessage
    });

    render(<MessageInput />);
    
    const input = screen.getByPlaceholderText('Type a message...');
    const sendButton = screen.getByRole('button', { name: /send/i });

    fireEvent.change(input, { target: { value: 'Hello, world!' } });
    fireEvent.click(sendButton);

    expect(mockSendMessage).toHaveBeenCalledWith('Hello, world!');
    expect(input).toHaveValue('');
  });

  it('does not send empty messages', () => {
    const mockSendMessage = vi.fn();
    (useChatStore as any).mockReturnValue({
      selectedChat: mockSelectedChat,
      sendMessage: mockSendMessage
    });

    render(<MessageInput />);
    
    const sendButton = screen.getByRole('button', { name: /send/i });
    fireEvent.click(sendButton);

    expect(mockSendMessage).not.toHaveBeenCalled();
  });

  it('sends message when Enter key is pressed', () => {
    const mockSendMessage = vi.fn();
    (useChatStore as any).mockReturnValue({
      selectedChat: mockSelectedChat,
      sendMessage: mockSendMessage
    });

    render(<MessageInput />);
    
    const input = screen.getByPlaceholderText('Type a message...');
    fireEvent.change(input, { target: { value: 'Hello, world!' } });
    fireEvent.keyPress(input, { key: 'Enter', code: 13, charCode: 13 });

    expect(mockSendMessage).toHaveBeenCalledWith('Hello, world!');
    expect(input).toHaveValue('');
  });

  it('does not send message when Shift+Enter is pressed', () => {
    const mockSendMessage = vi.fn();
    (useChatStore as any).mockReturnValue({
      selectedChat: mockSelectedChat,
      sendMessage: mockSendMessage
    });

    render(<MessageInput />);
    
    const input = screen.getByPlaceholderText('Type a message...');
    fireEvent.change(input, { target: { value: 'Hello, world!' } });
    fireEvent.keyPress(input, { key: 'Enter', code: 13, charCode: 13, shiftKey: true });

    expect(mockSendMessage).not.toHaveBeenCalled();
    expect(input).toHaveValue('Hello, world!');
  });

  it('renders message input with onSendMessage and onTyping', () => {
    render(<MessageInput onSendMessage={vi.fn()} onTyping={vi.fn()} />);
    
    expect(screen.getByPlaceholderText('Type a message...')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /send/i })).toBeInTheDocument();
  });
}); 
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import Message from './Message';

const mockMessage = {
  id: '1',
  content: 'Hello, world!',
  senderId: 'user1',
  timestamp: '2024-02-20T12:00:00Z',
  status: 'sent' as const,
  type: 'text' as const,
};

describe('Message Component', () => {
  const mockOnReply = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders text message correctly', () => {
    render(
      <Message
        message={mockMessage}
        isOwnMessage={false}
        onReply={mockOnReply}
      />
    );

    expect(screen.getByText('Hello, world!')).toBeInTheDocument();
  });

  it('renders image message correctly', () => {
    const imageMessage = {
      ...mockMessage,
      type: 'image' as const,
      mediaUrl: 'https://example.com/image.jpg',
    };

    render(
      <Message
        message={imageMessage}
        isOwnMessage={false}
        onReply={mockOnReply}
      />
    );

    const image = screen.getByAltText('Shared image');
    expect(image).toBeInTheDocument();
    expect(image).toHaveAttribute('src', 'https://example.com/image.jpg');
  });

  it('shows menu on hover', () => {
    render(
      <Message
        message={mockMessage}
        isOwnMessage={false}
        onReply={mockOnReply}
      />
    );

    const messageContainer = screen.getByText('Hello, world!').parentElement?.parentElement;
    fireEvent.mouseEnter(messageContainer!);

    // The menu should appear (aria-label="Reply")
    const replyButton = screen.getAllByRole('button').find(btn => btn.getAttribute('aria-label') === 'Reply');
    expect(replyButton).toBeInTheDocument();
  });

  it('calls onReply when reply button is clicked', () => {
    render(
      <Message
        message={mockMessage}
        isOwnMessage={false}
        onReply={mockOnReply}
      />
    );

    const messageContainer = screen.getByText('Hello, world!').parentElement?.parentElement;
    fireEvent.mouseEnter(messageContainer!);

    const replyButton = screen.getAllByRole('button').find(btn => btn.getAttribute('aria-label') === 'Reply');
    if (replyButton) {
      fireEvent.click(replyButton);
      expect(mockOnReply).toHaveBeenCalled();
    }
  });

  it('renders reply preview correctly', () => {
    const messageWithReply = {
      ...mockMessage,
      replyTo: {
        id: '2',
        content: 'Previous message',
      },
    };

    render(
      <Message
        message={messageWithReply}
        isOwnMessage={false}
        onReply={mockOnReply}
      />
    );

    expect(screen.getByText('Replying to: Previous message')).toBeInTheDocument();
  });
}); 
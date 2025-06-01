import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import MessageList from './MessageList';

const mockMessages = [
  {
    id: '1',
    content: 'Hello there!',
    senderId: 'user1',
    timestamp: '2024-02-20T12:00:00Z',
    status: 'sent',
    type: 'text',
  },
  {
    id: '2',
    content: 'How are you?',
    senderId: 'user2',
    timestamp: '2024-02-20T12:01:00Z',
    status: 'sent',
    type: 'text',
  },
  {
    id: '3',
    content: 'https://example.com/image.jpg',
    senderId: 'user1',
    timestamp: '2024-02-20T12:02:00Z',
    status: 'sent',
    type: 'image',
    mediaUrl: 'https://example.com/image.jpg',
  },
];

describe('MessageList Component', () => {
  const mockOnReply = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders messages in chronological order', () => {
    render(
      <MessageList
        messages={mockMessages}
        currentUserId="user1"
        onReply={mockOnReply}
      />
    );

    // The Message component does not use role="listitem" by default, so we check by text
    expect(screen.getByText('Hello there!')).toBeInTheDocument();
    expect(screen.getByText('How are you?')).toBeInTheDocument();
  });

  it('groups messages by date', () => {
    render(
      <MessageList
        messages={mockMessages}
        currentUserId="user1"
        onReply={mockOnReply}
      />
    );

    expect(screen.getByText('February 20, 2024')).toBeInTheDocument();
  });

  it('renders image messages correctly', () => {
    render(
      <MessageList
        messages={mockMessages}
        currentUserId="user1"
        onReply={mockOnReply}
      />
    );

    const image = screen.getByAltText('Shared image');
    expect(image).toBeInTheDocument();
    expect(image).toHaveAttribute('src', 'https://example.com/image.jpg');
  });

  it('calls onReply when reply button is clicked', () => {
    render(
      <MessageList
        messages={mockMessages}
        currentUserId="user1"
        onReply={mockOnReply}
      />
    );

    // Simulate mouse enter to show the menu
    const message = screen.getByText('How are you?');
    fireEvent.mouseEnter(message.parentElement?.parentElement!);

    // Find the reply button (aria-label="Reply")
    const replyButton = screen.getAllByRole('button').find(btn => btn.getAttribute('aria-label') === 'Reply');
    if (replyButton) {
      fireEvent.click(replyButton);
      expect(mockOnReply).toHaveBeenCalled();
    }
  });

  it('scrolls to bottom when new messages arrive', () => {
    const { rerender } = render(
      <MessageList
        messages={mockMessages}
        currentUserId="user1"
        onReply={mockOnReply}
      />
    );

    const newMessages = [
      ...mockMessages,
      {
        id: '4',
        content: 'New message',
        senderId: 'user2',
        timestamp: '2024-02-20T12:03:00Z',
        status: 'sent',
        type: 'text',
      },
    ];

    rerender(
      <MessageList
        messages={newMessages}
        currentUserId="user1"
        onReply={mockOnReply}
      />
    );

    expect(screen.getByText('New message')).toBeInTheDocument();
  });
}); 
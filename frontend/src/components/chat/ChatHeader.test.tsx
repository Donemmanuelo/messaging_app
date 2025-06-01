import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import ChatHeader from './ChatHeader';

const mockChat = {
  id: '1',
  name: 'Test Group',
  isGroup: true,
  participants: [
    { id: 'user1', username: 'alice', displayName: 'Alice', status: 'online' as const },
    { id: 'user2', username: 'bob', displayName: 'Bob', status: 'offline' as const },
  ],
  createdAt: '2024-02-20T12:00:00Z',
  updatedAt: '2024-02-20T12:00:00Z',
  unreadCount: 0,
};

const mockUser = { id: 'user1', username: 'alice', displayName: 'Alice', status: 'online' as const };

describe('ChatHeader Component', () => {
  const mockOnStartCall = vi.fn();
  const mockOnViewProfile = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders group chat name and member count', () => {
    render(
      <ChatHeader
        chat={mockChat}
        currentUser={mockUser}
        onStartCall={mockOnStartCall}
        onViewProfile={mockOnViewProfile}
      />
    );
    expect(screen.getByText('Test Group')).toBeInTheDocument();
    expect(screen.getByText('2 members')).toBeInTheDocument();
  });

  it('renders direct chat participant name and status', () => {
    const directChat = {
      ...mockChat,
      isGroup: false,
      participants: [
        { id: 'user1', username: 'alice', displayName: 'Alice', status: 'online' as const },
        { id: 'user2', username: 'bob', displayName: 'Bob', status: 'offline' as const },
      ],
      name: 'Alice & Bob',
    };
    render(
      <ChatHeader
        chat={directChat}
        currentUser={mockUser}
        onStartCall={mockOnStartCall}
        onViewProfile={mockOnViewProfile}
      />
    );
    expect(screen.getByText('Bob')).toBeInTheDocument();
    expect(screen.getByText('Offline')).toBeInTheDocument();
  });

  it('calls onStartCall when audio or video button is clicked', () => {
    const directChat = {
      ...mockChat,
      isGroup: false,
      participants: [
        { id: 'user1', username: 'alice', displayName: 'Alice', status: 'online' as const },
        { id: 'user2', username: 'bob', displayName: 'Bob', status: 'offline' as const },
      ],
      name: 'Alice & Bob',
    };
    render(
      <ChatHeader
        chat={directChat}
        currentUser={mockUser}
        onStartCall={mockOnStartCall}
        onViewProfile={mockOnViewProfile}
      />
    );
    const audioButton = screen.getAllByRole('button').find(btn => btn.innerHTML.includes('PhoneIcon'));
    const videoButton = screen.getAllByRole('button').find(btn => btn.innerHTML.includes('VideoCameraIcon'));
    if (audioButton) fireEvent.click(audioButton);
    if (videoButton) fireEvent.click(videoButton);
    expect(mockOnStartCall).toHaveBeenCalled();
  });

  it('calls onViewProfile when View Profile is clicked in menu', () => {
    render(
      <ChatHeader
        chat={mockChat}
        currentUser={mockUser}
        onStartCall={mockOnStartCall}
        onViewProfile={mockOnViewProfile}
      />
    );
    const menuButton = screen.getAllByRole('button').find(btn => btn.innerHTML.includes('EllipsisVerticalIcon'));
    if (menuButton) fireEvent.click(menuButton);
    const viewProfileButton = screen.getByText('View Profile');
    fireEvent.click(viewProfileButton);
    expect(mockOnViewProfile).toHaveBeenCalled();
  });
}); 
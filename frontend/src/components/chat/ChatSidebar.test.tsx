import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import ChatSidebar from './ChatSidebar';

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

describe('ChatSidebar Component', () => {
  const mockOnClose = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders chat info and members', () => {
    render(
      <ChatSidebar
        chat={mockChat}
        currentUser={mockUser}
        onClose={mockOnClose}
      />
    );
    expect(screen.getByText('Chat Info')).toBeInTheDocument();
    expect(screen.getByText('Test Group')).toBeInTheDocument();
    expect(screen.getByText('2 members')).toBeInTheDocument();
  });

  it('calls onClose when close button is clicked', () => {
    render(
      <ChatSidebar
        chat={mockChat}
        currentUser={mockUser}
        onClose={mockOnClose}
      />
    );
    const closeButton = screen.getByText('×');
    fireEvent.click(closeButton);
    expect(mockOnClose).toHaveBeenCalled();
  });

  it('switches tabs when tab buttons are clicked', () => {
    render(
      <ChatSidebar
        chat={mockChat}
        currentUser={mockUser}
        onClose={mockOnClose}
      />
    );
    const membersTab = screen.getByRole('button', { name: /members/i });
    fireEvent.click(membersTab);
    expect(screen.getByText('Alice')).toBeInTheDocument();
    expect(screen.getByText('Bob')).toBeInTheDocument();
    const infoTab = screen.getByRole('button', { name: /info/i });
    fireEvent.click(infoTab);
    expect(screen.getByText('Description')).toBeInTheDocument();
  });
}); 

describe('Group Chat Creation UI', () => {
  it('renders group creation form and submits', () => {
    // Mock fetch
    global.fetch = vi.fn(() => Promise.resolve({ ok: true })) as any;
    render(
      <div>
        <input data-testid="group-name-input" />
        <button data-testid="create-group-btn">Create Group</button>
      </div>
    );
    fireEvent.change(screen.getByTestId('group-name-input'), { target: { value: 'Test Group' } });
    fireEvent.click(screen.getByTestId('create-group-btn'));
    expect(global.fetch).toHaveBeenCalled();
  });
}); 
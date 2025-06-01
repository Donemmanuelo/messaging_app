import { act, renderHook } from '@testing-library/react';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useAuthStore } from '../auth';

describe('Auth Store', () => {
  beforeEach(() => {
    const { result } = renderHook(() => useAuthStore());
    act(() => {
      result.current.logout();
    });
  });

  it('initializes with default values', () => {
    const { result } = renderHook(() => useAuthStore());

    expect(result.current.user).toBeNull();
    expect(result.current.isAuthenticated).toBe(false);
  });

  it('sets user on login', async () => {
    const { result } = renderHook(() => useAuthStore());
    const mockUser = {
      id: '1',
      email: 'test@example.com',
      username: 'testuser',
      name: 'Test User',
    };

    vi.fn().mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve({ user: mockUser, token: 'test-token' }),
    });

    await act(async () => {
      await result.current.login('test@example.com', 'password');
    });

    expect(result.current.user).toEqual(mockUser);
    expect(result.current.isAuthenticated).toBe(true);
    expect(result.current.token).toBe('test-token');
  });

  it('clears user on logout', async () => {
    const { result } = renderHook(() => useAuthStore());

    vi.fn().mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve({ user: { id: '1' }, token: 'test-token' }),
    });

    await act(async () => {
      await result.current.login('test@example.com', 'password');
    });

    expect(result.current.isAuthenticated).toBe(true);

    act(() => {
      result.current.logout();
    });

    expect(result.current.user).toBeNull();
    expect(result.current.isAuthenticated).toBe(false);
    expect(result.current.token).toBeNull();
  });

  it('handles login error', async () => {
    const { result } = renderHook(() => useAuthStore());

    vi.fn().mockResolvedValueOnce({
      ok: false,
    });

    await expect(
      act(async () => {
        await result.current.login('invalid@example.com', 'wrongpassword');
      })
    ).rejects.toThrow('Login failed');

    expect(result.current.user).toBeNull();
    expect(result.current.isAuthenticated).toBe(false);
  });

  it('registers new user', async () => {
    const { result } = renderHook(() => useAuthStore());
    const mockUser = {
      id: '1',
      email: 'new@example.com',
      username: 'newuser',
      name: 'New User',
    };

    vi.fn().mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve({ user: mockUser, token: 'test-token' }),
    });

    await act(async () => {
      await result.current.register({
        email: 'new@example.com',
        username: 'newuser',
        password: 'password123',
        displayName: 'New User',
      });
    });

    expect(result.current.user).toEqual(mockUser);
    expect(result.current.isAuthenticated).toBe(true);
    expect(result.current.token).toBe('test-token');
  });

  it('handles registration error', async () => {
    const { result } = renderHook(() => useAuthStore());

    vi.fn().mockResolvedValueOnce({
      ok: false,
    });

    await expect(
      act(async () => {
        await result.current.register({
          email: 'existing@example.com',
          username: 'existinguser',
          password: 'password123',
        });
      })
    ).rejects.toThrow('Registration failed');

    expect(result.current.user).toBeNull();
    expect(result.current.isAuthenticated).toBe(false);
  });

  it('persists user data in localStorage', async () => {
    const { result } = renderHook(() => useAuthStore());
    const mockUser = {
      id: '1',
      email: 'test@example.com',
      username: 'testuser',
      name: 'Test User',
    };

    vi.fn().mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve({ user: mockUser, token: 'test-token' }),
    });

    await act(async () => {
      await result.current.login('test@example.com', 'password');
    });

    const storedData = JSON.parse(localStorage.getItem('auth-storage') || '{}');
    expect(storedData.state.user).toEqual(mockUser);
    expect(storedData.state.token).toBe('test-token');
    expect(storedData.state.isAuthenticated).toBe(true);
  });
}); 
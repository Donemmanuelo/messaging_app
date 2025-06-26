import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface User {
  id: string;
  username: string;
  email: string;
}

interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
  login: (phone_number: string, otp_code: string) => Promise<void>;
  requestOtp: (phone_number: string) => Promise<void>;
  verifyOtp: (phone_number: string, otp_code: string) => Promise<void>;
  logout: () => void;
  clearError: () => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      user: null,
      token: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,

      requestOtp: async (phone_number: string) => {
        set({ isLoading: true, error: null });
        try {
          const response = await fetch('/api/auth/request_otp', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ phone_number }),
          });
          if (!response.ok) {
            const data = await response.json();
            throw new Error(data.message || 'Failed to send OTP');
          }
          set({ isLoading: false });
        } catch (error) {
          set({
            error: error instanceof Error ? error.message : 'Failed to send OTP',
            isLoading: false,
          });
          throw error;
        }
      },

      verifyOtp: async (phone_number: string, otp_code: string) => {
        set({ isLoading: true, error: null });
        try {
          const response = await fetch('/api/auth/verify_otp', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ phone_number, otp_code }),
          });
          const data = await response.json();
          if (!response.ok) {
            throw new Error(data.message || 'OTP verification failed');
          }
          set({
            user: data.user,
            token: data.token,
            isAuthenticated: true,
            isLoading: false,
          });
        } catch (error) {
          set({
            error: error instanceof Error ? error.message : 'OTP verification failed',
            isLoading: false,
          });
          throw error;
        }
      },

      login: async (phone_number: string, otp_code: string) => {
        // For compatibility, just call verifyOtp
        return await (useAuthStore.getState().verifyOtp(phone_number, otp_code));
      },

      logout: () => {
        set({
          user: null,
          token: null,
          isAuthenticated: false,
          error: null,
        });
      },

      clearError: () => {
        set({ error: null });
      },
    }),
    {
      name: 'auth-storage',
    }
  )
); 
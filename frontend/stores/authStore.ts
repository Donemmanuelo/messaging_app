import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import axios from 'axios'
import toast from 'react-hot-toast'

interface User {
  id: string
  username: string
  email: string
  display_name?: string
  avatar_url?: string
  status?: string
  is_online: boolean
}

interface AuthState {
  user: User | null
  accessToken: string | null
  refreshToken: string | null
  isLoading: boolean
  login: (email: string, password: string) => Promise<boolean>
  register: (username: string, email: string, password: string, displayName?: string) => Promise<boolean>
  logout: () => void
  updateProfile: (data: Partial<User>) => Promise<void>
}

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8000'

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      accessToken: null,
      refreshToken: null,
      isLoading: false,

      login: async (email: string, password: string) => {
        set({ isLoading: true })
        try {
          const response = await axios.post(`${API_URL}/api/auth/login`, {
            email,
            password,
          })

          const { user, access_token, refresh_token } = response.data
          
          set({
            user,
            accessToken: access_token,
            refreshToken: refresh_token,
            isLoading: false,
          })

          // Set default axios header
          axios.defaults.headers.common['Authorization'] = `Bearer ${access_token}`
          
          toast.success('Login successful!')
          return true
        } catch (error: any) {
          set({ isLoading: false })
          toast.error(error.response?.data?.message || 'Login failed')
          return false
        }
      },

      register: async (username: string, email: string, password: string, displayName?: string) => {
        set({ isLoading: true })
        try {
          const response = await axios.post(`${API_URL}/api/auth/register`, {
            username,
            email,
            password,
            display_name: displayName,
          })

          const { user, access_token, refresh_token } = response.data
          
          set({
            user,
            accessToken: access_token,
            refreshToken: refresh_token,
            isLoading: false,
          })

          axios.defaults.headers.common['Authorization'] = `Bearer ${access_token}`
          
          toast.success('Registration successful!')
          return true
        } catch (error: any) {
          set({ isLoading: false })
          toast.error(error.response?.data?.message || 'Registration failed')
          return false
        }
      },

      logout: () => {
        set({
          user: null,
          accessToken: null,
          refreshToken: null,
        })
        delete axios.defaults.headers.common['Authorization']
        toast.success('Logged out successfully')
      },

      updateProfile: async (data: Partial<User>) => {
        try {
          const response = await axios.post(`${API_URL}/api/users/profile`, data)
          set({ user: response.data })
          toast.success('Profile updated successfully')
        } catch (error: any) {
          toast.error(error.response?.data?.message || 'Failed to update profile')
        }
      },
    }),
    {
      name: 'auth-storage',
      partialize: (state) => ({
        user: state.user,
        accessToken: state.accessToken,
        refreshToken: state.refreshToken,
      }),
    }
  )
)

// Set axios interceptor for token refresh
axios.interceptors.response.use(
  (response) => response,
  async (error) => {
    if (error.response?.status === 401) {
      const { refreshToken } = useAuthStore.getState()
      if (refreshToken) {
        try {
          const response = await axios.post(`${API_URL}/api/auth/refresh`, {
            refresh_token: refreshToken,
          })
          const { access_token } = response.data
          useAuthStore.setState({ accessToken: access_token })
          axios.defaults.headers.common['Authorization'] = `Bearer ${access_token}`
          return axios.request(error.config)
        } catch (refreshError) {
          useAuthStore.getState().logout()
        }
      }
    }
    return Promise.reject(error)
  }
)
'use client'

import { useState } from 'react'
import { useRouter } from 'next/navigation'
import Link from 'next/link'
import { useAuthStore } from '@/stores/authStore'

export default function LoginPage() {
  const router = useRouter()
  const { requestOtp, login, isLoading, error } = useAuthStore()
  const [phone, setPhone] = useState('')
  const [otp, setOtp] = useState('')
  const [otpSent, setOtpSent] = useState(false)
  const [localError, setLocalError] = useState('')

  const handleRequestOtp = async (e: React.FormEvent) => {
    e.preventDefault()
    setLocalError('')
    try {
      await requestOtp(phone)
      setOtpSent(true)
    } catch (err) {
      setLocalError('Failed to send OTP')
    }
  }

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault()
    setLocalError('')
    try {
      await login(phone, otp)
      router.push('/chat')
    } catch (err) {
      setLocalError('Invalid OTP or phone number')
    }
  }

  return (
    <div className="min-h-screen bg-whatsapp-teal flex items-center justify-center px-4">
      <div className="max-w-md w-full bg-white rounded-lg shadow-lg p-8">
        <div className="text-center mb-8">
          <h1 className="text-3xl font-bold text-whatsapp-teal">WhatsApp Clone</h1>
          <p className="text-gray-600 mt-2">Sign in with your phone number</p>
        </div>

        <form onSubmit={otpSent ? handleLogin : handleRequestOtp} className="space-y-6">
          <div>
            <label htmlFor="phone" className="block text-sm font-medium text-gray-700">
              Phone Number
            </label>
            <input
              id="phone"
              type="tel"
              className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-whatsapp-green focus:border-whatsapp-green"
              placeholder="Enter your phone number"
              value={phone}
              onChange={e => setPhone(e.target.value)}
              required
            />
          </div>
          {otpSent && (
            <div>
              <label htmlFor="otp" className="block text-sm font-medium text-gray-700">
                OTP Code
              </label>
              <input
                id="otp"
                type="text"
                className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-whatsapp-green focus:border-whatsapp-green"
                placeholder="Enter the OTP you received"
                value={otp}
                onChange={e => setOtp(e.target.value)}
                required
              />
            </div>
          )}
          {(localError || error) && (
            <p className="mt-1 text-sm text-red-600">{localError || error}</p>
          )}
          <button
            type="submit"
            disabled={isLoading}
            className="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-whatsapp-green hover:bg-whatsapp-green-dark focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-whatsapp-green disabled:opacity-50"
          >
            {isLoading ? (otpSent ? 'Signing in...' : 'Sending OTP...') : (otpSent ? 'Sign in' : 'Send OTP')}
          </button>
        </form>

        <div className="mt-6 text-center">
          <p className="text-sm text-gray-600">
            Don't have an account?{' '}
            <Link href="/auth/register" className="font-medium text-whatsapp-green hover:text-whatsapp-green-dark">
              Sign up
            </Link>
          </p>
        </div>
      </div>
    </div>
  )
}
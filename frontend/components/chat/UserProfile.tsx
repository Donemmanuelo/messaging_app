'use client'

import { useState } from 'react'
import { useAuthStore } from '@/stores/authStore'
import { XMarkIcon, PencilIcon } from '@heroicons/react/24/outline'
import { useForm } from 'react-hook-form'

interface UserProfileProps {
  onClose: () => void
}

interface ProfileForm {
  display_name: string
  status: string
}

export default function UserProfile({ onClose }: UserProfileProps) {
  const { user, updateProfile, logout } = useAuthStore()
  const [isEditing, setIsEditing] = useState(false)
  const { register, handleSubmit, formState: { errors } } = useForm<ProfileForm>({
    defaultValues: {
      display_name: user?.display_name || '',
      status: user?.status || ''
    }
  })

  const onSubmit = async (data: ProfileForm) => {
    await updateProfile(data)
    setIsEditing(false)
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg w-full max-w-md mx-4">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b">
          <h2 className="text-lg font-semibold">Profile</h2>
          <button
            onClick={onClose}
            className="p-1 rounded-full hover:bg-gray-100"
          >
            <XMarkIcon className="w-6 h-6" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6">
          {/* Avatar */}
          <div className="flex justify-center mb-6">
            <div className="relative">
              <img
                src={user?.avatar_url || '/default-avatar.png'}
                alt="Profile"
                className="w-24 h-24 rounded-full"
              />
              <button className="absolute bottom-0 right-0 bg-whatsapp-green text-white p-2 rounded-full hover:bg-whatsapp-green-dark">
                <PencilIcon className="w-4 h-4" />
              </button>
            </div>
          </div>

          {/* Profile Form */}
          <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Username
              </label>
              <input
                type="text"
                value={user?.username || ''}
                disabled
                className="w-full px-3 py-2 border border-gray-300 rounded-md bg-gray-50"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Email
              </label>
              <input
                type="email"
                value={user?.email || ''}
                disabled
                className="w-full px-3 py-2 border border-gray-300 rounded-md bg-gray-50"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Display Name
              </label>
              <input
                {...register('display_name')}
                type="text"
                disabled={!isEditing}
                className={`w-full px-3 py-2 border border-gray-300 rounded-md ${
                  isEditing ? 'bg-white' : 'bg-gray-50'
                }`}
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Status
              </label>
              <input
                {...register('status')}
                type="text"
                disabled={!isEditing}
                placeholder="Hey there! I am using WhatsApp Clone."
                className={`w-full px-3 py-2 border border-gray-300 rounded-md ${
                  isEditing ? 'bg-white' : 'bg-gray-50'
                }`}
              />
            </div>

            {/* Action Buttons */}
            <div className="flex space-x-3 pt-4">
              {isEditing ? (
                <>
                  <button
                    type="submit"
                    className="flex-1 bg-whatsapp-green text-white py-2 px-4 rounded-md hover:bg-whatsapp-green-dark"
                  >
                    Save
                  </button>
                  <button
                    type="button"
                    onClick={() => setIsEditing(false)}
                    className="flex-1 bg-gray-300 text-gray-700 py-2 px-4 rounded-md hover:bg-gray-400"
                  >
                    Cancel
                  </button>
                </>
              ) : (
                <button
                  type="button"
                  onClick={() => setIsEditing(true)}
                  className="flex-1 bg-whatsapp-green text-white py-2 px-4 rounded-md hover:bg-whatsapp-green-dark"
                >
                  Edit Profile
                </button>
              )}
            </div>
          </form>

          {/* Logout Button */}
          <button
            onClick={logout}
            className="w-full mt-4 bg-red-500 text-white py-2 px-4 rounded-md hover:bg-red-600"
          >
            Logout
          </button>
        </div>
      </div>
    </div>
  )
}
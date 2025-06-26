-- Add phone number and OTP fields to users table
ALTER TABLE users
ADD COLUMN phone_number VARCHAR(20) UNIQUE,
ADD COLUMN otp_code VARCHAR(10),
ADD COLUMN otp_expires_at TIMESTAMP; 
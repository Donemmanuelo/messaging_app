import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const messageLatency = new Trend('message_latency');
const mediaUploadLatency = new Trend('media_upload_latency');

// Test configuration
export const options = {
  stages: [
    { duration: '1m', target: 50 },  // Ramp up to 50 users
    { duration: '3m', target: 50 },  // Stay at 50 users
    { duration: '1m', target: 100 }, // Ramp up to 100 users
    { duration: '3m', target: 100 }, // Stay at 100 users
    { duration: '1m', target: 0 },   // Ramp down to 0 users
  ],
  thresholds: {
    'errors': ['rate<0.1'],          // Error rate should be less than 10%
    'message_latency': ['p(95)<500'], // 95% of requests should be below 500ms
    'media_upload_latency': ['p(95)<2000'], // 95% of uploads should be below 2s
  },
};

const BASE_URL = __ENV.API_URL || 'http://localhost:3000';
const TOKEN = __ENV.AUTH_TOKEN;

export function setup() {
  // Create test users and get authentication tokens
  const res = http.post(`${BASE_URL}/api/auth/register`, {
    username: `test_user_${Date.now()}`,
    password: 'test_password',
  });
  
  check(res, {
    'user created': (r) => r.status === 201,
  });
  
  return { token: res.json('token') };
}

export default function(data) {
  const params = {
    headers: {
      'Authorization': `Bearer ${data.token}`,
      'Content-Type': 'application/json',
    },
  };

  // Test message sending
  const messageStart = new Date();
  const messageRes = http.post(
    `${BASE_URL}/api/messages`,
    JSON.stringify({
      content: `Test message ${Date.now()}`,
      receiver_id: 'test_receiver_id',
    }),
    params
  );
  messageLatency.add(new Date() - messageStart);

  check(messageRes, {
    'message sent': (r) => r.status === 201,
  });

  // Test message retrieval
  if (messageRes.status === 201) {
    const messageId = messageRes.json('id');
    const getRes = http.get(`${BASE_URL}/api/messages/${messageId}`, params);
    
    check(getRes, {
      'message retrieved': (r) => r.status === 200,
    });
  }

  // Test media upload (every 5th request)
  if (__ITER % 5 === 0) {
    const mediaStart = new Date();
    const formData = {
      file: http.file('test_image.jpg', 'image/jpeg'),
    };
    
    const uploadRes = http.post(`${BASE_URL}/api/media`, formData, {
      ...params,
      headers: {
        ...params.headers,
        'Content-Type': 'multipart/form-data; boundary=----WebKitFormBoundary',
      },
    });
    
    mediaUploadLatency.add(new Date() - mediaStart);
    
    check(uploadRes, {
      'media uploaded': (r) => r.status === 201,
    });
  }

  // Add some sleep to prevent overwhelming the server
  sleep(1);
} 
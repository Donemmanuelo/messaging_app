// frontend/src/lib/e2ee.ts

// Generate an RSA-OAEP key pair
export async function generateKeyPair() {
  return await window.crypto.subtle.generateKey(
    {
      name: "RSA-OAEP",
      modulusLength: 4096,
      publicExponent: new Uint8Array([1, 0, 1]),
      hash: "SHA-256",
    },
    true,
    ["encrypt", "decrypt"]
  );
}

// Export a key to base64
export async function exportKey(key: CryptoKey, type: "spki" | "pkcs8") {
  const exported = await window.crypto.subtle.exportKey(type, key);
  return btoa(String.fromCharCode(...new Uint8Array(exported)));
}

// Import a public key from base64
export async function importPublicKey(base64: string) {
  const binary = Uint8Array.from(atob(base64), c => c.charCodeAt(0));
  return await window.crypto.subtle.importKey(
    "spki",
    binary,
    { name: "RSA-OAEP", hash: "SHA-256" },
    true,
    ["encrypt"]
  );
}

// Import a private key from base64
export async function importPrivateKey(base64: string) {
  const binary = Uint8Array.from(atob(base64), c => c.charCodeAt(0));
  return await window.crypto.subtle.importKey(
    "pkcs8",
    binary,
    { name: "RSA-OAEP", hash: "SHA-256" },
    true,
    ["decrypt"]
  );
}

// Store private key in localStorage
export function storePrivateKey(base64: string) {
  localStorage.setItem("privateKey", base64);
}

// Retrieve private key from localStorage
export function getPrivateKey() {
  return localStorage.getItem("privateKey");
}

// Upload public key to backend
export async function uploadPublicKey(userId: string, publicKeyBase64: string, accessToken: string) {
  await fetch(`/api/users/${userId}/public_key`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "Authorization": `Bearer ${accessToken}`,
    },
    body: JSON.stringify({ public_key: publicKeyBase64 }),
  });
}

// Encrypt a message with a recipient's public key
export async function encryptMessage(message: string, recipientPublicKey: CryptoKey) {
  const encoder = new TextEncoder();
  const data = encoder.encode(message);
  const encrypted = await window.crypto.subtle.encrypt(
    { name: "RSA-OAEP" },
    recipientPublicKey,
    data
  );
  return btoa(String.fromCharCode(...new Uint8Array(encrypted)));
}

// Decrypt a message with the user's private key
export async function decryptMessage(encryptedBase64: string, privateKey: CryptoKey) {
  const encryptedData = Uint8Array.from(atob(encryptedBase64), c => c.charCodeAt(0));
  const decrypted = await window.crypto.subtle.decrypt(
    { name: "RSA-OAEP" },
    privateKey,
    encryptedData
  );
  const decoder = new TextDecoder();
  return decoder.decode(decrypted);
} 
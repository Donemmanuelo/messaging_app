/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './pages/**/*.{js,ts,jsx,tsx,mdx}',
    './components/**/*.{js,ts,jsx,tsx,mdx}',
    './app/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {
      colors: {
        whatsapp: {
          green: '#25D366',
          'green-dark': '#128C7E',
          'green-light': '#DCF8C6',
          teal: '#075E54',
          blue: '#34B7F1',
          'gray-light': '#ECE5DD',
          'gray-dark': '#3C3C3C',
        }
      },
      backgroundImage: {
        'chat-pattern': "url('/chat-bg.png')",
      }
    },
  },
  plugins: [],
}
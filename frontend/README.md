# Messaging App Frontend (Next.js)

## Features
- Login/Register
- Chat list & chat window
- Group chats
- File upload
- Real-time messaging (WebSocket)

## Setup
```sh
npm install
```

## Run (Dev)
```sh
npm run dev
```

## Deployment

### Vercel
- Push your repo to GitHub.
- Import to [vercel.com](https://vercel.com/).
- Set `NEXT_PUBLIC_API_URL` in Vercel dashboard to your backend URL (e.g. `https://your-backend.com`).

### Netlify
- Push your repo to GitHub.
- Import to [netlify.com](https://netlify.com/).
- Set `NEXT_PUBLIC_API_URL` in Netlify dashboard to your backend URL.

### Docker
```sh
docker build -t messaging-frontend .
docker run -p 3000:3000 -e NEXT_PUBLIC_API_URL=http://your-backend:8080 messaging-frontend
```

This is a [Next.js](https://nextjs.org) project bootstrapped with [`create-next-app`](https://nextjs.org/docs/app/api-reference/cli/create-next-app).

## Getting Started

First, run the development server:

```bash
npm run dev
# or
yarn dev
# or
pnpm dev
# or
bun dev
```

Open [http://localhost:3000](http://localhost:3000) with your browser to see the result.

You can start editing the page by modifying `app/page.tsx`. The page auto-updates as you edit the file.

This project uses [`next/font`](https://nextjs.org/docs/app/building-your-application/optimizing/fonts) to automatically optimize and load [Geist](https://vercel.com/font), a new font family for Vercel.

## Learn More

To learn more about Next.js, take a look at the following resources:

- [Next.js Documentation](https://nextjs.org/docs) - learn about Next.js features and API.
- [Learn Next.js](https://nextjs.org/learn) - an interactive Next.js tutorial.

You can check out [the Next.js GitHub repository](https://github.com/vercel/next.js) - your feedback and contributions are welcome!

## Deploy on Vercel

The easiest way to deploy your Next.js app is to use the [Vercel Platform](https://vercel.com/new?utm_medium=default-template&filter=next.js&utm_source=create-next-app&utm_campaign=create-next-app-readme) from the creators of Next.js.

Check out our [Next.js deployment documentation](https://nextjs.org/docs/app/building-your-application/deploying) for more details.

## Environment Variables

Copy `.env.example` to `.env.local` and set the following:

- `NEXT_PUBLIC_API_URL`: Backend API base URL (e.g., http://localhost:8080/api)
- `NEXT_PUBLIC_WS_URL`: WebSocket URL (e.g., ws://localhost:8080/ws)

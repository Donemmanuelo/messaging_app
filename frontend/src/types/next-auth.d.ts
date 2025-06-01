import NextAuth from "next-auth";

declare module "next-auth" {
  interface Session {
    accessToken?: string;
    user: {
      id: string;
      username: string;
      displayName: string;
      avatar: string;
      email: string;
    };
  }
} 
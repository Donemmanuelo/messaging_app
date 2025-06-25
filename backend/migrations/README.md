# WhatsApp Clone – Database Migrations

## Migration Order
Apply the migrations in the following order (handled automatically by `sqlx migrate run`):

1. 20240620100000_create_users.sql
2. 20240620100001_create_sessions.sql
3. 20240620100002_create_chats.sql
4. 20240620100003_create_chat_participants.sql
5. 20240620100004_create_messages.sql
6. 20240620100005_create_message_receipts.sql
7. 20240620100006_create_push_subscriptions.sql
8. 20240620100007_create_contacts.sql
9. 20240620100008_create_e2ee_keys.sql
10. 20240620100009_create_indexes.sql
11. 20240620100010_create_groups.sql
12. 20240620100011_create_group_members.sql
13. 20240620100012_create_message_reactions.sql
14. 20240620100013_create_message_reads.sql

## What Each Migration Does
- **create_users.sql**: Users table, with email, phone, password, profile, and indexes
- **create_sessions.sql**: Sessions table for session/token storage
- **create_chats.sql**: Chats table for 1-to-1 and group conversations, with group metadata
- **create_chat_participants.sql**: Participants table (many-to-many users ↔ chats), with roles and join time
- **create_messages.sql**: Messages table, supports text, media, reply-to, and timestamps
- **create_message_receipts.sql**: Message receipts (seen/delivered per user)
- **create_push_subscriptions.sql**: Push subscriptions for web push/Firebase tokens
- **create_contacts.sql**: Contacts/friend requests system
- **create_e2ee_keys.sql**: Table for storing encrypted keys for end-to-end encryption
- **create_indexes.sql**: Indexes for performance
- **create_groups.sql**: Groups table for group metadata
- **create_group_members.sql**: Group members table for group membership and roles
- **create_message_reactions.sql**: Message reactions table for emoji reactions
- **create_message_reads.sql**: Message reads table for tracking which users have read which messages

## How to Run Migrations

1. Make sure your `.env` file contains the correct `DATABASE_URL` for your Postgres database.
2. From the `backend` directory, run:

```sh
cargo sqlx migrate run
```

This will apply all migrations in order.

## Notes
- If you need to reset the database, you may need to drop all tables and re-run the migrations.
- If you add new features, create a new migration file using:
  ```sh
  cargo sqlx migrate add <description>
  ```
- Only a superuser or the database owner can run `CREATE EXTENSION` (for `pgcrypto`). 
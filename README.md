# Posts API

This API, developed in Rust, offers a straightforward way to interact with posts. It integrates Axum, JWT authentication via jsonwebtoken, a PostgreSQL database, and uses sqlx for database queries. Configuration is managed with .env, while logging is handled by tracing and log_env. Additionally, Argon2 is used for secure password hashing.

## Get Started

1. Clone the Repository

   ```bash
   git clone github.com/victorbonato/posts-api && cd posts-api
   ```

2. Start PostgreSQL

   ```bash
   make postgres
   ```

3. Run Migrations

   ```bash
   make migrate
   ```

4. Launch the Application

   ```bash
   make run
   ```

## User API Routes

- **Create User**

  - `POST /api/users`
  - Body: `{ "user": { "username": "string", "password": "string" } }`

- **Login User**

  - `POST /api/users/login`
  - Body: `{ "user": { "username": "string", "password": "string" } }`

  Important: You need to add the authentication token you receive to the headers of your http client, like this: `Authorization: Bearer {token_here}`

- **Get Current User**

  - `GET /api/user`

- **Update User**
  - `PATCH /api/user`
  - Body: `{ "user": { "username": "string", "password": "string" } }`

## Post API Routes

- **List All Posts**

  - `GET /api/posts`

- **Get User Posts**

  - `GET /api/:username/posts`
  - Path Variable: `username`

- **Add Post**

  - `POST /api/posts`
  - Body: `{ "post": { "title": "string", "content": "string" } }`

- **Delete Post**

  - `DELETE /api/posts/:post_id`
  - Path Variable: `post_id` (integer)

- **Update Post**
  - `PATCH /api/posts/:post_id`
  - Path Variable: `post_id` (integer)
  - Body: `{ "post": { "user_id": "Uuid", "title": "string", "content": "string" } }`

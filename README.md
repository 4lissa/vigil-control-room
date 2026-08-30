# VIGIL

VIGIL is a control room for teams deploying to production.

It handles two types of events:

* **Releases** — planned deployments validated step by step.
* **Incidents** — production issues that need to be investigated and resolved.

Incidents and releases are linked: an incident can block a release in progress.
VIGIL also includes a rule engine that can turn external events, such as a failed GitHub CI run, into automatic actions.

---

## Architecture

```text
                  External services
                       GitHub
                         │
                      Webhook
                         ▼
┌─────────────────────────────────────────────────┐
│                Rust / Axum Server               │
│                                                 │
│  Webhooks → Rule Engine → VIGIL / HTTP actions  │
│                                                 │
│  REST → Services → Repositories → PostgreSQL    │
│                                                 │
│              WebSocket Hub                      │
└────────────────────┬────────────────────────────┘
                     │
              REST + WebSockets
              ┌──────┴──────┐
              ▼             ▼
        Next.js Web       Tauri
          Client          Desktop
```

The desktop client loads the same web application inside a native Tauri window. It only adds desktop-specific features such as a tray icon and native notifications.

---

## Stack

| Component      | Technology           |
| -------------- | -------------------- |
| Server         | Rust / Axum          |
| Web client     | Next.js / TypeScript |
| Desktop client | Tauri                |
| Database       | PostgreSQL           |
| Real-time      | WebSockets           |
| Containers     | Docker Compose       |
| CI/CD          | GitHub Actions       |

### Why Rust / Axum?

VIGIL contains several state machines for incidents, releases and permissions. Rust's type system helps make these transitions safer, while `sqlx` provides compile-time SQL checking. Axum also provides a simple async model for both REST and WebSockets.

### Why Tauri?

The server is already written in Rust, and Tauri allows us to reuse the web client instead of maintaining a separate desktop UI. It also produces smaller applications than Electron by using the operating system's WebView.

### Why PostgreSQL?

Several users can modify data concurrently, for example when assigning incidents or validating release steps. PostgreSQL is well suited for this and also provides useful `ENUM` and `JSONB` types for our state and rule data.

---

## Installation

### Docker Compose

```bash
cp server/.env.example server/.env
```

Fill in:

```text
GITHUB_CLIENT_ID
GITHUB_CLIENT_SECRET
TOKEN_ENCRYPTION_KEY
KICKOFF_TOKEN
```

Then:

```bash
docker compose up --build
```

* Web client: `http://localhost:8081`
* Server: `http://localhost:8080`
* Desktop binary: `http://localhost:8081/client.dmg`

PostgreSQL migrations are run automatically when the server starts.

### Local setup

Server:

```bash
cd server
cargo run
```

Web client:

```bash
cd client_web
pnpm install
pnpm dev
```

Desktop:

```bash
cd client_desktop/src-tauri
cargo tauri dev
```

The desktop client requires the web client to be running on port `8081`.

---

# REST API

All routes use `http://localhost:8080` as their base URL.

Authenticated routes require:

```http
Authorization: Bearer <session_id>
```

Errors use the following format:

```json
{
  "error": {
    "code": "...",
    "message": "..."
  }
}
```

## Auth

| Method          | Route                          | Auth | Description               |
| --------------- | ------------------------------ | ---- | ------------------------- |
| POST            | `/register`                    |      | Register                  |
| POST            | `/login`                       |      | Login                     |
| GET             | `/github`                      |      | GitHub OAuth              |
| GET             | `/github/callback`             |      | OAuth callback            |
| POST            | `/logout`                      | ✓    | Logout                    |
| GET             | `/me`                          | ✓    | Get current user          |
| PATCH           | `/me`                          | ✓    | Update profile            |
| GET/POST/DELETE | `/connected-services/:service` | ✓    | Manage connected services |

## Teams

| Method | Route                                 | Auth    | Description               |
| ------ | ------------------------------------- | ------- | ------------------------- |
| POST   | `/teams`                              | ✓       | Create team               |
| GET    | `/teams`                              | ✓       | List teams                |
| POST   | `/teams/join`                         | ✓       | Join with invitation code |
| GET    | `/teams/:teamId`                      | ✓       | Team details              |
| GET    | `/teams/:teamId/members`              | ✓       | List members              |
| POST   | `/teams/:teamId/invite`               | Manager | Generate invitation       |
| POST   | `/teams/:teamId/transfer`             | Manager | Transfer Manager role     |
| GET    | `/teams/:teamId/bans`                 | ✓       | List bans                 |
| POST   | `/teams/:teamId/members/:userId/kick` | Manager | Kick member               |
| POST   | `/teams/:teamId/members/:userId/ban`  | Manager | Ban member                |
| DELETE | `/teams/:teamId/members/:userId/ban`  | Manager | Remove ban                |

## Incidents

| Method | Route                          | Auth       | Description        |
| ------ | ------------------------------ | ---------- | ------------------ |
| GET    | `/teams/:teamId/incidents`     | ✓          | List incidents     |
| POST   | `/teams/:teamId/incidents`     | Manager    | Create incident    |
| GET    | `/teams/:teamId/incidents/:id` | ✓          | Incident details   |
| POST   | `.../:id/acknowledge`          | Responder+ | Acknowledge        |
| POST   | `.../:id/escalate`             | Responder+ | Increase severity  |
| POST   | `.../:id/resolve`              | Manager    | Resolve            |
| POST   | `.../:id/assign`               | Manager    | Assign/unassign    |
| GET    | `.../:id/timeline`             | ✓          | Get timeline       |
| POST   | `.../:id/timeline`             | Responder+ | Add timeline entry |
| PATCH  | `.../:id/timeline/:entryId`    | Author     | Edit own entry     |

### Reactions

```text
GET    /reactions/available
GET    /teams/:teamId/incidents/:id/timeline/reactions
POST   /teams/:teamId/incidents/:id/timeline/:entryId/reactions
DELETE /teams/:teamId/incidents/:id/timeline/:entryId/reactions/:emoji
```

Available reactions:

```text
+1  -1  eyes  warning  check  fire
```

## Releases

| Method | Route                               | Auth       | Description        |
| ------ | ----------------------------------- | ---------- | ------------------ |
| GET    | `/teams/:teamId/releases`           | ✓          | List releases      |
| POST   | `/teams/:teamId/releases`           | Manager    | Create release     |
| GET    | `/teams/:teamId/releases/:id`       | ✓          | Release details    |
| GET    | `/teams/:teamId/releases/:id/steps` | ✓          | List steps         |
| POST   | `.../:id/steps/:stepId/validate`    | Responder+ | Validate next step |
| POST   | `.../:id/cancel`                    | Manager    | Cancel release     |

## Messages

| Method | Route               | Auth | Description        |
| ------ | ------------------- | ---- | ------------------ |
| GET    | `/messages`         | ✓    | List conversations |
| GET    | `/messages/:userId` | ✓    | Get conversation   |
| POST   | `/messages/:userId` | ✓    | Send message       |

## Rule Engine

| Method | Route                          | Auth    | Description    |
| ------ | ------------------------------ | ------- | -------------- |
| GET    | `/teams/:teamId/rules`         | Manager | List rules     |
| POST   | `/teams/:teamId/rules`         | Manager | Create rule    |
| PATCH  | `/teams/:teamId/rules/:ruleId` | Manager | Enable/disable |
| DELETE | `/teams/:teamId/rules/:ruleId` | Manager | Delete rule    |
| POST   | `/webhooks/github/:ruleId`     | HMAC    | GitHub webhook |

Currently supported:

* **Action:** GitHub
* **REActions:** Create a VIGIL Incident, or send an HTTP POST

Tokens and webhook secrets are encrypted using AES-256-GCM.

## Other

| Method | Route                   | Description                            |
| ------ | ----------------------- | -------------------------------------- |
| GET    | `/about.json`           | Service catalog and kickoff token hash |
| GET    | `/health`               | Health check                           |
| GET    | `/ws?token=<sessionId>` | WebSocket connection                   |

---

# Database

PostgreSQL migrations are located in `server/migrations/` and run automatically on startup.

```sql
-- Users and authentication
users (id, username, email, password_hash, github_id, language, created_at, updated_at)
sessions (id, user_id -> users, created_at, expires_at)

-- Teams and membership
teams (id, name, invitation_code, created_by -> users, created_at)
team_members (id, team_id -> teams, user_id -> users, role, joined_at)
team_bans (id, team_id -> teams, user_id -> users, banned_by -> users, until, created_at)

-- Incidents and their timeline
incidents (
    id, team_id -> teams, title, description, state, severity,
    created_by -> users, assigned_to -> users,
    release_id -> releases, created_at, resolved_at
)

timeline_entries (
    id, incident_id -> incidents, author_id -> users,
    content, created_at, edited_at
)

timeline_reactions (
    id, entry_id -> timeline_entries, user_id -> users,
    emoji, created_at
)

-- Releases and ordered steps
releases (id, team_id -> teams, name, state, created_by -> users, created_at, completed_at)
release_steps (id, release_id -> releases, name, position, validated_at, validated_by -> users)

-- Direct messages
messages (id, sender_id -> users, recipient_id -> users, content, created_at)

-- Rule engine
connected_services (id, user_id -> users, service, encrypted_token, created_at)

rules (
    id, team_id -> teams, name, enabled,
    trigger_service, trigger_event, trigger_filters,
    encrypted_webhook_secret,
    reaction_type, reaction_payload,
    created_by -> users, created_at
)
```

UUIDs are used for IDs. Foreign keys use `ON DELETE CASCADE` where appropriate. A partial unique index ensures that each team has only one Manager.

---

# Codebase

The server follows the same structure for each feature:

```text
server/src/features/
├── auth/
├── teams/
├── incidents/
├── releases/
├── messages/
└── rule_engine/
```

Each feature contains:

```text
routes.rs   → route definitions
handler.rs  → HTTP request handling
service.rs  → business logic and permissions
repo.rs     → PostgreSQL queries
model.rs    → domain models
dto.rs      → API request/response types
```

Shared server code is located in `server/src/shared/`:

```text
ws/hub.rs       → WebSocket broadcaster
ws/event.rs     → WebSocket event types
ws/handler.rs   → /ws connection handling
crypto.rs       → token encryption
middleware.rs   → authentication
```

The web client mirrors the same feature structure:

```text
client_web/src/features/
```

* `api.ts` handles HTTP calls.
* `hooks.ts` exposes them through React Query.
* `components/` contains UI components.

The Tauri client is only the native shell:

```text
client_desktop/src-tauri/
```

It contains no UI code.

---

## Testing

### Server

```bash
cargo test
cargo clippy
cargo fmt --check
cargo llvm-cov
```

### Web client

```bash
pnpm test
pnpm lint
pnpm format:check
pnpm test:coverage
```

The project maintains more than **70% line coverage**.

GitHub Actions runs tests and linting on pushes, integration tests and coverage on merges to `dev`/`main`, and builds Docker images and the desktop application for version tags.

---

## Additional documentation

* `WEBSOCKET_SPEC.md` — WebSocket events and payloads
* `HOWTOCONTRIBUTE.md` — how to extend the rule engine and WebSockets
* `UI_GUIDELINES.md` — UI rules and screenshots

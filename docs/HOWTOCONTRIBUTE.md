# How to Contribute

This document explains how to extend the main parts of VIGIL:

* Rule engine services
* Rule engine Actions
* Rule engine REActions
* WebSocket events

---

## Add a new service

A service is an external platform that can trigger a VIGIL rule.

For example, to add GitLab:

1. Create `server/src/features/rule_engine/webhooks/gitlab.rs`.
   Add the webhook payload, a `build_context()` function and the required signature verification.

2. Add a new webhook route in `rule_engine/routes.rs`:

   ```text
   POST /webhooks/gitlab/{rule_id}
   ```

3. Update `rule_engine/service.rs`:

   * add the supported trigger;
   * verify the webhook;
   * match the rule;
   * execute its REAction.

4. Add the service to `service_catalog()` in `rule_engine/model.rs`.

5. Add any required fields to `CreateRuleForm.tsx`.

The service will then be available through `/about.json`.

---

## Add a new Action

An Action is an event provided by a service, such as a GitHub workflow failure.

1. Add the new trigger to `SUPPORTED_TRIGGERS` in `rule_engine/service.rs`.

2. Add it to the corresponding service in `service_catalog()`.

3. If necessary, update the webhook's `build_context()` to expose the required data.

4. Add any required fields to `CreateRuleForm.tsx`.

---

## Add a new REAction

A REAction is what VIGIL does when a rule is triggered.

For example, to add a Discord message:

1. Add a new variant to `ReactionType` in `rule_engine/model.rs`.

2. Add the implementation to `execute_reaction()` in `rule_engine/service.rs`.

3. If the service requires authentication, add its token type to the connected services and use the existing encrypted token storage.

4. Add the REAction to `service_catalog()`.

5. Add it to the reaction selector in `CreateRuleForm.tsx`.

---

## Add a WebSocket event

1. Add a new variant to `WsEvent` in:

   ```text
   server/src/shared/ws/event.rs
   ```

   Events use `snake_case` automatically.

2. Send the event through the WebSocket Hub:

   ```rust
   broadcast_to_team(...)
   broadcast_to_watchers(...)
   send_to_user(...)
   ```

3. Add the event to:

   ```text
   client_web/src/shared/lib/ws-types.ts
   ```

4. Handle the event in the appropriate feature, usually in `hooks.ts`.

5. If needed, add a native notification in `shared/lib/notifications.ts`.

6. Document the event in `WEBSOCKET_SPEC.md`, including:

   * payload;
   * trigger condition;
   * target clients.

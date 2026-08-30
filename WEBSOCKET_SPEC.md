# WebSocket Specification

VIGIL uses one WebSocket connection per client:

```text
GET /ws?token=<sessionId>
```

The session ID is the same token used for REST authentication. The server returns `401` if the token is missing, invalid or expired.

If the connection is lost, the client automatically reconnects with exponential backoff, from 1 second up to 30 seconds.

## Client → Server

```json
{ "type": "ping" }
{ "type": "watch", "incident_id": "uuid" }
{ "type": "unwatch", "incident_id": "uuid" }
```

* `ping`: keeps the connection alive; the server replies with `pong`.
* `watch` / `unwatch`: starts or stops watching an incident. This is used for presence updates.

A client can only watch incidents belonging to one of its teams.

## Server → Client

There are three targets:

* **Team**: all connected members of the team.
* **Watchers**: clients watching the incident.
* **Direct**: only the users concerned.

| Event                      | Trigger                                    | Target   |
| -------------------------- | ------------------------------------------ | -------- |
| `pong`                     | `ping` received                            | Direct   |
| `incident_state_changed`   | Incident created, acknowledged or resolved | Team     |
| `incident_escalated`       | Severity increased                         | Team     |
| `incident_assigned`        | Incident assigned/unassigned               | Team     |
| `timeline_entry_added`     | Timeline entry added                       | Team     |
| `timeline_entry_edited`    | Entry edited                               | Team     |
| `reaction_added`           | Reaction added                             | Team     |
| `reaction_removed`         | Reaction removed                           | Team     |
| `presence_update`          | Watch/unwatch/disconnect                   | Watchers |
| `release_step_validated`   | Step validated                             | Team     |
| `release_state_changed`    | Release state changes                      | Team     |
| `private_message_received` | Private message sent                       | Direct   |
| `member_joined`            | Member joins                               | Team     |
| `member_kicked`            | Member kicked                              | Team     |
| `member_banned`            | Member banned                              | Team     |
| `rule_triggered`           | Rule succeeds                              | Team     |
| `rule_failed`              | Rule fails                                 | Team     |

### Incident events

```json
{
  "type": "incident_state_changed",
  "incident_id": "uuid",
  "new_state": "acknowledged",
  "by": "username"
}
```

```json
{
  "type": "incident_escalated",
  "incident_id": "uuid",
  "new_severity": "critical",
  "by": "username"
}
```

```json
{
  "type": "incident_assigned",
  "incident_id": "uuid",
  "assigned_to": "uuid-or-null",
  "by": "username"
}
```

### Timeline events

```json
{
  "type": "timeline_entry_added",
  "incident_id": "uuid",
  "entry_id": "uuid",
  "author": "username",
  "content": "Rolled back the last deploy",
  "created_at": 1755000000
}
```

```json
{
  "type": "timeline_entry_edited",
  "incident_id": "uuid",
  "entry_id": "uuid",
  "new_content": "Updated timeline entry",
  "edited_at": 1755000100
}
```

```json
{
  "type": "reaction_added",
  "incident_id": "uuid",
  "entry_id": "uuid",
  "emoji": "+1",
  "by": "username"
}
```

`reaction_removed` uses the same payload with `"type": "reaction_removed"`.

### Presence

```json
{
  "type": "presence_update",
  "incident_id": "uuid",
  "watchers": ["alissa", "bob"]
}
```

### Release events

```json
{
  "type": "release_step_validated",
  "release_id": "uuid",
  "step": "staging",
  "by": "username"
}
```

```json
{
  "type": "release_state_changed",
  "release_id": "uuid",
  "new_state": "blocked"
}
```

`release_state_changed` is sent when a release is created, completed, cancelled, blocked by an incident, or unblocked.

### Messages

```json
{
  "type": "private_message_received",
  "from": "alissa",
  "to": "bob",
  "content": "Got a sec?",
  "at": 1755000000
}
```

Sent only to the sender and recipient.

### Team events

```json
{
  "type": "member_joined",
  "team_id": "uuid",
  "member": "bob",
  "role": "observer"
}
```

```json
{
  "type": "member_kicked",
  "team_id": "uuid",
  "member": "bob",
  "by": "alissa"
}
```

```json
{
  "type": "member_banned",
  "team_id": "uuid",
  "member": "bob",
  "until": 1755000000,
  "by": "alissa"
}
```

`until` is `null` for a permanent ban.

### Rule engine

```json
{
  "type": "rule_triggered",
  "rule_name": "CI failure > Incident",
  "result": "incident_created",
  "incident_id": "uuid"
}
```

```json
{
  "type": "rule_failed",
  "rule_name": "CI failure > Incident",
  "error": "Failed to create incident"
}
```

### Keep-alive

```json
{
  "type": "pong"
}
```

`pong` is sent directly to the client that sent `ping`.

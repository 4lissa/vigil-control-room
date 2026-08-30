# UI Guidelines

## Color palette

Colors are defined as CSS variables in `client_web/src/app/globals.css`.

| Role      | Token                                      | Usage                                           |
| --------- | ------------------------------------------ | ----------------------------------------------- |
| Primary   | `--color-accent`                           | Primary buttons, links, active states           |
| Secondary | `--color-border`, `--color-text-secondary` | Secondary UI                                    |
| Success   | `--color-success`                          | Resolved and completed states                   |
| Warning   | `--color-warning`                          | High severity                                   |
| Danger    | `--color-danger`                           | Critical states, errors and destructive actions |
| Medium    | `--color-medium`                           | Medium severity                                 |

Components use these tokens instead of hard-coded colors.

---

## State and severity

States are always represented using **color + icon + text**, so the interface does not rely on color alone.

### Incident states

| State          | Color   | Icon          |
| -------------- | ------- | ------------- |
| `open`         | Accent  | Circle        |
| `acknowledged` | Medium  | Eye           |
| `escalated`    | Danger  | ArrowUpCircle |
| `resolved`     | Success | CheckCircle2  |

### Severity

| Level      | Color   | Icon      |
| ---------- | ------- | --------- |
| `low`      | Neutral | ArrowDown |
| `medium`   | Medium  | Minus     |
| `high`     | Warning | ArrowUp   |
| `critical` | Danger  | Flame     |

### Release states

| State         | Color   | Icon         |
| ------------- | ------- | ------------ |
| `created`     | Neutral | Circle       |
| `in_progress` | Accent  | PlayCircle   |
| `completed`   | Success | CheckCircle2 |
| `cancelled`   | Neutral | XCircle      |
| `blocked`     | Danger  | Lock         |

---

## Typography and spacing

The interface uses four text sizes:

* `title`: 24px
* `subtitle`: 16px
* `body`: 14px
* `caption`: 12px

Spacing follows Tailwind's default 4px scale to keep layouts consistent.

---

## Reusable components

| Component  | Variants / Rules                          |
| ---------- | ----------------------------------------- |
| `Button`   | `primary`, `secondary`, `danger`, loading |
| `Input`    | Label and optional error message          |
| `Badge`    | Status/severity with icon + label         |
| `Dialog`   | Confirmation and destructive actions      |
| `Dropdown` | Reusable dropdown menu                    |

Interactive elements use standard HTML controls such as `<button>`, `<a>`, `<form>` and `<select>` to keep the interface keyboard accessible.

---

## Dark patterns and mitigation

Destructive actions use confirmation dialogs with:

* the exact resource being affected;
* a clearly named destructive action;
* a separate cancel button;
* a danger style for the confirmation button.

| Action           | Confirmation             |
| ---------------- | ------------------------ |
| Kick member      | `Kick {username}`        |
| Ban member       | `Ban {username}`         |
| Transfer Manager | `Transfer to {username}` |
| Cancel Release   | `Confirm cancellation`   |
| Delete rule      | `Delete {rule name}`     |

Actions are not hidden behind unusual gestures or unclear menus.

---

## Screenshots

### 1. Incident list

![Incident list with state and severity badges](screenshots/incident-list-annotated.png)

**Annotations:**

1. State badge uses **color + icon + text**.
2. Severity uses a different visual level.
3. The same badge components are reused across the interface.

### 2. Destructive action

![Destructive action confirmation dialog](screenshots/delete-rule-dialog-annotated.png)

**Annotations:**

1. The dialog clearly identifies the resource being affected.
2. The destructive action uses the `danger` style.
3. The alternative action is clearly labelled and visually distinct.

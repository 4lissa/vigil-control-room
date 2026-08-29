import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook } from "@testing-library/react";
import { NextIntlClientProvider } from "next-intl";
import { useNativeNotifications } from "./notifications";
import { WsEvent } from "./ws-types";
import messages from "../../../messages/en.json";

const handlers: Array<(event: WsEvent) => void> = [];
const unsubscribe = vi.fn();

vi.mock("@/shared/providers/WebSocketProvider", () => ({
  useWebSocket: () => ({
    onMessage: (handler: (event: WsEvent) => void) => {
      handlers.push(handler);
      return unsubscribe;
    },
  }),
}));

const isTauri = vi.fn();
const isPermissionGranted = vi.fn();
const requestPermission = vi.fn();
const sendNotification = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  isTauri: () => isTauri(),
}));

vi.mock("@tauri-apps/plugin-notification", () => ({
  isPermissionGranted: () => isPermissionGranted(),
  requestPermission: () => requestPermission(),
  sendNotification: (options: unknown) => sendNotification(options),
}));

const emit = async (event: WsEvent) => {
  for (const handler of handlers) handler(event);
  await Promise.resolve();
  await Promise.resolve();
};

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <NextIntlClientProvider locale="en" messages={messages}>
    {children}
  </NextIntlClientProvider>
);

beforeEach(() => {
  handlers.length = 0;
  isTauri.mockReset().mockReturnValue(true);
  isPermissionGranted.mockReset().mockResolvedValue(true);
  requestPermission.mockReset().mockResolvedValue("granted");
  sendNotification.mockReset();
});

describe("useNativeNotifications", () => {
  it("does nothing when there is no current user id", async () => {
    renderHook(() => useNativeNotifications(undefined), { wrapper });
    expect(handlers).toHaveLength(0);
  });

  it("does nothing outside of the desktop app", async () => {
    isTauri.mockReturnValue(false);
    renderHook(() => useNativeNotifications("user-123"), { wrapper });

    await emit({
      type: "incident_assigned",
      incident_id: "i1",
      assigned_to: "user-123",
      by: "bob",
    });

    expect(sendNotification).not.toHaveBeenCalled();
  });

  it("notifies the assigned user when an incident is assigned to them", async () => {
    renderHook(() => useNativeNotifications("user-123"), { wrapper });

    await emit({
      type: "incident_assigned",
      incident_id: "i1",
      assigned_to: "user-123",
      by: "bob",
    });

    expect(sendNotification).toHaveBeenCalledWith({
      title: "Incident assigned to you",
      body: "You have been assigned to an incident. Open VIGIL to view it.",
    });
  });

  it("does not notify when the incident is assigned to someone else", async () => {
    renderHook(() => useNativeNotifications("user-123"), { wrapper });

    await emit({
      type: "incident_assigned",
      incident_id: "i1",
      assigned_to: "someone-else",
      by: "bob",
    });

    expect(sendNotification).not.toHaveBeenCalled();
  });

  it("notifies only when an incident escalates to critical, not to other severities", async () => {
    renderHook(() => useNativeNotifications("user-123"), { wrapper });

    await emit({
      type: "incident_escalated",
      incident_id: "i1",
      new_severity: "high",
      by: "bob",
    });
    expect(sendNotification).not.toHaveBeenCalled();

    await emit({
      type: "incident_escalated",
      incident_id: "i1",
      new_severity: "critical",
      by: "bob",
    });
    expect(sendNotification).toHaveBeenCalledOnce();
  });

  it("notifies only when a release becomes blocked, not on other state changes", async () => {
    renderHook(() => useNativeNotifications("user-123"), { wrapper });

    await emit({
      type: "release_state_changed",
      release_id: "r1",
      new_state: "completed",
    });
    expect(sendNotification).not.toHaveBeenCalled();

    await emit({
      type: "release_state_changed",
      release_id: "r1",
      new_state: "blocked",
    });
    expect(sendNotification).toHaveBeenCalledOnce();
  });

  it("ignores unrelated event types", async () => {
    renderHook(() => useNativeNotifications("user-123"), { wrapper });

    await emit({
      type: "member_kicked",
      team_id: "t1",
      member: "bob",
      by: "alissa",
    });

    expect(sendNotification).not.toHaveBeenCalled();
  });

  it("requests permission when not already granted, and notifies once granted", async () => {
    isPermissionGranted.mockResolvedValue(false);
    requestPermission.mockResolvedValue("granted");

    renderHook(() => useNativeNotifications("user-123"), { wrapper });
    await emit({
      type: "incident_assigned",
      incident_id: "i1",
      assigned_to: "user-123",
      by: "bob",
    });

    expect(requestPermission).toHaveBeenCalledOnce();
    expect(sendNotification).toHaveBeenCalledOnce();
  });

  it("does not notify when permission is requested and denied", async () => {
    isPermissionGranted.mockResolvedValue(false);
    requestPermission.mockResolvedValue("denied");

    renderHook(() => useNativeNotifications("user-123"), { wrapper });
    await emit({
      type: "incident_assigned",
      incident_id: "i1",
      assigned_to: "user-123",
      by: "bob",
    });

    expect(sendNotification).not.toHaveBeenCalled();
  });

  it("unsubscribes from the websocket when unmounted", () => {
    const { unmount } = renderHook(() => useNativeNotifications("user-123"), {
      wrapper,
    });
    unmount();
    expect(unsubscribe).toHaveBeenCalled();
  });
});

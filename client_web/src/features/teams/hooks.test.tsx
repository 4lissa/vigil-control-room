import { describe, it, expect, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/node";
import { useAuthStore } from "@/features/auth/store";
import {
  useTeams,
  useTeam,
  useTeamMembers,
  useCreateTeam,
  useJoinTeam,
  useGenerateInviteCode,
  useTransferManager,
} from "./hooks";

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
};

beforeEach(() => {
  useAuthStore.setState({ token: "my-session-token", user: null });
});

describe("useTeams", () => {
  it("fetches teams when token is present", async () => {
    const { result } = renderHook(() => useTeams(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toHaveLength(1);
    expect(result.current.data![0]).toMatchObject({ name: "My Team" });
  });

  it("does not fetch when token is null", async () => {
    useAuthStore.setState({ token: null, user: null });

    const { result } = renderHook(() => useTeams(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.fetchStatus).toBe("idle"));
    expect(result.current.data).toBeUndefined();
  });
});

describe("useTeam", () => {
  it("fetches a single team by id", async () => {
    const { result } = renderHook(() => useTeam("team-123"), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({
      id: "team-123",
      name: "My Team",
    });
  });
});

describe("useTeamMembers", () => {
  it("fetches members for a team", async () => {
    const { result } = renderHook(() => useTeamMembers("team-123"), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toHaveLength(1);
    expect(result.current.data![0]).toMatchObject({ role: "manager" });
  });
});

describe("useCreateTeam", () => {
  it("returns the created team and invalidates the teams cache on success", async () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
        mutations: { retry: false },
      },
    });
    queryClient.setQueryData(["teams"], []);
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );

    const { result } = renderHook(() => useCreateTeam(), { wrapper });

    result.current.mutate({ name: "My Team" });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({
      team: { name: "My Team" },
      member: { role: "manager" },
    });
    expect(queryClient.getQueryState(["teams"])?.isInvalidated).toBe(true);
  });

  it("fails when name is taken", async () => {
    server.use(
      http.post("http://localhost:8080/teams", () => {
        return HttpResponse.json(
          { error: { code: "CONFLICT", message: "Name already taken" } },
          { status: 409 },
        );
      }),
    );

    const { result } = renderHook(() => useCreateTeam(), {
      wrapper: createWrapper(),
    });

    result.current.mutate({ name: "My Team" });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error?.message).toBe("Name already taken");
  });
});

describe("useJoinTeam", () => {
  it("returns team on success", async () => {
    const { result } = renderHook(() => useJoinTeam(), {
      wrapper: createWrapper(),
    });

    result.current.mutate({ code: "ABC123" });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({ id: "team-123" });
  });

  it("fails on invalid code", async () => {
    server.use(
      http.post("http://localhost:8080/teams/join", () => {
        return HttpResponse.json(
          { error: { code: "NOT_FOUND", message: "Invalid invitation code" } },
          { status: 404 },
        );
      }),
    );

    const { result } = renderHook(() => useJoinTeam(), {
      wrapper: createWrapper(),
    });

    result.current.mutate({ code: "WRONG" });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error?.message).toBe("Invalid invitation code");
  });
});

describe("useGenerateInviteCode", () => {
  it("returns invitation code on success", async () => {
    const { result } = renderHook(() => useGenerateInviteCode("team-123"), {
      wrapper: createWrapper(),
    });

    result.current.mutate();

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({ invitation_code: "ABC123" });
  });
});

describe("useTransferManager", () => {
  it("invalidates the team members cache on success", async () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
        mutations: { retry: false },
      },
    });
    queryClient.setQueryData(["teams", "team-123", "members"], []);
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );

    const { result } = renderHook(() => useTransferManager("team-123"), {
      wrapper,
    });

    result.current.mutate({ user_id: "user-456" });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(
      queryClient.getQueryState(["teams", "team-123", "members"])
        ?.isInvalidated,
    ).toBe(true);
  });

  it("fails when target is not a member", async () => {
    server.use(
      http.post("http://localhost:8080/teams/:teamId/transfer", () => {
        return HttpResponse.json(
          { error: { code: "NOT_FOUND", message: "Member not found" } },
          { status: 404 },
        );
      }),
    );

    const { result } = renderHook(() => useTransferManager("team-123"), {
      wrapper: createWrapper(),
    });

    result.current.mutate({ user_id: "user-999" });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error?.message).toBe("Member not found");
  });
});

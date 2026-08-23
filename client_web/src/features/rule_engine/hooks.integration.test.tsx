import { describe, it, expect, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/node";
import { setToken } from "@/features/auth/token";
import { useRules, useCreateRule, useUpdateRule, useDeleteRule } from "./hooks";

const teamId = "team-123";
const ruleId = "rule-123";

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  const wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );

  return { wrapper, queryClient };
};

beforeEach(() => {
  setToken("my-session-token");
});

describe("useRules", () => {
  it("fetches rules for a team", async () => {
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useRules(teamId), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toHaveLength(1);
    expect(result.current.data![0]).toMatchObject({
      name: "CI failure > Incident",
    });
  });
});

describe("useCreateRule", () => {
  it("returns the created rule and invalidates the rules cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    queryClient.setQueryData(["teams", teamId, "rules"], []);

    const { result } = renderHook(() => useCreateRule(teamId), { wrapper });

    result.current.mutate({
      name: "CI failure > Incident",
      enabled: true,
      trigger: {
        service: "github",
        event: "workflow_run",
        filters: {
          "workflow_run.conclusion": "failure",
          "repository.full_name": "my-org/my-repo",
        },
      },
      webhook_secret: "shhh",
      reaction: {
        type: "vigil_create_incident",
        payload: { title: "CI broken", severity: "high" },
      },
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({
      name: "CI failure > Incident",
    });
    expect(
      queryClient.getQueryState(["teams", teamId, "rules"])?.isInvalidated,
    ).toBe(true);
  });

  it("fails for a non-manager", async () => {
    server.use(
      http.post("http://localhost:8080/teams/:teamId/rules", () => {
        return HttpResponse.json(
          {
            error: {
              code: "FORBIDDEN",
              message: "Only managers can configure rules",
            },
          },
          { status: 403 },
        );
      }),
    );

    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useCreateRule(teamId), { wrapper });

    result.current.mutate({
      name: "CI failure > Incident",
      enabled: true,
      trigger: { service: "github", event: "workflow_run", filters: {} },
      webhook_secret: "shhh",
      reaction: { type: "vigil_create_incident", payload: {} },
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error?.message).toBe(
      "Only managers can configure rules",
    );
  });
});

describe("useUpdateRule", () => {
  it("returns the updated rule and invalidates the rules cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    queryClient.setQueryData(["teams", teamId, "rules"], []);

    const { result } = renderHook(() => useUpdateRule(teamId), { wrapper });

    result.current.mutate({ ruleId, body: { enabled: false } });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({ enabled: false });
    expect(
      queryClient.getQueryState(["teams", teamId, "rules"])?.isInvalidated,
    ).toBe(true);
  });
});

describe("useDeleteRule", () => {
  it("invalidates the rules cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    queryClient.setQueryData(["teams", teamId, "rules"], []);

    const { result } = renderHook(() => useDeleteRule(teamId), { wrapper });

    result.current.mutate(ruleId);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(
      queryClient.getQueryState(["teams", teamId, "rules"])?.isInvalidated,
    ).toBe(true);
  });
});

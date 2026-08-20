import { describe, it, expect, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/node";
import { setToken } from "@/features/auth/token";
import {
  useIncidents,
  useIncident,
  useCreateIncident,
  useAcknowledgeIncident,
  useEscalateIncident,
  useResolveIncident,
  useAssignResponder,
  useTimelineEntries,
  useAddTimelineEntry,
  useEditTimelineEntry,
  useAvailableEmojis,
  useReactions,
  useAddReaction,
  useRemoveReaction,
} from "./hooks";

const teamId = "team-123";
const incidentId = "incident-123";

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

describe("useIncidents", () => {
  it("fetches incidents for a team", async () => {
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useIncidents(teamId), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toHaveLength(1);
    expect(result.current.data![0]).toMatchObject({ title: "DB down" });
  });
});

describe("useIncident", () => {
  it("fetches a single incident by id", async () => {
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useIncident(teamId, incidentId), {
      wrapper,
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({
      id: incidentId,
      state: "open",
    });
  });
});

describe("useCreateIncident", () => {
  it("returns the created incident and invalidates the incidents cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    queryClient.setQueryData(["teams", teamId, "incidents"], []);

    const { result } = renderHook(() => useCreateIncident(teamId), {
      wrapper,
    });

    result.current.mutate({ title: "DB down", severity: "high" });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({ title: "DB down" });
    expect(
      queryClient.getQueryState(["teams", teamId, "incidents"])?.isInvalidated,
    ).toBe(true);
  });

  it("fails for a non-manager", async () => {
    server.use(
      http.post("http://localhost:8080/teams/:teamId/incidents", () => {
        return HttpResponse.json(
          {
            error: {
              code: "FORBIDDEN",
              message: "Only managers can create an incident",
            },
          },
          { status: 403 },
        );
      }),
    );

    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useCreateIncident(teamId), {
      wrapper,
    });

    result.current.mutate({ title: "DB down", severity: "high" });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error?.message).toBe(
      "Only managers can create an incident",
    );
  });
});

describe("useAcknowledgeIncident", () => {
  it("returns the acknowledged incident", async () => {
    const { wrapper } = createWrapper();
    const { result } = renderHook(
      () => useAcknowledgeIncident(teamId, incidentId),
      { wrapper },
    );

    result.current.mutate();

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({ state: "acknowledged" });
  });

  it("fails if already acknowledged", async () => {
    server.use(
      http.post(
        "http://localhost:8080/teams/:teamId/incidents/:incidentId/acknowledge",
        () => {
          return HttpResponse.json(
            {
              error: {
                code: "CONFLICT",
                message: "Only an open incident can be acknowledged",
              },
            },
            { status: 409 },
          );
        },
      ),
    );

    const { wrapper } = createWrapper();
    const { result } = renderHook(
      () => useAcknowledgeIncident(teamId, incidentId),
      { wrapper },
    );

    result.current.mutate();

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error?.message).toBe(
      "Only an open incident can be acknowledged",
    );
  });
});

describe("useEscalateIncident", () => {
  it("returns the escalated incident with its new severity", async () => {
    const { wrapper } = createWrapper();
    const { result } = renderHook(
      () => useEscalateIncident(teamId, incidentId),
      { wrapper },
    );

    result.current.mutate({ severity: "critical" });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({
      state: "escalated",
      severity: "critical",
    });
  });
});

describe("useResolveIncident", () => {
  it("returns the resolved incident", async () => {
    const { wrapper } = createWrapper();
    const { result } = renderHook(
      () => useResolveIncident(teamId, incidentId),
      { wrapper },
    );

    result.current.mutate();

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({ state: "resolved" });
    expect(result.current.data?.resolved_at).not.toBeNull();
  });
});

describe("useAssignResponder", () => {
  it("returns the incident with the new assignee", async () => {
    const { wrapper } = createWrapper();
    const { result } = renderHook(
      () => useAssignResponder(teamId, incidentId),
      { wrapper },
    );

    result.current.mutate({ user_id: "123" });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({ assigned_to: "123" });
  });
});

describe("useTimelineEntries", () => {
  it("fetches timeline entries for an incident", async () => {
    const { wrapper } = createWrapper();
    const { result } = renderHook(
      () => useTimelineEntries(teamId, incidentId),
      { wrapper },
    );

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toHaveLength(1);
    expect(result.current.data![0]).toMatchObject({ content: "Investigating" });
  });
});

describe("useAddTimelineEntry", () => {
  it("returns the created entry and invalidates the timeline cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    queryClient.setQueryData(
      ["teams", teamId, "incidents", incidentId, "timeline"],
      [],
    );

    const { result } = renderHook(
      () => useAddTimelineEntry(teamId, incidentId),
      { wrapper },
    );

    result.current.mutate({ content: "Investigating" });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({ content: "Investigating" });
    expect(
      queryClient.getQueryState([
        "teams",
        teamId,
        "incidents",
        incidentId,
        "timeline",
      ])?.isInvalidated,
    ).toBe(true);
  });
});

describe("useEditTimelineEntry", () => {
  it("returns the updated entry", async () => {
    const { wrapper } = createWrapper();
    const { result } = renderHook(
      () => useEditTimelineEntry(teamId, incidentId),
      { wrapper },
    );

    result.current.mutate({
      entryId: "entry-123",
      body: { content: "Updated" },
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({
      content: "Updated",
      edited_at: expect.any(Number),
    });
  });

  it("fails for a non-author", async () => {
    server.use(
      http.patch(
        "http://localhost:8080/teams/:teamId/incidents/:incidentId/timeline/:entryId",
        () => {
          return HttpResponse.json(
            {
              error: {
                code: "FORBIDDEN",
                message: "Only the author can edit their timeline entry",
              },
            },
            { status: 403 },
          );
        },
      ),
    );

    const { wrapper } = createWrapper();
    const { result } = renderHook(
      () => useEditTimelineEntry(teamId, incidentId),
      { wrapper },
    );

    result.current.mutate({
      entryId: "entry-123",
      body: { content: "Hacked" },
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error?.message).toBe(
      "Only the author can edit their timeline entry",
    );
  });
});

describe("useAvailableEmojis", () => {
  it("fetches the fixed emoji set", async () => {
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useAvailableEmojis(), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toEqual([
      "+1",
      "-1",
      "eyes",
      "warning",
      "check",
      "fire",
    ]);
  });
});

describe("useReactions", () => {
  it("fetches the aggregated reactions for an incident", async () => {
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useReactions(teamId, incidentId), {
      wrapper,
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toHaveLength(1);
    expect(result.current.data![0]).toMatchObject({
      emoji: "+1",
      usernames: ["alissa", "bob"],
      reacted_by_me: true,
    });
  });
});

describe("useAddReaction", () => {
  it("returns the created reaction and invalidates the reactions cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    queryClient.setQueryData(
      ["teams", teamId, "incidents", incidentId, "reactions"],
      [],
    );

    const { result } = renderHook(() => useAddReaction(teamId, incidentId), {
      wrapper,
    });

    result.current.mutate({ entryId: "entry-123", emoji: "+1" });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({ emoji: "+1" });
    expect(
      queryClient.getQueryState([
        "teams",
        teamId,
        "incidents",
        incidentId,
        "reactions",
      ])?.isInvalidated,
    ).toBe(true);
  });

  it("fails when the same emoji was already added", async () => {
    server.use(
      http.post(
        "http://localhost:8080/teams/:teamId/incidents/:incidentId/timeline/:entryId/reactions",
        () => {
          return HttpResponse.json(
            {
              error: {
                code: "CONFLICT",
                message: "You already reacted with this emoji",
              },
            },
            { status: 409 },
          );
        },
      ),
    );

    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useAddReaction(teamId, incidentId), {
      wrapper,
    });

    result.current.mutate({ entryId: "entry-123", emoji: "+1" });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error?.message).toBe(
      "You already reacted with this emoji",
    );
  });
});

describe("useRemoveReaction", () => {
  it("invalidates the reactions cache on success", async () => {
    const { wrapper, queryClient } = createWrapper();
    queryClient.setQueryData(
      ["teams", teamId, "incidents", incidentId, "reactions"],
      [],
    );

    const { result } = renderHook(() => useRemoveReaction(teamId, incidentId), {
      wrapper,
    });

    result.current.mutate({ entryId: "entry-123", emoji: "+1" });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(
      queryClient.getQueryState([
        "teams",
        teamId,
        "incidents",
        incidentId,
        "reactions",
      ])?.isInvalidated,
    ).toBe(true);
  });
});

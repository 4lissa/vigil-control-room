import { describe, it, expect, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/node";
import { setToken } from "@/features/auth/token";
import {
  useReleases,
  useRelease,
  useReleaseSteps,
  useCreateRelease,
  useValidateStep,
  useCancelRelease,
} from "./hooks";

const teamId = "team-123";
const releaseId = "release-123";

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

describe("useReleases", () => {
  it("fetches releases for a team", async () => {
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useReleases(teamId), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toHaveLength(1);
    expect(result.current.data![0]).toMatchObject({ name: "v1.0.0" });
  });
});

describe("useRelease", () => {
  it("fetches a single release by id", async () => {
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useRelease(teamId, releaseId), {
      wrapper,
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({
      id: releaseId,
      state: "in_progress",
    });
  });
});

describe("useReleaseSteps", () => {
  it("fetches steps for a release", async () => {
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useReleaseSteps(teamId, releaseId), {
      wrapper,
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toHaveLength(1);
    expect(result.current.data![0]).toMatchObject({ name: "staging" });
  });
});

describe("useCreateRelease", () => {
  it("returns the created release and invalidates the releases cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    queryClient.setQueryData(["teams", teamId, "releases"], []);

    const { result } = renderHook(() => useCreateRelease(teamId), {
      wrapper,
    });

    result.current.mutate({ name: "v1.0.0", steps: ["build", "staging"] });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({ name: "v1.0.0" });
    expect(
      queryClient.getQueryState(["teams", teamId, "releases"])?.isInvalidated,
    ).toBe(true);
  });

  it("fails for a non-manager", async () => {
    server.use(
      http.post("http://localhost:8080/teams/:teamId/releases", () => {
        return HttpResponse.json(
          {
            error: {
              code: "FORBIDDEN",
              message: "Only managers can create a release",
            },
          },
          { status: 403 },
        );
      }),
    );

    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useCreateRelease(teamId), {
      wrapper,
    });

    result.current.mutate({ name: "v1.0.0", steps: ["build"] });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error?.message).toBe(
      "Only managers can create a release",
    );
  });
});

describe("useValidateStep", () => {
  it("returns the validated step and invalidates the release cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    queryClient.setQueryData(
      ["teams", teamId, "releases", releaseId, "steps"],
      [],
    );

    const { result } = renderHook(() => useValidateStep(teamId, releaseId), {
      wrapper,
    });

    result.current.mutate("step-123");

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data?.validated_at).not.toBeNull();
    expect(
      queryClient.getQueryState([
        "teams",
        teamId,
        "releases",
        releaseId,
        "steps",
      ])?.isInvalidated,
    ).toBe(true);
  });

  it("fails when the step is out of order", async () => {
    server.use(
      http.post(
        "http://localhost:8080/teams/:teamId/releases/:releaseId/steps/:stepId/validate",
        () => {
          return HttpResponse.json(
            {
              error: {
                code: "CONFLICT",
                message: "This step cannot be validated yet",
              },
            },
            { status: 409 },
          );
        },
      ),
    );

    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useValidateStep(teamId, releaseId), {
      wrapper,
    });

    result.current.mutate("step-456");

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error?.message).toBe(
      "This step cannot be validated yet",
    );
  });
});

describe("useCancelRelease", () => {
  it("returns the cancelled release", async () => {
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useCancelRelease(teamId, releaseId), {
      wrapper,
    });

    result.current.mutate();

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({ state: "cancelled" });
  });
});

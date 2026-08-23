import { describe, it, expect, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/node";
import { getToken, setToken } from "./token";
import {
  useConnectService,
  useConnectedServiceStatus,
  useDisconnectService,
  useLogin,
  useLogout,
  useMe,
  useRegister,
  useUpdateProfile,
} from "./hooks";

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  const wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}> {children} </QueryClientProvider>
  );

  return { wrapper, queryClient };
};

beforeEach(() => {
  localStorage.clear();
});

describe("useRegister", () => {
  it("sets token and primes the me cache on success", async () => {
    const { wrapper, queryClient } = createWrapper();
    const { result } = renderHook(() => useRegister(), { wrapper });

    result.current.mutate({
      username: "alissa",
      email: "alissa@example.com",
      password: "password123",
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({
      token: "my-session-token",
      user: { username: "alissa", email: "alissa@example.com" },
    });

    expect(getToken()).toBe("my-session-token");
    expect(queryClient.getQueryData(["me"])).toMatchObject({
      username: "alissa",
    });
  });

  it("does not set token or cache on failure", async () => {
    server.use(
      http.post("http://localhost:8080/register", () => {
        return HttpResponse.json(
          { error: { code: "CONFLICT", message: "Email already taken" } },
          { status: 409 },
        );
      }),
    );

    const { wrapper, queryClient } = createWrapper();
    const { result } = renderHook(() => useRegister(), { wrapper });

    result.current.mutate({
      username: "alissa",
      email: "alissa@example.com",
      password: "password123",
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error?.message).toBe("Email already taken");

    expect(getToken()).toBeNull();
    expect(queryClient.getQueryData(["me"])).toBeUndefined();
  });
});

describe("useLogin", () => {
  it("sets token and primes the me cache on success", async () => {
    const { wrapper, queryClient } = createWrapper();
    const { result } = renderHook(() => useLogin(), { wrapper });

    result.current.mutate({
      email: "alissa@example.com",
      password: "password123",
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(getToken()).toBe("my-session-token");
    expect(queryClient.getQueryData(["me"])).toMatchObject({
      username: "alissa",
    });
  });

  it("does not set token or cache on failure", async () => {
    server.use(
      http.post("http://localhost:8080/login", () => {
        return HttpResponse.json(
          {
            error: {
              code: "UNAUTHORIZED",
              message: "Authentication required",
            },
          },
          { status: 401 },
        );
      }),
    );

    const { wrapper, queryClient } = createWrapper();
    const { result } = renderHook(() => useLogin(), { wrapper });

    result.current.mutate({ email: "alissa@example.com", password: "wrong" });

    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(getToken()).toBeNull();
    expect(queryClient.getQueryData(["me"])).toBeUndefined();
  });
});

describe("useLogout", () => {
  it("clears token and cache on success", async () => {
    setToken("my-session-token");
    const { wrapper, queryClient } = createWrapper();
    queryClient.setQueryData(["me"], { username: "alissa" });

    const { result } = renderHook(() => useLogout(), { wrapper });

    result.current.mutate();

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(getToken()).toBeNull();
    expect(queryClient.getQueryData(["me"])).toBeUndefined();
  });
});

describe("useMe", () => {
  it("fetches current user when token is present", async () => {
    setToken("my-session-token");

    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useMe(), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({
      username: "alissa",
      email: "alissa@example.com",
    });
  });

  it("fails without fetching when no token is stored", async () => {
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useMe(), { wrapper });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.data).toBeUndefined();
  });
});

describe("useUpdateProfile", () => {
  it("updates cache on success", async () => {
    setToken("my-session-token");
    const { wrapper, queryClient } = createWrapper();

    const { result } = renderHook(() => useUpdateProfile(), { wrapper });

    result.current.mutate({ username: "alissa_updated" });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({ username: "alissa_updated" });
    expect(queryClient.getQueryData(["me"])).toMatchObject({
      username: "alissa_updated",
    });
  });
});

describe("useConnectService", () => {
  it("succeeds for a supported service", async () => {
    setToken("my-session-token");
    const { wrapper } = createWrapper();

    const { result } = renderHook(() => useConnectService(), { wrapper });

    result.current.mutate({ service: "http", body: { token: "my-token" } });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
  });

  it("fails for an unsupported service", async () => {
    server.use(
      http.post("http://localhost:8080/connected-services/:service", () => {
        return HttpResponse.json(
          {
            error: {
              code: "VALIDATION_ERROR",
              message: "Unsupported service: discord",
            },
          },
          { status: 422 },
        );
      }),
    );

    setToken("my-session-token");
    const { wrapper } = createWrapper();

    const { result } = renderHook(() => useConnectService(), { wrapper });

    result.current.mutate({ service: "discord", body: { token: "my-token" } });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error?.message).toBe("Unsupported service: discord");
  });
});

describe("useConnectedServiceStatus", () => {
  it("reports connected once a service was connected", async () => {
    server.use(
      http.get("http://localhost:8080/connected-services/:service", () => {
        return HttpResponse.json({ connected: true });
      }),
    );

    setToken("my-session-token");
    const { wrapper } = createWrapper();

    const { result } = renderHook(() => useConnectedServiceStatus("http"), {
      wrapper,
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toMatchObject({ connected: true });
  });
});

describe("useDisconnectService", () => {
  it("invalidates the connected-services cache on success", async () => {
    setToken("my-session-token");
    const { wrapper, queryClient } = createWrapper();
    queryClient.setQueryData(["connected-services", "http"], {
      connected: true,
    });

    const { result } = renderHook(() => useDisconnectService(), { wrapper });

    result.current.mutate("http");

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(
      queryClient.getQueryState(["connected-services", "http"])?.isInvalidated,
    ).toBe(true);
  });
});

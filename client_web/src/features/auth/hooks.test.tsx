import { describe, it, expect, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/node";
import { useAuthStore } from "./store";
import {
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

  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}> {children} </QueryClientProvider>
  );
};

beforeEach(() => {
  useAuthStore.setState({ token: null, user: null });
});

describe("useRegister", () => {
  it("calls register endpoint and returns user", async () => {
    const { result } = renderHook(() => useRegister(), {
      wrapper: createWrapper(),
    });

    result.current.mutate({
      username: "alissa",
      email: "alissa@example.com",
      password: "password123",
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({
      username: "alissa",
      email: "alissa@example.com",
    });
  });

  it("sets error on failure", async () => {
    server.use(
      http.post("http://localhost:8080/register", () => {
        return HttpResponse.json(
          { error: { code: "CONFLICT", message: "Email already taken" } },
          { status: 409 },
        );
      }),
    );

    const { result } = renderHook(() => useRegister(), {
      wrapper: createWrapper(),
    });

    result.current.mutate({
      username: "alissa",
      email: "alissa@example.com",
      password: "password123",
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error?.message).toBe("Email already taken");
  });
});

describe("useLogin", () => {
  it("sets token and user in store on success", async () => {
    const { result } = renderHook(() => useLogin(), {
      wrapper: createWrapper(),
    });

    result.current.mutate({
      email: "alissa@example.com",
      password: "password123",
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    const { token, user } = useAuthStore.getState();
    expect(token).toBe("my-session-token");
    expect(user).toMatchObject({ username: "alissa" });
  });

  it("does not set store on failure", async () => {
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

    const { result } = renderHook(() => useLogin(), {
      wrapper: createWrapper(),
    });

    result.current.mutate({ email: "alissa@example.com", password: "wrong" });

    await waitFor(() => expect(result.current.isError).toBe(true));

    const { token, user } = useAuthStore.getState();
    expect(token).toBeNull();
    expect(user).toBeNull();
  });
});

describe("useLogout", () => {
  it("clears store on success", async () => {
    useAuthStore.setState({
      token: "my-session-token",
      user: {
        id: "123",
        username: "alissa",
        email: "alissa@example.com",
        language: "en",
      },
    });

    const { result } = renderHook(() => useLogout(), {
      wrapper: createWrapper(),
    });

    result.current.mutate();

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    const { token, user } = useAuthStore.getState();
    expect(token).toBeNull();
    expect(user).toBeNull();
  });
});

describe("useMe", () => {
  it("fetches current user when token is present", async () => {
    useAuthStore.setState({ token: "my-session-token", user: null });

    const { result } = renderHook(() => useMe(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({
      username: "alissa",
      email: "alissa@example.com",
    });
  });

  it("does not fetch when token is null", async () => {
    const { result } = renderHook(() => useMe(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.fetchStatus).toBe("idle"));
    expect(result.current.data).toBeUndefined();
  });
});

describe("useUpdateProfile", () => {
  it("updates cache on success", async () => {
    useAuthStore.setState({ token: "my-session-token", user: null });

    const queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
        mutations: { retry: false },
      },
    });

    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>
        {" "}
        {children}{" "}
      </QueryClientProvider>
    );

    const { result } = renderHook(() => useUpdateProfile(), { wrapper });

    result.current.mutate({ username: "alissa_updated" });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({ username: "alissa_updated" });
  });
});

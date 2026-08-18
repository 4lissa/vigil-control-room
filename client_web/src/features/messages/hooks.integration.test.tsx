import { describe, it, expect, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/node";
import { setToken } from "@/features/auth/token";
import {
  useContacts,
  useConversation,
  useConversations,
  useSendMessage,
} from "./hooks";

const userId = "456";

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

describe("useConversations", () => {
  it("fetches the list of conversations", async () => {
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useConversations(), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toHaveLength(1);
    expect(result.current.data![0]).toMatchObject({
      content: "Hey, got a minute?",
    });
  });
});

describe("useConversation", () => {
  it("fetches the conversation with a user", async () => {
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useConversation(userId), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toHaveLength(1);
    expect(result.current.data![0]).toMatchObject({
      content: "Hey, got a minute?",
    });
  });
});

describe("useSendMessage", () => {
  it("returns the sent message and invalidates the messages cache", async () => {
    const { wrapper, queryClient } = createWrapper();
    queryClient.setQueryData(["messages"], []);
    queryClient.setQueryData(["messages", userId], []);

    const { result } = renderHook(() => useSendMessage(userId), { wrapper });

    result.current.mutate({ content: "Hey, got a minute?" });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({
      content: "Hey, got a minute?",
    });
    expect(queryClient.getQueryState(["messages"])?.isInvalidated).toBe(true);
    expect(queryClient.getQueryState(["messages", userId])?.isInvalidated).toBe(
      true,
    );
  });

  it("fails when the recipient does not share a team", async () => {
    server.use(
      http.post("http://localhost:8080/messages/:userId", () => {
        return HttpResponse.json(
          {
            error: {
              code: "FORBIDDEN",
              message: "You do not share a team with this user",
            },
          },
          { status: 403 },
        );
      }),
    );

    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useSendMessage(userId), { wrapper });

    result.current.mutate({ content: "Hey" });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error?.message).toBe(
      "You do not share a team with this user",
    );
  });
});

describe("useContacts", () => {
  it("lists teammates across all teams, excluding the current user", async () => {
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useContacts("999"), { wrapper });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(result.current.contacts).toEqual([
      { user_id: "123", username: "alissa" },
    ]);
  });

  it("excludes the current user from their own contacts", async () => {
    const { wrapper } = createWrapper();
    const { result } = renderHook(() => useContacts("123"), { wrapper });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(result.current.contacts).toEqual([]);
  });
});

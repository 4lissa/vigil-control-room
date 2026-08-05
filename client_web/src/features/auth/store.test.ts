import { describe, it, expect, beforeEach } from "vitest";
import { useAuthStore } from "./store";

beforeEach(() => {
  useAuthStore.setState({ token: null, user: null });
});

describe("useAuthStore", () => {
  const mockUser = {
    id: "123",
    username: "alissa",
    email: "alissa@example.com",
    language: "fr",
  };

  it("initializes with null token and user", () => {
    const { token, user } = useAuthStore.getState();
    expect(token).toBeNull();
    expect(user).toBeNull();
  });

  it("setAuth sets token and user", () => {
    useAuthStore.getState().setAuth("my-token", mockUser);

    const { token, user } = useAuthStore.getState();
    expect(token).toBe("my-token");
    expect(user).toEqual(mockUser);
  });

  it("clearAuth resets token and user to null", () => {
    useAuthStore.getState().setAuth("my-token", mockUser);
    useAuthStore.getState().clearAuth();

    const { token, user } = useAuthStore.getState();
    expect(token).toBeNull();
    expect(user).toBeNull();
  });
});

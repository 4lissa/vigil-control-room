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

  it("persists auth to localStorage under the vigil-auth key", () => {
    useAuthStore.getState().setAuth("my-token", mockUser);

    const stored = JSON.parse(localStorage.getItem("vigil-auth") ?? "{}");
    expect(stored.state.token).toBe("my-token");
    expect(stored.state.user).toEqual(mockUser);
  });

  it("clears persisted auth from localStorage on clearAuth", () => {
    useAuthStore.getState().setAuth("my-token", mockUser);
    useAuthStore.getState().clearAuth();

    const stored = JSON.parse(localStorage.getItem("vigil-auth") ?? "{}");
    expect(stored.state.token).toBeNull();
    expect(stored.state.user).toBeNull();
  });
});

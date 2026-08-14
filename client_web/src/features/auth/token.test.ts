import { describe, it, expect, beforeEach } from "vitest";
import { getToken, setToken, clearToken } from "./token";

beforeEach(() => {
  localStorage.clear();
});

describe("token storage", () => {
  it("returns null when no token is stored", () => {
    expect(getToken()).toBeNull();
  });

  it("setToken persists the token to localStorage", () => {
    setToken("my-token");

    expect(getToken()).toBe("my-token");
    expect(localStorage.getItem("vigil-token")).toBe("my-token");
  });

  it("clearToken removes the stored token", () => {
    setToken("my-token");
    clearToken();

    expect(getToken()).toBeNull();
  });
});

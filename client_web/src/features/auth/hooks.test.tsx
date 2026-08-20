import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook } from "@testing-library/react";
import { getToken, clearToken } from "./token";
import { useHandleOAuthCallback } from "./hooks";

const replace = vi.fn();
let mockSearchParams = new URLSearchParams();

vi.mock("next/navigation", () => ({
  useRouter: () => ({ replace }),
  useSearchParams: () => mockSearchParams,
}));

beforeEach(() => {
  clearToken();
  replace.mockClear();
  mockSearchParams = new URLSearchParams();
});

describe("useHandleOAuthCallback", () => {
  it("stores the token and redirects to /incidents when present", () => {
    mockSearchParams = new URLSearchParams({ token: "session-token" });
    renderHook(() => useHandleOAuthCallback());

    expect(getToken()).toBe("session-token");
    expect(replace).toHaveBeenCalledWith("/incidents");
  });

  it("redirects to /login without storing anything when there is no token", () => {
    renderHook(() => useHandleOAuthCallback());

    expect(getToken()).toBeNull();
    expect(replace).toHaveBeenCalledWith("/login");
  });
});

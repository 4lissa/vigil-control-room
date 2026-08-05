import { describe, it, expect } from "vitest";
import { queryClient } from "./query-client";
import { ApiError } from "./api-client";

describe("queryClient", () => {
  describe("retry logic", () => {
    const retryFn = queryClient.getDefaultOptions().queries?.retry as (
      failureCount: number,
      error: unknown,
    ) => boolean;

    it("does not retry on 401", () => {
      const error = new ApiError(
        "UNAUTHORIZED",
        "Authentication required",
        401,
      );
      expect(retryFn(0, error)).toBe(false);
    });

    it("does not retry on 403", () => {
      const error = new ApiError(
        "FORBIDDEN",
        "Only managers can kick members",
        403,
      );
      expect(retryFn(0, error)).toBe(false);
    });

    it("does not retry on 404", () => {
      const error = new ApiError("NOT_FOUND", "User not found", 404);
      expect(retryFn(0, error)).toBe(false);
    });

    it("retries on network error up to 2 times", () => {
      const error = new Error("Network error");
      expect(retryFn(0, error)).toBe(true);
      expect(retryFn(1, error)).toBe(true);
      expect(retryFn(2, error)).toBe(false);
    });

    it("retries on 500 up to 2 times", () => {
      const error = new ApiError(
        "INTERNAL_ERROR",
        "An internal error occurred",
        500,
      );
      expect(retryFn(0, error)).toBe(true);
      expect(retryFn(1, error)).toBe(true);
      expect(retryFn(2, error)).toBe(false);
    });
  });
});

import { describe, it, expect } from "vitest";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/node";
import { apiClient, ApiError } from "./api-client";

describe("apiClient", () => {
  describe("successful responses", () => {
    it("returns parsed JSON on 200", async () => {
      const result = await apiClient.get<{
        id: string;
        username: string;
        email: string;
        language: string;
      }>("/me", "token");

      expect(result).toEqual({
        id: "123",
        username: "alissa",
        email: "alissa@example.com",
        language: "en",
      });
    });

    it("returns undefined on 204", async () => {
      const result = await apiClient.post<void>("/logout", {}, "token");
      expect(result).toBeUndefined();
    });
  });

  describe("error responses", () => {
    it("throws ApiError with code and message on 401", async () => {
      server.use(
        http.get("http://localhost:8080/me", () => {
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

      try {
        await apiClient.get("/me", "bad-token");
      } catch (e) {
        expect(e).toBeInstanceOf(ApiError);
        const err = e as ApiError;
        expect(err.code).toBe("UNAUTHORIZED");
        expect(err.message).toBe("Authentication required");
        expect(err.status).toBe(401);
      }
    });

    it("throws ApiError with UNKNOWN_ERROR when error field is missing", async () => {
      server.use(
        http.get("http://localhost:8080/me", () => {
          return HttpResponse.json({}, { status: 400 });
        }),
      );

      try {
        await apiClient.get("/me");
      } catch (e) {
        expect(e).toBeInstanceOf(ApiError);
        const err = e as ApiError;
        expect(err.code).toBe("UNKNOWN_ERROR");
      }
    });

    it("throws ApiError with UNKNOWN_ERROR when body is not parseable", async () => {
      server.use(
        http.get("http://localhost:8080/me", () => {
          return new HttpResponse("not json", { status: 500 });
        }),
      );

      try {
        await apiClient.get("/me");
      } catch (e) {
        expect(e).toBeInstanceOf(ApiError);
        const err = e as ApiError;
        expect(err.code).toBe("UNKNOWN_ERROR");
        expect(err.status).toBe(500);
      }
    });
  });

  describe("request methods", () => {
    it("sends POST and returns response", async () => {
      const result = await apiClient.post<{ token: string }>("/register", {
        username: "alissa",
        email: "alissa@example.com",
        password: "password123",
      });

      expect(result).toMatchObject({
        token: "my-session-token",
        user: {
          id: "123",
          username: "alissa",
          email: "alissa@example.com",
          language: "en",
        },
      });
    });

    it("sends PATCH and returns response", async () => {
      const result = await apiClient.patch<{
        id: string;
        username: string;
        email: string;
        language: string;
      }>("/me", { username: "alissa_updated" }, "token");

      expect(result).toMatchObject({ username: "alissa_updated" });
    });
  });
});

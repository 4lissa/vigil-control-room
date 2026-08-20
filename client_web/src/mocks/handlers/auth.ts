import { http, HttpResponse } from "msw";

export const authHandlers = [
  http.post("http://localhost:8080/register", () => {
    return HttpResponse.json(
      {
        token: "my-session-token",
        user: {
          id: "123",
          username: "alissa",
          email: "alissa@example.com",
          language: "en",
          has_password: true,
        },
      },
      { status: 201 },
    );
  }),

  http.post("http://localhost:8080/login", () => {
    return HttpResponse.json({
      token: "my-session-token",
      user: {
        id: "123",
        username: "alissa",
        email: "alissa@example.com",
        language: "en",
        has_password: true,
      },
    });
  }),

  http.get("http://localhost:8080/me", () => {
    return HttpResponse.json({
      id: "123",
      username: "alissa",
      email: "alissa@example.com",
      language: "en",
      has_password: true,
    });
  }),

  http.post("http://localhost:8080/logout", () => {
    return new HttpResponse(null, { status: 204 });
  }),

  http.patch("http://localhost:8080/me", () => {
    return HttpResponse.json({
      id: "123",
      username: "alissa_updated",
      email: "alissa@example.com",
      language: "en",
      has_password: true,
    });
  }),
];

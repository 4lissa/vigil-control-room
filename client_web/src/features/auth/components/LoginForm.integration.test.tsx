import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { NextIntlClientProvider } from "next-intl";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/node";
import { getToken } from "@/features/auth/token";
import { LoginForm } from "./LoginForm";
import messages from "../../../../messages/en.json";

const push = vi.fn();

vi.mock("next/navigation", () => ({
  useRouter: () => ({ push }),
}));

const renderForm = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  return render(
    <NextIntlClientProvider locale="en" messages={messages}>
      <QueryClientProvider client={queryClient}>
        <LoginForm />
      </QueryClientProvider>
    </NextIntlClientProvider>,
  );
};

beforeEach(() => {
  push.mockClear();
});

describe("LoginForm", () => {
  it("logs in with the entered credentials and redirects to /incidents", async () => {
    let requestedBody: unknown;
    server.use(
      http.post("http://localhost:8080/login", async ({ request }) => {
        requestedBody = await request.json();
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
    );

    const user = userEvent.setup();
    renderForm();

    await user.type(screen.getByLabelText("Email"), "alissa@example.com");
    await user.type(screen.getByLabelText("Password"), "password123");
    await user.click(screen.getByRole("button", { name: "Sign in" }));

    await waitFor(() =>
      expect(requestedBody).toEqual({
        email: "alissa@example.com",
        password: "password123",
      }),
    );
    await waitFor(() => expect(push).toHaveBeenCalledWith("/incidents"));
    expect(getToken()).toBe("my-session-token");
  });

  it("shows an error banner when the credentials are rejected", async () => {
    server.use(
      http.post("http://localhost:8080/login", () =>
        HttpResponse.json(
          { error: { code: "UNAUTHORIZED", message: "Invalid credentials" } },
          { status: 401 },
        ),
      ),
    );

    const user = userEvent.setup();
    renderForm();

    await user.type(screen.getByLabelText("Email"), "alissa@example.com");
    await user.type(screen.getByLabelText("Password"), "wrong-password");
    await user.click(screen.getByRole("button", { name: "Sign in" }));

    await waitFor(() =>
      expect(screen.getByText("Invalid credentials")).toBeInTheDocument(),
    );
    expect(push).not.toHaveBeenCalled();
  });
});

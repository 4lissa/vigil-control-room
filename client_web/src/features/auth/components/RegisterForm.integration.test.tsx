import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { NextIntlClientProvider } from "next-intl";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/node";
import { RegisterForm } from "./RegisterForm";
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
        <RegisterForm />
      </QueryClientProvider>
    </NextIntlClientProvider>,
  );
};

beforeEach(() => {
  push.mockClear();
});

describe("RegisterForm", () => {
  it("registers with the entered fields and redirects to /incidents", async () => {
    let requestedBody: unknown;
    server.use(
      http.post("http://localhost:8080/register", async ({ request }) => {
        requestedBody = await request.json();
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
    );

    const user = userEvent.setup();
    renderForm();

    await user.type(screen.getByLabelText("Username"), "alissa");
    await user.type(screen.getByLabelText("Email"), "alissa@example.com");
    await user.type(screen.getByLabelText("Password"), "password123");
    await user.click(screen.getByRole("button", { name: "Create account" }));

    await waitFor(() =>
      expect(requestedBody).toEqual({
        username: "alissa",
        email: "alissa@example.com",
        password: "password123",
      }),
    );
    await waitFor(() => expect(push).toHaveBeenCalledWith("/incidents"));
  });

  it("shows an error banner when registration fails", async () => {
    server.use(
      http.post("http://localhost:8080/register", () =>
        HttpResponse.json(
          { error: { code: "CONFLICT", message: "Email already taken" } },
          { status: 409 },
        ),
      ),
    );

    const user = userEvent.setup();
    renderForm();

    await user.type(screen.getByLabelText("Username"), "alissa");
    await user.type(screen.getByLabelText("Email"), "alissa@example.com");
    await user.type(screen.getByLabelText("Password"), "password123");
    await user.click(screen.getByRole("button", { name: "Create account" }));

    await waitFor(() =>
      expect(screen.getByText("Email already taken")).toBeInTheDocument(),
    );
    expect(push).not.toHaveBeenCalled();
  });
});

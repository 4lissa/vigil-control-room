import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { NextIntlClientProvider } from "next-intl";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/node";
import { setToken } from "@/features/auth/token";
import { JoinTeamForm } from "./JoinTeamForm";
import messages from "../../../../messages/en.json";

const push = vi.fn();

vi.mock("next/navigation", () => ({
  useRouter: () => ({ push }),
}));

const renderForm = (onSuccess = vi.fn()) => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  return {
    onSuccess,
    ...render(
      <NextIntlClientProvider locale="en" messages={messages}>
        <QueryClientProvider client={queryClient}>
          <JoinTeamForm onSuccess={onSuccess} />
        </QueryClientProvider>
      </NextIntlClientProvider>,
    ),
  };
};

beforeEach(() => {
  setToken("my-session-token");
  push.mockClear();
});

describe("JoinTeamForm", () => {
  it("joins the team with the entered invitation code and navigates to it", async () => {
    let requestedBody: unknown;
    server.use(
      http.post("http://localhost:8080/teams/join", async ({ request }) => {
        requestedBody = await request.json();
        return HttpResponse.json({
          team: {
            id: "team-999",
            name: "My Team",
            created_by: "123",
            created_at: "2026-08-12T00:00:00Z",
          },
          member: {
            id: "member-1",
            team_id: "team-999",
            user_id: "123",
            role: "observer",
            joined_at: "2026-08-12T00:00:00Z",
          },
        });
      }),
    );

    const user = userEvent.setup();
    const { onSuccess } = renderForm();

    await user.type(screen.getByLabelText("Invitation code"), "ABC123");
    await user.click(screen.getByRole("button", { name: "Join team" }));

    await waitFor(() => expect(requestedBody).toEqual({ code: "ABC123" }));
    await waitFor(() => expect(onSuccess).toHaveBeenCalledOnce());
    expect(push).toHaveBeenCalledWith("/teams/team-999/incidents");
  });

  it("shows an error banner for an invalid invitation code", async () => {
    server.use(
      http.post("http://localhost:8080/teams/join", () =>
        HttpResponse.json(
          { error: { code: "NOT_FOUND", message: "Invalid invitation code" } },
          { status: 404 },
        ),
      ),
    );

    const user = userEvent.setup();
    renderForm();

    await user.type(screen.getByLabelText("Invitation code"), "WRONG");
    await user.click(screen.getByRole("button", { name: "Join team" }));

    await waitFor(() =>
      expect(screen.getByText("Invalid invitation code")).toBeInTheDocument(),
    );
  });
});

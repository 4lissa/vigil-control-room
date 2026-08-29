import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { NextIntlClientProvider } from "next-intl";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/node";
import { setToken } from "@/features/auth/token";
import { TeamMemberList } from "./TeamMemberList";
import { TeamMemberResponse } from "../types";
import messages from "../../../../messages/en.json";

const me: TeamMemberResponse = {
  id: "member-123",
  team_id: "team-123",
  user_id: "123",
  username: "alissa",
  role: "manager",
  joined_at: "2026-08-12T00:00:00Z",
};

const bob: TeamMemberResponse = {
  id: "member-456",
  team_id: "team-123",
  user_id: "456",
  username: "bob",
  role: "observer",
  joined_at: "2026-08-13T00:00:00Z",
};

const renderList = (
  currentUserRole: TeamMemberResponse["role"] | undefined,
) => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  return render(
    <NextIntlClientProvider locale="en" messages={messages}>
      <QueryClientProvider client={queryClient}>
        <TeamMemberList
          teamId="team-123"
          teamName="My Team"
          members={[me, bob]}
          currentUserId="123"
          currentUserRole={currentUserRole}
        />
      </QueryClientProvider>
    </NextIntlClientProvider>,
  );
};

beforeEach(() => {
  setToken("my-session-token");
  HTMLDialogElement.prototype.showModal = vi.fn();
  HTMLDialogElement.prototype.close = vi.fn();
});

describe("TeamMemberList", () => {
  it("shows the member count and each member's role", () => {
    renderList("manager");
    expect(screen.getByText("Members (2)")).toBeInTheDocument();
    expect(screen.getByText("Manager")).toBeInTheDocument();
    expect(screen.getByText("Observer")).toBeInTheDocument();
  });

  it("marks the current user without offering to moderate themselves", () => {
    renderList("manager");
    expect(screen.getByText("(you)")).toBeInTheDocument();
    expect(screen.getAllByText("Kick")).toHaveLength(1);
    expect(screen.getAllByText("Ban")).toHaveLength(1);
  });

  it("hides moderation actions entirely for non-managers", () => {
    renderList("observer");
    expect(screen.queryByText("Kick")).not.toBeInTheDocument();
    expect(screen.queryByText("Ban")).not.toBeInTheDocument();
  });

  it("kicks the targeted member after confirming the dialog", async () => {
    const user = userEvent.setup();
    renderList("manager");

    await user.click(screen.getByText("Kick"));
    expect(
      screen.getByText(
        "Remove bob from My Team? They can rejoin later with a new invitation code.",
      ),
    ).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", { name: "Kick bob", hidden: true }),
    );

    await waitFor(() =>
      expect(
        screen.queryByText(
          "Remove bob from My Team? They can rejoin later with a new invitation code.",
        ),
      ).not.toBeInTheDocument(),
    );
  });

  it("shows an error banner when the kick request fails", async () => {
    server.use(
      http.post(
        "http://localhost:8080/teams/:teamId/members/:userId/kick",
        () =>
          HttpResponse.json(
            { error: { code: "FORBIDDEN", message: "Not allowed" } },
            { status: 403 },
          ),
      ),
    );

    const user = userEvent.setup();
    renderList("manager");

    await user.click(screen.getByText("Kick"));
    await user.click(
      screen.getByRole("button", { name: "Kick bob", hidden: true }),
    );

    await waitFor(() =>
      expect(screen.getByText("Not allowed")).toBeInTheDocument(),
    );
  });

  it("bans the targeted member with the chosen expiration", async () => {
    let requestedBody: unknown;
    server.use(
      http.post(
        "http://localhost:8080/teams/:teamId/members/:userId/ban",
        async ({ request }) => {
          requestedBody = await request.json();
          return new HttpResponse(null, { status: 204 });
        },
      ),
    );

    const user = userEvent.setup();
    renderList("manager");

    await user.click(screen.getByText("Ban"));
    expect(screen.getByLabelText(/Lift ban on/)).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", { name: "Ban bob", hidden: true }),
    );

    await waitFor(() => expect(requestedBody).toEqual({ until: null }));
  });
});

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { NextIntlClientProvider } from "next-intl";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/node";
import { setToken } from "@/features/auth/token";
import { ReactionBar } from "./ReactionBar";
import { Emoji, ReactionSummaryResponse } from "../types";
import messages from "../../../../messages/en.json";

const availableEmojis: Emoji[] = [
  "+1",
  "-1",
  "eyes",
  "warning",
  "check",
  "fire",
];

const renderBar = (reactions: ReactionSummaryResponse[]) => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  return render(
    <NextIntlClientProvider locale="en" messages={messages}>
      <QueryClientProvider client={queryClient}>
        <ReactionBar
          teamId="team-123"
          incidentId="incident-123"
          entryId="entry-123"
          availableEmojis={availableEmojis}
          reactions={reactions}
        />
      </QueryClientProvider>
    </NextIntlClientProvider>,
  );
};

beforeEach(() => {
  setToken("my-session-token");
});

describe("ReactionBar", () => {
  it("renders every available emoji as a togglable button", () => {
    renderBar([]);
    for (const emoji of availableEmojis) {
      expect(
        screen.getByRole("button", { name: `React with ${emoji}` }),
      ).toBeInTheDocument();
    }
  });

  it("marks an emoji as pressed when the current user already reacted with it", () => {
    renderBar([
      {
        entry_id: "entry-123",
        emoji: "+1",
        usernames: ["alissa"],
        reacted_by_me: true,
      },
    ]);

    expect(
      screen.getByRole("button", {
        name: "React with +1 (alissa)",
      }),
    ).toHaveAttribute("aria-pressed", "true");
    expect(
      screen.getByRole("button", { name: "React with -1" }),
    ).toHaveAttribute("aria-pressed", "false");
  });

  it("adds a reaction when clicking an emoji the user hasn't reacted with", async () => {
    let requestedBody: unknown;
    server.use(
      http.post(
        "http://localhost:8080/teams/:teamId/incidents/:incidentId/timeline/:entryId/reactions",
        async ({ request }) => {
          requestedBody = await request.json();
          return HttpResponse.json(
            {
              id: "reaction-1",
              entry_id: "entry-123",
              user_id: "123",
              emoji: "fire",
              created_at: 1755000000,
            },
            { status: 201 },
          );
        },
      ),
    );

    const user = userEvent.setup();
    renderBar([]);
    await user.click(screen.getByRole("button", { name: "React with fire" }));

    await waitFor(() => expect(requestedBody).toEqual({ emoji: "fire" }));
  });

  it("removes the reaction when clicking an emoji already reacted with", async () => {
    let removedEmoji: string | undefined;
    server.use(
      http.delete(
        "http://localhost:8080/teams/:teamId/incidents/:incidentId/timeline/:entryId/reactions/:emoji",
        ({ params }) => {
          removedEmoji = params.emoji as string;
          return new HttpResponse(null, { status: 204 });
        },
      ),
    );

    const user = userEvent.setup();
    renderBar([
      {
        entry_id: "entry-123",
        emoji: "+1",
        usernames: ["alissa"],
        reacted_by_me: true,
      },
    ]);
    await user.click(
      screen.getByRole("button", { name: "React with +1 (alissa)" }),
    );

    await waitFor(() => expect(removedEmoji).toBe("+1"));
  });

  it("shows an error message when a reaction request fails", async () => {
    server.use(
      http.post(
        "http://localhost:8080/teams/:teamId/incidents/:incidentId/timeline/:entryId/reactions",
        () =>
          HttpResponse.json(
            { error: { code: "CONFLICT", message: "Already reacted" } },
            { status: 409 },
          ),
      ),
    );

    const user = userEvent.setup();
    renderBar([]);
    await user.click(screen.getByRole("button", { name: "React with eyes" }));

    await waitFor(() =>
      expect(screen.getByText("Already reacted")).toBeInTheDocument(),
    );
  });
});

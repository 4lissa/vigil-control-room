import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { NextIntlClientProvider } from "next-intl";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/node";
import { setToken } from "@/features/auth/token";
import { Timeline } from "./Timeline";
import {
  Emoji,
  ReactionSummaryResponse,
  TimelineEntryResponse,
} from "../types";
import { TeamMemberResponse, TeamRole } from "@/features/teams";
import messages from "../../../../messages/en.json";

const members: TeamMemberResponse[] = [
  {
    id: "member-123",
    team_id: "team-123",
    user_id: "123",
    username: "alissa",
    role: "manager",
    joined_at: "2026-08-12T00:00:00Z",
  },
  {
    id: "member-456",
    team_id: "team-123",
    user_id: "456",
    username: "bob",
    role: "responder",
    joined_at: "2026-08-13T00:00:00Z",
  },
];

const myEntry: TimelineEntryResponse = {
  id: "entry-1",
  incident_id: "incident-123",
  author_id: "123",
  content: "Investigating the outage",
  created_at: 1755000000,
  edited_at: null,
};

const bobsEntry: TimelineEntryResponse = {
  id: "entry-2",
  incident_id: "incident-123",
  author_id: "456",
  content: "Rolled back the deploy",
  created_at: 1755000100,
  edited_at: 1755000200,
};

const availableEmojis: Emoji[] = ["+1", "fire"];

const renderTimeline = (
  entries: TimelineEntryResponse[],
  currentUserRole: TeamRole | undefined,
  reactions: ReactionSummaryResponse[] = [],
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
        <Timeline
          teamId="team-123"
          incidentId="incident-123"
          entries={entries}
          members={members}
          currentUserId="123"
          currentUserRole={currentUserRole}
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

describe("Timeline — rendering", () => {
  it("shows an empty state when there are no entries", () => {
    renderTimeline([], "manager");
    expect(screen.getByText("No timeline entries yet.")).toBeInTheDocument();
  });

  it("resolves each entry's author name from the members list", () => {
    renderTimeline([myEntry, bobsEntry], "manager");
    expect(screen.getByText("alissa")).toBeInTheDocument();
    expect(screen.getByText("bob")).toBeInTheDocument();
  });

  it("falls back to 'Unknown' when the author no longer exists", () => {
    renderTimeline([{ ...myEntry, author_id: null }], "manager");
    expect(screen.getByText("Unknown")).toBeInTheDocument();
  });

  it("marks only edited entries with the edited suffix", () => {
    renderTimeline([myEntry, bobsEntry], "manager");
    expect(screen.getAllByText(/edited/)).toHaveLength(1);
  });

  it("only shows the add-entry form to responders and managers", () => {
    renderTimeline([], "observer");
    expect(
      screen.queryByLabelText("Add a timeline entry"),
    ).not.toBeInTheDocument();

    renderTimeline([], "responder");
    expect(screen.getByLabelText("Add a timeline entry")).toBeInTheDocument();
  });

  it("only offers an edit link on the current user's own entries", () => {
    renderTimeline([myEntry, bobsEntry], "manager");
    expect(screen.getAllByRole("button", { name: "Edit" })).toHaveLength(1);
  });
});

describe("Timeline — editing an entry", () => {
  it("edits the entry's content and exits edit mode on success", async () => {
    let requestedBody: unknown;
    server.use(
      http.patch(
        "http://localhost:8080/teams/:teamId/incidents/:incidentId/timeline/:entryId",
        async ({ request }) => {
          requestedBody = await request.json();
          return HttpResponse.json({
            ...myEntry,
            content: "Root cause found",
            edited_at: 1755000300,
          });
        },
      ),
    );

    const user = userEvent.setup();
    renderTimeline([myEntry], "manager");

    await user.click(screen.getByRole("button", { name: "Edit" }));
    const textarea = screen.getByLabelText("Edit timeline entry");
    await user.clear(textarea);
    await user.type(textarea, "Root cause found");
    await user.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() =>
      expect(requestedBody).toEqual({ content: "Root cause found" }),
    );
    await waitFor(() =>
      expect(
        screen.queryByLabelText("Edit timeline entry"),
      ).not.toBeInTheDocument(),
    );
  });

  it("cancels editing without submitting anything", async () => {
    const user = userEvent.setup();
    renderTimeline([myEntry], "manager");

    await user.click(screen.getByRole("button", { name: "Edit" }));
    await user.click(screen.getByRole("button", { name: "Cancel" }));

    expect(
      screen.queryByLabelText("Edit timeline entry"),
    ).not.toBeInTheDocument();
    expect(screen.getByText("Investigating the outage")).toBeInTheDocument();
  });
});

describe("Timeline — adding an entry", () => {
  it("adds a new entry and clears the textarea on success", async () => {
    let requestedBody: unknown;
    server.use(
      http.post(
        "http://localhost:8080/teams/:teamId/incidents/:incidentId/timeline",
        async ({ request }) => {
          requestedBody = await request.json();
          return HttpResponse.json(
            { ...myEntry, id: "entry-3", content: "New update" },
            { status: 201 },
          );
        },
      ),
    );

    const user = userEvent.setup();
    renderTimeline([], "responder");

    const textarea = screen.getByLabelText("Add a timeline entry");
    await user.type(textarea, "New update");
    await user.click(screen.getByRole("button", { name: "Add entry" }));

    await waitFor(() =>
      expect(requestedBody).toEqual({ content: "New update" }),
    );
  });

  it("does not submit an entry that is only whitespace", async () => {
    let called = false;
    server.use(
      http.post(
        "http://localhost:8080/teams/:teamId/incidents/:incidentId/timeline",
        () => {
          called = true;
          return HttpResponse.json(myEntry, { status: 201 });
        },
      ),
    );

    const user = userEvent.setup();
    renderTimeline([], "responder");

    await user.type(screen.getByLabelText("Add a timeline entry"), "   ");
    await user.click(screen.getByRole("button", { name: "Add entry" }));

    expect(called).toBe(false);
  });
});

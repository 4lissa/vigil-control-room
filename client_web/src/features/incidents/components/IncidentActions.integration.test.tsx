import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { NextIntlClientProvider } from "next-intl";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/node";
import { setToken } from "@/features/auth/token";
import { IncidentActions } from "./IncidentActions";
import { IncidentResponse } from "../types";
import { TeamMemberResponse, TeamRole } from "@/features/teams";
import messages from "../../../../messages/en.json";

const baseIncident: IncidentResponse = {
  id: "incident-123",
  team_id: "team-123",
  title: "DB down",
  description: "",
  severity: "high",
  state: "open",
  assigned_to: null,
  release_id: null,
  created_by: "123",
  created_at: 1755000000,
  resolved_at: null,
};

const members: TeamMemberResponse[] = [
  {
    id: "member-456",
    team_id: "team-123",
    user_id: "456",
    username: "bob",
    role: "responder",
    joined_at: "2026-08-13T00:00:00Z",
  },
];

const renderActions = (
  incident: Partial<IncidentResponse>,
  currentUserRole: TeamRole | undefined,
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
        <IncidentActions
          teamId="team-123"
          incident={{ ...baseIncident, ...incident }}
          members={members}
          currentUserRole={currentUserRole}
        />
      </QueryClientProvider>
    </NextIntlClientProvider>,
  );
};

beforeEach(() => {
  setToken("my-session-token");
});

describe("IncidentActions — permission and lifecycle matrix", () => {
  it("renders nothing for an observer", () => {
    const { container } = renderActions({ state: "open" }, "observer");
    expect(container).toBeEmptyDOMElement();
  });

  it("renders nothing once the incident is resolved, even for a manager", () => {
    const { container } = renderActions({ state: "resolved" }, "manager");
    expect(container).toBeEmptyDOMElement();
  });

  it("offers acknowledge to a responder while the incident is open", () => {
    renderActions({ state: "open" }, "responder");
    expect(
      screen.getByRole("button", { name: "Acknowledge" }),
    ).toBeInTheDocument();
  });

  it("no longer offers acknowledge once the incident is acknowledged", () => {
    renderActions({ state: "acknowledged" }, "responder");
    expect(
      screen.queryByRole("button", { name: "Acknowledge" }),
    ).not.toBeInTheDocument();
  });

  it("offers escalate to a responder only while acknowledged or escalated", () => {
    renderActions({ state: "open" }, "responder");
    expect(
      screen.queryByRole("button", { name: "Escalate" }),
    ).not.toBeInTheDocument();

    renderActions({ state: "acknowledged" }, "responder");
    expect(
      screen.getByRole("button", { name: "Escalate" }),
    ).toBeInTheDocument();
  });

  it("never offers resolve or assign to a responder, only to a manager", () => {
    renderActions({ state: "acknowledged" }, "responder");
    expect(
      screen.queryByRole("button", { name: "Resolve" }),
    ).not.toBeInTheDocument();
    expect(screen.queryByLabelText("Assign to")).not.toBeInTheDocument();

    renderActions({ state: "acknowledged" }, "manager");
    expect(screen.getByRole("button", { name: "Resolve" })).toBeInTheDocument();
    expect(screen.getByLabelText("Assign to")).toBeInTheDocument();
  });

  it("acknowledges the incident when clicked", async () => {
    let called = false;
    server.use(
      http.post(
        "http://localhost:8080/teams/:teamId/incidents/:incidentId/acknowledge",
        () => {
          called = true;
          return HttpResponse.json({ ...baseIncident, state: "acknowledged" });
        },
      ),
    );

    const user = userEvent.setup();
    renderActions({ state: "open" }, "responder");
    await user.click(screen.getByRole("button", { name: "Acknowledge" }));

    await waitFor(() => expect(called).toBe(true));
  });

  it("escalates with the severity selected in the dropdown", async () => {
    let requestedBody: unknown;
    server.use(
      http.post(
        "http://localhost:8080/teams/:teamId/incidents/:incidentId/escalate",
        async ({ request }) => {
          requestedBody = await request.json();
          return HttpResponse.json({ ...baseIncident, state: "escalated" });
        },
      ),
    );

    const user = userEvent.setup();
    renderActions({ state: "acknowledged" }, "manager");

    await user.selectOptions(screen.getByLabelText("New severity"), "critical");
    await user.click(screen.getByRole("button", { name: "Escalate" }));

    await waitFor(() =>
      expect(requestedBody).toEqual({ severity: "critical" }),
    );
  });

  it("assigns the responder selected in the dropdown", async () => {
    let requestedBody: unknown;
    server.use(
      http.post(
        "http://localhost:8080/teams/:teamId/incidents/:incidentId/assign",
        async ({ request }) => {
          requestedBody = await request.json();
          return HttpResponse.json({ ...baseIncident, assigned_to: "456" });
        },
      ),
    );

    const user = userEvent.setup();
    renderActions({ state: "open" }, "manager");

    await user.selectOptions(screen.getByLabelText("Assign to"), "456");
    await user.click(screen.getByRole("button", { name: "Assign" }));

    await waitFor(() => expect(requestedBody).toEqual({ user_id: "456" }));
  });

  it("can unassign by submitting the empty option", async () => {
    let requestedBody: unknown;
    server.use(
      http.post(
        "http://localhost:8080/teams/:teamId/incidents/:incidentId/assign",
        async ({ request }) => {
          requestedBody = await request.json();
          return HttpResponse.json({ ...baseIncident, assigned_to: null });
        },
      ),
    );

    const user = userEvent.setup();
    renderActions({ state: "open", assigned_to: "456" }, "manager");

    await user.selectOptions(screen.getByLabelText("Assign to"), "");
    await user.click(screen.getByRole("button", { name: "Assign" }));

    await waitFor(() => expect(requestedBody).toEqual({ user_id: null }));
  });

  it("shows an error banner when an action fails", async () => {
    server.use(
      http.post(
        "http://localhost:8080/teams/:teamId/incidents/:incidentId/acknowledge",
        () =>
          HttpResponse.json(
            { error: { code: "FORBIDDEN", message: "Not allowed" } },
            { status: 403 },
          ),
      ),
    );

    const user = userEvent.setup();
    renderActions({ state: "open" }, "responder");
    await user.click(screen.getByRole("button", { name: "Acknowledge" }));

    await waitFor(() =>
      expect(screen.getByText("Not allowed")).toBeInTheDocument(),
    );
  });
});

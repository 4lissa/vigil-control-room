import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/node";
import { setToken } from "@/features/auth/token";
import { Navbar } from "./Navbar";

const push = vi.fn();
let mockPathname = "/teams/team-123/incidents";
let mockParams: { teamId?: string } = { teamId: "team-123" };

vi.mock("next/navigation", () => ({
  useRouter: () => ({ push, replace: vi.fn() }),
  usePathname: () => mockPathname,
  useParams: () => mockParams,
}));

const renderNavbar = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <Navbar />
    </QueryClientProvider>,
  );
};

beforeEach(() => {
  setToken("my-session-token");
  push.mockClear();
  mockPathname = "/teams/team-123/incidents";
  mockParams = { teamId: "team-123" };
  HTMLDialogElement.prototype.showModal = vi.fn();
  HTMLDialogElement.prototype.close = vi.fn();
});

describe("Navbar", () => {
  it("shows the active team name and the current user's role", async () => {
    renderNavbar();

    await waitFor(() =>
      expect(screen.getByText("My Team")).toBeInTheDocument(),
    );
    await waitFor(() =>
      expect(screen.getByText("- manager")).toBeInTheDocument(),
    );
  });

  it("points Incidents and Releases at the current team", async () => {
    renderNavbar();

    await waitFor(() =>
      expect(screen.getByRole("link", { name: "Incidents" })).toHaveAttribute(
        "href",
        "/teams/team-123/incidents",
      ),
    );
    expect(screen.getByRole("link", { name: "Releases" })).toHaveAttribute(
      "href",
      "/teams/team-123/releases",
    );
  });

  it("marks Incidents as the active link on the incidents page", async () => {
    renderNavbar();

    await waitFor(() => {
      expect(
        screen.getByRole("link", { name: "Incidents" }).className,
      ).toContain("font-medium");
    });
    expect(
      screen.getByRole("link", { name: "Releases" }).className,
    ).not.toContain("font-medium");
  });

  it("switches team while staying on the same section", async () => {
    server.use(
      http.get("http://localhost:8080/teams", () => {
        return HttpResponse.json([
          {
            id: "team-123",
            name: "My Team",
            created_by: "123",
            created_at: "2026-08-12T00:00:00Z",
          },
          {
            id: "team-456",
            name: "Other Team",
            created_by: "123",
            created_at: "2026-08-13T00:00:00Z",
          },
        ]);
      }),
    );
    mockPathname = "/teams/team-123/releases";

    const user = userEvent.setup();
    renderNavbar();

    await waitFor(() =>
      expect(screen.getByText("My Team")).toBeInTheDocument(),
    );
    await user.click(screen.getByText("My Team"));
    await user.click(await screen.findByText("Other Team"));

    expect(push).toHaveBeenCalledWith("/teams/team-456/releases");
  });

  it("logs out and redirects to /login", async () => {
    const user = userEvent.setup();
    renderNavbar();

    await waitFor(() => expect(screen.getByText("A")).toBeInTheDocument());
    await user.click(screen.getByText("A").closest("button")!);
    await user.click(screen.getByText("Logout"));

    await waitFor(() => expect(push).toHaveBeenCalledWith("/login"));
  });

  it("falls back to the bare routes when the user has no team", async () => {
    server.use(
      http.get("http://localhost:8080/teams", () => HttpResponse.json([])),
    );
    mockPathname = "/incidents";
    mockParams = {};

    renderNavbar();

    await waitFor(() =>
      expect(screen.getByText("No team")).toBeInTheDocument(),
    );
    expect(screen.getByRole("link", { name: "Incidents" })).toHaveAttribute(
      "href",
      "/incidents",
    );
    expect(screen.getByRole("link", { name: "Releases" })).toHaveAttribute(
      "href",
      "/releases",
    );
  });

  it("opens the create team dialog from the team dropdown", async () => {
    const user = userEvent.setup();
    renderNavbar();

    await waitFor(() =>
      expect(screen.getByText("My Team")).toBeInTheDocument(),
    );
    await user.click(screen.getByText("My Team"));
    await user.click(screen.getByText("Create a team"));

    expect(screen.getByRole("dialog", { hidden: true })).toBeInTheDocument();
    expect(screen.getByLabelText("Team name")).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Create a team" }),
    ).not.toBeInTheDocument();
  });

  it("opens the join team dialog from the team dropdown", async () => {
    const user = userEvent.setup();
    renderNavbar();

    await waitFor(() =>
      expect(screen.getByText("My Team")).toBeInTheDocument(),
    );
    await user.click(screen.getByText("My Team"));
    await user.click(screen.getByText("Join a team"));

    expect(screen.getByLabelText("Invitation code")).toBeInTheDocument();
  });

  it("links each team's settings gear to its team page", async () => {
    const user = userEvent.setup();
    renderNavbar();

    await waitFor(() =>
      expect(screen.getByText("My Team")).toBeInTheDocument(),
    );
    await user.click(screen.getByText("My Team"));

    expect(
      screen.getByRole("link", { name: "My Team settings" }),
    ).toHaveAttribute("href", "/teams/team-123");
  });
});

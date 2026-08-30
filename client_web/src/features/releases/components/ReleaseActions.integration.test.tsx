import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { NextIntlClientProvider } from "next-intl";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/node";
import { setToken } from "@/features/auth/token";
import { ReleaseActions } from "./ReleaseActions";
import { ReleaseResponse } from "../types";
import messages from "../../../../messages/en.json";

const release: ReleaseResponse = {
  id: "release-123",
  team_id: "team-123",
  name: "v1.0.0",
  state: "in_progress",
  created_by: "123",
  created_at: 1755000000,
  completed_at: null,
};

const renderActions = (
  currentUserRole: "observer" | "responder" | "manager" | undefined,
  releaseOverrides: Partial<ReleaseResponse> = {},
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
        <ReleaseActions
          teamId="team-123"
          release={{ ...release, ...releaseOverrides }}
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

describe("ReleaseActions", () => {
  it("renders nothing for a non-manager", () => {
    const { container } = renderActions("responder");
    expect(container).toBeEmptyDOMElement();
  });

  it("renders nothing once the release is completed or cancelled", () => {
    const { container: completed } = renderActions("manager", {
      state: "completed",
    });
    expect(completed).toBeEmptyDOMElement();

    const { container: cancelled } = renderActions("manager", {
      state: "cancelled",
    });
    expect(cancelled).toBeEmptyDOMElement();
  });

  it("offers a cancel button naming the release for a manager", async () => {
    const user = userEvent.setup();
    renderActions("manager");

    await user.click(screen.getByRole("button", { name: "Cancel release" }));

    expect(
      screen.getByText(
        "Are you sure you want to cancel v1.0.0? This cannot be undone.",
      ),
    ).toBeInTheDocument();
  });

  it("cancels the release when the cancellation is confirmed", async () => {
    const user = userEvent.setup();
    renderActions("manager");

    await user.click(screen.getByRole("button", { name: "Cancel release" }));
    await user.click(
      screen.getByRole("button", {
        name: "Confirm cancellation",
        hidden: true,
      }),
    );

    await waitFor(() =>
      expect(
        screen.queryByText(
          "Are you sure you want to cancel v1.0.0? This cannot be undone.",
        ),
      ).not.toBeInTheDocument(),
    );
  });

  it("keeps the release when dismissing the confirmation", async () => {
    const user = userEvent.setup();
    renderActions("manager");

    await user.click(screen.getByRole("button", { name: "Cancel release" }));
    await user.click(
      screen.getByRole("button", { name: "Keep release", hidden: true }),
    );

    expect(
      screen.queryByText(
        "Are you sure you want to cancel v1.0.0? This cannot be undone.",
      ),
    ).not.toBeInTheDocument();
  });

  it("shows an error banner when cancellation fails", async () => {
    server.use(
      http.post(
        "http://localhost:8080/teams/:teamId/releases/:releaseId/cancel",
        () =>
          HttpResponse.json(
            { error: { code: "FORBIDDEN", message: "Not allowed" } },
            { status: 403 },
          ),
      ),
    );

    const user = userEvent.setup();
    renderActions("manager");

    await user.click(screen.getByRole("button", { name: "Cancel release" }));
    await user.click(
      screen.getByRole("button", {
        name: "Confirm cancellation",
        hidden: true,
      }),
    );

    await waitFor(() =>
      expect(screen.getByText("Not allowed")).toBeInTheDocument(),
    );
  });
});

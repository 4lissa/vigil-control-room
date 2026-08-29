import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { NextIntlClientProvider } from "next-intl";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/node";
import { setToken } from "@/features/auth/token";
import { ManagerActions } from "./ManagerActions";
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

const renderActions = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  return render(
    <NextIntlClientProvider locale="en" messages={messages}>
      <QueryClientProvider client={queryClient}>
        <ManagerActions
          teamId="team-123"
          members={[me, bob]}
          currentUserId="123"
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

describe("ManagerActions", () => {
  it("generates and displays an invitation code", async () => {
    const user = userEvent.setup();
    renderActions();

    await user.click(
      screen.getByRole("button", { name: "Generate invite code" }),
    );

    await waitFor(() => expect(screen.getByText("ABC123")).toBeInTheDocument());
  });

  it("copies the generated code to the clipboard", async () => {
    const user = userEvent.setup();
    renderActions();
    const writeText = vi
      .spyOn(navigator.clipboard, "writeText")
      .mockResolvedValue(undefined);

    await user.click(
      screen.getByRole("button", { name: "Generate invite code" }),
    );
    await waitFor(() => expect(screen.getByText("ABC123")).toBeInTheDocument());
    await user.click(screen.getByRole("button", { name: "Copy" }));

    expect(writeText).toHaveBeenCalledWith("ABC123");
  });

  it("only lists other members as transfer targets", () => {
    renderActions();

    const select = screen.getByLabelText("Transfer manager role");
    expect(screen.queryByText("alissa")).not.toBeInTheDocument();
    expect(select).toHaveTextContent("bob");
  });

  it("disables the transfer button until a member is selected", async () => {
    const user = userEvent.setup();
    renderActions();

    expect(
      screen.getByRole("button", { name: "Transfer manager" }),
    ).toBeDisabled();

    await user.selectOptions(
      screen.getByLabelText("Transfer manager role"),
      "456",
    );

    expect(
      screen.getByRole("button", { name: "Transfer manager" }),
    ).toBeEnabled();
  });

  it("confirms the transfer, naming the selected member, before submitting it", async () => {
    const user = userEvent.setup();
    renderActions();

    await user.selectOptions(
      screen.getByLabelText("Transfer manager role"),
      "456",
    );
    await user.click(screen.getByRole("button", { name: "Transfer manager" }));

    expect(
      screen.getByText(
        "Transfer the Manager role to bob? You will become a Responder.",
      ),
    ).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", { name: "Transfer to bob", hidden: true }),
    );

    await waitFor(() =>
      expect(
        screen.queryByText(
          "Transfer the Manager role to bob? You will become a Responder.",
        ),
      ).not.toBeInTheDocument(),
    );
  });

  it("shows an error banner when the transfer fails", async () => {
    server.use(
      http.post("http://localhost:8080/teams/:teamId/transfer", () =>
        HttpResponse.json(
          { error: { code: "FORBIDDEN", message: "Not allowed" } },
          { status: 403 },
        ),
      ),
    );

    const user = userEvent.setup();
    renderActions();

    await user.selectOptions(
      screen.getByLabelText("Transfer manager role"),
      "456",
    );
    await user.click(screen.getByRole("button", { name: "Transfer manager" }));
    await user.click(
      screen.getByRole("button", { name: "Transfer to bob", hidden: true }),
    );

    await waitFor(() =>
      expect(screen.getByText("Not allowed")).toBeInTheDocument(),
    );
  });
});

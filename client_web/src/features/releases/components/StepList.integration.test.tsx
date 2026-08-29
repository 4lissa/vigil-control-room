import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { NextIntlClientProvider } from "next-intl";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/node";
import { setToken } from "@/features/auth/token";
import { StepList } from "./StepList";
import { ReleaseStepResponse } from "../types";
import messages from "../../../../messages/en.json";

const validatedStep: ReleaseStepResponse = {
  id: "step-1",
  release_id: "release-123",
  name: "build",
  position: 0,
  validated_at: 1755000000,
  validated_by: "123",
};

const pendingStep: ReleaseStepResponse = {
  id: "step-2",
  release_id: "release-123",
  name: "staging",
  position: 1,
  validated_at: null,
  validated_by: null,
};

const renderList = (steps: ReleaseStepResponse[], canValidate: boolean) => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  return render(
    <NextIntlClientProvider locale="en" messages={messages}>
      <QueryClientProvider client={queryClient}>
        <StepList
          teamId="team-123"
          releaseId="release-123"
          steps={steps}
          canValidate={canValidate}
        />
      </QueryClientProvider>
    </NextIntlClientProvider>,
  );
};

beforeEach(() => {
  setToken("my-session-token");
});

describe("StepList", () => {
  it("shows the validated timestamp for a validated step", () => {
    renderList([validatedStep], false);
    expect(screen.getByText("build")).toBeInTheDocument();
    expect(screen.getByText(/validated/i)).toBeInTheDocument();
  });

  it("does not offer a validate button when the user cannot validate", () => {
    renderList([pendingStep], false);
    expect(
      screen.queryByRole("button", { name: "Validate" }),
    ).not.toBeInTheDocument();
  });

  it("offers a validate button only on the next unvalidated step", () => {
    renderList([validatedStep, pendingStep], true);
    expect(screen.getAllByRole("button", { name: "Validate" })).toHaveLength(1);
  });

  it("validates the step for the release the list was rendered for", async () => {
    let requestedReleaseId: string | undefined;
    let requestedStepId: string | undefined;
    server.use(
      http.post(
        "http://localhost:8080/teams/:teamId/releases/:releaseId/steps/:stepId/validate",
        ({ params }) => {
          requestedReleaseId = params.releaseId as string;
          requestedStepId = params.stepId as string;
          return HttpResponse.json({
            ...pendingStep,
            validated_at: 1755001000,
          });
        },
      ),
    );

    const user = userEvent.setup();
    renderList([pendingStep], true);

    await user.click(screen.getByRole("button", { name: "Validate" }));

    await waitFor(() => expect(requestedStepId).toBe("step-2"));
    expect(requestedReleaseId).toBe("release-123");
  });

  it("shows an error banner when validation fails", async () => {
    server.use(
      http.post(
        "http://localhost:8080/teams/:teamId/releases/:releaseId/steps/:stepId/validate",
        () =>
          HttpResponse.json(
            { error: { code: "FORBIDDEN", message: "Not allowed" } },
            { status: 403 },
          ),
      ),
    );

    const user = userEvent.setup();
    renderList([pendingStep], true);

    await user.click(screen.getByRole("button", { name: "Validate" }));

    await waitFor(() =>
      expect(screen.getByText("Not allowed")).toBeInTheDocument(),
    );
  });
});

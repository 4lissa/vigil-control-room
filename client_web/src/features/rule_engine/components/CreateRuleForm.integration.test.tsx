import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { NextIntlClientProvider } from "next-intl";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/node";
import { setToken } from "@/features/auth/token";
import { CreateRuleForm } from "./CreateRuleForm";
import { API_BASE_URL } from "@/shared/lib/api-client";
import messages from "../../../../messages/en.json";

const renderForm = (onSuccess = () => {}) => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  return render(
    <NextIntlClientProvider locale="en" messages={messages}>
      <QueryClientProvider client={queryClient}>
        <CreateRuleForm teamId="team-123" onSuccess={onSuccess} />
      </QueryClientProvider>
    </NextIntlClientProvider>,
  );
};

const fillCommonFields = async (user: ReturnType<typeof userEvent.setup>) => {
  await user.type(screen.getByLabelText("Rule name"), "CI failure > Incident");
  await user.type(
    screen.getByLabelText("GitHub repository (owner/repo)"),
    "my-org/my-repo",
  );
  await user.type(screen.getByLabelText("Webhook secret"), "s3cret");
};

beforeEach(() => {
  setToken("my-session-token");
});

describe("CreateRuleForm — reaction type switching", () => {
  it("shows the VIGIL incident fields by default", () => {
    renderForm();
    expect(screen.getByLabelText("Incident title")).toBeInTheDocument();
    expect(screen.getByLabelText("Incident severity")).toBeInTheDocument();
    expect(screen.queryByLabelText("Target URL")).not.toBeInTheDocument();
  });

  it("switches to the HTTP fields when the HTTP reaction is selected", async () => {
    const user = userEvent.setup();
    renderForm();

    await user.selectOptions(screen.getByLabelText("Reaction"), "http_post");

    expect(screen.getByLabelText("Target URL")).toBeInTheDocument();
    expect(screen.queryByLabelText("Incident title")).not.toBeInTheDocument();
    expect(
      screen.queryByLabelText("Incident severity"),
    ).not.toBeInTheDocument();
  });
});

describe("CreateRuleForm — submission payload", () => {
  it("builds the vigil_create_incident payload with the chosen severity", async () => {
    let requestedBody: Record<string, unknown> | undefined;
    server.use(
      http.post(
        "http://localhost:8080/teams/:teamId/rules",
        async ({ request }) => {
          requestedBody = (await request.json()) as Record<string, unknown>;
          return HttpResponse.json(
            {
              id: "rule-999",
              team_id: "team-123",
              name: "CI failure > Incident",
              enabled: true,
              trigger: {
                service: "github",
                event: "workflow_run",
                filters: { "repository.full_name": "my-org/my-repo" },
              },
              reaction: { type: "vigil_create_incident", payload: {} },
              created_by: "123",
              created_at: 1755000000,
            },
            { status: 201 },
          );
        },
      ),
    );

    const user = userEvent.setup();
    renderForm();
    await fillCommonFields(user);
    await user.selectOptions(
      screen.getByLabelText("Incident severity"),
      "critical",
    );
    await user.click(screen.getByRole("button", { name: "Create rule" }));

    await waitFor(() => expect(requestedBody).toBeDefined());
    expect(requestedBody).toMatchObject({
      name: "CI failure > Incident",
      webhook_secret: "s3cret",
      trigger: {
        service: "github",
        event: "workflow_run",
        filters: {
          "workflow_run.conclusion": "failure",
          "repository.full_name": "my-org/my-repo",
        },
      },
      reaction: {
        type: "vigil_create_incident",
        payload: expect.objectContaining({ severity: "critical" }),
      },
    });
  });

  it("builds the http_post payload with the target url instead of an incident title", async () => {
    let requestedBody: Record<string, unknown> | undefined;
    server.use(
      http.post(
        "http://localhost:8080/teams/:teamId/rules",
        async ({ request }) => {
          requestedBody = (await request.json()) as Record<string, unknown>;
          return HttpResponse.json(
            {
              id: "rule-999",
              team_id: "team-123",
              name: "x",
              enabled: true,
              trigger: {
                service: "github",
                event: "workflow_run",
                filters: {},
              },
              reaction: { type: "http_post", payload: {} },
              created_by: "123",
              created_at: 1755000000,
            },
            { status: 201 },
          );
        },
      ),
    );

    const user = userEvent.setup();
    renderForm();
    await fillCommonFields(user);
    await user.selectOptions(screen.getByLabelText("Reaction"), "http_post");
    await user.type(
      screen.getByLabelText("Target URL"),
      "https://example.com/hook",
    );
    await user.click(screen.getByRole("button", { name: "Create rule" }));

    await waitFor(() => expect(requestedBody).toBeDefined());
    const reaction = requestedBody!.reaction as {
      type: string;
      payload: Record<string, unknown>;
    };
    expect(reaction.type).toBe("http_post");
    expect(reaction.payload).toMatchObject({
      url: "https://example.com/hook",
    });
    expect(reaction.payload).not.toHaveProperty("title");
  });

  it("omits the body field entirely when left blank", async () => {
    let requestedBody: Record<string, unknown> | undefined;
    server.use(
      http.post(
        "http://localhost:8080/teams/:teamId/rules",
        async ({ request }) => {
          requestedBody = (await request.json()) as Record<string, unknown>;
          return HttpResponse.json(
            {
              id: "rule-999",
              team_id: "team-123",
              name: "x",
              enabled: true,
              trigger: {
                service: "github",
                event: "workflow_run",
                filters: {},
              },
              reaction: { type: "vigil_create_incident", payload: {} },
              created_by: "123",
              created_at: 1755000000,
            },
            { status: 201 },
          );
        },
      ),
    );

    const user = userEvent.setup();
    renderForm();
    await fillCommonFields(user);
    await user.clear(screen.getByLabelText("Incident description (optional)"));
    await user.click(screen.getByRole("button", { name: "Create rule" }));

    await waitFor(() => expect(requestedBody).toBeDefined());
    const reaction = requestedBody!.reaction as {
      payload: Record<string, unknown>;
    };
    expect(reaction.payload).not.toHaveProperty("body");
  });
});

describe("CreateRuleForm — after creation", () => {
  it("shows the webhook URL to configure on GitHub and lets the user finish", async () => {
    const onSuccess = vi.fn();
    const user = userEvent.setup();
    renderForm(onSuccess);
    await fillCommonFields(user);
    await user.click(screen.getByRole("button", { name: "Create rule" }));

    await waitFor(() =>
      expect(
        screen.getByText(`${API_BASE_URL}/webhooks/github/rule-123`),
      ).toBeInTheDocument(),
    );

    await user.click(screen.getByRole("button", { name: "Done" }));
    expect(onSuccess).toHaveBeenCalledOnce();
  });

  it("shows an error banner when the rule creation fails", async () => {
    server.use(
      http.post("http://localhost:8080/teams/:teamId/rules", () =>
        HttpResponse.json(
          { error: { code: "VALIDATION_ERROR", message: "Invalid rule" } },
          { status: 422 },
        ),
      ),
    );

    const user = userEvent.setup();
    renderForm();
    await fillCommonFields(user);
    await user.click(screen.getByRole("button", { name: "Create rule" }));

    await waitFor(() =>
      expect(screen.getByText("Invalid rule")).toBeInTheDocument(),
    );
  });
});

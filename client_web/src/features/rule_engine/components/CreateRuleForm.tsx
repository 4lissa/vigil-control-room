"use client";

import { useState } from "react";
import { useCreateRule } from "../hooks";
import { RuleResponse } from "../types";
import { API_BASE_URL } from "@/shared/lib/api-client";
import { Button } from "@/shared/ui/Button";
import { Input } from "@/shared/ui/Input";

type Severity = "low" | "medium" | "high" | "critical";

interface CreateRuleFormProps {
  teamId: string;
  onSuccess: () => void;
}

export const CreateRuleForm = ({ teamId, onSuccess }: CreateRuleFormProps) => {
  const [severity, setSeverity] = useState<Severity>("high");
  const [createdRule, setCreatedRule] = useState<RuleResponse | null>(null);
  const { mutate: createRule, isPending, error } = useCreateRule(teamId);

  const handleSubmit = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    const form = e.currentTarget;
    const name = (form.elements.namedItem("name") as HTMLInputElement).value;
    const repository = (
      form.elements.namedItem("repository") as HTMLInputElement
    ).value;
    const webhookSecret = (
      form.elements.namedItem("webhook_secret") as HTMLInputElement
    ).value;
    const title = (form.elements.namedItem("title") as HTMLInputElement).value;
    const body = (form.elements.namedItem("body") as HTMLTextAreaElement).value;

    createRule(
      {
        name,
        enabled: true,
        trigger: {
          service: "github",
          event: "workflow_run",
          filters: {
            "workflow_run.conclusion": "failure",
            "repository.full_name": repository,
          },
        },
        webhook_secret: webhookSecret,
        reaction: {
          type: "vigil_create_incident",
          payload: body ? { title, severity, body } : { title, severity },
        },
      },
      { onSuccess: (rule) => setCreatedRule(rule) },
    );
  };

  if (createdRule) {
    const webhookUrl = `${API_BASE_URL}/webhooks/github/${createdRule.id}`;

    return (
      <div className="flex flex-col gap-4">
        <p className="text-body text-[var(--color-text-primary)]">
          Rule created. On GitHub, add a webhook on{" "}
          <strong>{createdRule.trigger.filters["repository.full_name"]}</strong>{" "}
          with:
        </p>
        <div className="flex flex-col gap-1">
          <span className="text-caption font-medium text-[var(--color-text-secondary)]">
            Payload URL
          </span>
          <code className="px-3 py-2 text-body rounded-md border border-[var(--color-border)] bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] break-all select-all">
            {webhookUrl}
          </code>
        </div>
        <div className="flex flex-col gap-1">
          <span className="text-caption font-medium text-[var(--color-text-secondary)]">
            Content type
          </span>
          <code className="px-3 py-2 text-body rounded-md border border-[var(--color-border)] bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)]">
            application/json
          </code>
        </div>
        <p className="text-caption text-[var(--color-text-muted)]">
          Set the webhook secret to the same value you entered here, and select
          the &quot;Workflow runs&quot; event.
        </p>
        <Button variant="primary" onClick={onSuccess} className="w-full mt-2">
          Done
        </Button>
      </div>
    );
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-4">
      {error && (
        <div className="px-4 py-2 text-body rounded-md border border-[var(--color-danger-border)] bg-[var(--color-danger-bg)] text-[var(--color-danger)]">
          {error.message}
        </div>
      )}
      <Input name="name" label="Rule name" required />
      <Input
        name="repository"
        label="GitHub repository (owner/repo)"
        required
      />
      <Input name="webhook_secret" label="Webhook secret" required />
      <Input
        name="title"
        label="Incident title"
        defaultValue="CI broken on {{repository.full_name}}"
        required
      />
      <div className="flex flex-col gap-1">
        <label
          htmlFor="body"
          className="text-body font-medium text-[var(--color-text-secondary)]"
        >
          Incident description (optional)
        </label>
        <textarea
          id="body"
          name="body"
          rows={2}
          defaultValue="Workflow {{workflow_run.name}} failed — {{workflow_run.html_url}}"
          className="w-full px-3 py-2 text-body rounded-md border bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] border-[var(--color-border)] placeholder:text-[var(--color-text-muted)] hover:border-[var(--color-border-strong)] focus:outline-none focus:border-[var(--color-accent)]"
        />
      </div>
      <div className="flex flex-col gap-1">
        <label
          htmlFor="severity"
          className="text-body font-medium text-[var(--color-text-secondary)]"
        >
          Incident severity
        </label>
        <select
          id="severity"
          value={severity}
          onChange={(e) => setSeverity(e.target.value as Severity)}
          className="px-3 py-2 text-body rounded-md border border-[var(--color-border)] bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] focus:outline-none focus:border-[var(--color-accent)]"
        >
          <option value="low">Low</option>
          <option value="medium">Medium</option>
          <option value="high">High</option>
          <option value="critical">Critical</option>
        </select>
      </div>
      <Button
        type="submit"
        variant="primary"
        isLoading={isPending}
        className="w-full mt-2"
      >
        Create rule
      </Button>
    </form>
  );
};

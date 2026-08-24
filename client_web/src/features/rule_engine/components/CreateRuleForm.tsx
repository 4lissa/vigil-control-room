"use client";

import { useState } from "react";
import { useTranslations } from "next-intl";
import { useCreateRule } from "../hooks";
import { RuleResponse } from "../types";
import { API_BASE_URL } from "@/shared/lib/api-client";
import { Button } from "@/shared/ui/Button";
import { Input } from "@/shared/ui/Input";

type Severity = "low" | "medium" | "high" | "critical";
type ReactionType = "vigil_create_incident" | "http_post";

interface CreateRuleFormProps {
  teamId: string;
  onSuccess: () => void;
}

export const CreateRuleForm = ({ teamId, onSuccess }: CreateRuleFormProps) => {
  const t = useTranslations("rule_engine");
  const severityT = useTranslations("common.severities");
  const [reactionType, setReactionType] = useState<ReactionType>(
    "vigil_create_incident",
  );
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
    const body = (form.elements.namedItem("body") as HTMLTextAreaElement).value;

    const payload =
      reactionType === "vigil_create_incident"
        ? {
            title: (form.elements.namedItem("title") as HTMLInputElement).value,
            severity,
            ...(body ? { body } : {}),
          }
        : {
            url: (form.elements.namedItem("url") as HTMLInputElement).value,
            ...(body ? { body } : {}),
          };

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
        reaction: { type: reactionType, payload },
      },
      { onSuccess: (rule) => setCreatedRule(rule) },
    );
  };

  if (createdRule) {
    const webhookUrl = `${API_BASE_URL}/webhooks/github/${createdRule.id}`;

    return (
      <div className="flex flex-col gap-4">
        <p className="text-body text-[var(--color-text-primary)]">
          {t("ruleCreatedIntro", {
            repository: createdRule.trigger.filters["repository.full_name"],
          })}
        </p>
        <div className="flex flex-col gap-1">
          <span className="text-caption font-medium text-[var(--color-text-secondary)]">
            {t("payloadUrl")}
          </span>
          <code className="px-3 py-2 text-body rounded-md border border-[var(--color-border)] bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] break-all select-all">
            {webhookUrl}
          </code>
        </div>
        <div className="flex flex-col gap-1">
          <span className="text-caption font-medium text-[var(--color-text-secondary)]">
            {t("contentType")}
          </span>
          <code className="px-3 py-2 text-body rounded-md border border-[var(--color-border)] bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)]">
            application/json
          </code>
        </div>
        <p className="text-caption text-[var(--color-text-muted)]">
          {t("webhookSecretHint")}
        </p>
        <Button variant="primary" onClick={onSuccess} className="w-full mt-2">
          {t("done")}
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
      <Input name="name" label={t("ruleName")} required />
      <Input name="repository" label={t("githubRepository")} required />
      <Input name="webhook_secret" label={t("webhookSecret")} required />
      <div className="flex flex-col gap-1">
        <label
          htmlFor="reaction_type"
          className="text-body font-medium text-[var(--color-text-secondary)]"
        >
          {t("reaction")}
        </label>
        <select
          id="reaction_type"
          value={reactionType}
          onChange={(e) => setReactionType(e.target.value as ReactionType)}
          className="px-3 py-2 text-body rounded-md border border-[var(--color-border)] bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] focus:outline-none focus:border-[var(--color-accent)]"
        >
          <option value="vigil_create_incident">
            {t("vigilCreateIncidentOption")}
          </option>
          <option value="http_post">{t("httpPostOption")}</option>
        </select>
      </div>
      {reactionType === "vigil_create_incident" ? (
        <>
          <Input
            name="title"
            label={t("incidentTitle")}
            defaultValue="CI broken on {{repository.full_name}}"
            required
          />
          <div className="flex flex-col gap-1">
            <label
              htmlFor="severity"
              className="text-body font-medium text-[var(--color-text-secondary)]"
            >
              {t("incidentSeverity")}
            </label>
            <select
              id="severity"
              value={severity}
              onChange={(e) => setSeverity(e.target.value as Severity)}
              className="px-3 py-2 text-body rounded-md border border-[var(--color-border)] bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] focus:outline-none focus:border-[var(--color-accent)]"
            >
              <option value="low">{severityT("low")}</option>
              <option value="medium">{severityT("medium")}</option>
              <option value="high">{severityT("high")}</option>
              <option value="critical">{severityT("critical")}</option>
            </select>
          </div>
        </>
      ) : (
        <Input name="url" label={t("targetUrl")} type="url" required />
      )}
      <div className="flex flex-col gap-1">
        <label
          htmlFor="body"
          className="text-body font-medium text-[var(--color-text-secondary)]"
        >
          {reactionType === "vigil_create_incident"
            ? t("incidentDescriptionOptional")
            : t("requestBodyOptional")}
        </label>
        <textarea
          id="body"
          name="body"
          rows={2}
          defaultValue="Workflow {{workflow_run.name}} failed — {{workflow_run.html_url}}"
          className="w-full px-3 py-2 text-body rounded-md border bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] border-[var(--color-border)] placeholder:text-[var(--color-text-muted)] hover:border-[var(--color-border-strong)] focus:outline-none focus:border-[var(--color-accent)]"
        />
      </div>
      <Button
        type="submit"
        variant="primary"
        isLoading={isPending}
        className="w-full mt-2"
      >
        {t("createRule")}
      </Button>
    </form>
  );
};

"use client";

import { useState } from "react";
import { useTranslations } from "next-intl";
import { useCreateIncident } from "../hooks";
import { Severity } from "../types";
import { useReleases } from "@/features/releases";
import { Button } from "@/shared/ui/Button";
import { Input } from "@/shared/ui/Input";

interface CreateIncidentFormProps {
  teamId: string;
  onSuccess: () => void;
}

export const CreateIncidentForm = ({
  teamId,
  onSuccess,
}: CreateIncidentFormProps) => {
  const t = useTranslations("incidents");
  const severityT = useTranslations("common.severities");
  const [severity, setSeverity] = useState<Severity>("medium");
  const [releaseId, setReleaseId] = useState("");
  const {
    mutate: createIncident,
    isPending,
    error,
  } = useCreateIncident(teamId);
  const { data: releases } = useReleases(teamId);

  const linkableReleases = (releases ?? []).filter(
    (release) => release.state === "created" || release.state === "in_progress",
  );

  const handleSubmit = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    const form = e.currentTarget;
    const title = (form.elements.namedItem("title") as HTMLInputElement).value;
    const description = (
      form.elements.namedItem("description") as HTMLTextAreaElement
    ).value;

    createIncident(
      {
        title,
        description: description || undefined,
        severity,
        release_id: releaseId || undefined,
      },
      { onSuccess },
    );
  };

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-4">
      {error && (
        <div className="px-4 py-2 text-body rounded-md border border-[var(--color-danger-border)] bg-[var(--color-danger-bg)] text-[var(--color-danger)]">
          {error.message}
        </div>
      )}
      <Input name="title" label={t("title")} required />
      <div className="flex flex-col gap-1">
        <label
          htmlFor="description"
          className="text-body font-medium text-[var(--color-text-secondary)]"
        >
          {t("description")}
        </label>
        <textarea
          id="description"
          name="description"
          rows={3}
          className="w-full px-3 py-2 text-body rounded-md border bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] border-[var(--color-border)] placeholder:text-[var(--color-text-muted)] hover:border-[var(--color-border-strong)] focus:outline-none focus:border-[var(--color-accent)]"
        />
      </div>
      <div className="flex flex-col gap-1">
        <label
          htmlFor="severity"
          className="text-body font-medium text-[var(--color-text-secondary)]"
        >
          {t("severity")}
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
      {linkableReleases.length > 0 && (
        <div className="flex flex-col gap-1">
          <label
            htmlFor="release"
            className="text-body font-medium text-[var(--color-text-secondary)]"
          >
            {t("linkToRelease")}
          </label>
          <select
            id="release"
            value={releaseId}
            onChange={(e) => setReleaseId(e.target.value)}
            className="px-3 py-2 text-body rounded-md border border-[var(--color-border)] bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] focus:outline-none focus:border-[var(--color-accent)]"
          >
            <option value="">{t("none")}</option>
            {linkableReleases.map((release) => (
              <option key={release.id} value={release.id}>
                {release.name}
              </option>
            ))}
          </select>
        </div>
      )}
      <Button
        type="submit"
        variant="primary"
        isLoading={isPending}
        className="w-full mt-2"
      >
        {t("createIncident")}
      </Button>
    </form>
  );
};

"use client";

import { useState } from "react";
import { useCreateIncident } from "../hooks";
import { Severity } from "../types";
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
  const [severity, setSeverity] = useState<Severity>("medium");
  const {
    mutate: createIncident,
    isPending,
    error,
  } = useCreateIncident(teamId);

  const handleSubmit = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    const form = e.currentTarget;
    const title = (form.elements.namedItem("title") as HTMLInputElement).value;
    const description = (
      form.elements.namedItem("description") as HTMLTextAreaElement
    ).value;

    createIncident(
      { title, description: description || undefined, severity },
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
      <Input name="title" label="Title" required />
      <div className="flex flex-col gap-1">
        <label
          htmlFor="description"
          className="text-body font-medium text-[var(--color-text-secondary)]"
        >
          Description
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
          Severity
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
        Create incident
      </Button>
    </form>
  );
};

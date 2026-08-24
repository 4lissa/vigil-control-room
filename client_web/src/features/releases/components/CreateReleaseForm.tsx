"use client";

import { useTranslations } from "next-intl";
import { useCreateRelease } from "../hooks";
import { Button } from "@/shared/ui/Button";
import { Input } from "@/shared/ui/Input";

interface CreateReleaseFormProps {
  teamId: string;
  onSuccess: () => void;
}

export const CreateReleaseForm = ({
  teamId,
  onSuccess,
}: CreateReleaseFormProps) => {
  const t = useTranslations("releases");
  const { mutate: createRelease, isPending, error } = useCreateRelease(teamId);

  const handleSubmit = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    const form = e.currentTarget;
    const name = (form.elements.namedItem("name") as HTMLInputElement).value;
    const stepsRaw = (form.elements.namedItem("steps") as HTMLTextAreaElement)
      .value;
    const steps = stepsRaw
      .split("\n")
      .map((step) => step.trim())
      .filter((step) => step.length > 0);

    createRelease({ name, steps }, { onSuccess });
  };

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-4">
      {error && (
        <div className="px-4 py-2 text-body rounded-md border border-[var(--color-danger-border)] bg-[var(--color-danger-bg)] text-[var(--color-danger)]">
          {error.message}
        </div>
      )}
      <Input name="name" label={t("releaseName")} required />
      <div className="flex flex-col gap-1">
        <label
          htmlFor="steps"
          className="text-body font-medium text-[var(--color-text-secondary)]"
        >
          {t("stepsLabel")}
        </label>
        <textarea
          id="steps"
          name="steps"
          rows={4}
          defaultValue={"build\nstaging\ngo_no_go\nproduction"}
          className="w-full px-3 py-2 text-body rounded-md border bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] border-[var(--color-border)] placeholder:text-[var(--color-text-muted)] hover:border-[var(--color-border-strong)] focus:outline-none focus:border-[var(--color-accent)]"
        />
      </div>
      <Button
        type="submit"
        variant="primary"
        isLoading={isPending}
        className="w-full mt-2"
      >
        {t("createRelease")}
      </Button>
    </form>
  );
};

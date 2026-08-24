"use client";

import { useTranslations } from "next-intl";
import { CheckCircle2, Circle } from "lucide-react";
import { useValidateStep } from "../hooks";
import { ReleaseStepResponse } from "../types";
import { Button } from "@/shared/ui/Button";

interface StepListProps {
  teamId: string;
  releaseId: string;
  steps: ReleaseStepResponse[];
  canValidate: boolean;
}

const formatDate = (unixSeconds: number) =>
  new Date(unixSeconds * 1000).toLocaleString();

export const StepList = ({
  teamId,
  releaseId,
  steps,
  canValidate,
}: StepListProps) => {
  const t = useTranslations("releases");
  const {
    mutate: validateStep,
    isPending,
    error,
  } = useValidateStep(teamId, releaseId);

  const nextStepId = steps.find((step) => !step.validated_at)?.id;

  return (
    <div className="flex flex-col gap-4">
      <h2 className="text-subtitle font-medium text-[var(--color-text-primary)]">
        {t("steps")}
      </h2>

      {error && (
        <div className="px-4 py-2 text-body rounded-md border border-[var(--color-danger-border)] bg-[var(--color-danger-bg)] text-[var(--color-danger)]">
          {error.message}
        </div>
      )}

      <div className="rounded-md border border-[var(--color-border)] overflow-hidden">
        {steps.map((step) => {
          const isValidated = !!step.validated_at;
          const isNext = step.id === nextStepId;

          return (
            <div
              key={step.id}
              className="flex items-center justify-between gap-4 px-4 py-3 border-b border-[var(--color-border)] last:border-0"
            >
              <div className="flex items-center gap-2">
                {isValidated ? (
                  <CheckCircle2
                    size={16}
                    className="text-[var(--color-success)]"
                  />
                ) : (
                  <Circle
                    size={16}
                    className="text-[var(--color-text-muted)]"
                  />
                )}
                <span className="text-body text-[var(--color-text-primary)]">
                  {step.name}
                </span>
                {isValidated && (
                  <span className="text-caption text-[var(--color-text-muted)]">
                    {t("validated", { date: formatDate(step.validated_at!) })}
                  </span>
                )}
              </div>
              {canValidate && isNext && (
                <Button
                  variant="secondary"
                  isLoading={isPending}
                  onClick={() => validateStep(step.id)}
                >
                  {t("validate")}
                </Button>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
};

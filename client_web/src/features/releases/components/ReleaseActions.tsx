"use client";

import { useState } from "react";
import { useTranslations } from "next-intl";
import { useCancelRelease } from "../hooks";
import { ReleaseResponse } from "../types";
import { TeamRole } from "@/features/teams";
import { Button } from "@/shared/ui/Button";
import { Dialog } from "@/shared/ui/Dialog";

interface ReleaseActionsProps {
  teamId: string;
  release: ReleaseResponse;
  currentUserRole: TeamRole | undefined;
}

export const ReleaseActions = ({
  teamId,
  release,
  currentUserRole,
}: ReleaseActionsProps) => {
  const t = useTranslations("releases");
  const commonT = useTranslations("common");
  const [confirmOpen, setConfirmOpen] = useState(false);
  const {
    mutate: cancel,
    isPending,
    error,
  } = useCancelRelease(teamId, release.id);

  const canCancel =
    currentUserRole === "manager" &&
    release.state !== "completed" &&
    release.state !== "cancelled";

  if (!canCancel) {
    return null;
  }

  return (
    <div className="flex flex-col gap-4">
      <h2 className="text-subtitle font-medium text-[var(--color-text-primary)]">
        {commonT("actions")}
      </h2>

      {error && (
        <div className="px-4 py-2 text-body rounded-md border border-[var(--color-danger-border)] bg-[var(--color-danger-bg)] text-[var(--color-danger)]">
          {error.message}
        </div>
      )}

      <Button
        variant="danger"
        className="w-fit"
        onClick={() => setConfirmOpen(true)}
      >
        {t("cancelRelease")}
      </Button>

      <Dialog
        isOpen={confirmOpen}
        onClose={() => setConfirmOpen(false)}
        title={t("cancelRelease")}
      >
        <div className="flex flex-col gap-4">
          <p className="text-body text-[var(--color-text-secondary)]">
            {t("cancelConfirm", { releaseName: release.name })}
          </p>
          <div className="flex gap-3">
            <Button
              variant="danger"
              isLoading={isPending}
              onClick={() =>
                cancel(undefined, { onSuccess: () => setConfirmOpen(false) })
              }
            >
              {t("confirmCancellation")}
            </Button>
            <Button variant="secondary" onClick={() => setConfirmOpen(false)}>
              {t("keepRelease")}
            </Button>
          </div>
        </div>
      </Dialog>
    </div>
  );
};

"use client";

import { useState } from "react";
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
        Actions
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
        Cancel release
      </Button>

      <Dialog
        isOpen={confirmOpen}
        onClose={() => setConfirmOpen(false)}
        title="Cancel release"
      >
        <div className="flex flex-col gap-4">
          <p className="text-body text-[var(--color-text-secondary)]">
            Are you sure you want to cancel <strong>{release.name}</strong>?
            This cannot be undone.
          </p>
          <div className="flex gap-3">
            <Button
              variant="danger"
              isLoading={isPending}
              onClick={() =>
                cancel(undefined, { onSuccess: () => setConfirmOpen(false) })
              }
            >
              Confirm cancellation
            </Button>
            <Button variant="secondary" onClick={() => setConfirmOpen(false)}>
              Keep release
            </Button>
          </div>
        </div>
      </Dialog>
    </div>
  );
};

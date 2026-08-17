"use client";

import { useState } from "react";
import { useGenerateInviteCode, useTransferManager } from "../hooks";
import { TeamMemberResponse } from "../types";
import { Button } from "@/shared/ui/Button";
import { Dialog } from "@/shared/ui/Dialog";

interface ManagerActionsProps {
  teamId: string;
  members: TeamMemberResponse[];
  currentUserId: string;
}

export const ManagerActions = ({
  teamId,
  members,
  currentUserId,
}: ManagerActionsProps) => {
  const [inviteCode, setInviteCode] = useState<string | null>(null);
  const [selectedUserId, setSelectedUserId] = useState("");
  const [transferTarget, setTransferTarget] =
    useState<TeamMemberResponse | null>(null);

  const { mutate: generateCode, isPending: generatingCode } =
    useGenerateInviteCode(teamId);
  const {
    mutate: transfer,
    isPending: transferring,
    error: transferError,
  } = useTransferManager(teamId);

  const otherMembers = members.filter((m) => m.user_id !== currentUserId);

  const handleGenerateCode = () => {
    generateCode(undefined, {
      onSuccess: (data) => setInviteCode(data.invitation_code),
    });
  };

  const handleOpenTransfer = () => {
    const target = otherMembers.find((m) => m.user_id === selectedUserId);
    if (target) setTransferTarget(target);
  };

  const handleConfirmTransfer = () => {
    if (!transferTarget) return;
    transfer(
      { user_id: transferTarget.user_id },
      {
        onSuccess: () => {
          setTransferTarget(null);
          setSelectedUserId("");
        },
      },
    );
  };

  return (
    <div className="flex flex-col gap-6">
      <h2 className="text-subtitle font-medium text-[var(--color-text-primary)]">
        Manager actions
      </h2>

      <div className="flex flex-col gap-2">
        <p className="text-body text-[var(--color-text-secondary)]">
          Invitation code
        </p>
        {inviteCode ? (
          <div className="flex items-center gap-3">
            <code className="px-3 py-2 rounded-md border border-[var(--color-border)] bg-[var(--color-bg-tertiary)] text-body text-[var(--color-accent)] font-mono">
              {inviteCode}
            </code>
            <Button
              variant="secondary"
              onClick={() => navigator.clipboard.writeText(inviteCode)}
            >
              Copy
            </Button>
          </div>
        ) : (
          <Button
            variant="secondary"
            isLoading={generatingCode}
            onClick={handleGenerateCode}
            className="w-fit"
          >
            Generate invite code
          </Button>
        )}
      </div>

      <div className="flex flex-col gap-2">
        <label
          htmlFor="transfer-target"
          className="text-body text-[var(--color-text-secondary)]"
        >
          Transfer manager role
        </label>
        <div className="flex gap-3">
          <select
            id="transfer-target"
            value={selectedUserId}
            onChange={(e) => setSelectedUserId(e.target.value)}
            className="px-3 py-2 text-body rounded-md border border-[var(--color-border)] bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] focus:outline-none focus:border-[var(--color-accent)]"
          >
            <option value="">Select a member</option>
            {otherMembers.map((m) => (
              <option key={m.id} value={m.user_id}>
                {m.username ?? m.user_id}
              </option>
            ))}
          </select>
          <Button
            variant="danger"
            onClick={handleOpenTransfer}
            disabled={!selectedUserId}
          >
            Transfer manager
          </Button>
        </div>
      </div>

      <Dialog
        isOpen={!!transferTarget}
        onClose={() => setTransferTarget(null)}
        title="Transfer manager role"
      >
        <div className="flex flex-col gap-4">
          {transferError && (
            <div className="px-4 py-2 text-body rounded-md border border-[var(--color-danger-border)] bg-[var(--color-danger-bg)] text-[var(--color-danger)]">
              {transferError.message}
            </div>
          )}
          <p className="text-body text-[var(--color-text-primary)]">
            Transfer the Manager role to{" "}
            <strong>
              {transferTarget?.username ?? transferTarget?.user_id}
            </strong>
            ? You will become a Responder.
          </p>
          <div className="flex gap-3">
            <Button
              variant="danger"
              isLoading={transferring}
              onClick={handleConfirmTransfer}
            >
              Transfer to {transferTarget?.username ?? transferTarget?.user_id}
            </Button>
            <Button variant="secondary" onClick={() => setTransferTarget(null)}>
              Cancel
            </Button>
          </div>
        </div>
      </Dialog>
    </div>
  );
};

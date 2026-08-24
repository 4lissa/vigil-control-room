"use client";

import { useState } from "react";
import { useTranslations } from "next-intl";
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
  const t = useTranslations("teams");
  const commonT = useTranslations("common");
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
        {t("managerActions")}
      </h2>

      <div className="flex flex-col gap-2">
        <p className="text-body text-[var(--color-text-secondary)]">
          {t("invitationCode")}
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
              {t("copy")}
            </Button>
          </div>
        ) : (
          <Button
            variant="secondary"
            isLoading={generatingCode}
            onClick={handleGenerateCode}
            className="w-fit"
          >
            {t("generateInviteCode")}
          </Button>
        )}
      </div>

      <div className="flex flex-col gap-2">
        <label
          htmlFor="transfer-target"
          className="text-body text-[var(--color-text-secondary)]"
        >
          {t("transferManagerRole")}
        </label>
        <div className="flex gap-3">
          <select
            id="transfer-target"
            value={selectedUserId}
            onChange={(e) => setSelectedUserId(e.target.value)}
            className="px-3 py-2 text-body rounded-md border border-[var(--color-border)] bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] focus:outline-none focus:border-[var(--color-accent)]"
          >
            <option value="">{t("selectMember")}</option>
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
            {t("transferManager")}
          </Button>
        </div>
      </div>

      <Dialog
        isOpen={!!transferTarget}
        onClose={() => setTransferTarget(null)}
        title={t("transferManagerRole")}
      >
        <div className="flex flex-col gap-4">
          {transferError && (
            <div className="px-4 py-2 text-body rounded-md border border-[var(--color-danger-border)] bg-[var(--color-danger-bg)] text-[var(--color-danger)]">
              {transferError.message}
            </div>
          )}
          <p className="text-body text-[var(--color-text-primary)]">
            {t("transferConfirm", {
              member: transferTarget?.username ?? transferTarget?.user_id ?? "",
            })}
          </p>
          <div className="flex gap-3">
            <Button
              variant="danger"
              isLoading={transferring}
              onClick={handleConfirmTransfer}
            >
              {t("transferButton", {
                member:
                  transferTarget?.username ?? transferTarget?.user_id ?? "",
              })}
            </Button>
            <Button variant="secondary" onClick={() => setTransferTarget(null)}>
              {commonT("cancel")}
            </Button>
          </div>
        </div>
      </Dialog>
    </div>
  );
};

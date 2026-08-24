"use client";

import { useTranslations } from "next-intl";
import { useMe } from "@/features/auth";
import { ProfileForm } from "@/features/auth/components/ProfileForm";
import { ConnectHttpTokenForm } from "@/features/auth/components/ConnectHttpTokenForm";

export default function ProfilePage() {
  const t = useTranslations("auth");
  const commonT = useTranslations("common");
  const { data: user, isLoading } = useMe();

  if (isLoading) {
    return (
      <p className="text-body text-[var(--color-text-muted)]">
        {commonT("loading")}
      </p>
    );
  }

  if (!user) {
    return (
      <p className="text-body text-[var(--color-text-muted)]">
        {t("userNotFound")}
      </p>
    );
  }

  return (
    <div className="max-w-md flex flex-col gap-6">
      <h1 className="text-title font-medium text-[var(--color-text-primary)]">
        {t("profile")}
      </h1>

      <ProfileForm user={user} />

      <div className="border-t border-[var(--color-border)] pt-4">
        <p className="text-body text-[var(--color-text-secondary)]">
          {t("email")}
        </p>
        <p className="text-body text-[var(--color-text-muted)] mt-1">
          {user.email}
        </p>
      </div>

      <div className="border-t border-[var(--color-border)] pt-4 flex flex-col gap-4">
        <p className="text-body text-[var(--color-text-secondary)]">
          {t("connectedServices")}
        </p>
        <ConnectHttpTokenForm />
      </div>
    </div>
  );
}

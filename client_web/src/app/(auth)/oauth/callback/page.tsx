"use client";

import { Suspense } from "react";
import { useTranslations } from "next-intl";
import { useHandleOAuthCallback } from "@/features/auth/hooks";

const OAuthCallback = () => {
  const t = useTranslations("auth");
  useHandleOAuthCallback();
  return (
    <p className="text-body text-[var(--color-text-muted)]">{t("signingIn")}</p>
  );
};

export default function OAuthCallbackPage() {
  return (
    <main className="flex-1 flex items-center justify-center">
      <Suspense fallback={null}>
        <OAuthCallback />
      </Suspense>
    </main>
  );
}

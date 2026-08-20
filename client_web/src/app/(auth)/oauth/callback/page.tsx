"use client";

import { Suspense } from "react";
import { useHandleOAuthCallback } from "@/features/auth/hooks";

const OAuthCallback = () => {
  useHandleOAuthCallback();
  return (
    <p className="text-body text-[var(--color-text-muted)]">Signing you in…</p>
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

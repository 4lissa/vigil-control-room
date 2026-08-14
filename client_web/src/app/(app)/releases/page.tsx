"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { useDefaultTeamId } from "@/features/teams";

export default function ReleasesPage() {
  const router = useRouter();
  const defaultTeamId = useDefaultTeamId();

  useEffect(() => {
    if (defaultTeamId) router.replace(`/teams/${defaultTeamId}/releases`);
  }, [defaultTeamId, router]);

  if (defaultTeamId) return null;

  return (
    <div className="flex flex-col items-center justify-center h-full gap-4 text-center">
      <p className="text-subtitle text-[var(--color-text-secondary)]">
        You don’t belong to any team yet.
      </p>
      <p className="text-body text-[var(--color-text-muted)]">
        Create or join a team to get started.
      </p>
    </div>
  );
}

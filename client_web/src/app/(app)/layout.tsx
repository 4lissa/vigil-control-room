"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { useMe, useSyncLocale } from "@/features/auth/hooks";
import { useTeamRealtime } from "@/features/teams";
import { useReleasesGlobalRealtime } from "@/features/releases";
import { Navbar } from "@/shared/ui/Navbar";
import { WebSocketProvider } from "@/shared/providers/WebSocketProvider";
import { useNativeNotifications } from "@/shared/lib/notifications";

function AppContent({ children }: { children: React.ReactNode }) {
  const { data: user } = useMe();

  useTeamRealtime(user?.username);
  useReleasesGlobalRealtime();
  useSyncLocale();
  useNativeNotifications(user?.id);

  return (
    <div className="min-h-screen flex flex-col">
      <Navbar />
      <main className="flex-1 p-6">{children}</main>
    </div>
  );
}

export default function AppLayout({ children }: { children: React.ReactNode }) {
  const router = useRouter();
  const { isPending, isError } = useMe();

  useEffect(() => {
    if (isError) router.push("/login");
  }, [isError, router]);

  if (isPending || isError) return null;

  return (
    <WebSocketProvider>
      <AppContent>{children}</AppContent>
    </WebSocketProvider>
  );
}

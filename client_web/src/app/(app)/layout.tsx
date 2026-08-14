"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { useMe } from "@/features/auth/hooks";
import { Navbar } from "@/shared/ui/Navbar";
import { WebSocketProvider } from "@/shared/providers/WebSocketProvider";

export default function AppLayout({ children }: { children: React.ReactNode }) {
  const router = useRouter();
  const { isPending, isError } = useMe();

  useEffect(() => {
    if (isError) router.push("/login");
  }, [isError, router]);

  if (isPending || isError) return null;

  return (
    <WebSocketProvider>
      <div className="min-h-screen flex flex-col">
        <Navbar />
        <main className="flex-1 p-6">{children}</main>
      </div>
    </WebSocketProvider>
  );
}

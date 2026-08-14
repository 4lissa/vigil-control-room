"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { useMe } from "@/features/auth/hooks";

export default function AuthLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const router = useRouter();
  const { isSuccess } = useMe();

  useEffect(() => {
    if (isSuccess) router.push("/incidents");
  }, [isSuccess, router]);

  return (
    <div className="min-h-screen flex flex-col">
      <div className="p-6">
        <span className="text-subtitle font-bold text-[var(--color-accent)]">
          VIGIL
        </span>
      </div>
      {children}
    </div>
  );
}

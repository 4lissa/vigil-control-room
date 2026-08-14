"use client";

import { useMe } from "@/features/auth";
import { ProfileForm } from "@/features/auth/components/ProfileForm";

export default function ProfilePage() {
  const { data: user, isLoading } = useMe();

  if (isLoading) {
    return (
      <p className="text-body text-[var(--color-text-muted)]">Loading...</p>
    );
  }

  if (!user) {
    return (
      <p className="text-body text-[var(--color-text-muted)]">
        User not found.
      </p>
    );
  }

  return (
    <div className="max-w-md flex flex-col gap-6">
      <h1 className="text-title font-medium text-[var(--color-text-primary)]">
        Profile
      </h1>

      <ProfileForm user={user} />

      <div className="border-t border-[var(--color-border)] pt-4">
        <p className="text-body text-[var(--color-text-secondary)]">Email</p>
        <p className="text-body text-[var(--color-text-muted)] mt-1">
          {user.email}
        </p>
      </div>
    </div>
  );
}

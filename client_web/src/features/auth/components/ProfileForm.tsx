"use client";

import { useState } from "react";
import { useUpdateProfile } from "../hooks";
import { UserResponse } from "../types";
import { Button } from "@/shared/ui/Button";
import { Input } from "@/shared/ui/Input";

interface ProfileFormProps {
  user: UserResponse;
}

export const ProfileForm = ({ user }: ProfileFormProps) => {
  const {
    mutate: updateProfile,
    isPending,
    error,
    isSuccess,
  } = useUpdateProfile();
  const [passwordError, setPasswordError] = useState("");

  const handleSubmit = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    setPasswordError("");

    const form = e.currentTarget;
    const username = (form.elements.namedItem("username") as HTMLInputElement)
      .value;
    const old_password =
      (form.elements.namedItem("old_password") as HTMLInputElement | null)
        ?.value ?? "";
    const new_password =
      (form.elements.namedItem("new_password") as HTMLInputElement | null)
        ?.value ?? "";

    if (new_password && !old_password) {
      setPasswordError("Current password is required to set a new one");
      return;
    }

    if (old_password && !new_password) {
      setPasswordError("New password is required");
      return;
    }

    updateProfile({
      username: username || undefined,
      old_password: old_password || undefined,
      new_password: new_password || undefined,
    });
  };

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-4">
      {error && (
        <div className="px-4 py-2 text-body rounded-md border border-[var(--color-danger-border)] bg-[var(--color-danger-bg)] text-[var(--color-danger)]">
          {error.message}
        </div>
      )}
      {isSuccess && (
        <div className="px-4 py-2 text-body rounded-md border border-[var(--color-success-border)] bg-[var(--color-success-bg)] text-[var(--color-success)]">
          Profile updated
        </div>
      )}

      <Input name="username" label="Username" defaultValue={user.username} />

      {user.has_password && (
        <div className="border-t border-[var(--color-border)] pt-4 flex flex-col gap-4">
          <p className="text-body text-[var(--color-text-muted)]">
            Change password
          </p>
          <Input name="old_password" type="password" label="Current password" />
          <Input
            name="new_password"
            type="password"
            label="New password"
            error={passwordError}
          />
        </div>
      )}

      <Button
        type="submit"
        variant="primary"
        isLoading={isPending}
        className="w-full mt-2"
      >
        Save changes
      </Button>
    </form>
  );
};

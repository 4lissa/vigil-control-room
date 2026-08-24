"use client";

import { useState } from "react";
import { useTranslations } from "next-intl";
import { useUpdateProfile } from "../hooks";
import { UserResponse } from "../types";
import { Button } from "@/shared/ui/Button";
import { Input } from "@/shared/ui/Input";

interface ProfileFormProps {
  user: UserResponse;
}

export const ProfileForm = ({ user }: ProfileFormProps) => {
  const t = useTranslations("auth");
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
    const language = (form.elements.namedItem("language") as HTMLSelectElement)
      .value;

    if (new_password && !old_password) {
      setPasswordError(t("currentPasswordRequired"));
      return;
    }

    if (old_password && !new_password) {
      setPasswordError(t("newPasswordRequired"));
      return;
    }

    updateProfile({
      username: username || undefined,
      old_password: old_password || undefined,
      new_password: new_password || undefined,
      language,
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
          {t("profileUpdated")}
        </div>
      )}

      <Input
        name="username"
        label={t("username")}
        defaultValue={user.username}
      />

      <div className="flex flex-col gap-1">
        <label
          htmlFor="language"
          className="text-body font-medium text-[var(--color-text-secondary)]"
        >
          {t("language")}
        </label>
        <select
          id="language"
          name="language"
          defaultValue={user.language}
          className="px-3 py-2 text-body rounded-md border border-[var(--color-border)] bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] focus:outline-none focus:border-[var(--color-accent)]"
        >
          <option value="en">English</option>
          <option value="fr">Français</option>
        </select>
      </div>

      {user.has_password && (
        <div className="border-t border-[var(--color-border)] pt-4 flex flex-col gap-4">
          <p className="text-body text-[var(--color-text-muted)]">
            {t("changePassword")}
          </p>
          <Input
            name="old_password"
            type="password"
            label={t("currentPassword")}
          />
          <Input
            name="new_password"
            type="password"
            label={t("newPassword")}
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
        {t("saveChanges")}
      </Button>
    </form>
  );
};

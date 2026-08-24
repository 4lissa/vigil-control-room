"use client";

import { useTranslations } from "next-intl";
import { useRouter } from "next/navigation";
import { useLogin } from "../hooks";
import { Button } from "@/shared/ui/Button";
import { Input } from "@/shared/ui/Input";

export const LoginForm = () => {
  const t = useTranslations("auth");
  const router = useRouter();
  const { mutate: login, isPending, error } = useLogin();

  const handleSubmit = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();

    const form = e.currentTarget;
    const email = (form.elements.namedItem("email") as HTMLInputElement).value;
    const password = (form.elements.namedItem("password") as HTMLInputElement)
      .value;

    login(
      { email, password },
      {
        onSuccess: () => router.push("/incidents"),
      },
    );
  };

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-4">
      {error && (
        <div className="px-4 py-2 text-body rounded-md border border-[var(--color-danger-border)] bg-[var(--color-danger-bg)] text-[var(--color-danger)]">
          {error.message}
        </div>
      )}
      <Input name="email" type="email" label={t("email")} required />
      <Input name="password" type="password" label={t("password")} required />
      <Button
        type="submit"
        variant="primary"
        isLoading={isPending}
        className="w-full mt-2"
      >
        {t("signIn")}
      </Button>
    </form>
  );
};

"use client";

import { useRouter } from "next/navigation";
import { useRegister } from "../hooks";
import { Button } from "@/shared/ui/Button";
import { Input } from "@/shared/ui/Input";

export const RegisterForm = () => {
  const router = useRouter();
  const { mutate: register, isPending, error } = useRegister();

  const handleSubmit = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();

    const form = e.currentTarget;
    const username = (form.elements.namedItem("username") as HTMLInputElement)
      .value;
    const email = (form.elements.namedItem("email") as HTMLInputElement).value;
    const password = (form.elements.namedItem("password") as HTMLInputElement)
      .value;

    register(
      { username, email, password },
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
      <Input name="username" type="text" label="Username" required />
      <Input name="email" type="email" label="Email" required />
      <Input name="password" type="password" label="Password" required />
      <Button
        type="submit"
        variant="primary"
        isLoading={isPending}
        className="w-full mt-2"
      >
        Create account
      </Button>
    </form>
  );
};

"use client";

import { useTranslations } from "next-intl";
import { useRouter } from "next/navigation";
import { useJoinTeam } from "../hooks";
import { Button } from "@/shared/ui/Button";
import { Input } from "@/shared/ui/Input";

interface JoinTeamFormProps {
  onSuccess: () => void;
}

export const JoinTeamForm = ({ onSuccess }: JoinTeamFormProps) => {
  const t = useTranslations("teams");
  const router = useRouter();
  const { mutate: joinTeam, isPending, error } = useJoinTeam();

  const handleSubmit = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    const form = e.currentTarget;
    const code = (form.elements.namedItem("code") as HTMLInputElement).value;

    joinTeam(
      { code },
      {
        onSuccess: (data) => {
          onSuccess();
          router.push(`/teams/${data.team.id}/incidents`);
        },
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
      <Input name="code" label={t("invitationCode")} required />
      <Button
        type="submit"
        variant="primary"
        isLoading={isPending}
        className="w-full mt-2"
      >
        {t("joinTeam")}
      </Button>
    </form>
  );
};

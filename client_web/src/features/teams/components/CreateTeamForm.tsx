"use client";

import { useRouter } from "next/navigation";
import { useCreateTeam } from "../hooks";
import { Button } from "@/shared/ui/Button";
import { Input } from "@/shared/ui/Input";

interface CreateTeamFormProps {
  onSuccess: () => void;
}

export const CreateTeamForm = ({ onSuccess }: CreateTeamFormProps) => {
  const router = useRouter();
  const { mutate: createTeam, isPending, error } = useCreateTeam();

  const handleSubmit = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    const form = e.currentTarget;
    const name = (form.elements.namedItem("name") as HTMLInputElement).value;

    createTeam(
      { name },
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
      <Input name="name" label="Team name" required />
      <Button
        type="submit"
        variant="primary"
        isLoading={isPending}
        className="w-full mt-2"
      >
        Create team
      </Button>
    </form>
  );
};

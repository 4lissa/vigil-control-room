import { Button } from "@/shared/ui/Button";

interface RulesLinkProps {
  teamId: string;
}

export const RulesLink = ({ teamId }: RulesLinkProps) => (
  <Button variant="secondary" href={`/teams/${teamId}/rules`} className="w-fit">
    Automation rules
  </Button>
);

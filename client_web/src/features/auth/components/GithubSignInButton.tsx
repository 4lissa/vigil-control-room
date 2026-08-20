import { githubSignInUrl } from "../api";
import { Button } from "@/shared/ui/Button";

export const GithubSignInButton = () => (
  <Button href={githubSignInUrl} variant="secondary" className="w-full">
    Continue with GitHub
  </Button>
);

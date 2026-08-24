"use client";

import { useTranslations } from "next-intl";
import { githubSignInUrl } from "../api";
import { Button } from "@/shared/ui/Button";

export const GithubSignInButton = () => {
  const t = useTranslations("auth");

  return (
    <Button href={githubSignInUrl} variant="secondary" className="w-full">
      {t("continueWithGithub")}
    </Button>
  );
};

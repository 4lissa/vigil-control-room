"use client";

import { useTranslations } from "next-intl";
import { Button } from "@/shared/ui/Button";

export const DownloadDesktopButton = () => {
  const t = useTranslations("auth");

  return (
    <Button href="/client.dmg" variant="secondary" className="w-full">
      {t("downloadDesktopApp")}
    </Button>
  );
};

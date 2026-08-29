import { getTranslations } from "next-intl/server";
import { GithubSignInButton } from "@/features/auth/components/GithubSignInButton";
import { RegisterForm } from "@/features/auth/components/RegisterForm";
import { DownloadDesktopButton } from "@/features/auth/components/DownloadDesktopButton";

export default async function RegisterPage() {
  const t = await getTranslations("auth");

  return (
    <main className="flex-1 flex flex-col items-center justify-center gap-4">
      <div className="w-full max-w-sm p-8 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)]">
        <h1 className="text-title font-medium text-[var(--color-text-primary)] mb-6">
          {t("createAccount")}
        </h1>
        <RegisterForm />
        <div className="my-4 text-caption text-[var(--color-text-muted)] text-center">
          {t("or")}
        </div>
        <GithubSignInButton />
        <p className="mt-4 text-body text-[var(--color-text-muted)] text-center">
          {t("alreadyHaveAccount")}{" "}
          <a
            href="/login"
            className="text-[var(--color-accent)] hover:underline"
          >
            {t("signIn")}
          </a>
        </p>
      </div>
      <div className="w-full max-w-sm">
        <DownloadDesktopButton />
      </div>
    </main>
  );
}

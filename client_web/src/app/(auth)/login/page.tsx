import { GithubSignInButton } from "@/features/auth/components/GithubSignInButton";
import { LoginForm } from "@/features/auth/components/LoginForm";

export default function LoginPage() {
  return (
    <main className="flex-1 flex items-center justify-center">
      <div className="w-full max-w-md p-8 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)]">
        <h1 className="text-title font-medium text-[var(--color-text-primary)] mb-6">
          Sign in
        </h1>
        <LoginForm />
        <div className="my-4 text-caption text-[var(--color-text-muted)] text-center">
          or
        </div>
        <GithubSignInButton />
        <p className="mt-4 text-body text-[var(--color-text-muted)] text-center">
          Don’t have an account?{" "}
          <a
            href="/register"
            className="text-[var(--color-accent)] hover:underline"
          >
            Create account
          </a>
        </p>
      </div>
    </main>
  );
}

import { GithubSignInButton } from "@/features/auth/components/GithubSignInButton";
import { RegisterForm } from "@/features/auth/components/RegisterForm";

export default function RegisterPage() {
  return (
    <main className="flex-1 flex items-center justify-center">
      <div className="w-full max-w-sm p-8 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)]">
        <h1 className="text-title font-medium text-[var(--color-text-primary)] mb-6">
          Create account
        </h1>
        <RegisterForm />
        <div className="my-4 text-caption text-[var(--color-text-muted)] text-center">
          or
        </div>
        <GithubSignInButton />
        <p className="mt-4 text-body text-[var(--color-text-muted)] text-center">
          Already have an account?{" "}
          <a
            href="/login"
            className="text-[var(--color-accent)] hover:underline"
          >
            Sign in
          </a>
        </p>
      </div>
    </main>
  );
}

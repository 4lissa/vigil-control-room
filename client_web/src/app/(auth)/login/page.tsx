import { LoginForm } from "@/features/auth/components/LoginForm";

export default function LoginPage() {
  return (
    <main className="min-h-screen flex items-center justify-center bg-[var(--color-bg-primary)]">
      <div className="w-full max-w-md p-8 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)]">
        <h1 className="text-title font-medium text-[var(--color-text-primary)] mb-6">
          Sign in
        </h1>
        <LoginForm />
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

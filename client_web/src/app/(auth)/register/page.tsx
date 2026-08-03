import { RegisterForm } from "@/features/auth/components/RegisterForm";

export default function RegisterPage() {
  return (
    <main className="min-h-screen flex items-center justify-center bg-[var(--color-bg-primary)]">
      <div className="w-full max-w-sm p-8 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)]">
        <h1 className="text-title font-medium text-[var(--color-text-primary)] mb-6">
          Créer un compte
        </h1>
        <RegisterForm />
        <p className="mt-4 text-body text-[var(--color-text-muted)] text-center">
          Déjà un compte ?{" "}
          <a
            href="/login"
            className="text-[var(--color-accent)] hover:underline"
          >
            Se connecter
          </a>
        </p>
      </div>
    </main>
  );
}

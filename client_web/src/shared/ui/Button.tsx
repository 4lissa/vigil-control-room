import { ButtonHTMLAttributes } from "react";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "secondary" | "danger";
  isLoading?: boolean;
  href?: string;
}

export const Button = ({
  variant = "secondary",
  isLoading = false,
  disabled,
  children,
  className = "",
  href,
  ...props
}: ButtonProps) => {
  const base = `inline-flex items-center justify-center gap-2 px-4 py-2 text-body font-medium rounded-md border transition-colors focus-visible:outline-none disabled:opacity-50 ${
    isLoading ? "disabled:cursor-wait" : "disabled:cursor-not-allowed"
  }`;

  const variants = {
    primary:
      "bg-[var(--color-accent)] text-[var(--color-bg-primary)] border-[var(--color-accent)] hover:opacity-90",
    secondary:
      "bg-transparent text-[var(--color-text-primary)] border-[var(--color-border)] hover:border-[var(--color-border-strong)] hover:bg-[var(--color-bg-tertiary)]",
    danger:
      "bg-transparent text-[var(--color-danger)] border-[var(--color-danger-border)] hover:bg-[var(--color-danger-bg)]",
  };

  const classes = `${base} ${variants[variant]} ${className}`;

  if (href) {
    return (
      <a href={href} className={classes}>
        {children}
      </a>
    );
  }

  return (
    <button disabled={disabled || isLoading} className={classes} {...props}>
      {isLoading ? (
        <span className="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
      ) : null}
      {children}
    </button>
  );
};

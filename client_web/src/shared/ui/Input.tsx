import { InputHTMLAttributes } from "react";

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label: string;
  error?: string;
}

export const Input = ({
  label,
  error,
  id,
  className = "",
  ...props
}: InputProps) => {
  const inputId = id ?? label.toLowerCase().replace(/\s+/g, "-");

  return (
    <div className="flex flex-col gap-1">
      <label
        htmlFor={inputId}
        className="text-body font-medium text-[var(--color-text-secondary)]"
      >
        {label}
      </label>
      <input
        id={inputId}
        className={`
          w-full px-3 py-2 text-body rounded-md border
          bg-[var(--color-bg-tertiary)]
          text-[var(--color-text-primary)]
          border-[var(--color-border)]
          placeholder:text-[var(--color-text-muted)]
          hover:border-[var(--color-border-strong)]
          focus:outline-none focus:border-[var(--color-accent)]
          disabled:opacity-50 disabled:cursor-not-allowed
          ${error ? "border-[var(--color-danger)]" : ""}
          ${className}
        `}
        {...props}
      />
      {error ? (
        <span className="text-caption text-[var(--color-danger)]">{error}</span>
      ) : null}
    </div>
  );
};

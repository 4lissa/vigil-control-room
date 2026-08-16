import type { LucideIcon } from "lucide-react";

export type BadgeColor =
  "neutral" | "accent" | "medium" | "warning" | "danger" | "success";

interface BadgeProps {
  label: string;
  icon: LucideIcon;
  color: BadgeColor;
}

const colorClasses: Record<BadgeColor, string> = {
  neutral:
    "bg-[var(--color-bg-tertiary)] border-[var(--color-border)] text-[var(--color-text-secondary)]",
  accent:
    "bg-[var(--color-accent-bg)] border-[var(--color-accent-border)] text-[var(--color-accent)]",
  medium:
    "bg-[var(--color-medium-bg)] border-[var(--color-medium-border)] text-[var(--color-medium)]",
  warning:
    "bg-[var(--color-warning-bg)] border-[var(--color-warning-border)] text-[var(--color-warning)]",
  danger:
    "bg-[var(--color-danger-bg)] border-[var(--color-danger-border)] text-[var(--color-danger)]",
  success:
    "bg-[var(--color-success-bg)] border-[var(--color-success-border)] text-[var(--color-success)]",
};

export const Badge = ({ label, icon: Icon, color }: BadgeProps) => (
  <span
    className={`inline-flex items-center gap-1 px-2 py-1 rounded-md border text-caption font-medium ${colorClasses[color]}`}
  >
    <Icon size={12} />
    {label}
  </span>
);

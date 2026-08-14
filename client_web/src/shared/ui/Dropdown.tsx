"use client";

import { useEffect, useRef, useState, ReactNode } from "react";

interface DropdownProps {
  trigger: (props: { onClick: () => void; open: boolean }) => ReactNode;
  children: (close: () => void) => ReactNode;
  align?: "left" | "right";
  panelClassName?: string;
}

export const Dropdown = ({
  trigger,
  children,
  align = "left",
  panelClassName = "w-56",
}: DropdownProps) => {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handleClickOutside = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [open]);

  const toggle = () => setOpen((v) => !v);
  const close = () => setOpen(false);
  const alignClass = align === "right" ? "right-0" : "left-0";

  return (
    <div ref={ref} className="relative">
      {trigger({ onClick: toggle, open })}

      {open && (
        <div
          className={`absolute top-full ${alignClass} mt-1 rounded-md border border-[var(--color-border)] bg-[var(--color-bg-secondary)] shadow-lg z-50 ${panelClassName}`}
        >
          {children(close)}
        </div>
      )}
    </div>
  );
};

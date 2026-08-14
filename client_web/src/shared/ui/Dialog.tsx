import { useEffect, useRef } from "react";
import { Button } from "./Button";

interface DialogProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  children: React.ReactNode;
}

export const Dialog = ({ isOpen, onClose, title, children }: DialogProps) => {
  const dialogRef = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog || !isOpen) return;

    dialog.showModal();
    return () => dialog.close();
  }, [isOpen]);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;
    const handleCancel = (e: Event) => {
      e.preventDefault();
      onClose();
    };
    dialog.addEventListener("cancel", handleCancel);
    return () => dialog.removeEventListener("cancel", handleCancel);
  }, [onClose]);

  if (!isOpen) return null;

  const handleBackdropClick = (e: React.MouseEvent<HTMLDialogElement>) => {
    if (e.target === dialogRef.current) onClose();
  };

  return (
    <dialog
      ref={dialogRef}
      onClick={handleBackdropClick}
      className="m-auto w-full max-w-md rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-6 backdrop:bg-black/50 outline-none"
    >
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-subtitle font-medium text-[var(--color-text-primary)]">
          {title}
        </h2>
        <Button
          variant="secondary"
          onClick={onClose}
          className="border-none px-2 py-1 text-[var(--color-text-muted)]"
          aria-label="Close dialog"
        >
          ✕
        </Button>
      </div>
      {children}
    </dialog>
  );
};

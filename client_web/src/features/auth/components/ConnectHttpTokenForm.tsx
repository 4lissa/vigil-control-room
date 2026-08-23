"use client";

import {
  useConnectService,
  useConnectedServiceStatus,
  useDisconnectService,
} from "../hooks";
import { Badge } from "@/shared/ui/Badge";
import { Button } from "@/shared/ui/Button";
import { Input } from "@/shared/ui/Input";
import { CheckCircle2 } from "lucide-react";

export const ConnectHttpTokenForm = () => {
  const { data: status, isLoading: statusLoading } =
    useConnectedServiceStatus("http");
  const {
    mutate: connectService,
    isPending: connecting,
    error,
  } = useConnectService();
  const { mutate: disconnectService, isPending: disconnecting } =
    useDisconnectService();

  const handleSubmit = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    const form = e.currentTarget;
    const token = (form.elements.namedItem("token") as HTMLInputElement).value;

    connectService({ service: "http", body: { token } });
    form.reset();
  };

  if (statusLoading) {
    return (
      <p className="text-body text-[var(--color-text-muted)]">Loading...</p>
    );
  }

  if (status?.connected) {
    return (
      <div className="flex items-center gap-3">
        <Badge label="HTTP connected" icon={CheckCircle2} color="success" />
        <Button
          type="button"
          variant="secondary"
          isLoading={disconnecting}
          onClick={() => disconnectService("http")}
        >
          Disconnect
        </Button>
      </div>
    );
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-4">
      {error && (
        <div className="px-4 py-2 text-body rounded-md border border-[var(--color-danger-border)] bg-[var(--color-danger-bg)] text-[var(--color-danger)]">
          {error.message}
        </div>
      )}
      <Input name="token" type="password" label="HTTP personal token" />
      <Button
        type="submit"
        variant="secondary"
        isLoading={connecting}
        className="w-fit"
      >
        Connect
      </Button>
    </form>
  );
};

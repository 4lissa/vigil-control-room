"use client";

import { useState } from "react";
import Link from "next/link";
import { useParams, usePathname, useRouter } from "next/navigation";
import { Bell, Mail, Settings, User, LogOut, Check, Plus } from "lucide-react";
import { useTeams, useTeamMembers, useDefaultTeamId } from "@/features/teams";
import { useLogout, useMe } from "@/features/auth";
import { Dialog } from "./Dialog";
import { Dropdown } from "./Dropdown";
import { CreateTeamForm } from "@/features/teams/components/CreateTeamForm";
import { JoinTeamForm } from "@/features/teams/components/JoinTeamForm";

export const Navbar = () => {
  const router = useRouter();
  const pathname = usePathname();
  const params = useParams<{ teamId?: string }>();
  const { data: user } = useMe();
  const { data: teams } = useTeams();
  const { mutate: logout } = useLogout();

  const defaultTeamId = useDefaultTeamId();
  const currentTeamId = params.teamId ?? defaultTeamId;
  const activeTeam = teams?.find((t) => t.id === currentTeamId);
  const { data: activeTeamMembers } = useTeamMembers(activeTeam?.id ?? "");

  const incidentsHref = currentTeamId
    ? `/teams/${currentTeamId}/incidents`
    : "/incidents";
  const releasesHref = currentTeamId
    ? `/teams/${currentTeamId}/releases`
    : "/releases";
  const teamPageSuffix = pathname.endsWith("/releases")
    ? "/releases"
    : "/incidents";

  const activeRole = activeTeamMembers?.find(
    (member) => member.user_id === user?.id,
  )?.role;

  const [createOpen, setCreateOpen] = useState(false);
  const [joinOpen, setJoinOpen] = useState(false);

  const handleLogout = () => {
    logout(undefined, {
      onSuccess: () => router.push("/login"),
    });
  };

  const navLinkClass = (suffix: "/incidents" | "/releases") =>
    `text-body inline-flex items-center h-9 px-3 rounded-md transition-colors hover:bg-[var(--color-bg-tertiary)] ${
      pathname.endsWith(suffix)
        ? "font-medium text-[var(--color-text-primary)]"
        : "text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]"
    }`;

  const menuItemClass = (danger = false) =>
    `flex w-full items-center gap-2 text-left px-3 py-2 text-body ${
      danger
        ? "text-[var(--color-danger)] hover:bg-[var(--color-danger-bg)]"
        : "text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text-primary)]"
    }`;

  return (
    <>
      <nav className="h-14 border-b border-[var(--color-border)] bg-[var(--color-bg-secondary)] px-6 flex items-center gap-4">
        <span className="text-subtitle font-bold text-[var(--color-accent)]">
          VIGIL
        </span>

        <Dropdown
          trigger={({ onClick }) => (
            <button
              onClick={onClick}
              className="flex items-center h-9 px-3 rounded-md hover:bg-[var(--color-bg-tertiary)] transition-colors"
            >
              <span className="flex items-end gap-2">
                <span className="text-body font-medium leading-none text-[var(--color-text-primary)]">
                  {activeTeam ? activeTeam.name : "No team"}
                </span>
                {activeTeam && activeRole && (
                  <span className="text-caption leading-none text-[var(--color-text-muted)] capitalize">
                    - {activeRole}
                  </span>
                )}
              </span>
            </button>
          )}
        >
          {(close) => (
            <>
              {teams && teams.length > 0 && (
                <>
                  {teams.map((team) => (
                    <div
                      key={team.id}
                      className="flex items-center justify-between px-3 py-2 hover:bg-[var(--color-bg-tertiary)]"
                    >
                      <span
                        onClick={() => {
                          close();
                          router.push(`/teams/${team.id}${teamPageSuffix}`);
                        }}
                        className={`text-body cursor-pointer flex items-center gap-1 ${
                          activeTeam?.id === team.id
                            ? "text-[var(--color-accent)]"
                            : "text-[var(--color-text-primary)]"
                        }`}
                      >
                        {activeTeam?.id === team.id && <Check size={14} />}
                        {team.name}
                      </span>
                      <Link
                        href={`/teams/${team.id}`}
                        onClick={close}
                        className="text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)]"
                        aria-label={`${team.name} settings`}
                      >
                        <Settings size={14} />
                      </Link>
                    </div>
                  ))}
                  <div className="border-t border-[var(--color-border)]" />
                </>
              )}
              <button
                onClick={() => {
                  close();
                  setCreateOpen(true);
                }}
                className={menuItemClass()}
              >
                <Plus size={14} />
                Create a team
              </button>
              <button
                onClick={() => {
                  close();
                  setJoinOpen(true);
                }}
                className={menuItemClass()}
              >
                <Plus size={14} />
                Join a team
              </button>
            </>
          )}
        </Dropdown>

        <Link href={incidentsHref} className={navLinkClass("/incidents")}>
          Incidents
        </Link>
        <Link href={releasesHref} className={navLinkClass("/releases")}>
          Releases
        </Link>

        <div className="ml-auto flex items-center gap-4">
          <button
            className="inline-flex items-center h-9 px-3 rounded-md text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text-primary)] transition-colors"
            aria-label="Notifications"
          >
            <Bell size={18} />
          </button>
          <Link
            href="/messages"
            className="inline-flex items-center h-9 px-3 rounded-md text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text-primary)] transition-colors"
            aria-label="Messages"
          >
            <Mail size={18} />
          </Link>

          <Dropdown
            align="right"
            panelClassName="w-48"
            trigger={({ onClick }) => (
              <button
                onClick={onClick}
                className="flex items-center gap-2 h-9 px-3 rounded-md hover:bg-[var(--color-bg-tertiary)] transition-colors"
              >
                <div className="w-7 h-7 rounded-full bg-[var(--color-accent)] flex items-center justify-center text-caption font-bold text-[var(--color-bg-primary)]">
                  {user?.username?.[0]?.toUpperCase()}
                </div>
              </button>
            )}
          >
            {(close) => (
              <>
                <div className="px-3 py-2 border-b border-[var(--color-border)]">
                  <p className="text-body font-medium text-[var(--color-text-primary)]">
                    {user?.username}
                  </p>
                  <p className="text-caption text-[var(--color-text-muted)]">
                    {user?.email}
                  </p>
                </div>
                <Link
                  href="/profile"
                  onClick={close}
                  className={menuItemClass()}
                >
                  <User size={14} />
                  Profile
                </Link>
                <button onClick={handleLogout} className={menuItemClass(true)}>
                  <LogOut size={14} />
                  Logout
                </button>
              </>
            )}
          </Dropdown>
        </div>
      </nav>

      <Dialog
        isOpen={createOpen}
        onClose={() => setCreateOpen(false)}
        title="Create a team"
      >
        <CreateTeamForm onSuccess={() => setCreateOpen(false)} />
      </Dialog>
      <Dialog
        isOpen={joinOpen}
        onClose={() => setJoinOpen(false)}
        title="Join a team"
      >
        <JoinTeamForm onSuccess={() => setJoinOpen(false)} />
      </Dialog>
    </>
  );
};

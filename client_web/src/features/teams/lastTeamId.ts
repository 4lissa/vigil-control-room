import { TeamResponse } from "./types";

const LAST_TEAM_ID_KEY = "vigil-last-team";

export const getLastTeamId = (): string | null => {
  return localStorage.getItem(LAST_TEAM_ID_KEY);
};

export const setLastTeamId = (teamId: string): void => {
  localStorage.setItem(LAST_TEAM_ID_KEY, teamId);
};

export const getDefaultTeamId = (
  teams: TeamResponse[] | undefined,
): string | undefined => {
  const lastTeamId = getLastTeamId();
  if (lastTeamId && teams?.some((t) => t.id === lastTeamId)) {
    return lastTeamId;
  }
  return teams?.[0]?.id;
};

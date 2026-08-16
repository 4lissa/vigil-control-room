import { authHandlers } from "./handlers/auth";
import { teamsHandlers } from "./handlers/teams";
import { incidentsHandlers } from "./handlers/incidents";
import { releasesHandlers } from "./handlers/releases";

export const handlers = [
  ...authHandlers,
  ...teamsHandlers,
  ...incidentsHandlers,
  ...releasesHandlers,
];

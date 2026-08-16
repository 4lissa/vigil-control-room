import { authHandlers } from "./handlers/auth";
import { teamsHandlers } from "./handlers/teams";
import { incidentsHandlers } from "./handlers/incidents";

export const handlers = [
  ...authHandlers,
  ...teamsHandlers,
  ...incidentsHandlers,
];

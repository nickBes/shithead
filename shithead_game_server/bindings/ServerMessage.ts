import type { ClickedCardLocation } from "./ClickedCardLocation";
import type { ExposedLobbyInfo } from "./ExposedLobbyInfo";
import type { ExposedLobbyPlayerInfo } from "./ExposedLobbyPlayerInfo";

export type ServerMessage = { t: "clientId", id: number, } | { t: "lobbies", lobbies: Array<ExposedLobbyInfo>, } | { t: "joinLobby", id: number, } | { t: "error", err: string, } | { t: "playerJoinedLobby", player_info: ExposedLobbyPlayerInfo, } | { t: "playerLeftLobby", id: number, } | { t: "lobbyOwnerChanged", new_owner_id: number, } | { t: "ownerLeftLobby", new_owner_id: number, } | { t: "startGame" } | { t: "clickCard", location: ClickedCardLocation, };
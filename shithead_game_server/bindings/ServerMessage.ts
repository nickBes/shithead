import type { ExposedLobbyPlayerInfo } from "./ExposedLobbyPlayerInfo";
import type { ExposedLobbyInfo } from "./ExposedLobbyInfo";
import type { ClickedCardLocation } from "./ClickedCardLocation";

export type ServerMessage = { clientId: number } | { lobbies: Array<ExposedLobbyInfo> } | { joinLobby: number } | { error: string } | { playerJoinedLobby: ExposedLobbyPlayerInfo } | { playerLeftLobby: number } | { lobbyOwnerChanged: { new_owner_id: number, } } | { ownerLeftLobby: { new_owner_id: number, } } | { clickCard: ClickedCardLocation };
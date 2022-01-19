// AUTO-GENERATED by typescript-type-def

export default types;
export namespace types{
export type Usize=number;
export type ClientId=types.Usize;
export type LobbyId=types.Usize;

/**
 * The information about a lobby player that is exposed to the clients.
 */
export type ExposedLobbyPlayerInfo=
/**
 * The information about a lobby player that is exposed to the clients.
 */
{"id":types.ClientId;"username":string;};

/**
 * The information about a lobby that is exposed to the clients.
 */
export type ExposedLobbyInfo=
/**
 * The information about a lobby that is exposed to the clients.
 */
{"name":string;"id":types.LobbyId;"players":(types.ExposedLobbyPlayerInfo)[];"owner_id":types.ClientId;};
export type CardId=types.Usize;
export type U32=number;
export type ClickedCardLocation=("trash"|{"myCards":{"cardIndex":types.U32;};});
export type ServerMessage=({"clientId":types.ClientId;}|{"lobbies":(types.ExposedLobbyInfo)[];}|{"joinLobby":types.LobbyId;}|{"error":string;}|{"playerJoinedLobby":types.ExposedLobbyPlayerInfo;}|{"playerLeftLobby":types.ClientId;}|{"lobbyOwnerChanged":{"new_owner_id":types.ClientId;};}|{"ownerLeftLobby":{"new_owner_id":types.ClientId;};}|"startGame"|{"initialCards":{"cards_in_hand":(types.CardId)[];"three_up_cards":(types.CardId)[];};}|{"clickCard":types.ClickedCardLocation;});
export type ClientMessage=({"setUsername":string;}|"getLobbies"|{"joinLobby":types.LobbyId;}|{"createLobby":{"lobby_name":string;};}|"startGame");
}

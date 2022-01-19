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
export type LobbyOwnerChangedMessage={"newOwnerId":types.ClientId;};
export type OwnerLeftLobbyMessage={"newOwnerId":types.ClientId;};
export type U32=number;
export type ClickedCardLocation=({"location":"trash";}|({"location":"myCards";}&{"cardIndex":types.U32;}));
export type ServerMessage=({"t":"clientId";"c":types.ClientId;}|{"t":"lobbies";"c":(types.ExposedLobbyInfo)[];}|{"t":"joinLobby";"c":types.LobbyId;}|{"t":"error";"c":string;}|{"t":"playerJoinedLobby";"c":types.ExposedLobbyPlayerInfo;}|{"t":"playerLeftLobby";"c":types.ClientId;}|{"t":"lobbyOwnerChanged";"c":types.LobbyOwnerChangedMessage;}|{"t":"ownerLeftLobby";"c":types.OwnerLeftLobbyMessage;}|{"t":"startGame";}|{"t":"clickCard";"c":types.ClickedCardLocation;});
export type CreateLobbyRequest={"lobbyName":string;};
export type ClientMessage=({"t":"setUsername";"c":string;}|{"t":"getLobbies";}|{"t":"joinLobby";"c":types.LobbyId;}|{"t":"createLobby";"c":types.CreateLobbyRequest;}|{"t":"startGame";});
}

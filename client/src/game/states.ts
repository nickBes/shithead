import { ref } from "vue"
import type Socket from "./socket"

type GameStates = "inLobby" | "inGame" | "creatingLobby" | "inQuery"

// global states which can be accessed and modified from every component

export const states = {
    gameState: ref<GameStates | undefined>(),
    // game socket is not a ref as it shouldn't update other components
    // when created or re-assigned
    gameSocket: null as (Socket | null)
}
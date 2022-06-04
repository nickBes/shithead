import type Socket from "./socket"
import type types from "@/bindings/bindings"
import { ref } from "vue"


// global states which can be accessed and modified from every component
export const states = {
    // game socket is not a ref as it shouldn't update other components
    // when created or re-assigned
    gameSocket: undefined as (Socket | undefined),
    lobby: undefined as (types.LobbyId | undefined),
    id: undefined as (types.ClientId | undefined),
    name: undefined as (string | undefined)
}
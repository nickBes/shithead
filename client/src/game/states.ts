import type Socket from "./socket"


// global states which can be accessed and modified from every component
export const states = {
    // game socket is not a ref as it shouldn't update other components
    // when created or re-assigned
    gameSocket: null as (Socket | null)
}
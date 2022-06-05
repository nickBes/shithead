<script setup lang="ts">
import { states } from "@/game/states"
import { match, P } from "ts-pattern";
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";

const router = useRouter()
let lobbyName = ref<string>()

function createLobby(event : SubmitEvent) {
    event?.preventDefault()
    if (lobbyName.value != undefined) {
        states.gameSocket?.send({
            createLobby: {
                lobbyName: lobbyName.value
            }
        })
    }
}

onMounted(() => {
    states.gameSocket?.setOnMessage(message => {
        match(message)
            .with({joinLobby: P.any}, msg => {
                states.lobby = msg.joinLobby
                router.push(`/lobby/${msg.joinLobby}`)
            })
            .otherwise(msg => console.warn(`Recieved a non related message on lobby creator: ${JSON.stringify(msg)}`))
    })
})

</script>
<template>
    <p>This is lobby creator</p>
    <form @submit="(event) => createLobby(event as SubmitEvent)">
        <input v-model.lazy.trim="lobbyName" autocomplete="off" name="lobbyName" type="text"/>
        <button type="submit">Create New Lobby</button>
    </form>
</template>
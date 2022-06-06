<script setup lang="ts">
import type { OnMessageCallback } from '@/game/socket';
import { states } from '@/game/states';
import { match, P } from 'ts-pattern';
import { onMounted, onUnmounted } from 'vue';
import { useRouter, useRoute, onBeforeRouteLeave } from 'vue-router';

const router = useRouter()
const route = useRoute()
let rawLobbyId : any = route.params.id
let lobbyId : number

if (rawLobbyId && typeof rawLobbyId == "string") {
    lobbyId = parseInt(rawLobbyId)
    if (isNaN(lobbyId)) {
        router.push("/")
    }
} else {
    router.push("/")
}

onBeforeRouteLeave(() => {
    if (states.lobby == lobbyId && !states.isInGame) {
        states.gameSocket?.send("leaveLobby")
    }
    states.isAdmin.value = false
})

onMounted(() => {
    if (states.lobby != lobbyId) { // then we either joined or switched
        states.gameSocket?.messageHandlers.set("addToLobby", (message, sk) => {
            match(message)
                .with({joinLobby: P.any}, () => { // means we could join
                    states.lobby = lobbyId
                    sk.messageHandlers.delete("addToLobby")
                    states.gameSocket?.messageHandlers.set('handleLobbyMessages', handleLobbyMessages)
                })
                .otherwise(() => { // couldn't join, go to home
                    states.gameSocket?.messageHandlers.delete("addToLobby")
                    router.push("/")
                })
        })
        states.gameSocket?.send({joinLobby: lobbyId})
    } else {
        states.gameSocket?.messageHandlers.set('handleLobbyMessages', handleLobbyMessages)
    }
})

const handleLobbyMessages : OnMessageCallback = (message) => {
    match(message)
        .with({playerJoinedLobby: P.any}, ({playerJoinedLobby}) => {
            states.lastMessage.value = `${playerJoinedLobby.username} joined the lobby.`
            states.players.value.set(playerJoinedLobby.id, playerJoinedLobby.username);
        })
        .with({playerLeftLobby: P.any}, ({playerLeftLobby}) => {
            states.lastMessage.value = `${states.players.value.get(playerLeftLobby)} left the lobby.`
            states.players.value.delete(playerLeftLobby)
        })
        .with({ownerLeftLobby: P.any}, ({ownerLeftLobby}) => {
            if (states.id == ownerLeftLobby.new_owner_id) {
                states.lastMessage.value = "You're the new owner"
            } else {
                states.lastMessage.value = `${states.players.value.get(ownerLeftLobby.new_owner_id)} is the new owner.`
            }
        })
        .with("startGame", () => {
            states.isInGame = true
            router.push(`/game/${lobbyId}`)
        })
        // other messages are managed by other components
        .otherwise(() => {})
}


onUnmounted(() => {
    if(states.lobby == lobbyId) {
        states.gameSocket?.messageHandlers.delete("handleLobbyMessages")
    }
})

</script>
<template>
    <p>This is lobby #{{rawLobbyId}}</p>
    <ul>
        <li v-for="[id, name] in states.players.value" :key="id">{{name}}</li>
    </ul>
    <button v-if="states.isAdmin.value" @click="() => states.gameSocket?.send('startGame')">Start Game</button>
</template>
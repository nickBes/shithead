<script setup lang="ts">
import { notificationSettings, states } from '@/game/states';
import InGame from '@/components/InGame.vue';
import InLobby from '@/components/InLobby.vue';
import { match, P } from 'ts-pattern';
import { onMounted } from 'vue';
import { useRouter, useRoute, onBeforeRouteLeave } from 'vue-router';
import { useNotification } from 'naive-ui';
import type types from '@/bindings/bindings';

const router = useRouter()
const route = useRoute()
const notification = useNotification()
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
    if (states.lobbyId == lobbyId) {
        states.lobbyState.value = null
        states.players.value.clear()
        states.gameSocket?.send("leaveLobby")
    }
    states.isOwner.value = false
})

onMounted(() => {
    if (states.lobbyId != lobbyId) { // then we either joined or switched
        states.gameSocket?.messageHandlers.set("addToLobby", (message, sk) => {
            match(message)
                .with({joinLobby: P.any}, (msg) => { // means we could join
                    states.lobbyId = lobbyId
                    notification.success({title: "Successfully joined a lobby", ...notificationSettings})
                    states.players.value.set(states.id as types.ClientId, states.name.value as string)

                    msg.joinLobby.players.forEach(player => states.players.value.set(player.id, player.username))
                    sk.messageHandlers.delete("addToLobby")
                    states.lobbyState.value = 'inLobby'
                })
                .otherwise(() => { // couldn't join, go to home
                    states.gameSocket?.messageHandlers.delete("addToLobby")
                    router.push("/")
                })
        })
        states.gameSocket?.send({joinLobby: lobbyId})
    } else {
        states.players.value.set(states.id as types.ClientId, states.name.value as string)
        states.lobbyState.value = 'inLobby'
    }
})

</script>
<template>
    <p>This is lobby #{{rawLobbyId}}</p>
    <InLobby v-if="states.lobbyState.value == 'inLobby'"/>
    <InGame v-else-if="states.lobbyState.value == 'inGame'"></InGame>
</template>

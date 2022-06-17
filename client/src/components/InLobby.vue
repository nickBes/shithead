<script setup lang="ts">
import { notificationSettings, states } from '@/game/states';
import type { OnMessageCallback } from '@/game/socket';
import { useNotification } from 'naive-ui';
import { match, P } from 'ts-pattern';
import { onMounted, onUnmounted } from 'vue';

const notification = useNotification()

const handleLobbyMessages : OnMessageCallback = (message) => {
    let notificationMessages  = [] as string[]
    match(message)
        .with({playerJoinedLobby: P.any}, ({playerJoinedLobby}) => {
            states.players.value.set(playerJoinedLobby.id, playerJoinedLobby.username);
            notificationMessages.push(`${playerJoinedLobby.username} joined the lobby`)
        })
        .with({playerLeftLobby: P.any}, ({playerLeftLobby}) => {
            notificationMessages.push(`${states.players.value.get(playerLeftLobby)} left the lobby`)
            states.players.value.delete(playerLeftLobby)
        })
        .with({ownerLeftLobby: P.any}, ({ownerLeftLobby}) => {
            if (states.id == ownerLeftLobby.new_owner_id) {
                states.isOwner.value = true
                notificationMessages.push("You're the new owner")
            } else {
                notificationMessages.push(`${states.players.value.get(ownerLeftLobby.new_owner_id)} is the new owner`)
            }
        })
        .with("startGame", () => {
            states.lobbyState.value = 'inGame'
        })
        // other messages are managed by other components
        .otherwise(() => {})

    for (let nMsg of notificationMessages) {
        notification.info({title: nMsg, ...notificationSettings})
    }
}

onMounted(() => {
    states.gameSocket?.messageHandlers.set('handleLobbyMessages', handleLobbyMessages)
})

onUnmounted(() => {
    states.gameSocket?.messageHandlers.delete("handleLobbyMessages")
})

</script>
<template>
    <ul>
        <li v-for="[id, name] in states.players.value" :key="id">{{name}}</li>
    </ul>
    <button v-if="states.isOwner.value" @click="() => states.gameSocket?.send('startGame')">Start Game</button>
</template>
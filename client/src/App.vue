<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import { onMounted } from 'vue';
import { states } from './game/states'
import Socket, { type ServerCallback } from './game/socket'
import { P, isMatching } from 'ts-pattern';

const onSocketOpen : ServerCallback = (socket) => {
    states.lastMessage.value = "Connected to server."

    // will recieve the user's id and set it in the global state
    socket.messageHandlers.set("getClientId", (message, sk) => {
        if (isMatching({handshake: P.any}, message)) {
            states.id = message.handshake.id
            states.name = message.handshake.username
            sk.messageHandlers.delete("getClientId")
        }
    })

    // will update the last message state every time an error was recieved
    socket.messageHandlers.set("errorMessageHandler", (message) => {
        if (isMatching({error: P.any}, message)) {
                states.lastMessage.value = message.error
        }
    })
}

const onSocketClose : ServerCallback = (socket) => {
    states.lastMessage.value = "Server connection closed. Trying to reconnect."
    socket.messageHandlers.clear()
}

onMounted(() => {
    // connect when mounted
    states.gameSocket = new Socket('ws://localhost:7522', onSocketOpen, onSocketClose)
})

</script>

<template>
    <nav>
        <h1>Message:</h1>
        <p>{{states.lastMessage.value}}</p>
        <RouterLink to="/">Home</RouterLink>
        <RouterLink to="/lobbyCreator">Create Lobby</RouterLink>
    </nav>
    <RouterView/>
</template>

<style scoped>
    nav {
        display: flex;
        flex-direction: column;
    }
</style>

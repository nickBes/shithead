<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import { onMounted } from 'vue';
import { states } from './game/states'
import Socket from './game/socket'
import { P, isMatching } from 'ts-pattern';

onMounted(() => {
    // connect when mounted
    states.gameSocket = new Socket('ws://localhost:7522', (socket) => {
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
    })
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

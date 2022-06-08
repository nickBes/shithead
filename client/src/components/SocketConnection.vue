<script setup lang="ts">
import { onMounted } from 'vue';
import { P, isMatching } from 'ts-pattern';
import { useNotification } from "naive-ui"
import Socket, { type ServerCallback } from '@/game/socket'
import { notificationSettings, states } from "@/game/states"

const notification = useNotification()

const onSocketOpen : ServerCallback = (socket) => {
    notification.success({title: "Connected to server", ...notificationSettings})

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
                notification.error({title: message.error, ...notificationSettings})
        }
    })
}

const onSocketClose : ServerCallback = (socket) => {
    // states.lastMessage.value = "Server connection closed. Trying to reconnect."
    socket.messageHandlers.clear()
}

onMounted(() => {
    // connect when mounted
    states.gameSocket = new Socket('ws://localhost:7522', onSocketOpen, onSocketClose)
})
</script>

<template>
</template>
<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import { notificationPlacement } from './game/states'
import { darkTheme, NConfigProvider, NNotificationProvider } from "naive-ui"
import SocketConnection from "@/components/SocketConnection.vue"

</script>

<template>
    <n-config-provider :theme="darkTheme">
        <n-notification-provider :placement="notificationPlacement">
            <SocketConnection/>
            <nav>
                <RouterLink to="/">Home</RouterLink>
                <RouterLink to="/lobbyCreator">Create Lobby</RouterLink>
            </nav>
            <RouterView/>
        </n-notification-provider>
    </n-config-provider>
</template>

<style>
    a {
        text-decoration: none;
    }
    nav {
        display: flex;
        flex-direction: column;
    }
</style>

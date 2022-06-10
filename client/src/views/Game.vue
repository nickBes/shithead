<script setup lang="ts">
import { states } from '@/game/states';
import { onBeforeRouteLeave, useRoute, useRouter } from 'vue-router';

const router = useRouter()
const route = useRoute()
let rawGameId : any = route.params.id
let gameId : number

if (rawGameId && typeof rawGameId == "string") {
    gameId = parseInt(rawGameId)
    if (isNaN(gameId) || gameId != states.lobby) {
        router.push("/")
    }
} else {
    router.push("/")
}

onBeforeRouteLeave(() => {
    states.isInGame = false
})

</script>

<template>
    <p>This is the game page.</p>
</template>
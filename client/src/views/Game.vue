<script setup lang="ts">
import { states } from '@/game/states';
import { onBeforeRouteLeave } from 'vue-router';

onBeforeRouteLeave(() => {
    states.isInGame = false
})

</script>

<template>
    <p>This is the game page.</p>
</template>
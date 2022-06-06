import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      component: () => import("@/views/Home.vue")
    },
    {
      path: '/lobbyCreator',
      component: () => import("@/views/LobbyCreator.vue"),
    },
    {
      path: '/lobby/:id',
      component: () => import("@/views/Lobby.vue")
    },
    {
      path: '/game/:id',
      component: () => import("@/views/Game.vue")
    }
  ]
})

export default router
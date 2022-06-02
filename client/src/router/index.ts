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
    }
  ]
})

export default router
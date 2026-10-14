import { createRouter, createWebHistory } from 'vue-router'
import { auth, loadSession } from './lib/auth'
import AdminLayout from './layouts/AdminLayout.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/login',
      component: () => import('./pages/LoginPage.vue'),
      meta: { title: 'Sign in', guest: true },
    },
    {
      path: '/',
      component: AdminLayout,
      meta: { auth: true },
      children: [
        {
          path: '',
          name: 'dashboard',
          component: () => import('./pages/DashboardPage.vue'),
          meta: { title: 'Overview', admin: true },
        },
        {
          path: 'users',
          name: 'users',
          component: () => import('./pages/UsersPage.vue'),
          meta: { title: 'Users', admin: true },
        },
        {
          path: 'settings',
          name: 'settings',
          component: () => import('./pages/SettingsPage.vue'),
          meta: { title: 'Settings' },
        },
      ],
    },
    {
      path: '/:pathMatch(.*)*',
      component: () => import('./pages/NotFoundPage.vue'),
      meta: { title: 'Page not found' },
    },
  ],
})
router.beforeEach(async (to) => {
  if (!auth.ready) {
    try {
      await loadSession()
    } catch {
      if (to.path !== '/login') return '/login'
    }
  }
  if (to.meta.auth && !auth.user) return '/login'
  if (to.meta.guest && auth.user) return auth.user.role === 'admin' ? '/' : '/settings'
  if (to.meta.admin && auth.user?.role !== 'admin') return '/settings'
})
router.afterEach((to) => {
  document.title = `${to.meta.title || 'Admin'} · RustMVC`
})
window.addEventListener('session-expired', () => {
  auth.user = null
  auth.ready = false
  router.replace('/login')
})
export default router

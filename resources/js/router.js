import { createRouter, createWebHistory, createWebHashHistory } from 'vue-router'
import { auth, loadSession } from './lib/auth'
import { isDesktop } from './lib/api'
import AdminLayout from './layouts/AdminLayout.vue'

const router = createRouter({
  history: isDesktop ? createWebHashHistory() : createWebHistory(),
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
          path: 'guide',
          component: () => import('./pages/GuidePage.vue'),
          meta: { title: 'Getting started' },
        },
        {
          path: 'examples/elements',
          component: () => import('./pages/examples/ElementsPage.vue'),
          meta: { title: 'Elements & buttons' },
        },
        {
          path: 'examples/cards',
          component: () => import('./pages/examples/CardsPage.vue'),
          meta: { title: 'Cards' },
        },
        {
          path: 'examples/tables',
          component: () => import('./pages/examples/TablesPage.vue'),
          meta: { title: 'Tables' },
        },
        {
          path: 'examples/forms',
          component: () => import('./pages/examples/FormsPage.vue'),
          meta: { title: 'Forms' },
        },
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

<script setup>
import { ref, watch, nextTick, onMounted, onBeforeUnmount } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { auth, logout } from '../lib/auth'
import { initials } from '../lib/format'
import AppIcon from '../components/AppIcon.vue'
import FeedbackMessage from '../components/FeedbackMessage.vue'
const route = useRoute(),
  router = useRouter()
const open = ref(false),
  busy = ref(false),
  error = ref('')
const media = window.matchMedia('(max-width: 900px)')
const mobile = ref(media.matches),
  sidebar = ref(null),
  menuButton = ref(null)
function resized(event) {
  mobile.value = event.matches
  if (!event.matches) open.value = false
}
onMounted(() => media.addEventListener('change', resized))
onBeforeUnmount(() => media.removeEventListener('change', resized))
watch(open, async (value) => {
  await nextTick()
  if (!mobile.value) return
  if (value) sidebar.value?.querySelector('button, a[href]')?.focus()
  else menuButton.value?.focus()
})
function navigationKey(event) {
  if (!mobile.value || !open.value) return
  if (event.key === 'Escape') {
    event.preventDefault()
    open.value = false
    return
  }
  if (event.key !== 'Tab') return
  const items = [...sidebar.value.querySelectorAll('a[href], button:not(:disabled)')].filter(
    (element) => element.getClientRects().length > 0,
  )
  const first = items[0],
    last = items.at(-1)
  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault()
    last?.focus()
  }
  if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault()
    first?.focus()
  }
}
async function signOut() {
  busy.value = true
  try {
    await logout()
    router.push('/login')
  } catch (e) {
    error.value = e.message
  } finally {
    busy.value = false
  }
}
</script>
<template>
  <div class="app-shell">
    <button
      v-if="mobile && open"
      tabindex="-1"
      class="sidebar-backdrop"
      aria-label="Close navigation"
      @click="open = false"
    ></button>
    <aside
      ref="sidebar"
      class="sidebar"
      :class="{ open }"
      :inert="mobile && !open"
      :aria-hidden="mobile && !open ? true : undefined"
      @keydown="navigationKey"
    >
      <button
        v-if="mobile"
        class="icon-button mobile-nav-close"
        aria-label="Close menu"
        @click="open = false"
      >
        <AppIcon name="X" />
      </button>
      <RouterLink
        :to="auth.user?.role === 'admin' ? '/' : '/settings'"
        class="brand"
        @click="open = false"
        ><span class="brand-mark"><AppIcon name="Layers3" :size="23" /></span>Rust<span
          class="brand-light"
          >MVC</span
        ><span class="version">/ 01</span></RouterLink
      >
      <div class="workspace">
        <span class="workspace-symbol">W</span>
        <div><strong>Your workspace</strong><small>Administration</small></div>
        <AppIcon name="ChevronDown" :size="14" />
      </div>
      <p class="nav-label">WORKSPACE</p>
      <nav aria-label="Main navigation">
        <RouterLink
          v-if="auth.user?.role === 'admin'"
          to="/"
          exact-active-class="selected"
          @click="open = false"
          ><AppIcon name="LayoutDashboard" />Overview</RouterLink
        >
        <RouterLink
          v-if="auth.user?.role === 'admin'"
          to="/users"
          active-class="selected"
          @click="open = false"
          ><AppIcon name="Users" />Users</RouterLink
        >
        <RouterLink to="/settings" active-class="selected" @click="open = false"
          ><AppIcon name="Settings2" />Settings</RouterLink
        >
        <p class="nav-label">LEARN & BUILD</p>
        <RouterLink
          v-for="item in [
            { path: '/guide', label: 'Getting started', icon: 'Command' },
            { path: '/examples/elements', label: 'Elements & buttons', icon: 'Layers3' },
            { path: '/examples/cards', label: 'Cards', icon: 'LayoutDashboard' },
            { path: '/examples/tables', label: 'Tables', icon: 'Menu' },
            { path: '/examples/forms', label: 'Forms', icon: 'Pencil' },
          ]"
          :key="item.path"
          :to="item.path"
          active-class="selected"
          @click="open = false"
          ><AppIcon :name="item.icon" />{{ item.label }}</RouterLink
        >
      </nav>
      <div class="sidebar-bottom">
        <div class="build-note">
          <span class="status-square"></span> A foundation for your next idea.
          <p>Rust on the inside.<br />A familiar workflow everywhere.</p>
          <span class="mono">RUST + VUE / STARTER</span>
        </div>
        <div class="sidebar-account">
          <span class="avatar-tile">{{ initials(auth.user?.name) }}</span>
          <div>
            <strong>{{ auth.user?.name }}</strong
            ><small>{{ auth.user?.role === 'admin' ? 'Administrator' : 'Member' }}</small>
          </div>
          <button class="icon-button" aria-label="Sign out" :disabled="busy" @click="signOut">
            <AppIcon name="LogOut" />
          </button>
        </div>
      </div>
    </aside>
    <div class="main-shell" :inert="mobile && open">
      <header class="topbar">
        <div class="breadcrumbs">
          <button
            ref="menuButton"
            class="icon-button mobile-menu"
            aria-label="Open navigation"
            :aria-expanded="open"
            @click="open = true"
          >
            <AppIcon name="Menu" /></button
          ><span class="breadcrumb-root">Workspace</span><span class="breadcrumb-divider">/</span
          ><strong>{{ route.meta.title }}</strong>
        </div>
        <div class="topbar-right">
          <span class="environment"><span></span> Workspace</span
          ><RouterLink to="/settings" class="avatar-tile small" aria-label="Account settings">{{
            initials(auth.user?.name)
          }}</RouterLink>
        </div>
      </header>
      <main id="main-content" class="page-content">
        <FeedbackMessage :message="error" /><RouterView />
      </main>
      <footer class="page-footer">
        <span>RustMVC <span class="footer-divider">/</span> Built to make it yours.</span
        ><span class="mono">v0.1.0</span>
      </footer>
    </div>
  </div>
</template>

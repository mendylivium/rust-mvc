<script setup>
import { ref } from 'vue'
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
      v-if="open"
      class="sidebar-backdrop"
      aria-label="Close navigation"
      @click="open = false"
    ></button>
    <aside class="sidebar" :class="{ open }">
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
    <div class="main-shell">
      <header class="topbar">
        <div class="breadcrumbs">
          <button
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

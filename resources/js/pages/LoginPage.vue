<script setup>
import { reactive, ref, onMounted } from 'vue'
import { api, isDesktop } from '../lib/api'
import { useRouter } from 'vue-router'
import { auth, login } from '../lib/auth'
import AppIcon from '../components/AppIcon.vue'
import FeedbackMessage from '../components/FeedbackMessage.vue'
const firstRun = ref(false)
const checking = ref(isDesktop)
onMounted(async () => {
  if (!isDesktop) return
  try {
    firstRun.value = (await api('/desktop/setup')).needs_setup
  } catch (e) {
    error.value = e.message
  } finally {
    checking.value = false
  }
})
const form = reactive({ name: '', email: '', password: '' }),
  busy = ref(false),
  error = ref('')
const router = useRouter()
async function submit() {
  busy.value = true
  error.value = ''
  try {
    if (firstRun.value) await api('/desktop/setup', { method: 'POST', body: form })
    await login(form)
    router.push(auth.user.role === 'admin' ? '/' : '/settings')
  } catch (e) {
    error.value =
      Object.values(e.errors || {})
        .flat()
        .join(' ') || e.message
  } finally {
    busy.value = false
  }
}
</script>
<template>
  <div class="login-page">
    <section class="login-story">
      <div class="brand">
        <span class="brand-mark"><AppIcon name="Layers3" :size="24" /></span>Rust<span
          class="brand-light"
          >MVC</span
        >
      </div>
      <div class="login-story-content">
        <span class="eyebrow">A STRONG FOUNDATION</span>
        <h1>Less setup.<br />More building.</h1>
        <p>Your workspace starts here. A thoughtful admin experience, powered by Rust.</p>
        <div class="architecture-art" aria-hidden="true">
          <div></div>
          <div></div>
          <div></div>
          <span>STRUCTURE. SPEED. SIMPLICITY.</span>
        </div>
      </div>
      <span class="mono">RUST + VUE / STARTER</span>
    </section>
    <section class="login-form-panel">
      <div class="login-form-wrap">
        <span class="eyebrow">YOUR WORKSPACE, READY.</span>
        <h2>{{ firstRun ? 'Create your workspace.' : 'Welcome back.' }}</h2>
        <p class="muted">
          {{
            firstRun
              ? 'Create the first administrator for this desktop workspace.'
              : 'Sign in to manage your application.'
          }}
        </p>
        <FeedbackMessage :message="error" />
        <p v-if="checking" role="status">Opening your workspace…</p>
        <form v-else @submit.prevent="submit">
          <template v-if="firstRun"
            ><label class="field-label" for="setup-name">Full name</label
            ><input
              id="setup-name"
              v-model="form.name"
              class="input w-full"
              required
              minlength="2"
              maxlength="80"
              autocomplete="name"
          /></template>
          <label class="field-label" for="email">Email address</label
          ><input
            id="email"
            v-model="form.email"
            type="email"
            class="input w-full"
            placeholder="you@company.com"
            autocomplete="username"
            required
            maxlength="254"
          /><label class="field-label" for="password">Password</label
          ><input
            id="password"
            v-model="form.password"
            type="password"
            class="input w-full"
            placeholder="Enter your password"
            :autocomplete="firstRun ? 'new-password' : 'current-password'"
            :minlength="firstRun ? 12 : undefined"
            required
            maxlength="128"
          /><button class="btn btn-primary w-full login-submit" :disabled="busy">
            {{ busy ? 'Please wait…' : firstRun ? 'Create workspace' : 'Sign in to workspace'
            }}<AppIcon name="ArrowRight" />
          </button>
        </form>
        <div class="login-help">
          <AppIcon name="ShieldCheck" /><span
            >Access is managed by your workspace administrator.</span
          >
        </div>
      </div>
      <p class="login-copyright">RustMVC · Your next project starts here.</p>
    </section>
  </div>
</template>

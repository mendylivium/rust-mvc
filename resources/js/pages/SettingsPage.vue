<script setup>
import { reactive, ref } from 'vue'
import { auth } from '../lib/auth'
import { api } from '../lib/api'
import { initials } from '../lib/format'
import AppIcon from '../components/AppIcon.vue'
import FeedbackMessage from '../components/FeedbackMessage.vue'
const form = reactive({ name: auth.user?.name || '' }),
  busy = ref(false),
  error = ref(''),
  success = ref('')
async function save() {
  busy.value = true
  error.value = ''
  success.value = ''
  try {
    const data = await api('/profile', { method: 'PATCH', body: form })
    auth.user = data.user
    success.value = 'Your profile has been updated.'
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
  <section class="page-heading">
    <div>
      <div class="eyebrow">YOUR ACCOUNT</div>
      <h1>A space that feels like you.</h1>
      <p>Keep your profile up to date.</p>
    </div>
  </section>
  <div class="settings-grid">
    <section class="card settings-card">
      <div class="card-heading">
        <div>
          <h2>Profile details</h2>
          <p>How you appear in your workspace.</p>
        </div>
        <AppIcon name="Settings2" />
      </div>
      <div class="settings-body">
        <div class="profile-preview">
          <span class="avatar-tile large">{{ initials(auth.user?.name) }}</span>
          <div>
            <strong>{{ auth.user?.name }}</strong>
            <p class="muted">{{ auth.user?.email }}</p>
          </div>
        </div>
        <FeedbackMessage :message="error" /><FeedbackMessage :message="success" success />
        <form @submit.prevent="save">
          <label class="field-label" for="profile-name">Full name</label
          ><input
            id="profile-name"
            v-model="form.name"
            class="input w-full"
            required
            minlength="2"
            maxlength="80"
            autocomplete="name"
          /><label class="field-label" for="profile-email">Email address</label
          ><input id="profile-email" :value="auth.user?.email" class="input w-full" disabled />
          <p class="field-hint">Contact an administrator to change your email or password.</p>
          <div class="settings-actions">
            <button class="btn btn-primary" :disabled="busy">
              {{ busy ? 'Saving…' : 'Save changes' }}
            </button>
          </div>
        </form>
      </div>
    </section>
    <aside class="settings-note">
      <AppIcon name="ShieldCheck" :size="25" />
      <h3>Your access</h3>
      <p>
        You’re signed in as
        <strong>{{ auth.user?.role === 'admin' ? 'an administrator' : 'a member' }}</strong
        >.
      </p>
      <p>
        {{
          auth.user?.role === 'admin'
            ? 'You can manage users, assign roles, and view your workspace overview.'
            : 'You can access your account and update your profile.'
        }}
      </p>
    </aside>
  </div>
</template>

<script setup>
import { reactive, ref } from 'vue'
import { api } from '../lib/api'
import { auth, loadSession } from '../lib/auth'
import AppIcon from './AppIcon.vue'
import FeedbackMessage from './FeedbackMessage.vue'
import AppDialog from './AppDialog.vue'
const props = defineProps({ user: Object })
const emit = defineEmits(['close', 'saved'])
const form = reactive({
  name: props.user?.name || '',
  email: props.user?.email || '',
  password: '',
  role: props.user?.role || 'member',
  status: props.user?.status || 'active',
})
const busy = ref(false),
  error = ref(''),
  errors = ref({})
async function submit() {
  busy.value = true
  error.value = ''
  errors.value = {}
  try {
    await api(props.user ? `/users/${props.user.id}` : '/users', {
      method: props.user ? 'PUT' : 'POST',
      body: form,
    })
    if (props.user?.id === auth.user?.id) await loadSession()
    emit('saved')
  } catch (e) {
    error.value = e.message
    errors.value = e.errors || {}
  } finally {
    busy.value = false
  }
}
</script>
<template>
  <AppDialog labelledby="user-form-title" :busy="busy" @close="emit('close')"
    ><div class="modal-heading">
      <div>
        <span class="eyebrow">WORKSPACE ACCESS</span>
        <h2 id="user-form-title">{{ user ? 'Edit user' : 'Add a new user' }}</h2>
      </div>
      <button class="icon-button" aria-label="Close form" :disabled="busy" @click="emit('close')">
        <AppIcon name="X" />
      </button>
    </div>
    <p class="muted">
      {{
        user
          ? 'Update their details and access to this workspace.'
          : 'Give someone a place in your workspace.'
      }}
    </p>
    <FeedbackMessage :message="error" />
    <form @submit.prevent="submit">
      <label class="field-label" for="user-name">Full name</label
      ><input
        id="user-name"
        v-model="form.name"
        class="input w-full"
        autocomplete="name"
        required
        minlength="2"
        maxlength="80"
        autofocus
      /><small v-if="errors.name" class="field-error">{{ errors.name[0] }}</small
      ><label class="field-label" for="user-email">Email address</label
      ><input
        id="user-email"
        v-model="form.email"
        class="input w-full"
        type="email"
        autocomplete="email"
        required
        maxlength="254"
      /><small v-if="errors.email" class="field-error">{{ errors.email[0] }}</small
      ><label class="field-label" for="user-password">{{
        user ? 'New password (optional)' : 'Password'
      }}</label
      ><input
        id="user-password"
        v-model="form.password"
        type="password"
        class="input w-full"
        autocomplete="new-password"
        :required="!user"
        minlength="12"
        maxlength="128"
      /><small class="muted"
        >At least 12 characters.
        {{
          user ? 'Leave blank to keep the current password.' : 'Share securely with the new user.'
        }}</small
      ><small v-if="errors.password" class="field-error">{{ errors.password[0] }}</small>
      <div class="form-grid">
        <div>
          <label class="field-label" for="user-role">Role</label
          ><select
            id="user-role"
            v-model="form.role"
            class="select w-full"
            :disabled="user?.id === auth.user?.id"
          >
            <option value="member">Member</option>
            <option value="admin">Administrator</option></select
          ><small v-if="errors.role" class="field-error">{{ errors.role[0] }}</small>
        </div>
        <div>
          <label class="field-label" for="user-status">Status</label
          ><select
            id="user-status"
            v-model="form.status"
            class="select w-full"
            :disabled="user?.id === auth.user?.id"
          >
            <option value="active">Active</option>
            <option value="inactive">Inactive</option>
          </select>
        </div>
      </div>
      <div class="modal-actions">
        <button type="button" class="btn btn-ghost" :disabled="busy" @click="emit('close')">
          Cancel</button
        ><button class="btn btn-primary" :disabled="busy">
          {{ busy ? 'Saving…' : user ? 'Save changes' : 'Create user' }}
        </button>
      </div>
    </form></AppDialog
  >
</template>

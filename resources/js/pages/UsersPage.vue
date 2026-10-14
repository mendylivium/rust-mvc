<script setup>
import { onMounted, ref, watch, onBeforeUnmount } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { api } from '../lib/api'
import { auth } from '../lib/auth'
import { initials, date } from '../lib/format'
import AppIcon from '../components/AppIcon.vue'
import FeedbackMessage from '../components/FeedbackMessage.vue'
import UserForm from '../components/UserForm.vue'
import AppDialog from '../components/AppDialog.vue'
const route = useRoute(),
  router = useRouter()
const search = ref(String(route.query.search || '')),
  role = ref(''),
  page = ref(1),
  data = ref({ data: [], meta: { total: 0, last_page: 1 } })
const loading = ref(true),
  error = ref(''),
  success = ref(''),
  showForm = ref(false),
  editing = ref(null),
  deleting = ref(null),
  busy = ref(false)
let timer,
  request = 0
async function load() {
  const id = ++request
  loading.value = true
  error.value = ''
  try {
    const result = await api(
      `/users?${new URLSearchParams({ search: search.value, role: role.value, page: page.value })}`,
    )
    if (id === request) {
      data.value = result
      page.value = result.meta.page
    }
  } catch (e) {
    if (id === request) error.value = e.message
  } finally {
    if (id === request) loading.value = false
  }
}
function edit(user = null) {
  editing.value = user
  showForm.value = true
  success.value = ''
}
async function saved() {
  showForm.value = false
  success.value = editing.value ? 'User updated successfully.' : 'Your new user is ready.'
  await load()
}
async function remove() {
  busy.value = true
  try {
    await api(`/users/${deleting.value.id}`, { method: 'DELETE' })
    deleting.value = null
    success.value = 'User deleted.'
    await load()
  } catch (e) {
    error.value = e.message
    deleting.value = null
  } finally {
    busy.value = false
  }
}
watch([search, role], () => {
  clearTimeout(timer)
  ++request
  timer = setTimeout(() => {
    page.value = 1
    load()
  }, 250)
})
onBeforeUnmount(() => {
  clearTimeout(timer)
  ++request
})
onMounted(() => {
  load()
  if (route.query.create === '1') {
    edit()
    const { create, ...query } = route.query
    router.replace({ query })
  }
})
function navigate(direction) {
  page.value += direction
  load()
}
</script>
<template>
  <section class="page-heading">
    <div>
      <div class="eyebrow">PEOPLE & PERMISSIONS</div>
      <h1>Good things start with people.</h1>
      <p>Manage your team, their details, and their workspace access.</p>
    </div>
    <button class="btn btn-primary" @click="edit()">
      <AppIcon name="Plus" :size="16" />Add user
    </button>
  </section>
  <FeedbackMessage :message="error" /><FeedbackMessage :message="success" success />
  <section class="card users-card">
    <div class="card-heading">
      <div class="users-title">
        <h2>All users</h2>
        <span class="badge badge-ghost">{{ data.meta.total }}</span>
      </div>
      <span class="muted text-sm">One workspace. Your whole team.</span>
    </div>
    <div class="table-toolbar">
      <label class="search-field"
        ><AppIcon name="Search" :size="17" /><input
          v-model="search"
          type="search"
          placeholder="Search by name or email…"
          aria-label="Search users" /></label
      ><select v-model="role" class="select role-filter" aria-label="Filter by role">
        <option value="">All roles</option>
        <option value="admin">Administrators</option>
        <option value="member">Members</option>
      </select>
    </div>
    <div v-if="loading" class="loading-state" role="status">
      <span class="loading loading-spinner"></span>Loading users…
    </div>
    <div v-else-if="error" class="empty-state">
      <h3>We couldn’t load your users.</h3>
      <button class="btn" @click="load">Try again</button>
    </div>
    <div v-else-if="!data.data.length" class="empty-state">
      <AppIcon name="Users" :size="32" />
      <h3>No users found</h3>
      <p>Try a different search or add someone new.</p>
      <button class="btn btn-primary" @click="edit()">Add user</button>
    </div>
    <div v-else class="table-scroll">
      <table class="table">
        <thead>
          <tr>
            <th>NAME</th>
            <th>ROLE</th>
            <th>STATUS</th>
            <th>JOINED</th>
            <th class="text-right">ACTIONS</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="user in data.data" :key="user.id">
            <td>
              <div class="person">
                <span class="avatar-tile">{{ initials(user.name) }}</span>
                <div>
                  <strong
                    >{{ user.name }}
                    <span v-if="user.id === auth.user?.id" class="you-label">you</span></strong
                  ><small>{{ user.email }}</small>
                </div>
              </div>
            </td>
            <td>
              <span class="role-label">{{ user.role }}</span>
            </td>
            <td>
              <span
                class="badge"
                :class="user.status === 'active' ? 'badge-success' : 'badge-ghost'"
                >{{ user.status }}</span
              >
            </td>
            <td class="muted nowrap">{{ date(user.created_at) }}</td>
            <td>
              <div class="row-actions">
                <button class="icon-button" :aria-label="`Edit ${user.name}`" @click="edit(user)">
                  <AppIcon name="Pencil" :size="16" /></button
                ><button
                  class="icon-button danger"
                  :aria-label="`Delete ${user.name}`"
                  :disabled="user.id === auth.user?.id"
                  @click="deleting = user"
                >
                  <AppIcon name="Trash2" :size="16" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <div class="table-pagination">
      <span
        >{{ data.meta.total }} {{ data.meta.total === 1 ? 'user' : 'users' }} in your
        workspace</span
      >
      <div>
        <button
          class="btn btn-sm btn-ghost"
          aria-label="Previous page"
          :disabled="loading || page <= 1"
          @click="navigate(-1)"
        >
          <AppIcon name="ChevronLeft" :size="16" /></button
        ><span>Page {{ page }} of {{ data.meta.last_page }}</span
        ><button
          class="btn btn-sm btn-ghost"
          aria-label="Next page"
          :disabled="loading || page >= data.meta.last_page"
          @click="navigate(1)"
        >
          <AppIcon name="ChevronRight" :size="16" />
        </button>
      </div>
    </div>
  </section>
  <UserForm v-if="showForm" :user="editing" @close="showForm = false" @saved="saved" />
  <AppDialog v-if="deleting" labelledby="delete-title" :busy="busy" @close="deleting = null"
    ><h2 id="delete-title">Delete {{ deleting.name }}?</h2>
    <p class="muted">
      Their account and active sessions will be permanently removed. This cannot be undone.
    </p>
    <div class="modal-actions">
      <button class="btn btn-ghost" :disabled="busy" @click="deleting = null">Cancel</button
      ><button class="btn btn-error" :disabled="busy" @click="remove">
        {{ busy ? 'Deleting…' : 'Delete user' }}
      </button>
    </div></AppDialog
  >
</template>

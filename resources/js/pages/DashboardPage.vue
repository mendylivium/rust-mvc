<script setup>
import { computed, onMounted, ref } from 'vue'
import { api } from '../lib/api'
import { auth } from '../lib/auth'
import { initials, date } from '../lib/format'
import AppIcon from '../components/AppIcon.vue'
import FeedbackMessage from '../components/FeedbackMessage.vue'
const data = ref(null),
  error = ref(''),
  loading = ref(true)
const today = new Intl.DateTimeFormat('en', {
  month: 'long',
  day: 'numeric',
  year: 'numeric',
}).format(new Date())
const metrics = computed(() =>
  data.value
    ? [
        {
          label: 'Total users',
          value: data.value.stats.total,
          icon: 'Users',
          note: 'Across your workspace',
        },
        {
          label: 'Active accounts',
          value: data.value.stats.active,
          icon: 'UserCheck',
          note: 'Ready to access the app',
        },
        {
          label: 'Administrators',
          value: data.value.stats.admins,
          icon: 'ShieldCheck',
          note: 'With management access',
        },
        {
          label: 'New this month',
          value: data.value.stats.new_this_month,
          icon: 'UserPlus',
          note: 'New additions to your team',
        },
      ]
    : [],
)
const max = computed(() => Math.max(1, ...data.value.growth.map((entry) => entry[1])))
const monthLabel = (value) =>
  new Intl.DateTimeFormat('en', { month: 'short', timeZone: 'UTC' }).format(
    new Date(`${value}-01T00:00:00Z`),
  )
async function load() {
  loading.value = true
  error.value = ''
  try {
    data.value = await api('/dashboard')
  } catch (e) {
    error.value = e.message
  } finally {
    loading.value = false
  }
}
onMounted(load)
</script>
<template>
  <section class="page-heading">
    <div>
      <div class="eyebrow">WORKSPACE OVERVIEW</div>
      <h1>A little clarity. A lot of possibility.</h1>
      <p>Welcome back, {{ auth.user?.name.split(' ')[0] }}. Here’s where things stand.</p>
    </div>
    <RouterLink to="/users?create=1" class="btn btn-primary"
      ><AppIcon name="Plus" :size="16" />Add user</RouterLink
    >
  </section>
  <FeedbackMessage :message="error" /><button v-if="error" class="btn btn-sm" @click="load">
    Try again
  </button>
  <div v-if="loading" class="loading-state" role="status">
    <span class="loading loading-spinner"></span>Loading your workspace…
  </div>
  <template v-else-if="data">
    <div class="section-caption">
      <span>AT A GLANCE</span><span>{{ today }}</span>
    </div>
    <div class="metrics-grid">
      <article v-for="(metric, i) in metrics" :key="metric.label" class="card metric-card">
        <div class="metric-top">
          <span>{{ metric.label }}</span
          ><AppIcon :name="metric.icon" />
        </div>
        <div class="metric-value">
          {{ metric.value.toLocaleString() }}<span class="metric-index">0{{ i + 1 }}</span>
        </div>
        <p>{{ metric.note }}</p>
      </article>
    </div>
    <div class="overview-grid">
      <section class="card growth-card">
        <div class="card-heading">
          <div>
            <h2>Room to grow</h2>
            <p>User registrations over the last six months.</p>
          </div>
          <span class="badge badge-ghost">6 months</span>
        </div>
        <div
          class="bar-chart"
          role="img"
          :aria-label="data.growth.map(([month, count]) => `${month}: ${count} users`).join(', ')"
        >
          <div v-for="([month, count], i) in data.growth" :key="month" class="chart-column">
            <span class="bar-value">{{ count }}</span>
            <div class="bar-track">
              <div
                class="bar-fill"
                :class="{ latest: i === 5 }"
                :style="{ height: `${Math.max(count ? 5 : 0, (count / max) * 100)}%` }"
              ></div>
            </div>
            <span class="bar-label">{{ monthLabel(month) }}</span>
          </div>
        </div>
        <div class="chart-footer">
          <span class="chart-key"></span>New users<span class="muted"
            >Live workspace data · UTC</span
          >
        </div>
      </section>
      <section class="quick-card">
        <div class="quick-icon"><AppIcon name="Layers3" :size="26" /></div>
        <span class="eyebrow">MAKE IT YOURS</span>
        <h2>A good place<br />to start.</h2>
        <p>The essentials are in place. Bring your team in and build what comes next.</p>
        <RouterLink to="/users">Manage your users<AppIcon name="ArrowUpRight" /></RouterLink
        ><RouterLink to="/settings"
          >Personalize your profile<AppIcon name="ArrowUpRight"
        /></RouterLink>
        <div class="quick-decoration" aria-hidden="true">↗</div>
      </section>
    </div>
    <section class="card recent-card">
      <div class="card-heading">
        <div>
          <h2>The latest additions</h2>
          <p>A quick look at your newest workspace members.</p>
        </div>
        <RouterLink to="/users" class="text-link"
          >View all users<AppIcon name="ArrowRight" :size="16"
        /></RouterLink>
      </div>
      <div class="table-scroll">
        <table class="table">
          <thead>
            <tr>
              <th>NAME</th>
              <th>ROLE</th>
              <th>STATUS</th>
              <th>JOINED</th>
              <th><span class="sr-only">Action</span></th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="user in data.recent" :key="user.id">
              <td>
                <div class="person">
                  <span class="avatar-tile">{{ initials(user.name) }}</span>
                  <div>
                    <strong>{{ user.name }}</strong
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
              <td class="muted">{{ date(user.created_at) }}</td>
              <td>
                <RouterLink
                  :to="`/users?search=${encodeURIComponent(user.email)}`"
                  class="icon-button"
                  :aria-label="`View ${user.name}`"
                  ><AppIcon name="ArrowUpRight" :size="16"
                /></RouterLink>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>
  </template>
</template>

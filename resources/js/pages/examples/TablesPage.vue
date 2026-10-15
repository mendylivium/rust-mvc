<script setup>
import { computed, ref, watch } from 'vue'
const rows = [
  { id: 1, name: 'Workspace redesign', owner: 'Jamie', status: 'Active' },
  { id: 2, name: 'API documentation', owner: 'Alex', status: 'Done' },
  { id: 3, name: 'Customer portal', owner: 'Sam', status: 'Draft' },
  { id: 4, name: 'Billing dashboard', owner: 'Jamie', status: 'Active' },
  { id: 5, name: 'Team directory', owner: 'Alex', status: 'Done' },
  { id: 6, name: 'Notification center', owner: 'Sam', status: 'Draft' },
  { id: 7, name: 'Reports library', owner: 'Jamie', status: 'Active' },
]
const search = ref(''),
  status = ref(''),
  page = ref(1),
  ascending = ref(true),
  selected = ref([])
const filtered = computed(() =>
  rows
    .filter(
      (row) =>
        row.name.toLowerCase().includes(search.value.toLowerCase()) &&
        (!status.value || row.status === status.value),
    )
    .sort((a, b) =>
      ascending.value ? a.name.localeCompare(b.name) : b.name.localeCompare(a.name),
    ),
)
const pages = computed(() => Math.max(1, Math.ceil(filtered.value.length / 4)))
const visible = computed(() => filtered.value.slice((page.value - 1) * 4, page.value * 4))
watch([search, status, ascending], () => (page.value = 1))
function reset() {
  search.value = ''
  status.value = ''
  selected.value = []
  ascending.value = true
  page.value = 1
}
</script>
<template>
  <div class="page-heading">
    <div>
      <p class="eyebrow">UI EXAMPLES / 03</p>
      <h1>Tables</h1>
      <p class="muted">
        Search, filter, sort, select, and paginate. These seven projects are local sample data.
      </p>
    </div>
  </div>
  <section class="card example-panel">
    <h2>Sample projects</h2>
    <div class="example-toolbar">
      <label class="example-field"
        >Search projects<input
          v-model="search"
          type="search"
          class="input"
          placeholder="Search by project name" /></label
      ><label class="example-field"
        >Project status<select v-model="status" class="select" aria-label="Project status">
          <option value="">All statuses</option>
          <option>Active</option>
          <option>Done</option>
          <option>Draft</option>
        </select></label
      ><button class="btn btn-outline" @click="reset">Reset table</button>
    </div>
    <p role="status">
      {{ filtered.length }} results · {{ selected.length }} selected across all pages
    </p>
    <div class="table-scroll">
      <table class="table">
        <caption class="sr-only">
          Sample projects with sortable names and row selection
        </caption>
        <thead>
          <tr>
            <th scope="col">Select</th>
            <th scope="col" :aria-sort="ascending ? 'ascending' : 'descending'">
              <button class="btn btn-ghost btn-sm" @click="ascending = !ascending">
                Project {{ ascending ? '↑' : '↓' }}
              </button>
            </th>
            <th scope="col">Owner</th>
            <th scope="col">Status</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in visible" :key="row.id">
            <td>
              <input
                v-model="selected"
                class="checkbox checkbox-sm"
                type="checkbox"
                :value="row.id"
                :aria-label="`Select ${row.name}`"
              />
            </td>
            <th scope="row">{{ row.name }}</th>
            <td>{{ row.owner }}</td>
            <td>
              <span
                class="badge"
                :class="
                  row.status === 'Active'
                    ? 'badge-success'
                    : row.status === 'Draft'
                      ? 'badge-warning'
                      : 'badge-outline'
                "
                >{{ row.status }}</span
              >
            </td>
          </tr>
          <tr v-if="!visible.length">
            <td colspan="4">No projects match. Clear your search or choose another status.</td>
          </tr>
        </tbody>
      </table>
    </div>
    <div class="example-row example-pagination">
      <span>Page {{ page }} of {{ pages }}</span
      ><button class="btn btn-sm" :disabled="page === 1" @click="page--">Previous page</button
      ><button class="btn btn-sm" :disabled="page === pages" @click="page++">Next page</button>
    </div>
  </section>
</template>

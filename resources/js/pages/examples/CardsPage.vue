<script setup>
import { ref } from 'vue'
const saved = ref(false)
</script>
<template>
  <div class="page-heading">
    <div>
      <p class="eyebrow">UI EXAMPLES / 02</p>
      <h1>Cards</h1>
      <p class="muted">
        Layouts for metrics, projects, profiles, and empty collections. All content is sample data.
      </p>
    </div>
  </div>
  <div class="example-grid example-metrics">
    <section
      v-for="metric in [
        { label: 'Sample projects', value: '24', note: 'Across three teams' },
        { label: 'Completion', value: '75%', note: 'Illustrative progress' },
        { label: 'Open tasks', value: '12', note: 'Demo backlog' },
      ]"
      :key="metric.label"
      class="card example-panel"
    >
      <h2>{{ metric.label }}</h2>
      <strong class="example-metric">{{ metric.value }}</strong>
      <p class="muted">{{ metric.note }}</p>
    </section>
  </div>
  <div class="example-grid">
    <article class="card example-panel">
      <div class="example-row">
        <span class="badge badge-success">In progress</span
        ><span class="muted">Sample project</span>
      </div>
      <h2>Workspace redesign</h2>
      <p>A project summary with a status, owner, progress indicator, and one primary action.</p>
      <label for="project-progress">Project completion: 75%</label
      ><progress id="project-progress" class="progress progress-primary" value="75" max="100">
        75%</progress
      ><button class="btn btn-primary" :aria-pressed="saved" @click="saved = !saved">
        {{ saved ? 'Saved to collection' : 'Save project' }}
      </button>
      <p role="status">
        {{ saved ? 'Saved for this visit only.' : 'This example does not write to the database.' }}
      </p>
    </article>
    <article class="card example-panel">
      <span class="avatar-tile">JD</span>
      <h2>Jamie Dela Cruz</h2>
      <p class="muted">Product designer · Sample profile</p>
      <p>
        Use a profile card for an owner, team member, or contact. Connect the action to a meaningful
        destination.
      </p>
      <RouterLink class="btn btn-outline" to="/settings">View your settings</RouterLink>
    </article>
    <article class="card example-panel example-feature">
      <p class="eyebrow">BUILD SOMETHING YOURS</p>
      <h2>A familiar foundation.</h2>
      <p>
        Rust handles your API. Vue handles the interface. Start with the walkthrough and make the
        next page your own.
      </p>
      <RouterLink class="btn btn-primary" to="/guide">Open getting started</RouterLink>
    </article>
    <article class="card example-panel example-empty">
      <h2>No archived projects</h2>
      <p class="muted">
        An empty state should explain what belongs here and provide a useful next step.
      </p>
      <RouterLink class="btn btn-outline" to="/examples/tables">Explore sample projects</RouterLink>
    </article>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import AppDialog from '../../components/AppDialog.vue'
const message = ref(''),
  dialog = ref(false),
  panel = ref('Overview')
function confirmExample() {
  message.value = 'Example confirmed.'
  dialog.value = false
}
</script>
<template>
  <div class="page-heading">
    <div>
      <p class="eyebrow">UI EXAMPLES / 01</p>
      <h1>Elements & buttons</h1>
      <p class="muted">
        Reusable daisyUI building blocks. All actions on this page are local demos.
      </p>
    </div>
  </div>
  <div class="example-grid">
    <section class="card example-panel">
      <h2>Button styles</h2>
      <p class="muted">Choose a clear primary action for each section.</p>
      <div class="example-row">
        <button
          v-for="variant in ['primary', 'secondary', 'accent', 'neutral', 'ghost', 'outline']"
          :key="variant"
          class="btn"
          :class="`btn-${variant}`"
          @click="message = `${variant} button clicked.`"
        >
          {{ variant }}
        </button>
      </div>
      <div class="example-row">
        <button class="btn btn-sm" @click="message = 'Small button clicked.'">Small</button
        ><button class="btn" disabled>Disabled</button
        ><button class="btn" disabled>
          <span class="loading loading-spinner loading-xs" aria-hidden="true"></span>Loading</button
        ><button class="btn btn-primary" @click="dialog = true">Open dialog</button>
      </div>
      <p role="status" class="example-status">
        {{ message || 'Try a button to see its feedback here.' }}
      </p>
      <code>class="btn btn-primary"</code>
    </section>
    <section class="card example-panel">
      <h2>Badges & feedback</h2>
      <div class="example-row">
        <span class="badge badge-success">Active</span
        ><span class="badge badge-warning">Pending</span
        ><span class="badge badge-error">Failed</span><span class="badge badge-outline">Draft</span>
      </div>
      <div class="alert alert-info">Info: these are sample states.</div>
      <div class="alert alert-success">Success: your example is ready.</div>
      <div class="alert alert-warning">Warning: review before publishing.</div>
      <div class="alert alert-error">Error: please try again.</div>
    </section>
    <section class="card example-panel">
      <h2>Content switcher</h2>
      <div class="example-row" role="group" aria-label="Example content">
        <button
          v-for="item in ['Overview', 'Activity', 'Details']"
          :key="item"
          class="btn btn-sm"
          :class="{ 'btn-primary': panel === item }"
          :aria-pressed="panel === item"
          @click="panel = item"
        >
          {{ item }}
        </button>
      </div>
      <p role="status">
        {{ panel }}:
        {{
          {
            Overview: 'A quick summary of your workspace.',
            Activity: 'No recent activity in this demo.',
            Details: 'Replace this content with your own feature.',
          }[panel]
        }}
      </p>
      <label for="example-progress">Example progress: 65%</label
      ><progress id="example-progress" class="progress progress-primary" value="65" max="100">
        65%
      </progress>
    </section>
    <section class="card example-panel">
      <h2>Disclosure & empty state</h2>
      <details class="example-disclosure">
        <summary>Where do these styles live?</summary>
        <p>
          Theme tokens and shared layout styles live in resources/css/app.css. Page examples live in
          resources/js/pages/examples.
        </p>
      </details>
      <div class="example-empty">
        <h3>No items yet</h3>
        <p class="muted">Explain the next step when a collection is empty.</p>
        <button
          class="btn btn-outline"
          @click="message = 'Create action selected. Connect your own form here.'"
        >
          Create example
        </button>
      </div>
    </section>
  </div>
  <AppDialog v-if="dialog" labelledby="example-dialog-title" @close="dialog = false"
    ><h2 id="example-dialog-title">A reusable dialog</h2>
    <p>Native dialog focus management, Escape to close, and a clearly labelled action.</p>
    <div class="modal-actions">
      <button class="btn" @click="dialog = false">Cancel</button
      ><button class="btn btn-primary" @click="confirmExample">Confirm example</button>
    </div></AppDialog
  >
</template>

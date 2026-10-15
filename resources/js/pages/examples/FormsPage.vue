<script setup>
import { reactive, ref } from 'vue'
const defaults = () => ({
  name: '',
  email: '',
  team: 'Design',
  notes: '',
  notifications: true,
  visibility: 'Private',
  capacity: 5,
})
const form = reactive(defaults()),
  errors = ref({}),
  message = ref('')
function submit() {
  errors.value = {}
  message.value = ''
  if (form.name.trim().length < 2) errors.value.name = 'Enter at least two characters.'
  if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(form.email))
    errors.value.email = 'Enter a valid email address.'
  if (Object.keys(errors.value).length) {
    message.value = 'Please check the highlighted fields.'
    return
  }
  message.value = `Demo saved for ${form.name.trim()}. Nothing was sent to the server.`
}
function reset() {
  Object.assign(form, defaults())
  errors.value = {}
  message.value = ''
}
</script>
<template>
  <div class="page-heading">
    <div>
      <p class="eyebrow">UI EXAMPLES / 04</p>
      <h1>Forms</h1>
      <p class="muted">
        Labelled fields, validation, and selection controls. This demo keeps data in memory only.
      </p>
    </div>
  </div>
  <form
    class="card example-panel example-form"
    novalidate
    @submit.prevent="submit"
    @reset.prevent="reset"
  >
    <h2>Project preferences</h2>
    <p role="status">{{ message }}</p>
    <div class="example-grid">
      <label class="example-field"
        >Project name<input
          v-model="form.name"
          class="input"
          required
          maxlength="80"
          :aria-invalid="!!errors.name"
          :aria-describedby="errors.name ? 'demo-name-error' : undefined"
        /><span v-if="errors.name" id="demo-name-error" class="field-error">{{
          errors.name
        }}</span></label
      >
      <label class="example-field"
        >Contact email<input
          v-model="form.email"
          class="input"
          type="email"
          required
          maxlength="254"
          :aria-invalid="!!errors.email"
          :aria-describedby="errors.email ? 'demo-email-error' : undefined"
        /><span v-if="errors.email" id="demo-email-error" class="field-error">{{
          errors.email
        }}</span></label
      >
      <label class="example-field"
        >Team<select v-model="form.team" class="select">
          <option>Design</option>
          <option>Engineering</option>
          <option>Operations</option>
        </select></label
      >
      <label class="example-field"
        >Reference code<input class="input" value="DEMO-001" readonly
      /></label>
    </div>
    <label class="example-field"
      >Project notes<textarea
        v-model="form.notes"
        class="textarea"
        rows="3"
        maxlength="500"
        placeholder="A short description"
      ></textarea>
    </label>
    <fieldset>
      <legend>Visibility</legend>
      <div class="example-row">
        <label v-for="value in ['Private', 'Team']" :key="value" class="example-choice"
          ><input
            v-model="form.visibility"
            class="radio radio-sm"
            type="radio"
            name="visibility"
            :value="value"
          />{{ value }}</label
        >
      </div>
    </fieldset>
    <label class="example-choice"
      ><input
        v-model="form.notifications"
        class="toggle toggle-primary toggle-sm"
        type="checkbox"
      />Email notifications</label
    >
    <label class="example-field"
      >Team capacity: {{ form.capacity
      }}<input
        v-model.number="form.capacity"
        class="range range-primary range-sm"
        type="range"
        min="1"
        max="10"
    /></label>
    <div class="example-row">
      <button class="btn btn-primary" type="submit">Save demo</button
      ><button class="btn btn-outline" type="reset">Reset form</button>
    </div>
  </form>
</template>

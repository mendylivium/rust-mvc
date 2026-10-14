<script setup>
defineProps({ message: String, success: Boolean })
</script>
<template>
  <div v-if="message" :role="success ? 'status' : 'alert'" class="feedback" :class="{ success }">
    {{ message }}
  </div>
</template>

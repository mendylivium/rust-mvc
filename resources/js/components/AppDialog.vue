<script setup>
import { onMounted, onBeforeUnmount, ref } from 'vue'
defineProps({ labelledby: String, busy: Boolean })
const emit = defineEmits(['close'])
const dialog = ref(null)
let previousFocus
onMounted(() => {
  previousFocus = document.activeElement
  dialog.value.showModal()
})
onBeforeUnmount(() => {
  dialog.value?.close()
  previousFocus?.focus()
})
</script>
<template>
  <dialog
    ref="dialog"
    class="modal"
    :aria-labelledby="labelledby"
    @cancel.prevent="!busy && emit('close')"
  >
    <div class="modal-box"><slot /></div>
  </dialog>
</template>

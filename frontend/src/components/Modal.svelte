<script>
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();

  export let isVisible = false;
</script>

<style>
  .modal {
    position: absolute;
    top: 0;
    left: 0;
    min-width: 100%;
    height: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
    background-color: rgba(0, 0, 0, 0.8);
  }
</style>

{#if isVisible}
  <div class="modal">
    <slot />
  </div>
{/if}

<script lang="ts">
  import "../app.css";
  let { children } = $props();
</script>

<div class="h-screen w-screen overflow-hidden flex flex-col bg-[var(--color-canvas)] text-[var(--color-text-body)] transition-colors duration-150">
  {@render children?.()}
</div>

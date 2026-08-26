<script lang="ts">
  import "../app.css";
  let { children } = $props();
</script>

<div class="h-screen w-screen overflow-hidden flex flex-col bg-[#0d0e11] text-[#e2e4e9]">
  {@render children?.()}
</div>

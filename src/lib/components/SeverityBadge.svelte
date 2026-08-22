<script lang="ts">
  import type { Severity } from "$lib/types";

  let { severity, count = undefined }: { severity: Severity; count?: number } = $props();

  const config = {
    critical: {
      bg: "bg-red-950/40 text-red-300 border-red-800/50 hover:bg-red-950/60",
      dot: "bg-red-400",
      label: "Critical",
    },
    high: {
      bg: "bg-orange-950/40 text-orange-300 border-orange-800/50 hover:bg-orange-950/60",
      dot: "bg-orange-400",
      label: "High",
    },
    medium: {
      bg: "bg-amber-950/40 text-amber-300 border-amber-800/50 hover:bg-amber-950/60",
      dot: "bg-amber-400",
      label: "Medium",
    },
    low: {
      bg: "bg-blue-950/40 text-blue-300 border-blue-800/50 hover:bg-blue-950/60",
      dot: "bg-blue-400",
      label: "Low",
    },
    info: {
      bg: "bg-neutral-800/60 text-neutral-300 border-neutral-700/60 hover:bg-neutral-800",
      dot: "bg-neutral-400",
      label: "Info",
    },
  };
</script>

<span
  class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md text-[11px] font-medium border transition-colors {config[severity].bg}"
>
  <span class="w-1.5 h-1.5 rounded-full {config[severity].dot}"></span>
  <span>{config[severity].label}</span>
  {#if count !== undefined}
    <span class="ml-1 px-1.5 py-0.2 rounded bg-black/30 text-[10px] font-mono">
      {count}
    </span>
  {/if}
</span>


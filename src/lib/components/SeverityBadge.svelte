<script lang="ts">
  import type { Severity } from "$lib/types";

  let { severity, count = undefined }: { severity: Severity; count?: number } = $props();

  const config = {
    critical: {
      bg: "bg-rose-500/15 text-rose-400 border-rose-500/30",
      dot: "bg-rose-500",
      label: "CRITICAL",
    },
    high: {
      bg: "bg-orange-500/15 text-orange-400 border-orange-500/30",
      dot: "bg-orange-500",
      label: "HIGH",
    },
    medium: {
      bg: "bg-amber-500/15 text-amber-400 border-amber-500/30",
      dot: "bg-amber-500",
      label: "MEDIUM",
    },
    low: {
      bg: "bg-blue-500/15 text-blue-400 border-blue-500/30",
      dot: "bg-blue-500",
      label: "LOW",
    },
    info: {
      bg: "bg-slate-500/15 text-slate-400 border-slate-500/30",
      dot: "bg-slate-400",
      label: "INFO",
    },
  };
</script>

<span
  class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-md text-xs font-semibold uppercase tracking-wider border {config[severity].bg}"
>
  <span class="w-1.5 h-1.5 rounded-full {config[severity].dot}"></span>
  <span>{config[severity].label}</span>
  {#if count !== undefined}
    <span class="ml-1 px-1.5 py-0.2 rounded bg-black/40 text-[10px] font-mono font-bold">
      {count}
    </span>
  {/if}
</span>

<script lang="ts">
  import type { Severity } from "$lib/types";

  let { severity, count = undefined }: { severity: Severity; count?: number } = $props();

  const config = {
    critical: {
      bg: "bg-rose-500/10 text-rose-400 border-rose-500/30 hover:border-rose-500/50",
      dot: "bg-rose-400 shadow-[0_0_8px_rgba(244,63,94,0.7)] animate-pulse",
      label: "CRITICAL",
    },
    high: {
      bg: "bg-orange-500/10 text-orange-400 border-orange-500/30 hover:border-orange-500/50",
      dot: "bg-orange-400 shadow-[0_0_6px_rgba(251,146,60,0.6)]",
      label: "HIGH",
    },
    medium: {
      bg: "bg-amber-500/10 text-amber-400 border-amber-500/30 hover:border-amber-500/50",
      dot: "bg-amber-400 shadow-[0_0_6px_rgba(245,158,11,0.6)]",
      label: "MEDIUM",
    },
    low: {
      bg: "bg-blue-500/10 text-blue-400 border-blue-500/30 hover:border-blue-500/50",
      dot: "bg-blue-400 shadow-[0_0_6px_rgba(96,165,250,0.6)]",
      label: "LOW",
    },
    info: {
      bg: "bg-slate-800/60 text-slate-300 border-slate-700/60 hover:border-slate-600",
      dot: "bg-slate-400",
      label: "INFO",
    },
  };
</script>

<span
  class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-[11px] font-mono font-bold uppercase tracking-wider border transition-colors {config[severity].bg}"
>
  <span class="w-1.5 h-1.5 rounded-full {config[severity].dot}"></span>
  <span>{config[severity].label}</span>
  {#if count !== undefined}
    <span class="ml-1 px-1.5 py-0.2 rounded-full bg-black/40 text-[10px] font-mono font-bold">
      {count}
    </span>
  {/if}
</span>


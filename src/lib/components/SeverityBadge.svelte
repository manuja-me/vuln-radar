<script lang="ts">
  import type { Severity } from "$lib/types";

  let { severity, count = undefined }: { severity: Severity | "pass"; count?: number } = $props();

  const config: Record<string, { border: string; bg: string; text: string; dot: string; label: string }> = {
    critical: {
      border: "border-red-500/40 dark:border-red-500/40",
      bg: "bg-red-500/5 dark:bg-red-950/30",
      text: "text-red-600 dark:text-red-400",
      dot: "bg-red-500",
      label: "CRITICAL",
    },
    high: {
      border: "border-orange-500/40 dark:border-orange-500/40",
      bg: "bg-orange-500/5 dark:bg-orange-950/30",
      text: "text-orange-600 dark:text-orange-400",
      dot: "bg-orange-500",
      label: "HIGH",
    },
    medium: {
      border: "border-amber-500/40 dark:border-amber-500/40",
      bg: "bg-amber-500/5 dark:bg-amber-950/30",
      text: "text-amber-600 dark:text-amber-400",
      dot: "bg-amber-500",
      label: "MEDIUM",
    },
    low: {
      border: "border-blue-500/40 dark:border-blue-500/40",
      bg: "bg-blue-500/5 dark:bg-blue-950/30",
      text: "text-blue-600 dark:text-blue-400",
      dot: "bg-blue-500",
      label: "LOW",
    },
    info: {
      border: "border-zinc-400/40 dark:border-zinc-700/50",
      bg: "bg-zinc-500/5 dark:bg-zinc-900/40",
      text: "text-zinc-600 dark:text-zinc-400",
      dot: "bg-zinc-400 dark:bg-zinc-500",
      label: "INFO",
    },
    pass: {
      border: "border-emerald-500/40 dark:border-emerald-500/40",
      bg: "bg-emerald-500/5 dark:bg-emerald-950/30",
      text: "text-emerald-600 dark:text-emerald-400",
      dot: "bg-emerald-500",
      label: "PASS",
    },
  };
</script>

<span
  class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-none border text-[10px] font-mono font-bold tracking-wider uppercase transition-colors {config[severity]?.border ?? config.info.border} {config[severity]?.bg ?? config.info.bg} {config[severity]?.text ?? config.info.text}"
>
  <span class="w-1.5 h-1.5 rounded-none {config[severity]?.dot ?? config.info.dot}"></span>
  <span>{config[severity]?.label ?? severity}</span>
  {#if count !== undefined}
    <span class="ml-1 pl-1.5 border-l border-current/30 text-[10px] font-mono tabular-nums font-semibold">
      {count}
    </span>
  {/if}
</span>

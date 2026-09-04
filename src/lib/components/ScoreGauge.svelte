<script lang="ts">
  let { score = 100 }: { score: number } = $props();

  const grade = $derived(
    score >= 95 ? "A+" : score >= 85 ? "A" : score >= 75 ? "B" : score >= 60 ? "C" : score >= 40 ? "D" : "F"
  );

  const colorConfig = $derived(
    score >= 85
      ? {
          signalBar: "bg-emerald-500",
          text: "text-emerald-600 dark:text-emerald-400",
          borderAccent: "border-l-emerald-500",
          label: "HARDENED POSTURE",
          code: "SEC-01",
          sub: "Standard security headers & controls verified. No high-risk exposures.",
        }
      : score >= 70
      ? {
          signalBar: "bg-blue-500",
          text: "text-blue-600 dark:text-blue-400",
          borderAccent: "border-l-blue-500",
          label: "MODERATE POSTURE",
          code: "SEC-02",
          sub: "Standard defensive controls present; minor gaps in header or cookie hygiene.",
        }
      : score >= 50
      ? {
          signalBar: "bg-amber-500",
          text: "text-amber-600 dark:text-amber-400",
          borderAccent: "border-l-amber-500",
          label: "ELEVATED RISK",
          code: "SEC-03",
          sub: "Multiple critical protections missing. Outdated libraries or open exposures.",
        }
      : {
          signalBar: "bg-red-500",
          text: "text-red-600 dark:text-red-400",
          borderAccent: "border-l-red-500",
          label: "CRITICAL EXPOSURE",
          code: "SEC-04",
          sub: "High-impact vulnerabilities detected. Immediate remediation required.",
        }
  );
</script>

<div class="flex flex-col sm:flex-row sm:items-center justify-between p-5 bg-[var(--color-surface)] border border-[var(--color-hairline)] border-l-4 {colorConfig.borderAccent} rounded-none relative gap-5 transition-colors">
  <!-- Primary Typographic Numeric Score Block -->
  <div class="flex items-baseline gap-2">
    <span class="text-5xl sm:text-6xl font-black font-sans tracking-tighter tabular-nums text-[var(--color-text-headline)]">
      {score}
    </span>
    <div class="flex flex-col">
      <span class="text-xs font-mono text-[var(--color-text-muted)] uppercase tracking-wider font-semibold">/ 100</span>
      <span class="text-[11px] font-mono font-bold {colorConfig.text} uppercase tracking-widest mt-0.5">
        GRADE {grade}
      </span>
    </div>
  </div>

  <!-- Calibrated Metric Bar & Status Details -->
  <div class="flex-1 flex flex-col justify-center min-w-0 sm:border-l sm:border-[var(--color-hairline)] sm:pl-6">
    <div class="flex items-center justify-between mb-1.5">
      <span class="text-[11px] font-mono font-bold tracking-widest text-[var(--color-text-headline)] uppercase">
        {colorConfig.label}
      </span>
      <span class="text-[10px] font-mono text-[var(--color-text-muted)] tracking-wider">
        [{colorConfig.code}]
      </span>
    </div>

    <!-- Segmented 10-Tick Progress Scale -->
    <div class="w-full bg-[var(--color-hairline)] h-2 flex gap-0.5 p-0.5 rounded-none overflow-hidden my-1">
      {#each Array(10) as _, i}
        <div
          class="flex-1 h-full transition-all duration-300 {i * 10 < score ? colorConfig.signalBar : 'bg-transparent'}"
        ></div>
      {/each}
    </div>

    <p class="text-xs text-[var(--color-text-muted)] leading-relaxed mt-1">
      {colorConfig.sub}
    </p>
  </div>
</div>

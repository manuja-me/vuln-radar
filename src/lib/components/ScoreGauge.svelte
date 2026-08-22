<script lang="ts">
  let { score = 100 }: { score: number } = $props();

  const radius = 44;
  const circumference = 2 * Math.PI * radius;
  const strokeDashoffset = $derived(circumference - (score / 100) * circumference);

  const grade = $derived(
    score >= 95 ? "A+" : score >= 85 ? "A" : score >= 75 ? "B" : score >= 60 ? "C" : score >= 40 ? "D" : "F"
  );

  const colorConfig = $derived(
    score >= 85
      ? {
          stroke: "#10b981",
          text: "text-emerald-400",
          glow: "drop-shadow-[0_0_12px_rgba(16,185,129,0.35)]",
          bg: "bg-emerald-500/10",
          border: "border-emerald-500/30",
          label: "Hardened Posture",
          sub: "No critical misconfigurations or high-risk vulnerabilities detected.",
        }
      : score >= 70
      ? {
          stroke: "#06b6d4",
          text: "text-cyan-400",
          glow: "drop-shadow-[0_0_12px_rgba(6,182,212,0.35)]",
          bg: "bg-cyan-500/10",
          border: "border-cyan-500/30",
          label: "Moderate Posture",
          sub: "Standard security controls present; minor header or cookie risks found.",
        }
      : score >= 50
      ? {
          stroke: "#f59e0b",
          text: "text-amber-400",
          glow: "drop-shadow-[0_0_12px_rgba(245,158,11,0.35)]",
          bg: "bg-amber-500/10",
          border: "border-amber-500/30",
          label: "Elevated Risk",
          sub: "Multiple security defenses missing or outdated dependencies detected.",
        }
      : {
          stroke: "#f43f5e",
          text: "text-rose-400",
          glow: "drop-shadow-[0_0_12px_rgba(244,63,94,0.4)]",
          bg: "bg-rose-500/10",
          border: "border-rose-500/30",
          label: "Critical Exposure",
          sub: "High-impact vulnerabilities or secret leaks require immediate remediation.",
        }
  );
</script>

<div class="flex items-center gap-5 p-5 bg-slate-900/60 border border-slate-800 rounded-2xl relative overflow-hidden backdrop-blur-md">
  <!-- Subtle background glow -->
  <div class="absolute -right-8 -top-8 w-28 h-28 rounded-full blur-3xl opacity-20 pointer-events-none {colorConfig.bg}"></div>

  <!-- Radial Gauge -->
  <div class="relative flex items-center justify-center w-28 h-28 flex-shrink-0">
    <svg class="w-full h-full -rotate-90 transform {colorConfig.glow}" viewBox="0 0 104 104">
      <!-- Background track -->
      <circle
        cx="52"
        cy="52"
        r={radius}
        class="text-slate-800/80"
        stroke-width="7"
        stroke="currentColor"
        fill="transparent"
      />
      <!-- Active gauge stroke -->
      <circle
        cx="52"
        cy="52"
        r={radius}
        stroke={colorConfig.stroke}
        stroke-width="7"
        stroke-dasharray={circumference}
        stroke-dashoffset={strokeDashoffset}
        stroke-linecap="round"
        fill="transparent"
        class="transition-all duration-1000 ease-out"
      />
    </svg>
    <div class="absolute flex flex-col items-center justify-center">
      <span class="text-3xl font-black font-mono tracking-tight {colorConfig.text}">{score}</span>
      <span class="text-[9px] font-bold uppercase tracking-widest text-slate-500 font-mono">/ 100</span>
    </div>
  </div>

  <!-- Posture Information -->
  <div class="flex flex-col min-w-0">
    <div class="flex items-center gap-2">
      <span class="text-[11px] uppercase tracking-wider text-slate-400 font-bold font-mono">Security Health</span>
      <span class="px-2 py-0.5 text-xs font-mono font-bold rounded-md border {colorConfig.bg} {colorConfig.text} {colorConfig.border}">
        Grade {grade}
      </span>
    </div>
    <div class="text-base font-extrabold text-white mt-1 truncate">{colorConfig.label}</div>
    <p class="text-xs text-slate-400 mt-1 leading-relaxed max-w-[240px]">
      {colorConfig.sub}
    </p>
  </div>
</div>


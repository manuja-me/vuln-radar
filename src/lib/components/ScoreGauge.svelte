<script lang="ts">
  let { score = 100 }: { score: number } = $props();

  const radius = 40;
  const circumference = 2 * Math.PI * radius;
  const strokeDashoffset = $derived(circumference - (score / 100) * circumference);

  const grade = $derived(
    score >= 95 ? "A+" : score >= 85 ? "A" : score >= 75 ? "B" : score >= 60 ? "C" : score >= 40 ? "D" : "F"
  );

  const colorConfig = $derived(
    score >= 85
      ? {
          stroke: "#34d399",
          text: "text-emerald-400",
          bg: "bg-emerald-950/40",
          border: "border-emerald-800/40",
          label: "Hardened",
          sub: "No critical misconfigurations or high-risk vulnerabilities detected.",
        }
      : score >= 70
      ? {
          stroke: "#60a5fa",
          text: "text-blue-400",
          bg: "bg-blue-950/40",
          border: "border-blue-800/40",
          label: "Moderate",
          sub: "Standard security controls present; minor risks or header gaps found.",
        }
      : score >= 50
      ? {
          stroke: "#fbbf24",
          text: "text-amber-400",
          bg: "bg-amber-950/40",
          border: "border-amber-800/40",
          label: "Elevated Risk",
          sub: "Multiple security defenses missing or outdated components detected.",
        }
      : {
          stroke: "#f87171",
          text: "text-red-400",
          bg: "bg-red-950/40",
          border: "border-red-800/40",
          label: "Critical Exposure",
          sub: "High-impact vulnerabilities require immediate remediation.",
        }
  );
</script>

<div class="flex items-center gap-5 p-4 bg-[#202020] border border-[#2e2e2e] rounded-xl relative overflow-hidden">
  <!-- Radial Gauge -->
  <div class="relative flex items-center justify-center w-24 h-24 flex-shrink-0">
    <svg class="w-full h-full -rotate-90 transform" viewBox="0 0 100 100">
      <!-- Background track -->
      <circle
        cx="50"
        cy="50"
        r={radius}
        class="text-neutral-800"
        stroke-width="6"
        stroke="currentColor"
        fill="transparent"
      />
      <!-- Active gauge stroke -->
      <circle
        cx="50"
        cy="50"
        r={radius}
        stroke={colorConfig.stroke}
        stroke-width="6"
        stroke-dasharray={circumference}
        stroke-dashoffset={strokeDashoffset}
        stroke-linecap="round"
        fill="transparent"
        class="transition-all duration-700 ease-out"
      />
    </svg>
    <div class="absolute flex flex-col items-center justify-center">
      <span class="text-2xl font-bold font-mono text-white">{score}</span>
      <span class="text-[9px] text-neutral-400 font-mono">/ 100</span>
    </div>
  </div>

  <!-- Posture Information -->
  <div class="flex flex-col min-w-0">
    <div class="flex items-center gap-2">
      <span class="text-[11px] text-neutral-400 font-medium">Security Score</span>
      <span class="px-2 py-0.5 text-xs font-mono font-medium rounded-md border {colorConfig.bg} {colorConfig.text} {colorConfig.border}">
        Grade {grade}
      </span>
    </div>
    <div class="text-sm font-semibold text-white mt-1 truncate">{colorConfig.label}</div>
    <p class="text-xs text-neutral-400 mt-0.5 leading-relaxed max-w-[240px]">
      {colorConfig.sub}
    </p>
  </div>
</div>


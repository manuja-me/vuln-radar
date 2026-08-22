<script lang="ts">
  let { score = 100 }: { score: number } = $props();

  const radius = 42;
  const circumference = 2 * Math.PI * radius;
  const strokeDashoffset = $derived(circumference - (score / 100) * circumference);

  const grade = $derived(
    score >= 90 ? "A" : score >= 80 ? "B" : score >= 65 ? "C" : score >= 50 ? "D" : "F"
  );

  const colorConfig = $derived(
    score >= 85
      ? { stroke: "#10b981", text: "text-emerald-400", bg: "bg-emerald-500/10", border: "border-emerald-500/30", label: "Secure / Low Risk" }
      : score >= 70
      ? { stroke: "#06b6d4", text: "text-cyan-400", bg: "bg-cyan-500/10", border: "border-cyan-500/30", label: "Moderate Posture" }
      : score >= 50
      ? { stroke: "#f59e0b", text: "text-amber-400", bg: "bg-amber-500/10", border: "border-amber-500/30", label: "Attention Needed" }
      : { stroke: "#f43f5e", text: "text-rose-400", bg: "bg-rose-500/10", border: "border-rose-500/30", label: "Critical Weaknesses" }
  );
</script>

<div class="flex items-center gap-5 p-5 bg-slate-900/60 border border-slate-800 rounded-xl">
  <div class="relative flex items-center justify-center w-28 h-28">
    <svg class="w-full h-full -rotate-90 transform" viewBox="0 0 100 100">
      <!-- Background circle -->
      <circle
        cx="50"
        cy="50"
        r={radius}
        class="text-slate-800"
        stroke-width="8"
        stroke="currentColor"
        fill="transparent"
      />
      <!-- Progress circle -->
      <circle
        cx="50"
        cy="50"
        r={radius}
        stroke={colorConfig.stroke}
        stroke-width="8"
        stroke-dasharray={circumference}
        stroke-dashoffset={strokeDashoffset}
        stroke-linecap="round"
        fill="transparent"
        class="transition-all duration-700 ease-out"
      />
    </svg>
    <div class="absolute flex flex-col items-center justify-center">
      <span class="text-3xl font-extrabold tracking-tight {colorConfig.text}">{score}</span>
      <span class="text-[11px] font-semibold uppercase text-slate-400">Score</span>
    </div>
  </div>

  <div class="flex flex-col">
    <div class="flex items-center gap-2">
      <span class="text-xs uppercase tracking-wider text-slate-400 font-semibold">Security Health Rating</span>
      <span class="px-2 py-0.5 text-xs font-bold rounded border {colorConfig.bg} {colorConfig.text} {colorConfig.border}">
        Grade {grade}
      </span>
    </div>
    <div class="text-lg font-bold text-slate-100 mt-1">{colorConfig.label}</div>
    <p class="text-xs text-slate-400 mt-1 max-w-[220px]">
      Calculated from headers, cookies, third-party libraries, and client security indicators.
    </p>
  </div>
</div>

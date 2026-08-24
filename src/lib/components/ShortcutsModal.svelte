<script lang="ts">
  import { X, Keyboard, Command } from "lucide-svelte";

  let {
    isOpen = false,
    onClose,
  }: {
    isOpen: boolean;
    onClose: () => void;
  } = $props();

  const shortcuts = [
    { key: "⌘ / Ctrl + K", description: "Focus target URL audit input" },
    { key: "⌘ / Ctrl + B", description: "Open Batch Fleet Scanner" },
    { key: "⌘ / Ctrl + H", description: "Open Scan History Archive" },
    { key: "⌘ / Ctrl + M", description: "Open Continuous Watchdog" },
    { key: "⌘ / Ctrl + O", description: "Open Scan Parameters & Headers" },
    { key: "⌘ / Ctrl + E", description: "Export Report (Markdown, JSON, CSV, cURL)" },
    { key: "⌘ / Ctrl + P", description: "Open Executive PDF / Print Report" },
    { key: "Esc", description: "Close any open dialog or modal" },
  ];
</script>

{#if isOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4"
    onclick={(e) => {
      if (e.target === e.currentTarget) onClose();
    }}
    role="dialog"
    aria-modal="true"
  >
    <div class="bg-[#202020] border border-[#333333] rounded-xl w-full max-w-md shadow-2xl overflow-hidden text-[#e3e2e0]">
      <!-- Header -->
      <div class="p-4 border-b border-[#2e2e2e] flex items-center justify-between bg-[#191919]">
        <div class="flex items-center gap-2">
          <Keyboard class="w-4 h-4 text-neutral-300" />
          <h2 class="text-sm font-semibold text-white">Keyboard Shortcuts</h2>
        </div>
        <button
          type="button"
          onclick={onClose}
          class="p-1 text-neutral-400 hover:text-white hover:bg-[#282828] rounded-lg transition-colors cursor-pointer"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Shortcuts list -->
      <div class="p-4 space-y-2 max-h-[60vh] overflow-y-auto">
        {#each shortcuts as sc}
          <div class="flex items-center justify-between py-1.5 border-b border-[#282828] last:border-none">
            <span class="text-xs text-neutral-300">{sc.description}</span>
            <kbd class="px-2 py-0.5 bg-[#161616] border border-[#303030] rounded text-[11px] font-mono text-neutral-200 shadow-xs">
              {sc.key}
            </kbd>
          </div>
        {/each}
      </div>

      <!-- Footer -->
      <div class="p-3 border-t border-[#2e2e2e] bg-[#191919] text-center text-[11px] text-neutral-500 font-mono">
        Press <kbd class="px-1.5 py-0.5 bg-[#252525] border border-[#383838] rounded text-[10px] text-neutral-300">Esc</kbd> to close
      </div>
    </div>
  </div>
{/if}

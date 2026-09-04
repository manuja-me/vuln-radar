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
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4"
    onclick={(e) => {
      if (e.target === e.currentTarget) onClose();
    }}
    onkeydown={(e) => {
      if (e.key === "Escape") onClose();
    }}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <div class="bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none w-full max-w-md shadow-2xl overflow-hidden text-[var(--color-text-body)]">
      <!-- Header -->
      <div class="p-4 border-b border-[var(--color-hairline)] flex items-center justify-between bg-[var(--color-surface)]">
        <div class="flex items-center gap-2">
          <Keyboard class="w-4 h-4 text-[var(--color-signal-red)]" />
          <h2 class="text-xs font-black text-[var(--color-text-headline)] font-mono uppercase tracking-tight">Keyboard Shortcuts</h2>
        </div>
        <button
          type="button"
          onclick={onClose}
          class="p-1 text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)] rounded-none transition-colors cursor-pointer"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Shortcuts list -->
      <div class="p-4 space-y-1.5 max-h-[60vh] overflow-y-auto">
        {#each shortcuts as sc}
          <div class="flex items-center justify-between py-1.5 border-b border-[var(--color-hairline)] last:border-none">
            <span class="text-xs font-mono text-[var(--color-text-body)]">{sc.description}</span>
            <kbd class="px-2 py-0.5 bg-[var(--color-canvas)] border border-[var(--color-hairline)] rounded-none text-[11px] font-mono font-bold text-[var(--color-text-headline)]">
              {sc.key}
            </kbd>
          </div>
        {/each}
      </div>

      <!-- Footer -->
      <div class="p-3 border-t border-[var(--color-hairline)] bg-[var(--color-surface)] text-center text-[11px] text-[var(--color-text-muted)] font-mono uppercase">
        Press <kbd class="px-1.5 py-0.5 bg-[var(--color-canvas)] border border-[var(--color-hairline)] rounded-none text-[10px] text-[var(--color-text-headline)]">Esc</kbd> to close
      </div>
    </div>
  </div>
{/if}

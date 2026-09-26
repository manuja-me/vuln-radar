<script lang="ts">
  import WordlistSelector from "./WordlistSelector.svelte";
  import type { WordlistConfig } from "$lib/types";
  import { X, ListFilter, Check } from "lucide-svelte";

  let {
    isOpen = false,
    config,
    onApply,
    onClose,
  }: {
    isOpen: boolean;
    config: WordlistConfig;
    onApply: (newConfig: WordlistConfig) => void;
    onClose: () => void;
  } = $props();

  let tempConfig = $state<WordlistConfig>({
    selectedIds: ["common_paths", "sensitive_files", "api_documentation"],
    customPaths: [],
    activePreset: "balanced",
  });

  // Sync tempConfig when modal opens
  let previousIsOpen = false;
  $effect(() => {
    if (isOpen && !previousIsOpen) {
      tempConfig = {
        selectedIds: [...(config?.selectedIds || ["common_paths", "sensitive_files"])],
        customPaths: [...(config?.customPaths || [])],
        activePreset: config?.activePreset || "balanced",
      };
    }
    previousIsOpen = isOpen;
  });

  function handleSave() {
    onApply(tempConfig);
    onClose();
  }
</script>

{#if isOpen}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 p-4 animate-fade-in"
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
    <div
      class="bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none w-full max-w-4xl max-h-[90vh] flex flex-col shadow-2xl overflow-hidden text-[var(--color-text-body)]"
    >
      <!-- Modal Header -->
      <div class="px-5 py-3.5 border-b border-[var(--color-hairline)] flex items-center justify-between bg-[var(--color-surface)]">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-none bg-[var(--color-canvas)] border border-[var(--color-hairline)] flex items-center justify-center text-[var(--color-signal-red)]">
            <ListFilter class="w-4 h-4" />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="text-xs font-black text-[var(--color-text-headline)] uppercase tracking-tight font-mono">
                Wordlist & Path Discovery Engine
              </h2>
              <span class="px-1.5 py-0.2 text-[10px] bg-[var(--color-canvas)] text-[var(--color-text-muted)] rounded-none font-mono border border-[var(--color-hairline)] uppercase font-bold">
                RECON PROFILES
              </span>
            </div>
            <p class="text-[11px] text-[var(--color-text-muted)] font-mono uppercase">
              Configure directories, common paths, sensitive files, and custom .txt lists for target audits
            </p>
          </div>
        </div>

        <button
          type="button"
          onclick={onClose}
          class="p-1.5 text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)] rounded-none transition-colors cursor-pointer"
          aria-label="Close wordlist dialog"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Modal Body (Scrollable) -->
      <div class="flex-1 overflow-y-auto p-5">
        <WordlistSelector
          bind:config={tempConfig}
        />
      </div>

      <!-- Modal Footer -->
      <div class="px-5 py-3 border-t border-[var(--color-hairline)] flex items-center justify-between bg-[var(--color-surface)] flex-shrink-0">
        <div class="text-[11px] font-mono text-[var(--color-text-muted)]">
          Selected: <strong class="text-[var(--color-text-headline)]">{tempConfig.selectedIds.length}</strong> list(s)
          {#if tempConfig.customPaths.length > 0}
            | <strong class="text-[var(--color-text-headline)]">{tempConfig.customPaths.length}</strong> custom
          {/if}
        </div>

        <div class="flex items-center gap-2">
          <button
            type="button"
            onclick={onClose}
            class="px-3 py-1.5 text-xs font-mono font-bold uppercase bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] border border-[var(--color-hairline)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] transition-colors cursor-pointer"
          >
            CANCEL
          </button>
          <button
            type="button"
            onclick={handleSave}
            class="px-4 py-1.5 text-xs font-mono font-bold uppercase bg-[var(--color-text-headline)] text-[var(--color-canvas)] hover:opacity-90 flex items-center gap-1.5 transition-colors cursor-pointer"
          >
            <Check class="w-3.5 h-3.5" />
            <span>APPLY CONFIGURATION</span>
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

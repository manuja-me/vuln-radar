<script lang="ts">
  import { CheckCircle2, AlertCircle, Info, X } from "lucide-svelte";

  let {
    message = "",
    type = "info",
    visible = false,
    onDismiss,
  }: {
    message: string;
    type?: "success" | "error" | "info";
    visible: boolean;
    onDismiss: () => void;
  } = $props();
</script>

{#if visible}
  <div
    class="fixed bottom-6 right-6 z-50 flex items-center gap-2.5 px-4 py-2.5 bg-[var(--color-surface)] border border-[var(--color-hairline)] {type === 'success' ? 'border-l-4 border-l-emerald-500' : type === 'error' ? 'border-l-4 border-l-[var(--color-signal-red)]' : 'border-l-4 border-l-[var(--color-hairline-strong)]'} rounded-none shadow-2xl text-xs font-mono text-[var(--color-text-headline)] uppercase animate-fade-in"
    role="status"
    aria-live="polite"
  >
    {#if type === "success"}
      <CheckCircle2 class="w-4 h-4 text-emerald-500 flex-shrink-0" />
    {:else if type === "error"}
      <AlertCircle class="w-4 h-4 text-[var(--color-signal-red)] flex-shrink-0" />
    {:else}
      <Info class="w-4 h-4 text-[var(--color-text-muted)] flex-shrink-0" />
    {/if}

    <span>{message}</span>

    <button
      type="button"
      onclick={onDismiss}
      class="p-0.5 text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] rounded-none transition-colors ml-2 cursor-pointer"
      title="Dismiss"
    >
      <X class="w-3.5 h-3.5" />
    </button>
  </div>
{/if}

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
    class="fixed bottom-6 right-6 z-50 flex items-center gap-2.5 px-3.5 py-2.5 bg-[#222222] border border-[#333333] rounded-xl shadow-2xl text-xs font-medium text-[#e3e2e0] animate-fade-in"
    role="status"
    aria-live="polite"
  >
    {#if type === "success"}
      <CheckCircle2 class="w-4 h-4 text-emerald-400 flex-shrink-0" />
    {:else if type === "error"}
      <AlertCircle class="w-4 h-4 text-red-400 flex-shrink-0" />
    {:else}
      <Info class="w-4 h-4 text-blue-400 flex-shrink-0" />
    {/if}

    <span>{message}</span>

    <button
      type="button"
      onclick={onDismiss}
      class="p-0.5 text-neutral-400 hover:text-white rounded transition-colors ml-1 cursor-pointer"
      title="Dismiss"
    >
      <X class="w-3 h-3" />
    </button>
  </div>
{/if}

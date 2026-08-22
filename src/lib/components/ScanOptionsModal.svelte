<script lang="ts">
  import type { ScanOptions } from "$lib/types";
  import { X, Settings2, Plus, Trash2, Shield, Globe, Clock, Sliders } from "lucide-svelte";

  let {
    isOpen = false,
    options = $bindable(),
    onClose,
  }: {
    isOpen: boolean;
    options: ScanOptions;
    onClose: () => void;
  } = $props();

  let headerRows = $state<{ key: string; value: string }[]>([]);
  let userAgentInput = $state(options.user_agent || "");
  let timeoutSeconds = $state(options.timeout_seconds || 15);
  let includeSubdomains = $state(options.include_subdomains ?? true);

  $effect(() => {
    if (isOpen) {
      headerRows = (options.custom_headers || []).map(([key, value]) => ({
        key,
        value,
      }));
      if (headerRows.length === 0) {
        headerRows = [{ key: "", value: "" }];
      }
      userAgentInput = options.user_agent || "";
      timeoutSeconds = options.timeout_seconds || 15;
      includeSubdomains = options.include_subdomains ?? true;
    }
  });

  function addHeaderRow() {
    headerRows = [...headerRows, { key: "", value: "" }];
  }

  function removeHeaderRow(index: number) {
    headerRows = headerRows.filter((_, i) => i !== index);
    if (headerRows.length === 0) {
      headerRows = [{ key: "", value: "" }];
    }
  }

  function saveOptions() {
    const validHeaders = headerRows
      .filter((r) => r.key.trim().length > 0)
      .map((r) => [r.key.trim(), r.value.trim()] as [string, string]);

    options = {
      custom_headers: validHeaders.length > 0 ? validHeaders : undefined,
      user_agent: userAgentInput.trim() ? userAgentInput.trim() : undefined,
      timeout_seconds: Number(timeoutSeconds) || 15,
      include_subdomains: includeSubdomains,
    };
    onClose();
  }
</script>

{#if isOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-md p-4 animate-fade-in"
  >
    <div
      class="bg-slate-900/95 border border-slate-800 rounded-2xl w-full max-w-2xl max-h-[85vh] flex flex-col shadow-2xl overflow-hidden backdrop-blur-2xl"
    >
      <!-- Header -->
      <div class="p-5 border-b border-slate-800/80 flex items-center justify-between bg-slate-950/40">
        <div class="flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl bg-cyan-500/10 border border-cyan-500/30 flex items-center justify-center text-cyan-400 shadow-lg shadow-cyan-500/10">
            <Sliders class="w-4 h-4" />
          </div>
          <div>
            <h2 class="text-base font-black text-white tracking-tight">Audit & Recon Parameters</h2>
            <p class="text-[11px] text-slate-400">Configure request headers, authentication, and timeouts</p>
          </div>
        </div>
        <button
          type="button"
          onclick={onClose}
          class="p-1.5 text-slate-400 hover:text-slate-200 hover:bg-slate-800 rounded-lg transition-colors cursor-pointer"
        >
          <X class="w-5 h-5" />
        </button>
      </div>

      <!-- Body -->
      <div class="flex-1 overflow-y-auto p-6 space-y-6">
        
        <!-- Recon Options -->
        <div class="space-y-3">
          <span class="block text-xs font-mono font-bold text-slate-300 uppercase tracking-wider">
            Reconnaissance Features
          </span>
          <div class="p-4 bg-slate-950/60 border border-slate-800 rounded-xl flex items-center justify-between shadow-inner">
            <div>
              <div class="text-xs font-bold text-slate-200 font-mono">Subdomain Mapping via crt.sh</div>
              <div class="text-[11px] text-slate-400 mt-0.5">
                Query Certificate Transparency logs for public subdomain assets.
              </div>
            </div>
            <label class="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                bind:checked={includeSubdomains}
                class="sr-only peer"
              />
              <div class="w-10 h-5 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-cyan-500 shadow-inner"></div>
            </label>
          </div>
        </div>

        <!-- Request Timeout -->
        <div class="space-y-2">
          <span class="block text-xs font-mono font-bold text-slate-300 uppercase tracking-wider">
            Socket Request Timeout
          </span>
          <div class="grid grid-cols-4 gap-2">
            {#each [5, 15, 30, 60] as sec}
              <button
                type="button"
                onclick={() => (timeoutSeconds = sec)}
                class="py-2 text-xs font-mono font-semibold rounded-xl border transition-colors cursor-pointer {timeoutSeconds === sec ? 'bg-cyan-500/10 text-cyan-400 border-cyan-500/40 font-bold shadow-sm' : 'bg-slate-950/60 text-slate-400 border-slate-800 hover:border-slate-700'}"
              >
                {sec}s
              </button>
            {/each}
          </div>
        </div>

        <!-- Custom User-Agent -->
        <div class="space-y-2">
          <label
            for="user-agent"
            class="block text-xs font-mono font-bold text-slate-300 uppercase tracking-wider"
          >
            Custom User-Agent Signature
          </label>
          <input
            id="user-agent"
            type="text"
            bind:value={userAgentInput}
            placeholder="Default: Mozilla/5.0 ... VulnRadar/1.0"
            class="w-full px-3.5 py-2.5 bg-slate-950/80 border border-slate-800 focus:border-cyan-500 focus:ring-1 focus:ring-cyan-500/40 rounded-xl text-xs font-mono text-slate-200 placeholder-slate-600 focus:outline-none shadow-inner"
          />
        </div>

        <!-- Custom Headers Table -->
        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <span
              class="block text-xs font-mono font-bold text-slate-300 uppercase tracking-wider"
            >
              Custom HTTP Headers & Auth Tokens
            </span>
            <button
              type="button"
              onclick={addHeaderRow}
              class="text-xs text-cyan-400 hover:text-cyan-300 flex items-center gap-1 font-semibold cursor-pointer transition-colors"
            >
              <Plus class="w-3.5 h-3.5" />
              <span>Add Header Row</span>
            </button>
          </div>

          <div class="space-y-2">
            {#each headerRows as row, i}
              <div class="flex items-center gap-2">
                <input
                  type="text"
                  bind:value={row.key}
                  placeholder="Header Name (e.g. Authorization)"
                  class="flex-1 px-3 py-2 bg-slate-950/80 border border-slate-800 focus:border-cyan-500 rounded-lg text-xs font-mono text-slate-200 placeholder-slate-600 focus:outline-none"
                />
                <input
                  type="text"
                  bind:value={row.value}
                  placeholder="Value (e.g. Bearer token...)"
                  class="flex-1 px-3 py-2 bg-slate-950/80 border border-slate-800 focus:border-cyan-500 rounded-lg text-xs font-mono text-slate-200 placeholder-slate-600 focus:outline-none"
                />
                <button
                  type="button"
                  onclick={() => removeHeaderRow(i)}
                  class="p-2 text-slate-500 hover:text-rose-400 rounded-lg transition-colors cursor-pointer"
                >
                  <Trash2 class="w-4 h-4" />
                </button>
              </div>
            {/each}
          </div>
        </div>

      </div>

      <!-- Footer -->
      <div class="p-4 border-t border-slate-800/80 flex items-center justify-end gap-2 bg-slate-950/40">
        <button
          type="button"
          onclick={onClose}
          class="px-4 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded-xl text-xs font-semibold cursor-pointer transition-colors"
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={saveOptions}
          class="px-5 py-2 bg-gradient-to-r from-cyan-500 to-cyan-400 hover:from-cyan-400 hover:to-cyan-300 text-slate-950 font-bold rounded-xl text-xs transition-all shadow-lg shadow-cyan-500/20 cursor-pointer"
        >
          Apply Parameters
        </button>
      </div>

    </div>
  </div>
{/if}


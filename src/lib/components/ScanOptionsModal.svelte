<script lang="ts">
  import type { ScanOptions } from "$lib/types";
  import { X, Plus, Trash2, Sliders, Server, Globe } from "lucide-svelte";

  let {
    isOpen = false,
    options,
    onApply,
    onClose,
  }: {
    isOpen: boolean;
    options: ScanOptions;
    onApply?: (newOptions: ScanOptions) => void;
    onClose: () => void;
  } = $props();

  let headerRows = $state<{ key: string; value: string }[]>([{ key: "", value: "" }]);
  let userAgentInput = $state("");
  let timeoutSeconds = $state(15);
  let includeSubdomains = $state(true);

  // Port Scanning Config
  let enablePortScan = $state(false);
  let portScanProfile = $state("top20");
  let customPortsInput = $state("21, 22, 80, 443, 3000-3005, 8080, 8443");
  let portTimeoutMs = $state(800);

  // Synchronize internal state only when opening the modal
  let previousIsOpen = false;
  $effect(() => {
    if (isOpen && !previousIsOpen) {
      headerRows =
        options && options.custom_headers && options.custom_headers.length > 0
          ? options.custom_headers.map(([key, value]) => ({ key, value }))
          : [{ key: "", value: "" }];
      userAgentInput = options?.user_agent || "";
      timeoutSeconds = options?.timeout_seconds || 15;
      includeSubdomains = options?.include_subdomains ?? true;

      enablePortScan = options?.enable_port_scan ?? false;
      portScanProfile = options?.port_scan_profile || "top20";
      customPortsInput = options?.custom_ports || "21, 22, 80, 443, 3000-3005, 8080, 8443";
      portTimeoutMs = options?.port_timeout_ms || 800;
    }
    previousIsOpen = isOpen;
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

  function handleSave() {
    const validHeaders = headerRows
      .filter((r) => r.key.trim().length > 0)
      .map((r) => [r.key.trim(), r.value.trim()] as [string, string]);

    const newOptions: ScanOptions = {
      custom_headers: validHeaders.length > 0 ? validHeaders : undefined,
      user_agent: userAgentInput.trim() ? userAgentInput.trim() : undefined,
      timeout_seconds: Number(timeoutSeconds) || 15,
      include_subdomains: includeSubdomains,
      enable_port_scan: enablePortScan,
      port_scan_profile: portScanProfile,
      custom_ports: customPortsInput.trim() ? customPortsInput.trim() : undefined,
      port_timeout_ms: Number(portTimeoutMs) || 800,
    };

    if (onApply) {
      onApply(newOptions);
    }
    onClose();
  }
</script>

{#if isOpen}
  <!-- Backdrop (Clicking outside closes modal) -->
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4 animate-fade-in"
    onclick={(e) => {
      if (e.target === e.currentTarget) onClose();
    }}
    role="dialog"
    aria-modal="true"
  >
    <div
      class="bg-[#202020] border border-[#333333] rounded-xl w-full max-w-2xl max-h-[85vh] flex flex-col shadow-xl overflow-hidden text-[#e3e2e0]"
    >
      <!-- Header -->
      <div class="p-4 border-b border-[#2e2e2e] flex items-center justify-between bg-[#191919]">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-lg bg-[#282828] border border-[#383838] flex items-center justify-center text-neutral-300">
            <Sliders class="w-4 h-4" />
          </div>
          <div>
            <h2 class="text-sm font-semibold text-white tracking-tight">Audit & Recon Parameters</h2>
            <p class="text-[11px] text-neutral-400">Configure request headers, network port scanner, and timeouts</p>
          </div>
        </div>
        <button
          type="button"
          onclick={onClose}
          class="p-1.5 text-neutral-400 hover:text-white hover:bg-[#282828] rounded-lg transition-colors cursor-pointer"
          aria-label="Close dialog"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Body -->
      <div class="flex-1 overflow-y-auto p-5 space-y-5">
        
        <!-- Recon Options: Subdomains & Port Scanner -->
        <div class="space-y-3">
          <span class="block text-xs font-medium text-neutral-400 uppercase tracking-wider">
            Reconnaissance Features
          </span>
          
          <!-- Subdomain Toggle -->
          <div class="p-3.5 bg-[#191919] border border-[#2e2e2e] rounded-lg flex items-center justify-between">
            <div>
              <div class="text-xs font-medium text-neutral-200 flex items-center gap-2">
                <Globe class="w-3.5 h-3.5 text-neutral-400" />
                <span>Subdomain Mapping via crt.sh</span>
              </div>
              <div class="text-[11px] text-neutral-400 mt-0.5">
                Query Certificate Transparency logs for public subdomain assets.
              </div>
            </div>
            <label class="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                bind:checked={includeSubdomains}
                class="sr-only peer"
              />
              <div class="w-9 h-5 bg-[#333333] peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-blue-600"></div>
            </label>
          </div>

          <!-- Port Scanner Main Toggle -->
          <div class="p-3.5 bg-[#191919] border border-[#2e2e2e] rounded-lg space-y-3">
            <div class="flex items-center justify-between">
              <div>
                <div class="text-xs font-medium text-neutral-200 flex items-center gap-2">
                  <Server class="w-3.5 h-3.5 text-neutral-400" />
                  <span>Network Port Scanner (Nmap Engine)</span>
                </div>
                <div class="text-[11px] text-neutral-400 mt-0.5">
                  Probe target host for open TCP network ports, exposed databases, and service banners.
                </div>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  bind:checked={enablePortScan}
                  class="sr-only peer"
                />
                <div class="w-9 h-5 bg-[#333333] peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-blue-600"></div>
              </label>
            </div>

            {#if enablePortScan}
              <!-- Port Scan Profiles -->
              <div class="pt-3 border-t border-[#2e2e2e] space-y-3">
                <span class="block text-[11px] font-medium text-neutral-400 uppercase tracking-wider">
                  Target Port Profile
                </span>
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
                  <button
                    type="button"
                    onclick={() => (portScanProfile = "top20")}
                    class="p-2 rounded-lg border text-left transition-colors cursor-pointer {portScanProfile === 'top20' ? 'bg-[#2a2a2a] border-neutral-500 text-white' : 'bg-[#202020] border-[#2e2e2e] text-neutral-400 hover:border-[#3a3a3a]'}"
                  >
                    <div class="text-xs font-medium">Top 20 Ports</div>
                    <div class="text-[10px] text-neutral-400">Fast web & infra</div>
                  </button>
                  <button
                    type="button"
                    onclick={() => (portScanProfile = "top100")}
                    class="p-2 rounded-lg border text-left transition-colors cursor-pointer {portScanProfile === 'top100' ? 'bg-[#2a2a2a] border-neutral-500 text-white' : 'bg-[#202020] border-[#2e2e2e] text-neutral-400 hover:border-[#3a3a3a]'}"
                  >
                    <div class="text-xs font-medium">Top 100 Ports</div>
                    <div class="text-[10px] text-neutral-400">Standard Nmap set</div>
                  </button>
                  <button
                    type="button"
                    onclick={() => (portScanProfile = "databases")}
                    class="p-2 rounded-lg border text-left transition-colors cursor-pointer {portScanProfile === 'databases' ? 'bg-[#2a2a2a] border-neutral-500 text-white' : 'bg-[#202020] border-[#2e2e2e] text-neutral-400 hover:border-[#3a3a3a]'}"
                  >
                    <div class="text-xs font-medium">Databases</div>
                    <div class="text-[10px] text-neutral-400">DBs, Redis, APIs</div>
                  </button>
                  <button
                    type="button"
                    onclick={() => (portScanProfile = "custom")}
                    class="p-2 rounded-lg border text-left transition-colors cursor-pointer {portScanProfile === 'custom' ? 'bg-[#2a2a2a] border-neutral-500 text-white' : 'bg-[#202020] border-[#2e2e2e] text-neutral-400 hover:border-[#3a3a3a]'}"
                  >
                    <div class="text-xs font-medium">Custom List</div>
                    <div class="text-[10px] text-neutral-400">Specify range</div>
                  </button>
                </div>

                <!-- Custom port input -->
                {#if portScanProfile === "custom"}
                  <div class="space-y-1 pt-1">
                    <label for="custom-ports" class="block text-[11px] text-neutral-300">
                      Custom Ports & Ranges (e.g. 21, 22, 80, 443, 3000-3005, 8080)
                    </label>
                    <input
                      id="custom-ports"
                      type="text"
                      bind:value={customPortsInput}
                      placeholder="80, 443, 3000-3005, 8080, 8443"
                      class="w-full px-3 py-1.5 bg-[#202020] border border-[#2e2e2e] focus:border-neutral-500 rounded-lg text-xs font-mono text-neutral-200 placeholder-neutral-500 focus:outline-none"
                    />
                  </div>
                {/if}

                <!-- Socket timeout per probe -->
                <div class="space-y-1 pt-1">
                  <span class="block text-[11px] text-neutral-400">
                    Port Probe Socket Timeout
                  </span>
                  <div class="grid grid-cols-3 gap-2">
                    {#each [{ label: "Fast (500ms)", ms: 500 }, { label: "Balanced (800ms)", ms: 800 }, { label: "Thorough (1500ms)", ms: 1500 }] as t}
                      <button
                        type="button"
                        onclick={() => (portTimeoutMs = t.ms)}
                        class="py-1 text-xs font-mono rounded border transition-colors cursor-pointer {portTimeoutMs === t.ms ? 'bg-[#2a2a2a] text-white border-neutral-500 font-medium' : 'bg-[#202020] text-neutral-400 border-[#2e2e2e] hover:border-[#3a3a3a]'}"
                      >
                        {t.label}
                      </button>
                    {/each}
                  </div>
                </div>
              </div>
            {/if}
          </div>
        </div>

        <!-- Request Timeout -->
        <div class="space-y-1.5">
          <span class="block text-xs font-medium text-neutral-400 uppercase tracking-wider">
            HTTP Socket Timeout
          </span>
          <div class="grid grid-cols-4 gap-2">
            {#each [5, 15, 30, 60] as sec}
              <button
                type="button"
                onclick={() => (timeoutSeconds = sec)}
                class="py-1.5 text-xs font-mono font-medium rounded-lg border transition-colors cursor-pointer {timeoutSeconds === sec ? 'bg-[#2a2a2a] text-white border-neutral-500' : 'bg-[#191919] text-neutral-400 border-[#2e2e2e] hover:border-[#3a3a3a]'}"
              >
                {sec}s
              </button>
            {/each}
          </div>
        </div>

        <!-- Custom User-Agent -->
        <div class="space-y-1.5">
          <label
            for="user-agent"
            class="block text-xs font-medium text-neutral-400 uppercase tracking-wider"
          >
            Custom User-Agent Signature
          </label>
          <input
            id="user-agent"
            type="text"
            bind:value={userAgentInput}
            placeholder="Default: Mozilla/5.0 ... VulnRadar/1.0"
            class="w-full px-3 py-2 bg-[#191919] border border-[#2e2e2e] focus:border-neutral-500 rounded-lg text-xs font-mono text-neutral-200 placeholder-neutral-500 focus:outline-none"
          />
        </div>

        <!-- Custom Headers Table -->
        <div class="space-y-2">
          <div class="flex items-center justify-between">
            <span
              class="block text-xs font-medium text-neutral-400 uppercase tracking-wider"
            >
              Custom HTTP Headers & Auth Tokens
            </span>
            <button
              type="button"
              onclick={addHeaderRow}
              class="text-xs text-neutral-300 hover:text-white flex items-center gap-1 font-medium cursor-pointer transition-colors"
            >
              <Plus class="w-3.5 h-3.5" />
              <span>Add Header</span>
            </button>
          </div>

          <div class="space-y-2">
            {#each headerRows as row, i}
              <div class="flex items-center gap-2">
                <input
                  type="text"
                  bind:value={row.key}
                  placeholder="Header Name (e.g. Authorization)"
                  class="flex-1 px-3 py-1.5 bg-[#191919] border border-[#2e2e2e] focus:border-neutral-500 rounded-lg text-xs font-mono text-neutral-200 placeholder-neutral-500 focus:outline-none"
                />
                <input
                  type="text"
                  bind:value={row.value}
                  placeholder="Value (e.g. Bearer token...)"
                  class="flex-1 px-3 py-1.5 bg-[#191919] border border-[#2e2e2e] focus:border-neutral-500 rounded-lg text-xs font-mono text-neutral-200 placeholder-neutral-500 focus:outline-none"
                />
                <button
                  type="button"
                  onclick={() => removeHeaderRow(i)}
                  class="p-1.5 text-neutral-500 hover:text-red-400 rounded-lg transition-colors cursor-pointer"
                >
                  <Trash2 class="w-4 h-4" />
                </button>
              </div>
            {/each}
          </div>
        </div>

      </div>

      <!-- Footer -->
      <div class="p-3.5 border-t border-[#2e2e2e] flex items-center justify-end gap-2 bg-[#191919]">
        <button
          type="button"
          onclick={onClose}
          class="px-3.5 py-1.5 bg-[#252525] hover:bg-[#2e2e2e] text-neutral-300 rounded-lg text-xs font-medium cursor-pointer transition-colors"
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={handleSave}
          class="px-4 py-1.5 bg-white hover:bg-neutral-200 text-neutral-950 font-semibold rounded-lg text-xs transition-colors cursor-pointer shadow-sm"
        >
          Apply Parameters
        </button>
      </div>

    </div>
  </div>
{/if}


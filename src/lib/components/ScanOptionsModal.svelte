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
  let enablePortScan = $state(true);
  let portScanProfile = $state("top20");
  let customPortsInput = $state("21, 22, 80, 443, 3000-3005, 8080, 8443");
  let portTimeoutMs = $state(600);

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

      enablePortScan = options?.enable_port_scan ?? true;
      portScanProfile = options?.port_scan_profile || "top20";
      customPortsInput = options?.custom_ports || "21, 22, 80, 443, 3000-3005, 8080, 8443";
      portTimeoutMs = options?.port_timeout_ms || 600;
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
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 animate-fade-in"
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
      class="bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none w-full max-w-2xl max-h-[85vh] flex flex-col shadow-2xl overflow-hidden text-[var(--color-text-body)]"
    >
      <!-- Header -->
      <div class="p-4 border-b border-[var(--color-hairline)] flex items-center justify-between bg-[var(--color-surface)]">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-none bg-[var(--color-canvas)] border border-[var(--color-hairline)] flex items-center justify-center text-[var(--color-signal-red)]">
            <Sliders class="w-4 h-4" />
          </div>
          <div>
            <h2 class="text-xs font-black text-[var(--color-text-headline)] uppercase tracking-tight font-mono">Audit & Recon Parameters</h2>
            <p class="text-[11px] text-[var(--color-text-muted)] font-mono uppercase">Configure request headers, network port scanner, and timeouts</p>
          </div>
        </div>
        <button
          type="button"
          onclick={onClose}
          class="p-1.5 text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)] rounded-none transition-colors cursor-pointer"
          aria-label="Close dialog"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Body -->
      <div class="flex-1 overflow-y-auto p-5 space-y-5">
        
        <!-- Recon Options: Subdomains & Port Scanner -->
        <div class="space-y-3">
          <span class="block text-xs font-bold text-[var(--color-text-headline)] uppercase tracking-wider font-mono">
            Reconnaissance Features
          </span>
          
          <!-- Subdomain Toggle -->
          <div class="p-3.5 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none flex items-center justify-between">
            <div>
              <div class="text-xs font-bold text-[var(--color-text-headline)] flex items-center gap-2 font-mono uppercase">
                <Globe class="w-3.5 h-3.5 text-[var(--color-signal-red)]" />
                <span>Subdomain Mapping via crt.sh</span>
              </div>
              <div class="text-[11px] text-[var(--color-text-muted)] mt-0.5 font-mono">
                Query Certificate Transparency logs for public subdomain assets.
              </div>
            </div>
            <label class="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                bind:checked={includeSubdomains}
                class="sr-only peer"
              />
              <div class="w-9 h-5 bg-[var(--color-canvas)] border border-[var(--color-hairline)] peer-focus:outline-none rounded-none peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-[var(--color-text-muted)] after:rounded-none after:h-3.5 after:w-3.5 after:transition-all peer-checked:bg-[var(--color-text-headline)] peer-checked:after:bg-[var(--color-canvas)]"></div>
            </label>
          </div>

          <!-- Port Scanner Main Toggle -->
          <div class="p-3.5 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none space-y-3">
            <div class="flex items-center justify-between">
              <div>
                <div class="text-xs font-bold text-[var(--color-text-headline)] flex items-center gap-2 font-mono uppercase">
                  <Server class="w-3.5 h-3.5 text-[var(--color-signal-red)]" />
                  <span>Network Port Scanner (Nmap Engine)</span>
                </div>
                <div class="text-[11px] text-[var(--color-text-muted)] mt-0.5 font-mono">
                  Probe target host for open TCP network ports, exposed databases, and service banners.
                </div>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  bind:checked={enablePortScan}
                  class="sr-only peer"
                />
                <div class="w-9 h-5 bg-[var(--color-canvas)] border border-[var(--color-hairline)] peer-focus:outline-none rounded-none peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-[var(--color-text-muted)] after:rounded-none after:h-3.5 after:w-3.5 after:transition-all peer-checked:bg-[var(--color-text-headline)] peer-checked:after:bg-[var(--color-canvas)]"></div>
              </label>
            </div>

            {#if enablePortScan}
              <!-- Port Scan Profiles -->
              <div class="pt-3 border-t border-[var(--color-hairline)] space-y-3">
                <span class="block text-[11px] font-bold text-[var(--color-text-headline)] uppercase tracking-wider font-mono">
                  Target Port Profile
                </span>
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
                  <button
                    type="button"
                    onclick={() => (portScanProfile = "top20")}
                    class="p-2 rounded-none border text-left transition-colors cursor-pointer {portScanProfile === 'top20' ? 'bg-[var(--color-text-headline)] border-transparent text-[var(--color-canvas)] font-bold' : 'bg-[var(--color-canvas)] border-[var(--color-hairline)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)]'}"
                  >
                    <div class="text-xs font-mono font-bold uppercase">Top 20 Ports</div>
                    <div class="text-[10px] opacity-80 mt-0.5 font-mono uppercase">Fast web & core</div>
                  </button>
                  <button
                    type="button"
                    onclick={() => (portScanProfile = "top100")}
                    class="p-2 rounded-none border text-left transition-colors cursor-pointer {portScanProfile === 'top100' ? 'bg-[var(--color-text-headline)] border-transparent text-[var(--color-canvas)] font-bold' : 'bg-[var(--color-canvas)] border-[var(--color-hairline)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)]'}"
                  >
                    <div class="text-xs font-mono font-bold uppercase">Top 100 Ports</div>
                    <div class="text-[10px] opacity-80 mt-0.5 font-mono uppercase">Standard Nmap set</div>
                  </button>
                  <button
                    type="button"
                    onclick={() => (portScanProfile = "databases")}
                    class="p-2 rounded-none border text-left transition-colors cursor-pointer {portScanProfile === 'databases' ? 'bg-[var(--color-text-headline)] border-transparent text-[var(--color-canvas)] font-bold' : 'bg-[var(--color-canvas)] border-[var(--color-hairline)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)]'}"
                  >
                    <div class="text-xs font-mono font-bold uppercase">Databases</div>
                    <div class="text-[10px] opacity-80 mt-0.5 font-mono uppercase">DBs, Redis, APIs</div>
                  </button>
                  <button
                    type="button"
                    onclick={() => (portScanProfile = "custom")}
                    class="p-2 rounded-none border text-left transition-colors cursor-pointer {portScanProfile === 'custom' ? 'bg-[var(--color-text-headline)] border-transparent text-[var(--color-canvas)] font-bold' : 'bg-[var(--color-canvas)] border-[var(--color-hairline)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)]'}"
                  >
                    <div class="text-xs font-mono font-bold uppercase">Custom List</div>
                    <div class="text-[10px] opacity-80 mt-0.5 font-mono uppercase">Specify range</div>
                  </button>
                </div>

                <!-- Custom port input -->
                {#if portScanProfile === "custom"}
                  <div class="space-y-1 pt-1">
                    <label for="custom-ports" class="block text-[11px] font-bold text-[var(--color-text-headline)] font-mono uppercase">
                      Custom Ports & Ranges (e.g. 21, 22, 80, 443, 3000-3005, 8080)
                    </label>
                    <input
                      id="custom-ports"
                      type="text"
                      bind:value={customPortsInput}
                      placeholder="80, 443, 3000-3005, 8080, 8443"
                      class="w-full px-3 py-1.5 bg-[var(--color-canvas)] border border-[var(--color-hairline)] focus:border-[var(--color-hairline-strong)] rounded-none text-xs font-mono text-[var(--color-text-headline)] placeholder-[var(--color-text-muted)] uppercase focus:outline-none"
                    />
                  </div>
                {/if}

                <!-- Socket timeout per probe -->
                <div class="space-y-1 pt-1">
                  <span class="block text-[11px] font-bold text-[var(--color-text-headline)] uppercase font-mono">
                    Port Probe Socket Timeout
                  </span>
                  <div class="grid grid-cols-3 gap-2">
                    {#each [{ label: "FAST (500MS)", ms: 500 }, { label: "BALANCED (800MS)", ms: 800 }, { label: "THOROUGH (1500MS)", ms: 1500 }] as t}
                      <button
                        type="button"
                        onclick={() => (portTimeoutMs = t.ms)}
                        class="py-1 text-xs font-mono font-bold rounded-none border transition-colors cursor-pointer {portTimeoutMs === t.ms ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)] border-transparent' : 'bg-[var(--color-canvas)] text-[var(--color-text-muted)] border-[var(--color-hairline)] hover:text-[var(--color-text-headline)]'}"
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
          <span class="block text-xs font-bold text-[var(--color-text-headline)] uppercase tracking-wider font-mono">
            HTTP Socket Timeout
          </span>
          <div class="grid grid-cols-4 gap-2">
            {#each [5, 15, 30, 60] as sec}
              <button
                type="button"
                onclick={() => (timeoutSeconds = sec)}
                class="py-1.5 text-xs font-mono font-bold rounded-none border transition-colors cursor-pointer {timeoutSeconds === sec ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)] border-transparent' : 'bg-[var(--color-canvas)] text-[var(--color-text-muted)] border-[var(--color-hairline)] hover:text-[var(--color-text-headline)]'}"
              >
                {sec}S
              </button>
            {/each}
          </div>
        </div>

        <!-- Custom User-Agent -->
        <div class="space-y-1.5">
          <label
            for="user-agent"
            class="block text-xs font-bold text-[var(--color-text-headline)] uppercase tracking-wider font-mono"
          >
            Custom User-Agent Signature
          </label>
          <input
            id="user-agent"
            type="text"
            bind:value={userAgentInput}
            placeholder="DEFAULT: MOZILLA/5.0 ... VULNRADAR/1.0"
            class="w-full px-3 py-2 bg-[var(--color-canvas)] border border-[var(--color-hairline)] focus:border-[var(--color-hairline-strong)] rounded-none text-xs font-mono text-[var(--color-text-headline)] placeholder-[var(--color-text-muted)] uppercase focus:outline-none"
          />
        </div>

        <!-- Custom Headers Table -->
        <div class="space-y-2">
          <div class="flex items-center justify-between">
            <span
              class="block text-xs font-bold text-[var(--color-text-headline)] uppercase tracking-wider font-mono"
            >
              Custom HTTP Headers & Auth Tokens
            </span>
            <button
              type="button"
              onclick={addHeaderRow}
              class="px-2.5 py-1 bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] text-[var(--color-text-headline)] border border-[var(--color-hairline)] rounded-none text-xs font-mono font-bold uppercase flex items-center gap-1 cursor-pointer transition-colors"
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
                  placeholder="HEADER (E.G. AUTHORIZATION)"
                  class="flex-1 px-3 py-1.5 bg-[var(--color-canvas)] border border-[var(--color-hairline)] focus:border-[var(--color-hairline-strong)] rounded-none text-xs font-mono text-[var(--color-text-headline)] placeholder-[var(--color-text-muted)] uppercase focus:outline-none"
                />
                <input
                  type="text"
                  bind:value={row.value}
                  placeholder="VALUE (E.G. BEARER EYJHBGC...)"
                  class="flex-1 px-3 py-1.5 bg-[var(--color-canvas)] border border-[var(--color-hairline)] focus:border-[var(--color-hairline-strong)] rounded-none text-xs font-mono text-[var(--color-text-headline)] placeholder-[var(--color-text-muted)] focus:outline-none"
                />
                <button
                  type="button"
                  onclick={() => removeHeaderRow(i)}
                  class="p-1.5 text-[var(--color-text-muted)] hover:text-red-500 rounded-none transition-colors cursor-pointer"
                >
                  <Trash2 class="w-4 h-4" />
                </button>
              </div>
            {/each}
          </div>
        </div>

      </div>

      <!-- Footer -->
      <div class="p-3.5 border-t border-[var(--color-hairline)] flex items-center justify-end gap-2 bg-[var(--color-surface)]">
        <button
          type="button"
          onclick={onClose}
          class="px-3.5 py-1.5 bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] border border-[var(--color-hairline)] rounded-none text-xs font-mono font-bold uppercase cursor-pointer transition-colors"
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={handleSave}
          class="px-4 py-1.5 bg-[var(--color-text-headline)] hover:opacity-90 text-[var(--color-canvas)] font-mono font-bold uppercase rounded-none text-xs transition-opacity cursor-pointer"
        >
          Apply Parameters
        </button>
      </div>

    </div>
  </div>
{/if}


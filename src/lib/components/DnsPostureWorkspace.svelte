<script lang="ts">
  import type { DnsSecurityReport } from "$lib/types";
  import {
    ShieldCheck,
    ShieldAlert,
    Shield,
    Terminal,
    Copy,
    Check,
    Filter,
    Globe,
    Loader2,
    RefreshCw,
    AlertOctagon,
  } from "lucide-svelte";

  let {
    targetUrl = "",
    dnsReport = null,
    onUpdateReport,
  }: {
    targetUrl: string;
    dnsReport: DnsSecurityReport | null;
    onUpdateReport?: (newReport: DnsSecurityReport) => void;
  } = $props();

  let localReport = $state<DnsSecurityReport | null>(null);
  let isQuerying = $state(false);
  let queryError = $state<string | null>(null);
  let searchQuery = $state("");
  let copiedRaw = $state(false);

  // Derive target domain from URL or report
  const defaultDomain = $derived.by(() => {
    if (dnsReport?.domain) return dnsReport.domain;
    try {
      const parsed = new URL(targetUrl.startsWith("http") ? targetUrl : `https://${targetUrl}`);
      return parsed.hostname;
    } catch {
      return targetUrl.replace(/https?:\/\//, "").split("/")[0] || "example.com";
    }
  });

  let domainInput = $state("");

  $effect(() => {
    if (!domainInput && defaultDomain) {
      domainInput = defaultDomain;
    }
  });

  $effect(() => {
    if (dnsReport) {
      localReport = dnsReport;
    }
  });

  const activeDnsReport = $derived(localReport || dnsReport);
  const targetDomain = $derived(domainInput.trim() || activeDnsReport?.domain || defaultDomain);

  // RFC 1035 resource records from live backend DoH scan
  const allRecords = $derived.by(() => {
    if (activeDnsReport?.records && activeDnsReport.records.length > 0) {
      return activeDnsReport.records;
    }
    const recs = [];
    if (activeDnsReport?.spf_record) {
      recs.push({
        name: `${targetDomain}.`,
        ttl: 300,
        class: "IN",
        type: "TXT",
        data: activeDnsReport.spf_record,
        note: "SPF Policy",
      });
    }
    if (activeDnsReport?.dmarc_record) {
      recs.push({
        name: `_dmarc.${targetDomain}.`,
        ttl: 300,
        class: "IN",
        type: "TXT",
        data: activeDnsReport.dmarc_record,
        note: "DMARC Policy",
      });
    }
    return recs;
  });

  const filteredRecords = $derived.by(() => {
    if (!searchQuery.trim()) return allRecords;
    const q = searchQuery.toLowerCase();
    return allRecords.filter(
      (r) =>
        r.name.toLowerCase().includes(q) ||
        r.type.toLowerCase().includes(q) ||
        r.data.toLowerCase().includes(q) ||
        (r.note && r.note.toLowerCase().includes(q))
    );
  });

  async function handleQueryDns(domainToQuery?: string) {
    const dom = (domainToQuery || domainInput || targetDomain).trim();
    if (!dom || isQuerying) return;

    isQuerying = true;
    queryError = null;

    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const res = await invoke<DnsSecurityReport>("query_dns", { domain: dom });
      localReport = res;
      if (onUpdateReport) {
        onUpdateReport(res);
      }
    } catch (e: any) {
      queryError = typeof e === "string" ? e : (e?.message || "Failed to query live DNS records.");
    } finally {
      isQuerying = false;
    }
  }

  async function copyRawDump() {
    let dump = `; <<>> VulnRadar DNS <<>> ANY ${targetDomain}\n`;
    if (activeDnsReport?.latency_ms != null) {
      dump += `; (Query time: ${activeDnsReport.latency_ms}ms)\n\n`;
    } else {
      dump += `;\n\n`;
    }
    for (const r of allRecords) {
      dump += `${r.name.padEnd(28)} ${r.ttl}  ${r.class}  ${r.type.padEnd(8)} ${r.data}\n`;
    }
    try {
      await navigator.clipboard.writeText(dump);
      copiedRaw = true;
      setTimeout(() => (copiedRaw = false), 1800);
    } catch {}
  }
</script>

<div class="flex flex-col w-full gap-4 max-w-7xl mx-auto pb-10">
  <!-- Query Bar -->
  <section class="bg-surface-container-low p-3.5 rounded border border-surface-container-high shadow-sm">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div class="flex items-center gap-2 flex-1 min-w-[200px] sm:min-w-[280px] max-w-lg">
        <div class="h-9 flex items-center bg-surface-container-lowest px-3 rounded border border-surface-container-high flex-1">
          <Globe class="w-3.5 h-3.5 text-outline mr-2 flex-shrink-0" />
          <input
            type="text"
            bind:value={domainInput}
            placeholder="e.g. example.com"
            onkeydown={(e) => {
              if (e.key === "Enter") handleQueryDns();
            }}
            class="bg-transparent font-mono text-xs text-on-surface outline-none w-full"
          />
        </div>

        {#if activeDnsReport}
          <span class="h-9 flex items-center px-2.5 rounded font-mono text-xs font-medium border {activeDnsReport.dnssec_enabled ? 'bg-tertiary/10 text-tertiary border-tertiary/30' : 'bg-surface-container text-outline border-surface-container-high'} whitespace-nowrap">
            DNSSEC: {activeDnsReport.dnssec_enabled ? "Active" : "Disabled"}
          </span>
        {/if}
      </div>

      <div class="flex items-center gap-2">
        <button
          type="button"
          disabled={isQuerying}
          onclick={() => handleQueryDns()}
          class="h-9 flex items-center gap-1.5 px-4 bg-primary text-on-primary rounded font-mono text-xs font-bold transition-opacity cursor-pointer hover:opacity-90 disabled:opacity-50 whitespace-nowrap"
        >
          {#if isQuerying}
            <Loader2 class="w-3.5 h-3.5 animate-spin" />
            <span>Resolving...</span>
          {:else}
            <RefreshCw class="w-3.5 h-3.5" />
            <span>Query DNS</span>
          {/if}
        </button>

        {#if allRecords.length > 0}
          <button
            type="button"
            onclick={copyRawDump}
            class="h-9 flex items-center gap-1.5 px-3 bg-surface-container hover:bg-surface-container-high text-on-surface rounded font-mono text-xs transition-colors cursor-pointer border border-surface-container-high whitespace-nowrap"
          >
            {#if copiedRaw}
              <Check class="w-3.5 h-3.5 text-tertiary" />
              <span>Copied</span>
            {:else}
              <Copy class="w-3.5 h-3.5 text-outline" />
              <span>Copy Zone</span>
            {/if}
          </button>
        {/if}
      </div>
    </div>

    <!-- Active Report Metadata Bar -->
    {#if activeDnsReport}
      <div class="flex items-center gap-4 text-xs text-outline font-mono pt-3 mt-3 border-t border-surface-container-high flex-wrap">
        <span>Records: <strong class="text-on-surface">{allRecords.length}</strong></span>
        {#if activeDnsReport.latency_ms != null}
          <span>Latency: <strong class="text-on-surface">{activeDnsReport.latency_ms}ms</strong></span>
        {/if}
        {#if activeDnsReport.nameservers && activeDnsReport.nameservers.length > 0}
          <span class="truncate max-w-md">Nameservers: <strong class="text-on-surface">{activeDnsReport.nameservers.join(", ")}</strong></span>
        {/if}
      </div>
    {/if}
  </section>

  <!-- Error Banner -->
  {#if queryError}
    <div class="p-3 bg-surface-container-low border border-error/40 rounded flex items-center justify-between text-xs text-error font-mono">
      <div class="flex items-center gap-2">
        <AlertOctagon class="w-4 h-4 flex-shrink-0" />
        <span>{queryError}</span>
      </div>
      <button
        type="button"
        onclick={() => handleQueryDns()}
        class="px-2 py-0.5 bg-error text-surface-container-lowest font-bold rounded cursor-pointer uppercase text-[10px]"
      >
        Retry
      </button>
    </div>
  {/if}

  {#if !activeDnsReport && !isQuerying}
    <!-- Clean Standby State -->
    <div class="p-12 text-center bg-surface-container-low rounded border border-surface-container-high text-outline flex flex-col items-center justify-center gap-2">
      <Globe class="w-8 h-8 opacity-40 mb-1" />
      <span class="font-bold text-on-surface text-sm">DNS Posture & Record Inspector</span>
      <span class="text-xs max-w-md">Query the target domain to inspect authoritative DNS records, DNSSEC cryptographic validation, SPF sender policies, and DMARC spoof defenses.</span>
    </div>
  {:else if isQuerying && allRecords.length === 0}
    <!-- Querying State -->
    <div class="p-12 text-center bg-surface-container-low rounded border border-surface-container-high text-outline flex flex-col items-center justify-center gap-2">
      <Loader2 class="w-6 h-6 animate-spin text-primary" />
      <span class="font-bold text-on-surface text-sm">Querying authoritative DNS records for {targetDomain}...</span>
    </div>
  {:else}
    <!-- 3 Clean Posture Summary Cards -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
      <!-- Card 1: SPF Alignment -->
      <div class="bg-surface-container-low p-4 rounded border border-surface-container-high flex flex-col justify-between gap-3">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <ShieldCheck class="w-4 h-4 {activeDnsReport?.spf_record ? 'text-tertiary' : 'text-error'}" />
            <h3 class="text-xs font-bold text-on-surface uppercase tracking-wider font-mono">SPF Policy</h3>
          </div>
          <span class="px-2 py-0.5 text-[11px] font-mono rounded font-bold {activeDnsReport?.spf_record ? 'bg-tertiary/10 text-tertiary' : 'bg-error/10 text-error'}">
            {activeDnsReport?.spf_record ? "Configured" : "Missing"}
          </span>
        </div>

        {#if activeDnsReport?.spf_record}
          <div class="p-2.5 bg-surface-container-lowest rounded border border-surface-container-high font-mono text-xs text-on-surface-variant break-all">
            {activeDnsReport.spf_record}
          </div>
          <div class="text-[11px] text-outline">
            {#if activeDnsReport.spf_record.includes("-all")}
              Strict enforcement (<code class="text-tertiary">-all</code>): drops unauthorized mail.
            {:else if activeDnsReport.spf_record.includes("~all")}
              Soft fail (<code class="text-secondary">~all</code>): marks unauthorized mail.
            {:else}
              Neutral directive: spoofing possible.
            {/if}
          </div>
        {:else}
          <p class="text-xs text-error">No SPF record published. Domain is vulnerable to email spoofing.</p>
        {/if}
      </div>

      <!-- Card 2: DMARC Policy -->
      <div class="bg-surface-container-low p-4 rounded border border-surface-container-high flex flex-col justify-between gap-3">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <ShieldAlert class="w-4 h-4 {activeDnsReport?.dmarc_record ? 'text-tertiary' : 'text-amber-500'}" />
            <h3 class="text-xs font-bold text-on-surface uppercase tracking-wider font-mono">DMARC Policy</h3>
          </div>
          <span class="px-2 py-0.5 text-[11px] font-mono rounded font-bold {activeDnsReport?.dmarc_record ? 'bg-tertiary/10 text-tertiary' : 'bg-amber-500/10 text-amber-500'}">
            {activeDnsReport?.dmarc_policy ? `p=${activeDnsReport.dmarc_policy}` : "Missing"}
          </span>
        </div>

        {#if activeDnsReport?.dmarc_record}
          <div class="p-2.5 bg-surface-container-lowest rounded border border-surface-container-high font-mono text-xs text-on-surface-variant break-all">
            {activeDnsReport.dmarc_record}
          </div>
          <div class="text-[11px] text-outline">
            {#if activeDnsReport.dmarc_policy === "reject"}
              Strict enforcement: unauthorized mail claiming this domain is rejected.
            {:else if activeDnsReport.dmarc_policy === "quarantine"}
              Quarantine enforcement: unauthorized mail is delivered to spam.
            {:else}
              Monitoring mode only: mail is delivered without restriction.
            {/if}
          </div>
        {:else}
          <p class="text-xs text-amber-500">No _dmarc TXT record found. No domain impersonation protection.</p>
        {/if}
      </div>

      <!-- Card 3: DNSSEC Chain -->
      <div class="bg-surface-container-low p-4 rounded border border-surface-container-high flex flex-col justify-between gap-3">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <Shield class="w-4 h-4 {activeDnsReport?.dnssec_enabled ? 'text-tertiary' : 'text-outline'}" />
            <h3 class="text-xs font-bold text-on-surface uppercase tracking-wider font-mono">DNSSEC</h3>
          </div>
          <span class="px-2 py-0.5 text-[11px] font-mono rounded font-bold {activeDnsReport?.dnssec_enabled ? 'bg-tertiary/10 text-tertiary' : 'bg-surface-container text-outline'}">
            {activeDnsReport?.dnssec_enabled ? "Authenticated" : "Unsigned"}
          </span>
        </div>

        <div class="p-2.5 bg-surface-container-lowest rounded border border-surface-container-high font-mono text-xs text-on-surface space-y-1">
          <div class="flex justify-between">
            <span class="text-outline">Validation:</span>
            <span class="{activeDnsReport?.dnssec_enabled ? 'text-tertiary font-bold' : 'text-outline'}">
              {activeDnsReport?.dnssec_enabled ? "RRSIG / AD=1 Valid" : "No RRSIG signatures"}
            </span>
          </div>
          <div class="flex justify-between">
            <span class="text-outline">Transport:</span>
            <span class="text-on-surface-variant">DoH (RFC 8484)</span>
          </div>
        </div>
        <div class="text-[11px] text-outline">
          {activeDnsReport?.dnssec_enabled ? "Protects against DNS cache poisoning and spoofed answers." : "Zone is not cryptographically signed."}
        </div>
      </div>
    </div>

    <!-- Clean Records Table -->
    <div class="bg-surface-container-low rounded border border-surface-container-high overflow-hidden shadow-sm">
      <div class="px-4 py-2.5 bg-surface-container flex items-center justify-between gap-3 border-b border-surface-container-high">
        <div class="flex items-center gap-2">
          <Terminal class="w-4 h-4 text-outline" />
          <span class="text-xs font-bold text-on-surface uppercase font-mono tracking-wider">DNS Records ({filteredRecords.length})</span>
        </div>
        <div class="flex items-center bg-surface-container-lowest px-2.5 py-1 rounded gap-2 border border-surface-container-high">
          <Filter class="w-3 h-3 text-outline" />
          <input
            class="bg-transparent text-on-surface font-mono text-xs focus:outline-none w-36 placeholder:text-outline"
            placeholder="Filter records..."
            type="text"
            bind:value={searchQuery}
          />
        </div>
      </div>

      <div class="overflow-x-auto">
        <table class="w-full text-left text-xs font-mono">
          <thead>
            <tr class="border-b border-surface-container-high text-outline text-[11px] bg-surface-container-lowest">
              <th class="py-2 px-3 font-semibold">NAME</th>
              <th class="py-2 px-3 font-semibold w-16">TTL</th>
              <th class="py-2 px-3 font-semibold w-16">CLASS</th>
              <th class="py-2 px-3 font-semibold w-16">TYPE</th>
              <th class="py-2 px-3 font-semibold">DATA / VALUE</th>
              <th class="py-2 px-3 font-semibold">NOTE</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-surface-container-high/60">
            {#each filteredRecords as r}
              <tr class="hover:bg-surface-container/60 transition-colors">
                <td class="py-2 px-3 text-secondary font-medium truncate max-w-[200px]">{r.name}</td>
                <td class="py-2 px-3 text-outline">{r.ttl}</td>
                <td class="py-2 px-3 text-on-surface-variant">{r.class}</td>
                <td class="py-2 px-3">
                  <span class="px-1.5 py-0.5 rounded text-[10px] font-bold {r.type === 'A' ? 'bg-secondary/10 text-secondary' : r.type === 'AAAA' ? 'bg-secondary/10 text-secondary' : r.type === 'MX' ? 'bg-primary-container/10 text-primary-container' : r.type === 'TXT' ? 'bg-tertiary/10 text-tertiary' : r.type === 'NS' ? 'bg-primary/10 text-primary' : 'bg-surface-container text-outline'}">
                    {r.type}
                  </span>
                </td>
                <td class="py-2 px-3 text-on-surface break-all select-text">{r.data}</td>
                <td class="py-2 px-3 text-outline text-[11px] truncate max-w-[150px]">{r.note || ""}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {/if}
</div>

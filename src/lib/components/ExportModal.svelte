<script lang="ts">
  import type { ScanReport } from "$lib/types";
  import { X, Download, Copy, Check, FileText, Code2, Table, Terminal, Printer } from "lucide-svelte";

  let {
    isOpen = false,
    report,
    markdownContent = "",
    onOpenExecutiveReport,
    onClose,
  }: {
    isOpen: boolean;
    report: ScanReport | null;
    markdownContent: string;
    onOpenExecutiveReport?: () => void;
    onClose: () => void;
  } = $props();

  let activeTab: "markdown" | "json" | "csv" | "curl" = $state("markdown");
  let copied = $state(false);

  function generateCsv(rep: ScanReport | null): string {
    if (!rep) return "";
    const escapeCsv = (str: string | null | undefined) => {
      if (!str) return '""';
      let clean = str;
      // Prevent CSV / Excel Formula Injection (CWE-1236)
      if (/^[=+\-@\t\r]/.test(clean)) {
        clean = `'${clean}`;
      }
      return `"${clean.replace(/"/g, '""').replace(/\r?\n/g, " ")}"`;
    };

    const header = ["ID", "Title", "Severity", "Category", "OWASP Category", "CVE", "Description", "Impact", "Remediation", "Evidence"].join(",");
    const rows = rep.findings.map((f) => {
      return [
        escapeCsv(f.id),
        escapeCsv(f.title),
        escapeCsv(f.severity.toUpperCase()),
        escapeCsv(f.category),
        escapeCsv(f.owasp_category),
        escapeCsv(f.cve_id || "N/A"),
        escapeCsv(f.description),
        escapeCsv(f.impact),
        escapeCsv(f.remediation),
        escapeCsv(f.evidence || "N/A"),
      ].join(",");
    });

    return [header, ...rows].join("\n");
  }

  function generateCurl(rep: ScanReport | null): string {
    if (!rep) return "";
    return `# Reproduce Passive Audit Request\ncurl -i -s -k -L \\\n  -H "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) VulnRadar/1.0" \\\n  "${rep.target_url}"`;
  }

  const jsonContent = $derived(report ? JSON.stringify(report, null, 2) : "");
  const csvContent = $derived(generateCsv(report));
  const curlContent = $derived(generateCurl(report));

  const currentContent = $derived(
    activeTab === "markdown"
      ? markdownContent
      : activeTab === "json"
        ? jsonContent
        : activeTab === "csv"
          ? csvContent
          : curlContent
  );

  async function copyToClipboard() {
    try {
      await navigator.clipboard.writeText(currentContent);
      copied = true;
      setTimeout(() => {
        copied = false;
      }, 2000);
    } catch (e) {
      console.error("Failed to copy", e);
    }
  }

  function downloadReport() {
    if (!report) return;
    const ext = activeTab === "markdown" ? "md" : activeTab === "json" ? "json" : activeTab === "csv" ? "csv" : "sh";
    const mime =
      activeTab === "markdown"
        ? "text/markdown;charset=utf-8"
        : activeTab === "json"
          ? "application/json;charset=utf-8"
          : activeTab === "csv"
            ? "text/csv;charset=utf-8"
            : "text/x-sh;charset=utf-8";

    let host = "target";
    try {
      const urlToParse = report.target_url.startsWith("http") ? report.target_url : `https://${report.target_url}`;
      host = new URL(urlToParse).hostname.replace(/[^a-zA-Z0-9.-]/g, "_");
    } catch {
      host = report.target_url.replace(/[^a-zA-Z0-9.-]/g, "_");
    }
    const filename = `vulnradar-${host}-${new Date().toISOString().slice(0, 10)}.${ext}`;

    const blob = new Blob([currentContent], { type: mime });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  }
</script>

{#if isOpen && report}
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
    <div class="bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none w-full max-w-3xl max-h-[85vh] flex flex-col shadow-2xl overflow-hidden text-[var(--color-text-body)]">
      <!-- Header -->
      <div class="p-4 border-b border-[var(--color-hairline)] flex flex-col sm:flex-row sm:items-center justify-between gap-3 bg-[var(--color-surface)]">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-none bg-[var(--color-canvas)] border border-[var(--color-hairline)] flex items-center justify-center text-[var(--color-signal-red)]">
            <FileText class="w-4 h-4" />
          </div>
          <div>
            <h2 class="text-xs font-black text-[var(--color-text-headline)] font-mono uppercase tracking-tight">Export & Share Security Report</h2>
            <p class="text-[11px] text-[var(--color-text-muted)] font-mono truncate max-w-xs uppercase">{report.target_url}</p>
          </div>
        </div>

        <div class="flex items-center gap-2 self-end sm:self-auto">
          <!-- Format Tabs -->
          <div class="flex bg-[var(--color-canvas)] p-0.5 rounded-none border border-[var(--color-hairline)]">
            <button
              type="button"
              onclick={() => (activeTab = "markdown")}
              class="px-2.5 py-1 text-xs font-mono font-bold uppercase rounded-none flex items-center gap-1.5 transition-colors cursor-pointer {activeTab === 'markdown' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)]' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)]'}"
            >
              <FileText class="w-3.5 h-3.5" />
              <span>MARKDOWN</span>
            </button>
            <button
              type="button"
              onclick={() => (activeTab = "json")}
              class="px-2.5 py-1 text-xs font-mono font-bold uppercase rounded-none flex items-center gap-1.5 transition-colors cursor-pointer {activeTab === 'json' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)]' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)]'}"
            >
              <Code2 class="w-3.5 h-3.5" />
              <span>JSON</span>
            </button>
            <button
              type="button"
              onclick={() => (activeTab = "csv")}
              class="px-2.5 py-1 text-xs font-mono font-bold uppercase rounded-none flex items-center gap-1.5 transition-colors cursor-pointer {activeTab === 'csv' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)]' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)]'}"
            >
              <Table class="w-3.5 h-3.5" />
              <span>CSV</span>
            </button>
            <button
              type="button"
              onclick={() => (activeTab = "curl")}
              class="px-2.5 py-1 text-xs font-mono font-bold uppercase rounded-none flex items-center gap-1.5 transition-colors cursor-pointer {activeTab === 'curl' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)]' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)]'}"
            >
              <Terminal class="w-3.5 h-3.5" />
              <span>cURL</span>
            </button>
          </div>

          <button
            type="button"
            onclick={onClose}
            class="p-1.5 text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)] rounded-none transition-colors cursor-pointer"
            aria-label="Close export dialog"
          >
            <X class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- Preview Content -->
      <div class="flex-1 overflow-hidden p-4 flex flex-col bg-[var(--color-canvas)]">
        <pre class="flex-1 p-3.5 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none text-xs font-mono text-[var(--color-text-headline)] overflow-auto whitespace-pre-wrap">{currentContent}</pre>
      </div>

      <!-- Footer / Actions -->
      <div class="p-3.5 border-t border-[var(--color-hairline)] bg-[var(--color-surface)] flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div class="flex items-center gap-3">
          <div class="text-xs font-mono text-[var(--color-text-muted)] uppercase">
            FORMAT: <strong class="text-[var(--color-text-headline)] font-bold">{activeTab}</strong> • {report.total_findings} FINDINGS
          </div>

          {#if onOpenExecutiveReport}
            <button
              type="button"
              onclick={() => {
                onClose();
                onOpenExecutiveReport?.();
              }}
              class="px-2.5 py-1 bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] text-[var(--color-text-headline)] border border-[var(--color-hairline)] rounded-none text-xs font-mono font-bold uppercase flex items-center gap-1.5 transition-colors cursor-pointer"
            >
              <Printer class="w-3.5 h-3.5 text-[var(--color-text-muted)]" />
              <span>EXECUTIVE PDF VIEW</span>
            </button>
          {/if}
        </div>

        <div class="flex items-center gap-2 self-end sm:self-auto">
          <button
            type="button"
            onclick={copyToClipboard}
            class="px-3 py-1.5 bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] text-[var(--color-text-headline)] rounded-none text-xs font-mono font-bold uppercase flex items-center gap-1.5 transition-colors cursor-pointer border border-[var(--color-hairline)]"
          >
            {#if copied}
              <Check class="w-3.5 h-3.5 text-emerald-500" />
              <span class="text-emerald-500">COPIED</span>
            {:else}
              <Copy class="w-3.5 h-3.5" />
              <span>COPY</span>
            {/if}
          </button>

          <button
            type="button"
            onclick={downloadReport}
            class="px-3.5 py-1.5 bg-[var(--color-text-headline)] hover:opacity-90 text-[var(--color-canvas)] font-mono font-bold uppercase rounded-none text-xs flex items-center gap-1.5 transition-opacity cursor-pointer"
          >
            <Download class="w-3.5 h-3.5" />
            <span>DOWNLOAD .{activeTab === "markdown" ? "MD" : activeTab === "json" ? "JSON" : activeTab === "csv" ? "CSV" : "SH"}</span>
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

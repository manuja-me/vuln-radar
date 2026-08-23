<script lang="ts">
  import type { ScanReport } from "$lib/types";
  import { X, Download, Copy, Check, FileText, Code2, Table, Terminal } from "lucide-svelte";

  let {
    isOpen = false,
    report,
    markdownContent = "",
    onClose,
  }: {
    isOpen: boolean;
    report: ScanReport | null;
    markdownContent: string;
    onClose: () => void;
  } = $props();

  let activeTab: "markdown" | "json" | "csv" | "curl" = $state("markdown");
  let copied = $state(false);

  function generateCsv(rep: ScanReport | null): string {
    if (!rep) return "";
    const escapeCsv = (str: string | null | undefined) => {
      if (!str) return '""';
      return `"${str.replace(/"/g, '""').replace(/\r?\n/g, " ")}"`;
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

    const host = new URL(report.target_url).hostname.replace(/[^a-zA-Z0-9.-]/g, "_");
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
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4">
    <div class="bg-[#202020] border border-[#333333] rounded-xl w-full max-w-3xl max-h-[85vh] flex flex-col shadow-xl overflow-hidden text-[#e3e2e0]">
      <!-- Header -->
      <div class="p-4 border-b border-[#2e2e2e] flex items-center justify-between bg-[#191919]">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-lg bg-[#282828] border border-[#383838] flex items-center justify-center text-neutral-300">
            <FileText class="w-4 h-4" />
          </div>
          <div>
            <h2 class="text-sm font-semibold text-white">Export Security Report</h2>
            <p class="text-[11px] text-neutral-400 font-mono">{report.target_url}</p>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <!-- Format Tabs -->
          <div class="flex bg-[#141414] p-0.5 rounded-lg border border-[#2e2e2e]">
            <button
              type="button"
              onclick={() => (activeTab = "markdown")}
              class="px-2.5 py-1 text-xs font-medium rounded-md flex items-center gap-1.5 transition-colors cursor-pointer {activeTab === 'markdown' ? 'bg-[#2a2a2a] text-white font-semibold' : 'text-neutral-400 hover:text-neutral-200'}"
            >
              <FileText class="w-3.5 h-3.5" />
              Markdown
            </button>
            <button
              type="button"
              onclick={() => (activeTab = "json")}
              class="px-2.5 py-1 text-xs font-medium rounded-md flex items-center gap-1.5 transition-colors cursor-pointer {activeTab === 'json' ? 'bg-[#2a2a2a] text-white font-semibold' : 'text-neutral-400 hover:text-neutral-200'}"
            >
              <Code2 class="w-3.5 h-3.5" />
              JSON
            </button>
            <button
              type="button"
              onclick={() => (activeTab = "csv")}
              class="px-2.5 py-1 text-xs font-medium rounded-md flex items-center gap-1.5 transition-colors cursor-pointer {activeTab === 'csv' ? 'bg-[#2a2a2a] text-white font-semibold' : 'text-neutral-400 hover:text-neutral-200'}"
            >
              <Table class="w-3.5 h-3.5" />
              CSV
            </button>
            <button
              type="button"
              onclick={() => (activeTab = "curl")}
              class="px-2.5 py-1 text-xs font-medium rounded-md flex items-center gap-1.5 transition-colors cursor-pointer {activeTab === 'curl' ? 'bg-[#2a2a2a] text-white font-semibold' : 'text-neutral-400 hover:text-neutral-200'}"
            >
              <Terminal class="w-3.5 h-3.5" />
              cURL
            </button>
          </div>

          <button
            type="button"
            onclick={onClose}
            class="p-1.5 text-neutral-400 hover:text-white hover:bg-[#282828] rounded-lg transition-colors cursor-pointer"
          >
            <X class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- Preview Content -->
      <div class="flex-1 overflow-hidden p-4 flex flex-col bg-[#161616]">
        <pre class="flex-1 p-3.5 bg-[#141414] border border-[#2a2a2a] rounded-lg text-xs font-mono text-neutral-300 overflow-auto whitespace-pre-wrap">{currentContent}</pre>
      </div>

      <!-- Footer / Actions -->
      <div class="p-3.5 border-t border-[#2e2e2e] bg-[#191919] flex items-center justify-between">
        <div class="text-xs text-neutral-400">
          Format: <strong class="text-white uppercase font-mono">{activeTab}</strong> • {report.total_findings} findings discovered
        </div>

        <div class="flex items-center gap-2">
          <button
            type="button"
            onclick={copyToClipboard}
            class="px-3 py-1.5 bg-[#252525] hover:bg-[#2c2c2c] text-neutral-300 hover:text-white rounded-lg text-xs font-medium flex items-center gap-1.5 transition-colors cursor-pointer"
          >
            {#if copied}
              <Check class="w-3.5 h-3.5 text-emerald-400" />
              <span class="text-emerald-400">Copied</span>
            {:else}
              <Copy class="w-3.5 h-3.5" />
              <span>Copy</span>
            {/if}
          </button>

          <button
            type="button"
            onclick={downloadReport}
            class="px-3.5 py-1.5 bg-white hover:bg-neutral-200 text-neutral-950 font-semibold rounded-lg text-xs flex items-center gap-1.5 transition-colors cursor-pointer shadow-sm"
          >
            <Download class="w-3.5 h-3.5" />
            <span>Download .{activeTab === "markdown" ? "md" : activeTab === "json" ? "json" : activeTab === "csv" ? "csv" : "sh"}</span>
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

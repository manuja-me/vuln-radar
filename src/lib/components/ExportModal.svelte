<script lang="ts">
  import type { ScanReport } from "$lib/types";
  import { X, Download, Copy, Check, FileText, Code2 } from "lucide-svelte";

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

  let activeTab: "markdown" | "json" = $state("markdown");
  let copied = $state(false);

  const jsonContent = $derived(report ? JSON.stringify(report, null, 2) : "");
  const currentContent = $derived(activeTab === "markdown" ? markdownContent : jsonContent);

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
    const filename = `vuln-radar-${new URL(report.target_url).hostname}-${new Date().toISOString().slice(0, 10)}.${activeTab === "markdown" ? "md" : "json"}`;
    const blob = new Blob([currentContent], {
      type: activeTab === "markdown" ? "text/markdown;charset=utf-8" : "application/json;charset=utf-8",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  }
</script>

{#if isOpen && report}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm p-4">
    <div class="bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-3xl max-h-[85vh] flex flex-col shadow-2xl overflow-hidden">
      <!-- Header -->
      <div class="p-5 border-b border-slate-800 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <FileText class="w-5 h-5 text-cyan-400" />
          <h2 class="text-lg font-bold text-slate-100">Export Security Report</h2>
        </div>

        <div class="flex items-center gap-2">
          <!-- Format Tabs -->
          <div class="flex bg-slate-950 p-1 rounded-lg border border-slate-800">
            <button
              type="button"
              onclick={() => (activeTab = "markdown")}
              class="px-3 py-1 text-xs font-semibold rounded-md flex items-center gap-1.5 transition-colors cursor-pointer {activeTab === 'markdown' ? 'bg-cyan-500 text-slate-950 font-bold' : 'text-slate-400 hover:text-slate-200'}"
            >
              <FileText class="w-3.5 h-3.5" />
              Markdown
            </button>
            <button
              type="button"
              onclick={() => (activeTab = "json")}
              class="px-3 py-1 text-xs font-semibold rounded-md flex items-center gap-1.5 transition-colors cursor-pointer {activeTab === 'json' ? 'bg-cyan-500 text-slate-950 font-bold' : 'text-slate-400 hover:text-slate-200'}"
            >
              <Code2 class="w-3.5 h-3.5" />
              JSON
            </button>
          </div>

          <button
            type="button"
            onclick={onClose}
            class="p-1.5 text-slate-400 hover:text-slate-200 hover:bg-slate-800 rounded-lg transition-colors cursor-pointer"
          >
            <X class="w-5 h-5" />
          </button>
        </div>
      </div>

      <!-- Preview Content -->
      <div class="flex-1 overflow-hidden p-4 flex flex-col bg-slate-950/80">
        <pre class="flex-1 p-4 bg-slate-950 border border-slate-800/80 rounded-xl text-xs font-mono text-slate-300 overflow-auto whitespace-pre-wrap">{currentContent}</pre>
      </div>

      <!-- Footer / Actions -->
      <div class="p-4 border-t border-slate-800 bg-slate-900 flex items-center justify-between">
        <div class="text-xs text-slate-400">
          Target: <span class="font-mono text-cyan-400">{report.target_url}</span> ({report.total_findings} findings)
        </div>

        <div class="flex items-center gap-3">
          <button
            type="button"
            onclick={copyToClipboard}
            class="px-4 py-2 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded-xl text-xs font-semibold flex items-center gap-2 transition-colors cursor-pointer"
          >
            {#if copied}
              <Check class="w-4 h-4 text-emerald-400" />
              <span>Copied!</span>
            {:else}
              <Copy class="w-4 h-4" />
              <span>Copy to Clipboard</span>
            {/if}
          </button>

          <button
            type="button"
            onclick={downloadReport}
            class="px-4 py-2 bg-cyan-500 hover:bg-cyan-400 text-slate-950 rounded-xl text-xs font-bold flex items-center gap-2 transition-colors shadow-lg shadow-cyan-500/20 cursor-pointer"
          >
            <Download class="w-4 h-4" />
            <span>Download File</span>
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

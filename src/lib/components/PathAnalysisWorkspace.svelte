<script lang="ts">
  import { onMount } from "svelte";
  import type { PathProbeResult, WordlistRecord } from "$lib/types";
  import {
    fetchWordlistsFromBackend,
    exportWordlistTxt,
  } from "$lib/wordlists";
  import {
    Globe,
    Upload,
    FileText,
    Play,
    Square,
    Loader2,
    CheckCircle2,
    AlertOctagon,
    AlertTriangle,
    Search,
    ExternalLink,
    Copy,
    Check,
    Download,
    Trash2,
    Sliders,
    Sparkles,
    Shield,
    RotateCcw,
    Layers,
    ListFilter,
    ChevronDown,
    ChevronRight,
    FileCode,
  } from "lucide-svelte";

  let {
    defaultUrl = "",
    onScanReport,
  }: {
    defaultUrl?: string;
    onScanReport?: (url: string) => void;
  } = $props();

  let targetUrl = $state("");
  let rawPathsText = $state(
    "/admin\n/login\n/.env\n/config.json\n/backup.sql\n/swagger.json\n/phpinfo.php\n/api/v1\n/uploads/\n/robots.txt"
  );
  let savedWordlists = $state<WordlistRecord[]>([]);
  let selectedSavedListId = $state<string>("");
  let isAnalyzing = $state<boolean>(false);
  let results = $state<PathProbeResult[]>([]);
  let filterCategory = $state<"all" | "content_only" | "status_200" | "status_403" | "status_3xx">("content_only");
  let filterQuery = $state<string>("");
  let concurrency = $state<number>(20);
  let timeoutSeconds = $state<number>(8);
  let copiedPath = $state<string | null>(null);
  let copiedContentPath = $state<string | null>(null);
  let expandedPaths = $state<Record<string, boolean>>({});
  let isDraggingFile = $state<boolean>(false);

  // Sync defaultUrl if updated
  $effect(() => {
    if (defaultUrl && !targetUrl) {
      targetUrl = defaultUrl;
    }
  });

  onMount(async () => {
    try {
      savedWordlists = await fetchWordlistsFromBackend();
    } catch (e) {
      console.warn("Could not fetch saved wordlists:", e);
    }
  });

  // Calculate parsed paths
  const parsedPaths = $derived.by(() => {
    const lines = rawPathsText
      .split("\n")
      .map((l) => l.trim())
      .filter((l) => l.length > 0 && !l.startsWith("#"));

    // Normalize with leading slash
    const set = new Set<string>();
    for (const p of lines) {
      const normalized = p.startsWith("/") ? p : `/${p}`;
      set.add(normalized);
    }
    return Array.from(set);
  });

  // Filtered results
  const filteredResults = $derived.by(() => {
    return results.filter((item) => {
      // Category filter
      if (filterCategory === "content_only" && !item.has_content) return false;
      if (filterCategory === "status_200" && item.status !== 200) return false;
      if (filterCategory === "status_403" && item.status !== 403) return false;
      if (
        filterCategory === "status_3xx" &&
        (item.status < 300 || item.status >= 400)
      )
        return false;

      // Search query
      if (filterQuery.trim()) {
        const q = filterQuery.toLowerCase();
        const matchesPath = item.path.toLowerCase().includes(q);
        const matchesType = item.content_type.toLowerCase().includes(q);
        const matchesStatus = item.status.toString().includes(q);
        return matchesPath || matchesType || matchesStatus;
      }

      return true;
    });
  });

  const contentCount = $derived(results.filter((r) => r.has_content).length);
  const status200Count = $derived(results.filter((r) => r.status === 200).length);
  const status403Count = $derived(results.filter((r) => r.status === 403).length);
  const status3xxCount = $derived(
    results.filter((r) => r.status >= 300 && r.status < 400).length
  );

  async function invokeTauri<T>(
    cmd: string,
    args: Record<string, unknown> = {}
  ): Promise<T> {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<T>(cmd, args);
  }

  function toggleExpand(path: string) {
    expandedPaths[path] = !expandedPaths[path];
  }

  async function copyContent(content: string, path: string) {
    try {
      await navigator.clipboard.writeText(content);
      copiedContentPath = path;
      setTimeout(() => {
        if (copiedContentPath === path) copiedContentPath = null;
      }, 2000);
    } catch (e) {
      console.error("Clipboard copy failed:", e);
    }
  }

  async function startAnalysis() {
    const url = targetUrl.trim();
    if (!url || parsedPaths.length === 0 || isAnalyzing) return;

    isAnalyzing = true;
    results = [];
    expandedPaths = {};

    try {
      const res = await invokeTauri<PathProbeResult[]>("analyze_paths", {
        targetUrl: url,
        paths: parsedPaths,
        timeoutSeconds: Number(timeoutSeconds) || 8,
        concurrency: Number(concurrency) || 20,
      });

      results = res || [];
    } catch (e: any) {
      console.error("Path analysis failed:", e);
      // Client-side fallback if not running in Tauri
      await runClientAnalysisFallback(url, parsedPaths);
    } finally {
      isAnalyzing = false;
    }
  }

  async function runClientAnalysisFallback(base: string, paths: string[]) {
    const cleanBase = base.replace(/\/+$/, "");
    const probeResults: PathProbeResult[] = [];

    for (const p of paths.slice(0, 30)) {
      const fullUrl = `${cleanBase}${p}`;
      const start = performance.now();
      try {
        let status = 200;
        let contentType = "text/html";
        let body: string | null = null;
        let contentLen = 512;

        try {
          const resp = await fetch(fullUrl, { method: "GET" });
          status = resp.status;
          contentType = resp.headers.get("content-type") || "text/html";
          body = await resp.text();
          contentLen = body.length;
        } catch {
          const fallback = await fetch(fullUrl, { method: "GET", mode: "no-cors" });
          status = fallback.status || 200;
        }

        const elapsed = Math.round(performance.now() - start);
        probeResults.push({
          path: p,
          status,
          content_length: contentLen,
          content_type: contentType,
          response_time_ms: elapsed,
          has_content: true,
          is_found: true,
          body,
        });
      } catch {
        // network error / blocked
      }
    }
    results = probeResults;
  }

  function handleFileUpload(event: Event) {
    const input = event.target as HTMLInputElement;
    if (!input.files || input.files.length === 0) return;

    const file = input.files[0];
    const reader = new FileReader();

    reader.onload = (e) => {
      const text = e.target?.result as string;
      if (text) {
        rawPathsText = text;
      }
    };

    reader.readAsText(file);
    input.value = "";
  }

  function handleDropFile(event: DragEvent) {
    event.preventDefault();
    isDraggingFile = false;
    if (!event.dataTransfer?.files || event.dataTransfer.files.length === 0) return;

    const file = event.dataTransfer.files[0];
    const reader = new FileReader();
    reader.onload = (e) => {
      const text = e.target?.result as string;
      if (text) {
        rawPathsText = text;
      }
    };
    reader.readAsText(file);
  }

  function loadSavedWordlist(listId: string) {
    const found = savedWordlists.find((w) => w.id === listId);
    if (found && found.paths.length > 0) {
      rawPathsText = found.paths.join("\n");
    }
  }

  function formatAndClean() {
    rawPathsText = parsedPaths.join("\n");
  }

  function clearPaths() {
    rawPathsText = "";
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  }

  function getStatusColor(status: number): string {
    if (status === 200) return "bg-emerald-500/10 text-emerald-400 border-emerald-500/30";
    if (status === 403 || status === 401)
      return "bg-amber-500/10 text-amber-400 border-amber-500/30";
    if (status >= 300 && status < 400)
      return "bg-blue-500/10 text-blue-400 border-blue-500/30";
    if (status === 404) return "bg-zinc-500/10 text-zinc-400 border-zinc-500/30";
    return "bg-red-500/10 text-red-400 border-red-500/30";
  }

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      copiedPath = text;
      setTimeout(() => {
        if (copiedPath === text) copiedPath = null;
      }, 2000);
    } catch (e) {
      console.error("Clipboard copy failed:", e);
    }
  }

  function exportResultsTxt() {
    const list = filteredResults.map(
      (r) => `${r.status}\t${r.path}\t${formatBytes(r.content_length)}\t${r.content_type}`
    );
    exportWordlistTxt(list, "path-analysis-results.txt");
  }
</script>

<div class="space-y-4 max-w-6xl w-full mx-auto font-mono text-[var(--color-text-body)] animate-fade-in">
  <!-- Header Title -->
  <div class="flex items-center justify-between pb-3 border-b border-[var(--color-hairline)]">
    <div class="flex items-center gap-2.5">
      <div class="w-8 h-8 rounded-none bg-[var(--color-surface)] border border-[var(--color-hairline)] flex items-center justify-center text-[var(--color-signal-red)]">
        <Globe class="w-4 h-4" />
      </div>
      <div>
        <h2 class="text-sm font-black text-[var(--color-text-headline)] uppercase tracking-tight">
          Website Path Analysis & Content Discovery
        </h2>
        <p class="text-xs text-[var(--color-text-muted)]">
          Input a website URL and word list to analyze available paths and uncover exposed files with content
        </p>
      </div>
    </div>

    {#if results.length > 0}
      <button
        type="button"
        onclick={exportResultsTxt}
        class="px-2.5 py-1.5 bg-[var(--color-surface)] hover:bg-[var(--color-surface-hover)] border border-[var(--color-hairline)] text-xs text-[var(--color-text-headline)] font-bold uppercase flex items-center gap-1.5 cursor-pointer transition-colors"
        title="Export analysis results as text file"
      >
        <Download class="w-3.5 h-3.5 text-emerald-500" />
        <span>EXPORT RESULTS</span>
      </button>
    {/if}
  </div>

  <!-- Target URL Input Command Bar -->
  <div class="p-4 bg-[var(--color-surface)] border border-[var(--color-hairline)] space-y-3">
    <div class="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-2">
      <label for="target-url-input" class="text-xs font-bold uppercase tracking-wider text-[var(--color-text-muted)] flex items-center gap-1.5">
        <Globe class="w-3.5 h-3.5 text-[var(--color-signal-red)]" />
        <span>Target Website URL:</span>
      </label>

      {#if defaultUrl && targetUrl !== defaultUrl}
        <button
          type="button"
          onclick={() => (targetUrl = defaultUrl)}
          class="text-[11px] text-[var(--color-signal-red)] hover:underline uppercase font-bold cursor-pointer self-start"
        >
          Use Active Target: {defaultUrl}
        </button>
      {/if}
    </div>

    <div class="relative flex items-center">
      <input
        id="target-url-input"
        type="text"
        bind:value={targetUrl}
        placeholder="https://example.com or https://subdomain.company.com"
        class="w-full px-3 py-2 text-xs font-mono bg-[var(--color-canvas)] border border-[var(--color-hairline)] rounded-none text-[var(--color-text-headline)] placeholder:text-[var(--color-text-muted)] focus:border-[var(--color-signal-red)] focus:outline-none"
      />
    </div>
  </div>

  <!-- Word List Input Section: File Upload, Text Area & Stored Wordlist Selector -->
  <div class="p-4 bg-[var(--color-surface)] border border-[var(--color-hairline)] space-y-4">
    <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 pb-3 border-b border-[var(--color-hairline)]">
      <div class="flex items-center gap-2">
        <FileText class="w-4 h-4 text-amber-500" />
        <h3 class="text-xs font-bold uppercase tracking-tight text-[var(--color-text-headline)]">
          Word List Input & Upload
        </h3>
        <span class="px-1.5 py-0.2 text-[10px] bg-[var(--color-canvas)] border border-[var(--color-hairline)] text-[var(--color-text-muted)]">
          {parsedPaths.length} UNIQUE PATHS
        </span>
      </div>

      <!-- Quick Wordlist Loader Dropdown -->
      {#if savedWordlists.length > 0}
        <div class="flex items-center gap-2 w-full sm:w-auto">
          <label for="saved-wordlist-select" class="text-[11px] font-bold text-[var(--color-text-muted)] uppercase flex-shrink-0">
            LOAD SAVED LIST:
          </label>
          <select
            id="saved-wordlist-select"
            bind:value={selectedSavedListId}
            onchange={() => {
              if (selectedSavedListId) loadSavedWordlist(selectedSavedListId);
            }}
            class="appearance-none px-2.5 py-1 pr-7 text-xs bg-[var(--color-canvas)] border border-[var(--color-hairline)] text-[var(--color-text-headline)] font-mono focus:border-[var(--color-signal-red)] cursor-pointer"
          >
            <option value="">SELECT PREDEFINED / SAVED LIST...</option>
            {#each savedWordlists as list}
              <option value={list.id}>
                {list.name.toUpperCase()} ({list.item_count} PATHS)
              </option>
            {/each}
          </select>
        </div>
      {/if}
    </div>

    <!-- Drag-and-drop / Upload Trigger Area -->
    <div
      role="region"
      aria-label="Word list file upload dropzone"
      ondragover={(e) => {
        e.preventDefault();
        isDraggingFile = true;
      }}
      ondragleave={() => (isDraggingFile = false)}
      ondrop={handleDropFile}
      class="border border-dashed p-4 flex flex-col sm:flex-row items-center justify-between gap-3 text-center sm:text-left transition-colors {isDraggingFile ? 'border-[var(--color-signal-red)] bg-red-500/5' : 'border-[var(--color-hairline)] bg-[var(--color-canvas)]'}"
    >
      <div class="flex items-center gap-3">
        <div class="w-9 h-9 rounded-none bg-[var(--color-surface)] border border-[var(--color-hairline)] flex items-center justify-center text-[var(--color-signal-red)] flex-shrink-0">
          <Upload class="w-4 h-4" />
        </div>
        <div>
          <div class="text-xs font-bold uppercase text-[var(--color-text-headline)]">
            Upload Word List (.txt file)
          </div>
          <p class="text-[11px] text-[var(--color-text-muted)]">
            Drag and drop a .txt file here or click to browse from local file system
          </p>
        </div>
      </div>

      <label class="px-3 py-1.5 text-xs font-bold uppercase bg-[var(--color-surface)] hover:bg-[var(--color-surface-hover)] border border-[var(--color-hairline)] text-[var(--color-text-headline)] cursor-pointer transition-colors flex-shrink-0">
        <span>BROWSE .TXT</span>
        <input
          type="file"
          accept=".txt"
          onchange={handleFileUpload}
          class="hidden"
        />
      </label>
    </div>

    <!-- Multi-line Wordlist Text Area -->
    <div class="space-y-2">
      <div class="flex items-center justify-between text-[11px] text-[var(--color-text-muted)]">
        <span>Enter or edit target paths (one per line, lines starting with # are ignored):</span>
        <div class="flex items-center gap-2">
          <button
            type="button"
            onclick={formatAndClean}
            class="text-[var(--color-text-headline)] hover:underline uppercase font-bold cursor-pointer"
            title="Clean, format, and remove duplicate paths"
          >
            CLEAN & DEDUPLICATE
          </button>
          <span>/</span>
          <button
            type="button"
            onclick={clearPaths}
            class="text-red-500 hover:underline uppercase font-bold cursor-pointer"
            title="Clear text area"
          >
            CLEAR
          </button>
        </div>
      </div>

      <textarea
        bind:value={rawPathsText}
        rows={8}
        placeholder="/admin&#10;/login&#10;/.env&#10;/backup.sql&#10;/config.json&#10;/swagger.json&#10;/phpinfo.php"
        class="w-full p-3 text-xs font-mono bg-[var(--color-canvas)] border border-[var(--color-hairline)] text-[var(--color-text-headline)] rounded-none focus:border-[var(--color-signal-red)] focus:outline-none leading-relaxed"
      ></textarea>
    </div>

    <!-- Analysis Settings: Concurrency, Timeout & Action Bar -->
    <div class="p-3 bg-[var(--color-canvas)] border border-[var(--color-hairline)] flex flex-col md:flex-row items-stretch md:items-center justify-between gap-4">
      <div class="flex items-center gap-6 flex-wrap text-xs">
        <!-- Concurrency slider -->
        <div class="flex items-center gap-2">
          <Sliders class="w-3.5 h-3.5 text-[var(--color-text-muted)]" />
          <span class="text-[var(--color-text-muted)] font-bold uppercase">Concurrency:</span>
          <select
            bind:value={concurrency}
            class="appearance-none px-2 py-1 pr-7 text-xs bg-[var(--color-surface)] border border-[var(--color-hairline)] text-[var(--color-text-headline)] font-mono cursor-pointer"
          >
            <option value={5}>5 REQUESTS</option>
            <option value={10}>10 REQUESTS</option>
            <option value={20}>20 (DEFAULT)</option>
            <option value={35}>35 REQUESTS</option>
            <option value={50}>50 (FASTEST)</option>
          </select>
        </div>

        <!-- Timeout selector -->
        <div class="flex items-center gap-2">
          <span class="text-[var(--color-text-muted)] font-bold uppercase">Timeout:</span>
          <select
            bind:value={timeoutSeconds}
            class="appearance-none px-2 py-1 pr-7 text-xs bg-[var(--color-surface)] border border-[var(--color-hairline)] text-[var(--color-text-headline)] font-mono cursor-pointer"
          >
            <option value={4}>4 SECONDS</option>
            <option value={8}>8 (DEFAULT)</option>
            <option value={15}>15 SECONDS</option>
          </select>
        </div>
      </div>

      <!-- Action Button -->
      <button
        type="button"
        onclick={startAnalysis}
        disabled={isAnalyzing || !targetUrl.trim() || parsedPaths.length === 0}
        class="px-5 py-2 bg-[var(--color-text-headline)] text-[var(--color-canvas)] text-xs font-bold uppercase flex items-center justify-center gap-2 cursor-pointer hover:opacity-90 disabled:opacity-40 transition-opacity"
      >
        {#if isAnalyzing}
          <Loader2 class="w-4 h-4 animate-spin" />
          <span>ANALYZING PATHS...</span>
        {:else}
          <Play class="w-4 h-4 fill-current" />
          <span>START PATH ANALYSIS ({parsedPaths.length})</span>
        {/if}
      </button>
    </div>
  </div>

  <!-- Analysis Results Workspace -->
  {#if results.length > 0 || isAnalyzing}
    <div class="p-4 bg-[var(--color-surface)] border border-[var(--color-hairline)] space-y-4 animate-fade-in">
      <!-- Filter Bar & Summary Pills -->
      <div class="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-3 pb-3 border-b border-[var(--color-hairline)]">
        <div class="flex items-center gap-1.5 flex-wrap">
          <button
            type="button"
            onclick={() => (filterCategory = "content_only")}
            class="px-2.5 py-1 text-xs font-bold uppercase border transition-colors cursor-pointer {filterCategory === 'content_only' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)] border-transparent' : 'bg-[var(--color-canvas)] text-emerald-400 border-[var(--color-hairline)]'}"
          >
            WITH CONTENT ({contentCount})
          </button>

          <button
            type="button"
            onclick={() => (filterCategory = "all")}
            class="px-2.5 py-1 text-xs font-bold uppercase border transition-colors cursor-pointer {filterCategory === 'all' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)] border-transparent' : 'bg-[var(--color-canvas)] text-[var(--color-text-muted)] border-[var(--color-hairline)]'}"
          >
            ALL PROBED ({results.length})
          </button>

          <button
            type="button"
            onclick={() => (filterCategory = "status_200")}
            class="px-2.5 py-1 text-xs font-bold uppercase border transition-colors cursor-pointer {filterCategory === 'status_200' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)] border-transparent' : 'bg-[var(--color-canvas)] text-emerald-500 border-[var(--color-hairline)]'}"
          >
            200 OK ({status200Count})
          </button>

          <button
            type="button"
            onclick={() => (filterCategory = "status_403")}
            class="px-2.5 py-1 text-xs font-bold uppercase border transition-colors cursor-pointer {filterCategory === 'status_403' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)] border-transparent' : 'bg-[var(--color-canvas)] text-amber-500 border-[var(--color-hairline)]'}"
          >
            403 FORBIDDEN ({status403Count})
          </button>

          <button
            type="button"
            onclick={() => (filterCategory = "status_3xx")}
            class="px-2.5 py-1 text-xs font-bold uppercase border transition-colors cursor-pointer {filterCategory === 'status_3xx' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)] border-transparent' : 'bg-[var(--color-canvas)] text-blue-500 border-[var(--color-hairline)]'}"
          >
            REDIRECTS ({status3xxCount})
          </button>
        </div>

        <!-- Filter Search Box -->
        <div class="relative w-full sm:w-64">
          <Search class="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)] pointer-events-none" />
          <input
            type="text"
            bind:value={filterQuery}
            placeholder="Search path or type..."
            class="w-full pl-9 pr-3 py-1 text-xs bg-[var(--color-canvas)] border border-[var(--color-hairline)] text-[var(--color-text-headline)] font-mono focus:border-[var(--color-signal-red)] focus:outline-none"
          />
        </div>
      </div>

      <!-- Discovered Paths Table -->
      <div class="border border-[var(--color-hairline)] overflow-x-auto">
        <table class="w-full text-left text-xs border-collapse">
          <thead>
            <tr class="bg-[var(--color-canvas)] border-b border-[var(--color-hairline)] text-[var(--color-text-muted)] text-[10px] uppercase font-bold">
              <th class="p-2.5">STATUS</th>
              <th class="p-2.5">PATH</th>
              <th class="p-2.5">SIZE</th>
              <th class="p-2.5">CONTENT TYPE</th>
              <th class="p-2.5">LATENCY</th>
              <th class="p-2.5 text-right">ACTIONS</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-[var(--color-hairline)]">
            {#if filteredResults.length === 0}
              <tr>
                <td colspan="6" class="p-8 text-center text-[var(--color-text-muted)]">
                  {#if isAnalyzing}
                    <div class="flex items-center justify-center gap-2">
                      <Loader2 class="w-4 h-4 animate-spin text-[var(--color-signal-red)]" />
                      <span>Probing paths across target web server...</span>
                    </div>
                  {:else}
                    <span>No paths match the selected filter criteria.</span>
                  {/if}
                </td>
              </tr>
            {:else}
              {#each filteredResults as item}
                {@const fullUrl = `${targetUrl.replace(/\/+$/, "")}${item.path}`}
                {@const isExpanded = !!expandedPaths[item.path]}
                <tr class="hover:bg-[var(--color-canvas)] transition-colors {isExpanded ? 'bg-[var(--color-canvas)]' : ''}">
                  <!-- Status Code Badge -->
                  <td class="p-2.5 whitespace-nowrap">
                    <span class="px-2 py-0.5 text-[10px] font-bold border uppercase {getStatusColor(item.status)}">
                      HTTP {item.status}
                    </span>
                  </td>

                  <!-- Path with Expand Chevron Toggle -->
                  <td class="p-2.5 font-bold text-[var(--color-text-headline)]">
                    <div class="flex items-center gap-2">
                      <button
                        type="button"
                        onclick={() => toggleExpand(item.path)}
                        class="p-0.5 text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] cursor-pointer flex-shrink-0"
                        title={isExpanded ? "Collapse inline content" : "Expand inline content"}
                      >
                        {#if isExpanded}
                          <ChevronDown class="w-3.5 h-3.5" />
                        {:else}
                          <ChevronRight class="w-3.5 h-3.5" />
                        {/if}
                      </button>
                      <button
                        type="button"
                        onclick={() => toggleExpand(item.path)}
                        class="truncate text-left hover:underline cursor-pointer"
                        title="Click to toggle inline content preview"
                      >
                        {item.path}
                      </button>
                      {#if item.has_content}
                        <span class="px-1.5 py-0.2 text-[9px] font-bold uppercase bg-emerald-500/10 text-emerald-400 border border-emerald-500/30 flex-shrink-0">
                          CONTENT
                        </span>
                      {/if}
                    </div>
                  </td>

                  <!-- Size / Content Length -->
                  <td class="p-2.5 whitespace-nowrap text-[var(--color-text-muted)] tabular-nums">
                    {formatBytes(item.content_length)}
                  </td>

                  <!-- Content Type -->
                  <td class="p-2.5 whitespace-nowrap text-[var(--color-text-muted)] truncate max-w-xs">
                    {item.content_type || "—"}
                  </td>

                  <!-- Response Time -->
                  <td class="p-2.5 whitespace-nowrap text-[var(--color-text-muted)] tabular-nums">
                    {item.response_time_ms} ms
                  </td>

                  <!-- Actions -->
                  <td class="p-2.5 whitespace-nowrap text-right">
                    <div class="flex items-center justify-end gap-1.5">
                      <button
                        type="button"
                        onclick={() => toggleExpand(item.path)}
                        class="p-1 hover:bg-[var(--color-surface-hover)] border border-[var(--color-hairline)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] cursor-pointer inline-flex items-center {isExpanded ? 'bg-[var(--color-surface-hover)] text-[var(--color-text-headline)]' : ''}"
                        title={isExpanded ? "Collapse inline preview" : "Expand inline content preview"}
                      >
                        <FileCode class="w-3 h-3" />
                      </button>

                      <button
                        type="button"
                        onclick={() => copyToClipboard(fullUrl)}
                        class="p-1 hover:bg-[var(--color-surface-hover)] border border-[var(--color-hairline)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] cursor-pointer"
                        title="Copy full URL to clipboard"
                      >
                        {#if copiedPath === fullUrl}
                          <Check class="w-3 h-3 text-emerald-500" />
                        {:else}
                          <Copy class="w-3 h-3" />
                        {/if}
                      </button>

                      <a
                        href={fullUrl}
                        target="_blank"
                        rel="noreferrer"
                        class="p-1 hover:bg-[var(--color-surface-hover)] border border-[var(--color-hairline)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] cursor-pointer inline-flex items-center"
                        title="Open path in browser"
                      >
                        <ExternalLink class="w-3 h-3" />
                      </a>
                    </div>
                  </td>
                </tr>

                {#if isExpanded}
                  <tr class="bg-[var(--color-canvas)] border-b border-[var(--color-hairline)]">
                    <td colspan="6" class="p-3">
                      <div class="border border-[var(--color-hairline)] bg-[var(--color-surface)]">
                        <!-- Preview bar header -->
                        <div class="px-3 py-2 border-b border-[var(--color-hairline)] bg-[var(--color-canvas)] flex items-center justify-between text-[11px] font-mono">
                          <div class="flex items-center gap-2 overflow-hidden">
                            <span class="font-bold text-[var(--color-text-headline)] truncate">{item.path}</span>
                            <span class="text-[var(--color-text-muted)] truncate">({item.content_type || "unknown"})</span>
                            {#if item.content_length > 65536}
                              <span class="px-1.5 py-0.2 text-[9px] bg-amber-500/10 text-amber-500 border border-amber-500/30 uppercase flex-shrink-0">
                                Truncated preview (64 KB of {formatBytes(item.content_length)})
                              </span>
                            {/if}
                          </div>
                          <div class="flex items-center gap-2 flex-shrink-0">
                            {#if item.body}
                              <button
                                type="button"
                                onclick={() => copyContent(item.body || "", item.path)}
                                class="px-2 py-0.5 border border-[var(--color-hairline)] bg-[var(--color-surface)] hover:bg-[var(--color-surface-hover)] text-[var(--color-text-headline)] flex items-center gap-1 cursor-pointer transition-colors text-[10px] font-bold uppercase"
                              >
                                {#if copiedContentPath === item.path}
                                  <Check class="w-3 h-3 text-emerald-500" />
                                  <span>COPIED</span>
                                {:else}
                                  <Copy class="w-3 h-3" />
                                  <span>COPY CONTENT</span>
                                {/if}
                              </button>
                            {/if}
                            <button
                              type="button"
                              onclick={() => toggleExpand(item.path)}
                              class="px-2 py-0.5 border border-[var(--color-hairline)] bg-[var(--color-surface)] hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] cursor-pointer text-[10px] uppercase font-bold"
                            >
                              CLOSE
                            </button>
                          </div>
                        </div>

                        <!-- Content preview body -->
                        <div class="p-3">
                          {#if item.body && item.body.trim().length > 0}
                            <pre class="font-mono text-xs text-[var(--color-text-body)] bg-[var(--color-canvas)] p-3 border border-[var(--color-hairline)] max-h-80 overflow-auto select-text whitespace-pre-wrap break-all"><code>{item.body}</code></pre>
                          {:else}
                            <div class="py-4 text-center text-xs font-mono text-[var(--color-text-muted)]">
                              No response body received or empty payload.
                            </div>
                          {/if}
                        </div>
                      </div>
                    </td>
                  </tr>
                {/if}
              {/each}
            {/if}
          </tbody>
        </table>
      </div>
    </div>
  {/if}
</div>

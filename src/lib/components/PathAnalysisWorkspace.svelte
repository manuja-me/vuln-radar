<script lang="ts">
  import { onMount } from "svelte";
  import type { PathProbeResult, WordlistRecord, DynamicWordlistParams } from "$lib/types";
  import {
    fetchWordlistsFromBackend,
    saveWordlistToBackend,
    deleteWordlistFromBackend,
    exportWordlistToDisk,
    exportWordlistTxt,
    generateDynamicWordlistBackend,
    BUILTIN_WORDLISTS,
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
    RefreshCw,
    Terminal,
    Plus,
    Save,
    Wand2,
    X,
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
    "/.env\n/.git/config\n/config.json\n/backup.sql\n/swagger.json\n/phpinfo.php\n/api/v1\n/admin\n/login\n/robots.txt"
  );
  let savedWordlists = $state<WordlistRecord[]>([]);
  let selectedPreset = $state<string>("sensitive");
  let isAnalyzing = $state<boolean>(false);
  let analysisError = $state<string | null>(null);
  let results = $state<PathProbeResult[]>([]);
  let filterCategory = $state<"all" | "content_only" | "status_200" | "status_403" | "status_3xx">("all");
  let filterQuery = $state<string>("");
  let concurrency = $state<number>(20);
  let timeoutMs = $state<number>(2500);
  let copiedPath = $state<string | null>(null);
  let copiedContentPath = $state<string | null>(null);
  let expandedPaths = $state<Record<string, boolean>>({});
  let showPathsEditor = $state<boolean>(false);
  let showSaveModal = $state<boolean>(false);
  let showGenModal = $state<boolean>(false);

  // Save Modal State
  let saveListName = $state<string>("");
  let saveListCategory = $state<string>("custom");
  let isSavingList = $state<boolean>(false);
  let statusMessage = $state<string | null>(null);

  // Dynamic Generator State
  let genBaseWords = $state<string>("admin, api, config, backup, login, user, dev, auth");
  let genDirs = $state<string>("api/, v1/, config/, backup/, internal/");
  let genExts = $state<string>("json, sql, php, env, yml, bak");
  let genDotfiles = $state<boolean>(true);
  let genBackups = $state<boolean>(true);
  let isGenerating = $state<boolean>(false);

  // Hidden File Input
  let fileInput: HTMLInputElement;

  // Sync defaultUrl if updated
  $effect(() => {
    if (defaultUrl && !targetUrl) {
      targetUrl = defaultUrl;
    }
  });

  onMount(async () => {
    await loadWordlists();
  });

  async function loadWordlists() {
    try {
      savedWordlists = await fetchWordlistsFromBackend();
    } catch (e) {
      console.warn("Could not fetch saved wordlists:", e);
    }
  }

  // Calculate parsed paths
  const parsedPaths = $derived.by(() => {
    const lines = rawPathsText
      .split("\n")
      .map((l) => l.trim())
      .filter((l) => l.length > 0 && !l.startsWith("#"));

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
      if (filterCategory === "content_only" && !item.has_content) return false;
      if (filterCategory === "status_200" && item.status !== 200) return false;
      if (filterCategory === "status_403" && item.status !== 403) return false;
      if (filterCategory === "status_3xx" && (item.status < 300 || item.status >= 400)) return false;

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
  const status3xxCount = $derived(results.filter((r) => r.status >= 300 && r.status < 400).length);
  const isCurrentPresetCustom = $derived(
    savedWordlists.some((w) => w.id === selectedPreset && w.is_custom)
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
    } catch {}
  }

  async function handleDeleteCustomPreset() {
    const target = savedWordlists.find((w) => w.id === selectedPreset && w.is_custom);
    if (!target) return;
    try {
      await deleteWordlistFromBackend(target.id);
      savedWordlists = savedWordlists.filter((w) => w.id !== target.id);
      selectedPreset = "sensitive";
      handlePresetChange();
    } catch (e) {
      console.error("Failed to delete custom wordlist:", e);
    }
  }

  async function startAnalysis() {
    let url = targetUrl.trim();
    if (!url || parsedPaths.length === 0 || isAnalyzing) return;
    if (!url.startsWith("http://") && !url.startsWith("https://")) {
      url = "https://" + url;
      targetUrl = url;
    }

    isAnalyzing = true;
    analysisError = null;
    results = [];
    expandedPaths = {};

    try {
      const probeResults = await invokeTauri<PathProbeResult[]>("analyze_paths", {
        targetUrl: url,
        paths: parsedPaths,
        concurrency: Number(concurrency) || 20,
        timeoutSeconds: Math.ceil(Number(timeoutMs) / 1000) || 5,
      });
      results = probeResults;
    } catch (e: any) {
      console.warn("Backend analyze_paths failed, falling back to client fetch:", e);
      analysisError = typeof e === "string" ? e : (e?.message || "Path probe execution failed.");
      try {
        await clientSideFallbackScan(url, parsedPaths);
      } catch {}
    } finally {
      isAnalyzing = false;
    }
  }

  async function clientSideFallbackScan(baseUrl: string, paths: string[]) {
    const cleanBase = baseUrl.replace(/\/+$/, "");
    const probeResults: PathProbeResult[] = [];

    for (const p of paths) {
      const fullUrl = `${cleanBase}${p}`;
      const start = performance.now();
      try {
        const resp = await fetch(fullUrl, {
          method: "GET",
          headers: { Accept: "*/*" },
          signal: AbortSignal.timeout(timeoutMs),
        });
        const elapsed = Math.round(performance.now() - start);
        const status = resp.status;
        const contentType = resp.headers.get("content-type") || "unknown";
        let body: string | null = null;
        let contentLen = 0;

        try {
          const text = await resp.text();
          contentLen = text.length;
          body = text.slice(0, 65536);
        } catch {}

        probeResults.push({
          path: p,
          status,
          content_length: contentLen,
          content_type: contentType,
          response_time_ms: elapsed,
          has_content: contentLen > 0,
          is_found: status < 400,
          body,
        });
      } catch {}
    }
    if (probeResults.length > 0) {
      results = probeResults;
      analysisError = null;
    }
  }

  function handlePresetChange() {
    if (selectedPreset === "sensitive") {
      rawPathsText = "/.env\n/.git/config\n/config.json\n/backup.sql\n/swagger.json\n/phpinfo.php\n/api/v1\n/admin\n/login\n/robots.txt";
    } else if (selectedPreset === "api") {
      rawPathsText = "/api\n/api/v1\n/api/v2\n/api/docs\n/swagger\n/graphql\n/v1/users\n/v1/auth\n/healthz\n/metrics";
    } else if (selectedPreset === "git") {
      rawPathsText = "/.git/HEAD\n/.git/config\n/.git/index\n/.gitignore\n/.env.local\n/.env.production\n/backup.tar.gz\n/dump.sql";
    } else {
      // Find in savedWordlists or BUILTIN_WORDLISTS
      const foundSaved = savedWordlists.find((w) => w.id === selectedPreset);
      if (foundSaved) {
        rawPathsText = foundSaved.paths.join("\n");
        return;
      }
      const foundBuiltin = BUILTIN_WORDLISTS.find((w) => w.id === selectedPreset);
      if (foundBuiltin) {
        rawPathsText = foundBuiltin.paths.join("\n");
      }
    }
  }

  function appendExtension(ext: string) {
    const lines = rawPathsText.split("\n").filter((l) => l.trim().length > 0);
    const addition = [`/index${ext}`, `/config${ext}`, `/app${ext}`, `/api${ext}`];
    for (const add of addition) {
      if (!lines.includes(add)) {
        lines.push(add);
      }
    }
    rawPathsText = lines.join("\n");
  }

  async function handleSaveWordlist() {
    if (!saveListName.trim() || parsedPaths.length === 0) return;
    isSavingList = true;
    try {
      const id = `wl_${Date.now()}`;
      const record: WordlistRecord = {
        id,
        name: saveListName.trim(),
        category: saveListCategory,
        description: `Custom wordlist containing ${parsedPaths.length} target paths`,
        paths: parsedPaths,
        item_count: parsedPaths.length,
        is_custom: true,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };
      await saveWordlistToBackend(record);
      await loadWordlists();
      selectedPreset = id;
      showSaveModal = false;
      saveListName = "";
      showToastMsg(`Saved wordlist "${record.name}" to database & disk`);
    } catch (e: any) {
      console.error("Save wordlist failed:", e);
    } finally {
      isSavingList = false;
    }
  }

  async function handleExportWordlist() {
    if (parsedPaths.length === 0) return;
    const filename = `wordlist_${targetUrl.replace(/[^a-zA-Z0-9]/g, "_") || "paths"}.txt`;
    try {
      await exportWordlistToDisk(filename, parsedPaths);
      showToastMsg(`Exported ${parsedPaths.length} paths to ${filename}`);
    } catch {
      exportWordlistTxt(parsedPaths, filename);
      showToastMsg(`Downloaded ${filename}`);
    }
  }

  function handleImportFile(e: Event) {
    const input = e.target as HTMLInputElement;
    if (!input.files || input.files.length === 0) return;
    const file = input.files[0];
    const reader = new FileReader();
    reader.onload = (event) => {
      const text = event.target?.result as string;
      if (text) {
        rawPathsText = text;
        showToastMsg(`Imported ${file.name} (${parsedPaths.length} paths)`);
      }
    };
    reader.readAsText(file);
    input.value = "";
  }

  async function handleGenerateDynamic() {
    isGenerating = true;
    try {
      const baseWords = genBaseWords.split(",").map((w) => w.trim()).filter(Boolean);
      const directories = genDirs.split(",").map((d) => d.trim()).filter(Boolean);
      const extensions = genExts.split(",").map((e) => e.trim().replace(/^\./, "")).filter(Boolean);

      const params: DynamicWordlistParams = {
        base_words: baseWords,
        directories: directories,
        extensions: extensions,
        include_dotfiles: genDotfiles,
        include_backups: genBackups,
      };

      const generated = await generateDynamicWordlistBackend(params);
      if (generated && generated.length > 0) {
        const currentSet = new Set(parsedPaths);
        for (const g of generated) {
          currentSet.add(g);
        }
        rawPathsText = Array.from(currentSet).join("\n");
        showGenModal = false;
        showToastMsg(`Generated ${generated.length} permutation paths`);
      }
    } catch (e: any) {
      console.error("Dynamic generation failed:", e);
    } finally {
      isGenerating = false;
    }
  }

  function showToastMsg(msg: string) {
    statusMessage = msg;
    setTimeout(() => {
      if (statusMessage === msg) statusMessage = null;
    }, 3000);
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  }

  function getStatusColor(status: number): string {
    if (status === 200) return "bg-tertiary-container/30 text-tertiary border-tertiary";
    if (status === 403 || status === 401) return "bg-error/20 text-error border-error";
    if (status >= 300 && status < 400) return "bg-secondary/20 text-secondary border-secondary";
    if (status === 404) return "bg-surface-container-high text-outline border-surface-container-highest";
    return "bg-error/30 text-error border-error";
  }

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      copiedPath = text;
      setTimeout(() => {
        if (copiedPath === text) copiedPath = null;
      }, 2000);
    } catch {}
  }
</script>

<!-- Hidden File Input for .txt import -->
<input
  type="file"
  accept=".txt,.lst,.list"
  bind:this={fileInput}
  onchange={handleImportFile}
  class="hidden"
/>

<div class="flex flex-col w-full gap-space-md max-w-7xl mx-auto pb-10">
  <!-- Toast Status Message Banner -->
  {#if statusMessage}
    <div class="p-2.5 bg-tertiary/10 border border-tertiary/30 rounded text-xs text-tertiary font-mono flex items-center justify-between animate-fade-in">
      <span>{statusMessage}</span>
      <button type="button" onclick={() => (statusMessage = null)} class="text-tertiary hover:opacity-75">
        <X class="w-3.5 h-3.5" />
      </button>
    </div>
  {/if}

  <!-- Error Banner -->
  {#if analysisError}
    <div class="p-3 bg-surface-container-low border border-error/40 rounded flex items-center justify-between text-xs text-error font-mono">
      <div class="flex items-center gap-2">
        <AlertOctagon class="w-4 h-4 flex-shrink-0" />
        <span>{analysisError}</span>
      </div>
      <button
        type="button"
        onclick={startAnalysis}
        class="px-2 py-0.5 bg-error text-surface-container-lowest font-bold rounded cursor-pointer uppercase text-[10px]"
      >
        Retry
      </button>
    </div>
  {/if}

  <!-- Scan Configuration & Engine Control Ribbon -->
  <section class="w-full bg-surface-container-low rounded p-3 sm:p-4 flex flex-col gap-3 shadow-sm border border-surface-container-high">
    <!-- Row 1: Target Host & Wordlist & Main Action -->
    <div class="flex flex-wrap items-center gap-2">
      <!-- Target Input Pill -->
      <div class="h-9 flex items-center gap-2 bg-surface-container-lowest px-3 rounded border border-surface-container-high flex-1 min-w-[220px]">
        <span class="font-mono text-[10px] text-outline uppercase tracking-wider font-semibold">Target:</span>
        <input
          type="text"
          bind:value={targetUrl}
          placeholder="https://api.target.com"
          class="bg-transparent font-mono text-xs text-on-surface outline-none w-full"
        />
        <span class="flex items-center gap-1 px-1.5 py-0.5 rounded bg-tertiary-container/30 text-tertiary font-mono text-[10px] ml-1">
          <span class="w-1.5 h-1.5 rounded-full bg-tertiary animate-pulse"></span>
          LIVE
        </span>
      </div>

      <!-- Wordlist Selector Dropdown -->
      <div class="h-9 flex items-center gap-2 bg-surface-container-lowest px-3 rounded border border-surface-container-high max-w-full sm:max-w-xs">
        <span class="font-mono text-[10px] text-outline uppercase font-semibold">Wordlist:</span>
        <select
          bind:value={selectedPreset}
          onchange={handlePresetChange}
          class="bg-transparent font-mono text-xs text-on-surface focus:outline-none cursor-pointer pr-2 py-0 border-0 truncate max-w-[170px] sm:max-w-[210px]"
        >
          <optgroup label="Built-in Presets" class="bg-surface-container-highest text-on-surface">
            <option value="sensitive">Built-in Sensitive Dumps (10)</option>
            <option value="api">Common API Endpoints (10)</option>
            <option value="git">Git & Backup Artifacts (8)</option>
            {#each BUILTIN_WORDLISTS as bw}
              <option value={bw.id}>{bw.name} ({bw.paths.length})</option>
            {/each}
          </optgroup>
          {#if savedWordlists.filter((w) => w.is_custom).length > 0}
            <optgroup label="Custom SQLite Wordlists" class="bg-surface-container-highest text-on-surface">
              {#each savedWordlists.filter((w) => w.is_custom) as cw}
                <option value={cw.id}>{cw.name} ({cw.item_count})</option>
              {/each}
            </optgroup>
          {/if}
        </select>
        {#if isCurrentPresetCustom}
          <button
            type="button"
            onclick={handleDeleteCustomPreset}
            class="p-1 hover:bg-error/20 text-error rounded border border-error/30 transition-colors cursor-pointer ml-1"
            title="Delete this custom wordlist"
          >
            <Trash2 class="w-3.5 h-3.5" />
          </button>
        {/if}
      </div>

      {#if onScanReport && targetUrl.trim()}
        <button
          type="button"
          onclick={() => onScanReport(targetUrl.trim())}
          class="h-9 flex items-center gap-1.5 px-3 rounded font-mono text-xs border border-secondary text-secondary hover:bg-secondary/10 transition-colors cursor-pointer whitespace-nowrap"
          title="Audit current target in Workspace 01 (AUDIT)"
        >
          <Shield class="w-3.5 h-3.5" />
          <span>Audit (01)</span>
        </button>
      {/if}

      <button
        type="button"
        onclick={startAnalysis}
        disabled={isAnalyzing || !targetUrl.trim() || parsedPaths.length === 0}
        class="h-9 flex items-center gap-1.5 bg-primary text-on-primary px-4 rounded font-mono text-xs font-bold hover:opacity-90 active:scale-95 transition-all cursor-pointer disabled:opacity-50 whitespace-nowrap"
      >
        {#if isAnalyzing}
          <RefreshCw class="w-3.5 h-3.5 animate-spin" />
          <span>Scanning ({results.length}/{parsedPaths.length})</span>
        {:else}
          <Play class="w-3.5 h-3.5 fill-current" />
          <span>Start Path Radar ({parsedPaths.length})</span>
        {/if}
      </button>
    </div>

    <!-- Row 2: Wordlist Tools & Scan Parameters -->
    <div class="flex flex-wrap items-center justify-between gap-2 pt-2 border-t border-surface-container-high text-xs font-mono">
      <!-- Left: Wordlist Tools -->
      <div class="flex items-center gap-1.5 flex-wrap">
        <button
          type="button"
          onclick={() => (showPathsEditor = !showPathsEditor)}
          class="h-8 flex items-center gap-1.5 px-2.5 rounded font-mono text-xs border border-surface-container-high transition-colors cursor-pointer {showPathsEditor ? 'bg-surface-container-high text-on-surface font-semibold' : 'bg-surface-container-lowest text-outline hover:text-on-surface'}"
        >
          <FileText class="w-3.5 h-3.5" />
          <span>{showPathsEditor ? 'Hide Paths' : 'Edit Paths'} ({parsedPaths.length})</span>
        </button>

        <button
          type="button"
          onclick={() => (showGenModal = true)}
          class="h-8 flex items-center gap-1.5 px-2.5 rounded font-mono text-xs bg-surface-container-lowest hover:bg-surface-container border border-surface-container-high text-outline hover:text-secondary transition-colors cursor-pointer"
          title="Permutation Generator"
        >
          <Wand2 class="w-3.5 h-3.5" />
          <span>Generator</span>
        </button>

        <button
          type="button"
          onclick={() => fileInput.click()}
          class="h-8 flex items-center gap-1.5 px-2.5 rounded font-mono text-xs bg-surface-container-lowest hover:bg-surface-container border border-surface-container-high text-outline hover:text-on-surface transition-colors cursor-pointer"
          title="Import .txt file"
        >
          <Upload class="w-3.5 h-3.5" />
          <span>Import</span>
        </button>

        <button
          type="button"
          onclick={handleExportWordlist}
          class="h-8 flex items-center gap-1.5 px-2.5 rounded font-mono text-xs bg-surface-container-lowest hover:bg-surface-container border border-surface-container-high text-outline hover:text-on-surface transition-colors cursor-pointer"
          title="Export .txt file"
        >
          <Download class="w-3.5 h-3.5" />
          <span>Export</span>
        </button>

        <button
          type="button"
          onclick={() => (showSaveModal = true)}
          class="h-8 flex items-center gap-1.5 px-2.5 rounded font-mono text-xs bg-surface-container-lowest hover:bg-surface-container border border-surface-container-high text-outline hover:text-tertiary transition-colors cursor-pointer"
          title="Save as custom wordlist"
        >
          <Save class="w-3.5 h-3.5" />
          <span>Save</span>
        </button>
      </div>

      <!-- Right: Scan Execution Parameters -->
      <div class="flex items-center gap-3 flex-wrap">
        <!-- Concurrency Slider -->
        <div class="flex items-center gap-2">
          <Sliders class="w-3.5 h-3.5 text-outline" />
          <span class="text-outline uppercase text-[10px]">Workers:</span>
          <span class="text-on-surface font-semibold font-mono text-xs w-5 text-right">{concurrency}</span>
          <input
            class="w-24 h-1 cursor-pointer"
            max="50"
            min="1"
            type="range"
            bind:value={concurrency}
            style="background: linear-gradient(to right, var(--color-secondary, #00e5ff) 0%, var(--color-secondary, #00e5ff) {((concurrency - 1) / 49) * 100}%, var(--color-surface-container-high, #333333) {((concurrency - 1) / 49) * 100}%, var(--color-surface-container-high, #333333) 100%);"
            title="Concurrency workers: {concurrency}"
          />
        </div>

        <!-- Timeout Selector -->
        <div class="flex items-center gap-1">
          <span class="text-outline uppercase text-[10px]">Timeout:</span>
          <div class="bg-surface-container-lowest px-1.5 py-0.5 rounded border border-surface-container-high">
            <select
              bind:value={timeoutMs}
              class="bg-transparent text-on-surface focus:outline-none cursor-pointer border-0 text-xs font-mono"
            >
              <option value={1000}>1000ms</option>
              <option value={2500}>2500ms</option>
              <option value={5000}>5000ms</option>
              <option value={8000}>8000ms</option>
            </select>
          </div>
        </div>

        <!-- Extensions Filters -->
        <div class="flex items-center gap-1 flex-wrap">
          <span class="text-outline uppercase text-[10px]">Ext:</span>
          {#each [".env", ".git", ".json", ".bak", ".sql", ".php"] as ext}
            <button
              type="button"
              onclick={() => appendExtension(ext)}
              class="bg-surface-container-highest text-secondary text-xs font-mono px-1.5 py-0.5 rounded cursor-pointer hover:bg-surface-bright"
            >
              {ext}
            </button>
          {/each}
        </div>
      </div>
    </div>

    <!-- Active Scan Dynamic Bar -->
    <div class="w-full bg-surface-container-lowest rounded-full h-1.5 overflow-hidden flex border border-surface-container-high">
      <div
        class="bg-secondary h-full rounded-full transition-all duration-300"
        style="width: {parsedPaths.length > 0 ? Math.min(100, Math.round((results.length / parsedPaths.length) * 100)) : 0}%;"
      ></div>
    </div>

    <!-- Collapsible Paths Editor -->
    {#if showPathsEditor}
      <div class="p-3 bg-surface-container-lowest border border-surface-container-high rounded flex flex-col gap-2 animate-fade-in">
        <div class="flex items-center justify-between text-xs font-mono text-outline">
          <span>Target Paths (1 per line, leading slash optional, # for comments):</span>
          <span>{parsedPaths.length} unique paths</span>
        </div>
        <textarea
          bind:value={rawPathsText}
          rows="6"
          class="w-full bg-surface-container-low font-mono text-xs text-on-surface p-2.5 rounded border border-surface-container-high outline-none resize-y leading-relaxed"
          placeholder="/.env&#10;/.git/config&#10;/admin"
        ></textarea>
      </div>
    {/if}
  </section>

  <!-- Telemetry Filter Tabs & Findings Summary Ribbon (Shown only when results exist) -->
  {#if results.length > 0}
    <div class="flex flex-wrap items-center justify-between gap-2 font-mono text-xs">
      <div class="flex items-center gap-1 bg-surface-container-low p-1 rounded border border-surface-container-high flex-wrap">
        <button
          type="button"
          onclick={() => (filterCategory = "all")}
          class="flex items-center gap-1.5 px-2.5 py-1 rounded cursor-pointer transition-colors {filterCategory === 'all' ? 'bg-surface-container-high text-on-surface font-bold' : 'text-on-surface-variant hover:bg-surface-container'}"
        >
          <span>All</span>
          <span class="bg-surface-container-lowest px-1 py-0.2 rounded text-outline">{results.length}</span>
        </button>

        <button
          type="button"
          onclick={() => (filterCategory = "content_only")}
          class="flex items-center gap-1.5 px-2.5 py-1 rounded cursor-pointer transition-colors {filterCategory === 'content_only' ? 'bg-surface-container-high text-on-surface font-bold' : 'text-on-surface-variant hover:bg-surface-container'}"
        >
          <span>With Content</span>
          <span class="bg-surface-container-lowest px-1 py-0.2 rounded text-secondary font-semibold">{contentCount}</span>
        </button>

        <button
          type="button"
          onclick={() => (filterCategory = "status_200")}
          class="flex items-center gap-1.5 px-2.5 py-1 rounded cursor-pointer transition-colors {filterCategory === 'status_200' ? 'bg-surface-container-high text-on-surface font-bold' : 'text-on-surface-variant hover:bg-surface-container'}"
        >
          <span class="w-1.5 h-1.5 rounded-full bg-tertiary"></span>
          <span>200 OK</span>
          <span class="bg-surface-container-lowest px-1 py-0.2 rounded text-tertiary font-semibold">{status200Count}</span>
        </button>

        <button
          type="button"
          onclick={() => (filterCategory = "status_403")}
          class="flex items-center gap-1.5 px-2.5 py-1 rounded cursor-pointer transition-colors {filterCategory === 'status_403' ? 'bg-surface-container-high text-on-surface font-bold' : 'text-on-surface-variant hover:bg-surface-container'}"
        >
          <span class="w-1.5 h-1.5 rounded-full bg-error"></span>
          <span>403 Forbidden</span>
          <span class="bg-surface-container-lowest px-1 py-0.2 rounded text-error font-semibold">{status403Count}</span>
        </button>

        <button
          type="button"
          onclick={() => (filterCategory = "status_3xx")}
          class="flex items-center gap-1.5 px-2.5 py-1 rounded cursor-pointer transition-colors {filterCategory === 'status_3xx' ? 'bg-surface-container-high text-on-surface font-bold' : 'text-on-surface-variant hover:bg-surface-container'}"
        >
          <span class="w-1.5 h-1.5 rounded-full bg-secondary"></span>
          <span>3xx Redirects</span>
          <span class="bg-surface-container-lowest px-1 py-0.2 rounded text-secondary font-semibold">{status3xxCount}</span>
        </button>
      </div>

      <!-- Search Query Input -->
      <div class="h-9 flex items-center bg-surface-container-lowest px-3 rounded gap-2 border border-surface-container-high min-w-[200px]">
        <Search class="w-3.5 h-3.5 text-outline" />
        <input
          type="text"
          bind:value={filterQuery}
          placeholder="Filter path, type..."
          class="bg-transparent font-mono text-xs text-on-surface outline-none w-full placeholder:text-outline"
        />
      </div>
    </div>
  {/if}

  <!-- Path Results Table with Inline Accordion -->
  <div class="w-full bg-surface-container-low rounded overflow-hidden shadow-sm flex flex-col border border-surface-container-high">
    <div class="overflow-x-auto">
      <table class="w-full text-left text-xs border-collapse font-mono">
        <thead>
          <tr class="bg-surface-container-high border-b border-surface-container-highest text-outline font-label-sm">
            <th class="p-3">STATUS</th>
            <th class="p-3">PROBED TARGET PATH</th>
            <th class="p-3">SIZE</th>
            <th class="p-3">CONTENT TYPE</th>
            <th class="p-3">LATENCY</th>
            <th class="p-3 text-right">ACTIONS</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-surface-container-high">
          {#if filteredResults.length === 0}
            <tr>
              <td colspan="6" class="p-8 text-center text-outline">
                {#if isAnalyzing}
                  <div class="flex items-center justify-center gap-2">
                    <Loader2 class="w-4 h-4 animate-spin text-primary" />
                    <span>Probing {parsedPaths.length} paths across target web server...</span>
                  </div>
                {:else}
                  <span>No path probes match the selected filter. Click "START PATH RADAR" above to begin.</span>
                {/if}
              </td>
            </tr>
          {:else}
            {#each filteredResults as item (item.path)}
              {@const fullUrl = `${targetUrl.replace(/\/+$/, "")}${item.path}`}
              {@const isExpanded = !!expandedPaths[item.path]}
              <tr class="hover:bg-surface-container transition-colors {isExpanded ? 'bg-surface-container/60' : ''}">
                <!-- Status Badge -->
                <td class="p-3 whitespace-nowrap">
                  <span class="px-2 py-0.5 font-label-sm rounded uppercase font-semibold border {getStatusColor(item.status)}">
                    HTTP {item.status}
                  </span>
                </td>

                <!-- Path Name & Accordion Trigger -->
                <td class="p-3 font-semibold text-on-surface">
                  <div class="flex items-center gap-2">
                    <button
                      type="button"
                      onclick={() => toggleExpand(item.path)}
                      class="p-0.5 text-outline hover:text-on-surface cursor-pointer flex-shrink-0"
                      title={isExpanded ? "Collapse inline preview" : "Expand inline preview"}
                    >
                      {#if isExpanded}
                        <ChevronDown class="w-3.5 h-3.5 text-secondary" />
                      {:else}
                        <ChevronRight class="w-3.5 h-3.5" />
                      {/if}
                    </button>
                    <button
                      type="button"
                      onclick={() => toggleExpand(item.path)}
                      class="truncate text-left hover:underline cursor-pointer text-secondary font-code-inline"
                    >
                      {item.path}
                    </button>
                    {#if item.has_content}
                      <span class="px-1.5 py-0.2 text-[9px] font-bold uppercase bg-tertiary-container/20 text-tertiary border border-tertiary/40 rounded flex-shrink-0">
                        CONTENT
                      </span>
                    {/if}
                  </div>
                </td>

                <!-- Size -->
                <td class="p-3 whitespace-nowrap text-outline tabular-nums font-code-inline">
                  {formatBytes(item.content_length)}
                </td>

                <!-- Content Type -->
                <td class="p-3 whitespace-nowrap text-outline truncate max-w-xs font-code-inline">
                  {item.content_type || "—"}
                </td>

                <!-- Latency -->
                <td class="p-3 whitespace-nowrap text-outline tabular-nums font-code-inline">
                  {item.response_time_ms} ms
                </td>

                <!-- Actions -->
                <td class="p-3 whitespace-nowrap text-right">
                  <div class="flex items-center justify-end gap-1.5">
                    <button
                      type="button"
                      onclick={() => toggleExpand(item.path)}
                      class="p-1 hover:bg-surface-container-high border border-surface-container-high text-outline hover:text-on-surface rounded cursor-pointer inline-flex items-center {isExpanded ? 'bg-surface-container-high text-on-surface' : ''}"
                      title={isExpanded ? "Collapse inline preview" : "Expand inline content preview"}
                    >
                      <FileCode class="w-3.5 h-3.5" />
                    </button>

                    <button
                      type="button"
                      onclick={() => copyToClipboard(fullUrl)}
                      class="p-1 hover:bg-surface-container-high border border-surface-container-high text-outline hover:text-on-surface rounded cursor-pointer"
                      title="Copy full URL to clipboard"
                    >
                      {#if copiedPath === fullUrl}
                        <Check class="w-3.5 h-3.5 text-tertiary" />
                      {:else}
                        <Copy class="w-3.5 h-3.5" />
                      {/if}
                    </button>

                    {#if onScanReport}
                      <button
                        type="button"
                        onclick={() => onScanReport(fullUrl)}
                        class="p-1 hover:bg-surface-container-high border border-surface-container-high text-secondary hover:text-on-surface rounded cursor-pointer inline-flex items-center"
                        title="Audit endpoint in Workspace 01"
                      >
                        <Shield class="w-3.5 h-3.5" />
                      </button>
                    {/if}

                    <a
                      href={fullUrl}
                      target="_blank"
                      rel="noreferrer"
                      class="p-1 hover:bg-surface-container-high border border-surface-container-high text-outline hover:text-on-surface rounded cursor-pointer inline-flex items-center"
                      title="Open in browser"
                    >
                      <ExternalLink class="w-3.5 h-3.5" />
                    </a>
                  </div>
                </td>
              </tr>

              <!-- EXPANDED INLINE ACCORDION DRAWER -->
              {#if isExpanded}
                <tr class="bg-surface-container-lowest border-b border-surface-container-highest">
                  <td colspan="6" class="p-space-md">
                    <div class="border border-surface-container-high rounded bg-surface-container-low p-space-md flex flex-col gap-space-sm">
                      <div class="flex items-center justify-between border-b border-surface-container-high pb-2">
                        <div class="flex items-center gap-2">
                          <Terminal class="w-3.5 h-3.5 text-secondary" />
                          <span class="font-code-inline text-xs font-bold text-on-surface">{fullUrl}</span>
                          <span class="text-outline text-xs">({item.content_type || "unknown"})</span>
                        </div>
                        <div class="flex items-center gap-2">
                          {#if item.body}
                            <button
                              type="button"
                              onclick={() => copyContent(item.body || "", item.path)}
                              class="px-2 py-0.5 bg-surface-container-high hover:bg-surface-bright text-on-surface font-label-sm rounded flex items-center gap-1 cursor-pointer transition-colors"
                            >
                              {#if copiedContentPath === item.path}
                                <Check class="w-3 h-3 text-tertiary" />
                                <span class="text-tertiary">Copied</span>
                              {:else}
                                <Copy class="w-3 h-3" />
                                <span>Copy Payload</span>
                              {/if}
                            </button>
                          {/if}
                          {#if onScanReport}
                            <button
                              type="button"
                              onclick={() => onScanReport(fullUrl)}
                              class="px-2 py-0.5 border border-secondary text-secondary hover:bg-secondary/10 font-label-sm font-semibold rounded flex items-center gap-1 cursor-pointer transition-colors"
                              title="Audit endpoint in Workspace 01"
                            >
                              <Shield class="w-3 h-3" />
                              <span>Audit in 01</span>
                            </button>
                          {/if}
                          <a
                            href={fullUrl}
                            target="_blank"
                            rel="noreferrer"
                            class="px-2 py-0.5 bg-secondary text-surface-container-lowest font-label-sm font-semibold rounded flex items-center gap-1 hover:opacity-90"
                          >
                            <ExternalLink class="w-3 h-3" />
                            <span>Open Web View</span>
                          </a>
                        </div>
                      </div>

                      <!-- Response Snippet Box -->
                      {#if item.body}
                        <pre class="p-3 bg-surface-container-lowest rounded font-code-inline text-xs text-on-surface overflow-x-auto max-h-64 whitespace-pre-wrap break-all leading-relaxed select-text border border-surface-container-high">{item.body}</pre>
                      {:else}
                        <div class="p-4 text-center text-outline text-xs">
                          No body payload returned for HTTP {item.status}.
                        </div>
                      {/if}
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
</div>

<!-- Save Custom Wordlist Modal -->
{#if showSaveModal}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 p-4 animate-fade-in">
    <div class="bg-surface-container-low border border-surface-container-high rounded p-6 max-w-md w-full space-y-4 shadow-xl text-on-surface">
      <div class="flex items-center justify-between pb-2 border-b border-surface-container-high">
        <h3 class="font-bold text-sm uppercase font-mono">Save Custom Wordlist</h3>
        <button type="button" onclick={() => (showSaveModal = false)} class="text-outline hover:text-on-surface">
          <X class="w-4 h-4" />
        </button>
      </div>

      <div class="space-y-3 font-mono text-xs">
        <div>
          <label for="save-wordlist-name" class="text-outline uppercase text-[11px] block mb-1">Wordlist Name</label>
          <input
            id="save-wordlist-name"
            type="text"
            bind:value={saveListName}
            placeholder="e.g. My Target API Routes"
            class="w-full bg-surface-container-lowest p-2 rounded border border-surface-container-high text-on-surface outline-none"
          />
        </div>

        <div>
          <label for="save-wordlist-category" class="text-outline uppercase text-[11px] block mb-1">Category</label>
          <select
            id="save-wordlist-category"
            bind:value={saveListCategory}
            class="w-full bg-surface-container-lowest p-2 rounded border border-surface-container-high text-on-surface outline-none cursor-pointer"
          >
            <option value="custom">Custom</option>
            <option value="api">API Endpoints</option>
            <option value="paths">General Paths</option>
            <option value="backups">Backups</option>
            <option value="files">Files & Secrets</option>
          </select>
        </div>

        <div class="text-outline text-[11px]">
          Saving {parsedPaths.length} paths to local SQLite database and filesystem storage.
        </div>
      </div>

      <div class="flex items-center justify-end gap-2 pt-2 border-t border-surface-container-high">
        <button
          type="button"
          onclick={() => (showSaveModal = false)}
          class="px-3 py-1.5 text-xs text-outline hover:text-on-surface rounded cursor-pointer"
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={handleSaveWordlist}
          disabled={!saveListName.trim() || isSavingList}
          class="px-4 py-1.5 bg-primary text-on-primary font-bold text-xs rounded cursor-pointer disabled:opacity-50"
        >
          {isSavingList ? "Saving..." : "Save Wordlist"}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Dynamic Permutation Generator Modal -->
{#if showGenModal}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 p-4 animate-fade-in">
    <div class="bg-surface-container-low border border-surface-container-high rounded p-6 max-w-lg w-full space-y-4 shadow-xl text-on-surface">
      <div class="flex items-center justify-between pb-2 border-b border-surface-container-high">
        <div class="flex items-center gap-2">
          <Wand2 class="w-4 h-4 text-primary" />
          <h3 class="font-bold text-sm uppercase font-mono">Dynamic Wordlist Generator</h3>
        </div>
        <button type="button" onclick={() => (showGenModal = false)} class="text-outline hover:text-on-surface">
          <X class="w-4 h-4" />
        </button>
      </div>

      <div class="space-y-3 font-mono text-xs">
        <div>
          <label for="gen-base-words" class="text-outline uppercase text-[11px] block mb-1">Base Keywords (comma-separated)</label>
          <input
            id="gen-base-words"
            type="text"
            bind:value={genBaseWords}
            class="w-full bg-surface-container-lowest p-2 rounded border border-surface-container-high text-on-surface outline-none"
          />
        </div>

        <div>
          <label for="gen-directories" class="text-outline uppercase text-[11px] block mb-1">Prefix Directories (comma-separated)</label>
          <input
            id="gen-directories"
            type="text"
            bind:value={genDirs}
            class="w-full bg-surface-container-lowest p-2 rounded border border-surface-container-high text-on-surface outline-none"
          />
        </div>

        <div>
          <label for="gen-extensions" class="text-outline uppercase text-[11px] block mb-1">Extensions (comma-separated)</label>
          <input
            id="gen-extensions"
            type="text"
            bind:value={genExts}
            class="w-full bg-surface-container-lowest p-2 rounded border border-surface-container-high text-on-surface outline-none"
          />
        </div>

        <div class="flex items-center gap-6 pt-1 text-xs font-mono">
          <label class="flex items-center gap-2 cursor-pointer text-on-surface-variant hover:text-on-surface select-none group">
            <input type="checkbox" bind:checked={genDotfiles} class="sr-only" />
            <div class="w-3.5 h-3.5 rounded flex items-center justify-center border transition-colors {genDotfiles ? 'bg-primary border-primary text-on-primary' : 'bg-surface-container-lowest border-surface-container-high group-hover:border-outline'}">
              {#if genDotfiles}
                <Check class="w-2.5 h-2.5 stroke-[3]" />
              {/if}
            </div>
            <span>Include Dotfiles (e.g. .env, .git)</span>
          </label>
          <label class="flex items-center gap-2 cursor-pointer text-on-surface-variant hover:text-on-surface select-none group">
            <input type="checkbox" bind:checked={genBackups} class="sr-only" />
            <div class="w-3.5 h-3.5 rounded flex items-center justify-center border transition-colors {genBackups ? 'bg-primary border-primary text-on-primary' : 'bg-surface-container-lowest border-surface-container-high group-hover:border-outline'}">
              {#if genBackups}
                <Check class="w-2.5 h-2.5 stroke-[3]" />
              {/if}
            </div>
            <span>Include Backups (.bak, .old)</span>
          </label>
        </div>
      </div>

      <div class="flex items-center justify-end gap-2 pt-2 border-t border-surface-container-high">
        <button
          type="button"
          onclick={() => (showGenModal = false)}
          class="px-3 py-1.5 text-xs text-outline hover:text-on-surface rounded cursor-pointer"
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={handleGenerateDynamic}
          disabled={isGenerating}
          class="px-4 py-1.5 bg-primary text-on-primary font-bold text-xs rounded cursor-pointer disabled:opacity-50 flex items-center gap-1.5"
        >
          {#if isGenerating}
            <Loader2 class="w-3.5 h-3.5 animate-spin" />
            <span>Generating...</span>
          {:else}
            <Sparkles class="w-3.5 h-3.5" />
            <span>Generate & Append</span>
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}

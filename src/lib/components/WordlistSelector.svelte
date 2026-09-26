<script lang="ts">
  import { onMount } from "svelte";
  import {
    BUILTIN_WORDLISTS,
    WORDLIST_PRESETS,
    computeUniquePaths,
    exportWordlistTxt,
    fetchWordlistsFromBackend,
    saveWordlistToBackend,
    deleteWordlistFromBackend,
    generateDynamicWordlistBackend,
    exportWordlistToDisk,
    downloadWordlistFromServer,
  } from "$lib/wordlists";
  import type { WordlistCategory, WordlistConfig, WordlistItem, WordlistRecord } from "$lib/types";
  import {
    CheckSquare,
    Square,
    Folder,
    FileText,
    Key,
    Database,
    Code,
    Activity,
    Upload,
    Download,
    Search,
    RotateCcw,
    ChevronDown,
    ChevronUp,
    Copy,
    Check,
    ListFilter,
    Layers,
    Sparkles,
    Trash2,
    Plus,
    Wand2,
    Save,
    HardDrive,
    Server,
    Loader2,
  } from "lucide-svelte";

  let {
    config = $bindable<WordlistConfig>({
      selectedIds: ["common_paths", "sensitive_files", "api_documentation"],
      customPaths: [],
      activePreset: "balanced",
    }),
    onChange,
  }: {
    config?: WordlistConfig;
    onChange?: (newConfig: WordlistConfig) => void;
  } = $props();

  let wordlists = $state<WordlistItem[]>(BUILTIN_WORDLISTS);
  let selectedCategory = $state<string>("all");
  let selectedPreset = $state<string>(config?.activePreset || "balanced");
  let searchQuery = $state<string>("");
  let expandedWordlistId = $state<string | null>(null);
  let customInputText = $state<string>(config?.customPaths?.join("\n") || "");
  let showCustomPanel = $state<boolean>(false);
  let showGeneratorPanel = $state<boolean>(false);
  let copiedWordlistId = $state<string | null>(null);
  let isLoadingFromDb = $state<boolean>(false);
  let isSavingToDb = $state<boolean>(false);
  let isGenerating = $state<boolean>(false);
  let saveFeedbackMessage = $state<string | null>(null);

  // Dynamic Generator Form State
  let genBaseWords = $state<string>("admin, api, config, backup, login, user, portal, auth, dev, test");
  let genDirectories = $state<string>("api/, v1/, config/, backup/, internal/");
  let genExtensions = $state<string>("json, sql, php, env, yml, bak");
  let genIncludeDotfiles = $state<boolean>(true);
  let genIncludeBackups = $state<boolean>(true);
  let genResults = $state<string[]>([]);
  let genListName = $state<string>("Dynamic Recon Wordlist");

  // Custom List Database Save Modal/Form
  let customSaveName = $state<string>("");
  let customSaveCategory = $state<string>("custom");

  // Sync preset if prop updates
  $effect(() => {
    if (config?.activePreset && config.activePreset !== selectedPreset) {
      selectedPreset = config.activePreset;
    }
  });

  onMount(async () => {
    await reloadWordlistsFromBackend();
  });

  async function reloadWordlistsFromBackend() {
    isLoadingFromDb = true;
    try {
      const records = await fetchWordlistsFromBackend();
      if (records && records.length > 0) {
        wordlists = records.map((rec) => ({
          id: rec.id,
          name: rec.name,
          category: (rec.category as any) || "paths",
          description: rec.description,
          paths: rec.paths,
          tags: [rec.category.toUpperCase(), `${rec.item_count} items`],
          isCustom: rec.is_custom,
        }));
      }
    } catch (e) {
      console.warn("Could not load wordlists from database backend:", e);
    } finally {
      isLoadingFromDb = false;
    }
  }

  // Calculate unique combined paths
  const totalUniquePaths = $derived(
    computeUniquePaths(config.selectedIds, config.customPaths)
  );

  // Filter wordlists based on category dropdown & search query
  const filteredWordlists = $derived(
    wordlists.filter((item) => {
      // Category filter
      if (selectedCategory !== "all" && item.category !== selectedCategory) {
        return false;
      }

      // Search query filter
      if (!searchQuery.trim()) return true;
      const q = searchQuery.toLowerCase();
      const matchesName = item.name.toLowerCase().includes(q);
      const matchesDesc = item.description.toLowerCase().includes(q);
      const matchesTags = item.tags?.some((t) => t.toLowerCase().includes(q));
      const matchesPaths = item.paths.some((p) => p.toLowerCase().includes(q));

      return matchesName || matchesDesc || matchesTags || matchesPaths;
    })
  );

  function isSelected(id: string): boolean {
    return config.selectedIds.includes(id);
  }

  function toggleWordlist(id: string) {
    let nextIds: string[];
    if (config.selectedIds.includes(id)) {
      nextIds = config.selectedIds.filter((item) => item !== id);
    } else {
      nextIds = [...config.selectedIds, id];
    }

    config.selectedIds = nextIds;
    config.activePreset = "custom";
    selectedPreset = "custom";
    triggerChange();
  }

  function handlePresetChange(presetId: string) {
    selectedPreset = presetId;
    config.activePreset = presetId;
    const preset = WORDLIST_PRESETS.find((p) => p.id === presetId);
    if (preset) {
      config.selectedIds = [...preset.selectedIds];
    }
    triggerChange();
  }

  function selectAll() {
    config.selectedIds = wordlists.map((w) => w.id);
    config.activePreset = "full";
    selectedPreset = "full";
    triggerChange();
  }

  function deselectAll() {
    config.selectedIds = [];
    config.activePreset = "custom";
    selectedPreset = "custom";
    triggerChange();
  }

  function resetToDefault() {
    const balanced = WORDLIST_PRESETS.find((p) => p.id === "balanced");
    config.selectedIds = balanced ? [...balanced.selectedIds] : ["common_paths", "sensitive_files"];
    config.customPaths = [];
    customInputText = "";
    config.activePreset = "balanced";
    selectedPreset = "balanced";
    selectedCategory = "all";
    searchQuery = "";
    triggerChange();
  }

  function handleCustomTextChange() {
    const lines = customInputText
      .split("\n")
      .map((l) => l.trim())
      .filter((l) => l.length > 0 && !l.startsWith("#"));

    // Deduplicate
    config.customPaths = Array.from(new Set(lines));
    triggerChange();
  }

  function handleFileUpload(event: Event) {
    const input = event.target as HTMLInputElement;
    if (!input.files || input.files.length === 0) return;

    const file = input.files[0];
    const reader = new FileReader();

    reader.onload = (e) => {
      const text = e.target?.result as string;
      if (text) {
        customInputText = text;
        handleCustomTextChange();
        showCustomPanel = true;
      }
    };

    reader.readAsText(file);
    input.value = "";
  }

  async function handleExport() {
    try {
      await exportWordlistToDisk("vulnradar-selected-wordlist.txt", totalUniquePaths);
      flashMessage("Wordlist exported successfully to file system");
    } catch {
      exportWordlistTxt(totalUniquePaths, "vulnradar-selected-wordlist.txt");
    }
  }

  async function handleDownloadWordlist(id: string, name: string, paths: string[]) {
    const safeFilename = `${id.replace(/[^a-zA-Z0-9_-]/g, "_")}.txt`;
    await downloadWordlistFromServer(id, safeFilename, paths);
    flashMessage(`Downloaded "${name}" via HTTP server`);
  }

  async function handleGenerateDynamic() {
    isGenerating = true;
    try {
      const base_words = genBaseWords
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);
      const directories = genDirectories
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);
      const extensions = genExtensions
        .split(",")
        .map((s) => s.trim().replace(/^\.+/, ""))
        .filter(Boolean);

      const generated = await generateDynamicWordlistBackend({
        base_words,
        directories,
        extensions,
        prefixes: [""],
        include_dotfiles: genIncludeDotfiles,
        include_backups: genIncludeBackups,
      });

      genResults = generated;
    } finally {
      isGenerating = false;
    }
  }

  function applyGeneratedToCustom() {
    if (genResults.length === 0) return;
    const combined = Array.from(new Set([...config.customPaths, ...genResults]));
    config.customPaths = combined;
    customInputText = combined.join("\n");
    triggerChange();
    flashMessage(`Added ${genResults.length} dynamic paths to active target queue`);
  }

  async function saveGeneratedAsWordlist() {
    if (genResults.length === 0) return;
    isSavingToDb = true;
    try {
      const id = `dynamic_${Date.now()}`;
      const record: WordlistRecord = {
        id,
        name: genListName.trim() || `Dynamic Wordlist (${genResults.length})`,
        category: "paths",
        description: `Dynamically generated from keywords with ${genResults.length} permutation paths`,
        paths: genResults,
        item_count: genResults.length,
        is_custom: true,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };

      await saveWordlistToBackend(record);
      await reloadWordlistsFromBackend();
      config.selectedIds = [...config.selectedIds, id];
      triggerChange();
      flashMessage(`Saved "${record.name}" to SQLite & File System`);
    } finally {
      isSavingToDb = false;
    }
  }

  async function saveCustomTextAsWordlist() {
    if (config.customPaths.length === 0 || !customSaveName.trim()) return;
    isSavingToDb = true;
    try {
      const id = `custom_${Date.now()}`;
      const record: WordlistRecord = {
        id,
        name: customSaveName.trim(),
        category: customSaveCategory,
        description: `User-imported custom wordlist containing ${config.customPaths.length} target paths`,
        paths: config.customPaths,
        item_count: config.customPaths.length,
        is_custom: true,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };

      await saveWordlistToBackend(record);
      await reloadWordlistsFromBackend();
      config.selectedIds = [...config.selectedIds, id];
      triggerChange();
      customSaveName = "";
      flashMessage(`Saved "${record.name}" to Database & Disk (.txt)`);
    } finally {
      isSavingToDb = false;
    }
  }

  async function handleDeleteCustomWordlist(id: string, name: string) {
    if (!confirm(`Delete wordlist "${name}" from SQLite and file system?`)) return;
    await deleteWordlistFromBackend(id);
    await reloadWordlistsFromBackend();
    config.selectedIds = config.selectedIds.filter((item) => item !== id);
    triggerChange();
    flashMessage(`Deleted "${name}"`);
  }

  function flashMessage(msg: string) {
    saveFeedbackMessage = msg;
    setTimeout(() => {
      if (saveFeedbackMessage === msg) saveFeedbackMessage = null;
    }, 3500);
  }

  async function copyPathsToClipboard(id: string, paths: string[]) {
    try {
      await navigator.clipboard.writeText(paths.join("\n"));
      copiedWordlistId = id;
      setTimeout(() => {
        if (copiedWordlistId === id) copiedWordlistId = null;
      }, 2000);
    } catch (e) {
      console.error("Failed to copy paths:", e);
    }
  }

  function triggerChange() {
    if (onChange) {
      onChange({
        selectedIds: config.selectedIds,
        customPaths: config.customPaths,
        activePreset: config.activePreset,
      });
    }
  }

  function getCategoryIcon(cat: WordlistItem["category"]) {
    switch (cat) {
      case "paths":
        return Folder;
      case "directories":
        return Layers;
      case "files":
        return FileText;
      case "backups":
        return Database;
      case "api":
        return Code;
      case "debug":
        return Activity;
      default:
        return FileText;
    }
  }

  function getCategoryColor(cat: WordlistItem["category"]) {
    switch (cat) {
      case "paths":
        return "text-blue-500 border-blue-500/30 bg-blue-500/10";
      case "directories":
        return "text-cyan-500 border-cyan-500/30 bg-cyan-500/10";
      case "files":
        return "text-red-500 border-red-500/30 bg-red-500/10";
      case "backups":
        return "text-amber-500 border-amber-500/30 bg-amber-500/10";
      case "api":
        return "text-purple-500 border-purple-500/30 bg-purple-500/10";
      case "debug":
        return "text-emerald-500 border-emerald-500/30 bg-emerald-500/10";
      default:
        return "text-zinc-400 border-zinc-500/30 bg-zinc-500/10";
    }
  }
</script>

<div class="flex flex-col gap-4 font-mono text-[var(--color-text-body)]">
  <!-- Toast / Feedback Bar -->
  {#if saveFeedbackMessage}
    <div class="px-3.5 py-2 bg-emerald-500/10 border border-emerald-500/30 text-emerald-400 text-xs flex items-center justify-between gap-2 animate-fade-in">
      <div class="flex items-center gap-2">
        <Check class="w-3.5 h-3.5 text-emerald-400" />
        <span>{saveFeedbackMessage}</span>
      </div>
      <button
        type="button"
        onclick={() => (saveFeedbackMessage = null)}
        class="text-emerald-400 hover:text-white cursor-pointer"
      >
        ×
      </button>
    </div>
  {/if}

  <!-- Top Control Bar: Category Dropdown, Preset Selector, Search & Stats -->
  <div class="p-4 bg-[var(--color-surface)] border border-[var(--color-hairline)] flex flex-col gap-3">
    <!-- Row 1: Dropdown Selectors & Actions -->
    <div class="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-3">
      <!-- Category Filter Dropdown -->
      <div class="flex flex-col sm:flex-row items-start sm:items-center gap-2 flex-1">
        <label for="wordlist-cat-select" class="text-xs font-bold uppercase tracking-wider text-[var(--color-text-muted)] flex items-center gap-1.5 flex-shrink-0">
          <ListFilter class="w-3.5 h-3.5 text-[var(--color-signal-red)]" />
          <span>Category:</span>
        </label>
        <select
          id="wordlist-cat-select"
          bind:value={selectedCategory}
          class="w-full sm:w-auto appearance-none px-2.5 py-1.5 pr-7 text-xs bg-[var(--color-canvas)] border border-[var(--color-hairline)] rounded-none text-[var(--color-text-headline)] font-mono focus:border-[var(--color-signal-red)] focus:outline-none cursor-pointer"
        >
          <option value="all">ALL CATEGORIES ({wordlists.length})</option>
          <option value="paths">COMMON WEB PATHS</option>
          <option value="directories">DIRECTORY & FOLDER NAMES</option>
          <option value="files">SENSITIVE FILES & DOTFILES</option>
          <option value="backups">DATABASE DUMPS & BACKUPS</option>
          <option value="api">API & OPENAPI SPECIFICATIONS</option>
          <option value="debug">DEBUGGERS & TELEMETRY</option>
          <option value="custom">USER CUSTOM WORDLISTS</option>
        </select>

        <!-- Preset Selection Dropdown -->
        <label for="wordlist-preset-select" class="text-xs font-bold uppercase tracking-wider text-[var(--color-text-muted)] flex items-center gap-1.5 flex-shrink-0 sm:ml-2">
          <Sparkles class="w-3.5 h-3.5 text-amber-500" />
          <span>Preset:</span>
        </label>
        <select
          id="wordlist-preset-select"
          value={selectedPreset}
          onchange={(e) => handlePresetChange((e.target as HTMLSelectElement).value)}
          class="w-full sm:w-auto appearance-none px-2.5 py-1.5 pr-7 text-xs bg-[var(--color-canvas)] border border-[var(--color-hairline)] rounded-none text-[var(--color-text-headline)] font-mono focus:border-[var(--color-signal-red)] focus:outline-none cursor-pointer"
        >
          {#each WORDLIST_PRESETS as preset}
            <option value={preset.id}>{preset.name.toUpperCase()}</option>
          {/each}
        </select>
      </div>

      <!-- Quick Action Buttons -->
      <div class="flex items-center gap-2 flex-wrap sm:flex-nowrap flex-shrink-0">
        <button
          type="button"
          onclick={selectAll}
          class="px-2 py-1 text-[11px] font-bold uppercase bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] border border-[var(--color-hairline)] text-[var(--color-text-headline)] transition-colors cursor-pointer"
          title="Select all wordlists"
        >
          SELECT ALL
        </button>
        <button
          type="button"
          onclick={deselectAll}
          class="px-2 py-1 text-[11px] font-bold uppercase bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] border border-[var(--color-hairline)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] transition-colors cursor-pointer"
          title="Clear all selections"
        >
          DESELECT ALL
        </button>
        <button
          type="button"
          onclick={resetToDefault}
          class="p-1 text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] border border-[var(--color-hairline)] bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] transition-colors cursor-pointer"
          title="Reset to recommended defaults"
        >
          <RotateCcw class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>

    <!-- Row 2: Search Input & Generator / File Actions -->
    <div class="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-3 pt-2 border-t border-[var(--color-hairline)]">
      <!-- Search Filter -->
      <div class="relative flex-1">
        <Search class="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)] pointer-events-none" />
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Search wordlists by name, keyword, or path (e.g., .env, /admin)..."
          class="w-full pl-9 pr-3 py-1.5 text-xs bg-[var(--color-canvas)] border border-[var(--color-hairline)] rounded-none text-[var(--color-text-headline)] font-mono placeholder:text-[var(--color-text-muted)] focus:border-[var(--color-signal-red)] focus:outline-none"
        />
      </div>

      <!-- Action Toggles -->
      <div class="flex items-center gap-2 flex-wrap sm:flex-nowrap flex-shrink-0">
        <!-- Dynamic Generator Toggle -->
        <button
          type="button"
          onclick={() => {
            showGeneratorPanel = !showGeneratorPanel;
            if (showGeneratorPanel && genResults.length === 0) handleGenerateDynamic();
          }}
          class="px-2.5 py-1.5 text-xs font-bold uppercase border border-[var(--color-hairline)] flex items-center gap-1.5 transition-colors cursor-pointer {showGeneratorPanel ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)]' : 'bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] text-amber-500'}"
          title="Dynamic Wordlist Generator Engine"
        >
          <Wand2 class="w-3 h-3 text-amber-500" />
          <span>GENERATOR</span>
        </button>

        <!-- Hidden file input for .txt file -->
        <label
          class="px-2.5 py-1.5 text-xs font-bold uppercase bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] border border-[var(--color-hairline)] text-[var(--color-text-headline)] flex items-center gap-1.5 transition-colors cursor-pointer"
          title="Import wordlist from .txt file on filesystem"
        >
          <Upload class="w-3 h-3 text-[var(--color-signal-red)]" />
          <span>LOAD .TXT</span>
          <input
            type="file"
            accept=".txt"
            onchange={handleFileUpload}
            class="hidden"
          />
        </label>

        <button
          type="button"
          onclick={handleExport}
          class="px-2.5 py-1.5 text-xs font-bold uppercase bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] border border-[var(--color-hairline)] text-[var(--color-text-headline)] flex items-center gap-1.5 transition-colors cursor-pointer"
          title="Export active paths to filesystem disk"
        >
          <Download class="w-3 h-3 text-emerald-500" />
          <span>EXPORT .TXT</span>
        </button>

        <button
          type="button"
          onclick={() => (showCustomPanel = !showCustomPanel)}
          class="px-2.5 py-1.5 text-xs font-bold uppercase border border-[var(--color-hairline)] flex items-center gap-1.5 transition-colors cursor-pointer {showCustomPanel ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)]' : 'bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] text-[var(--color-text-headline)]'}"
          title="Toggle custom wordlist editor"
        >
          <Plus class="w-3 h-3" />
          <span>CUSTOM ({config.customPaths.length})</span>
        </button>
      </div>
    </div>
  </div>

  <!-- Dynamic Wordlist Generator Panel (Backend Service) -->
  {#if showGeneratorPanel}
    <div class="p-4 bg-[var(--color-surface)] border border-[var(--color-hairline)] border-l-2 border-l-amber-500 flex flex-col gap-4 animate-fade-in">
      <div class="flex items-center justify-between">
        <div>
          <h3 class="text-xs font-bold uppercase tracking-tight text-[var(--color-text-headline)] flex items-center gap-2">
            <Wand2 class="w-3.5 h-3.5 text-amber-500" />
            <span>Backend Dynamic Wordlist Generator</span>
          </h3>
          <p class="text-[11px] text-[var(--color-text-muted)] mt-0.5">
            Generates permutations from base keywords, directory prefixes, and file extensions using the Rust engine.
          </p>
        </div>
        <span class="text-[10px] font-mono uppercase px-2 py-0.5 bg-amber-500/10 text-amber-400 border border-amber-500/30">
          ALGORITHMIC PERMUTATION
        </span>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
        <div>
          <label for="gen-keywords" class="text-[11px] font-bold text-[var(--color-text-muted)] uppercase block mb-1">
            Base Keywords (comma-separated):
          </label>
          <input
            id="gen-keywords"
            type="text"
            bind:value={genBaseWords}
            class="w-full px-2.5 py-1.5 text-xs bg-[var(--color-canvas)] border border-[var(--color-hairline)] font-mono text-[var(--color-text-headline)] focus:border-amber-500 focus:outline-none"
          />
        </div>

        <div>
          <label for="gen-directories" class="text-[11px] font-bold text-[var(--color-text-muted)] uppercase block mb-1">
            Directory Prefixes (comma-separated):
          </label>
          <input
            id="gen-directories"
            type="text"
            bind:value={genDirectories}
            class="w-full px-2.5 py-1.5 text-xs bg-[var(--color-canvas)] border border-[var(--color-hairline)] font-mono text-[var(--color-text-headline)] focus:border-amber-500 focus:outline-none"
          />
        </div>

        <div>
          <label for="gen-extensions" class="text-[11px] font-bold text-[var(--color-text-muted)] uppercase block mb-1">
            Extensions (comma-separated):
          </label>
          <input
            id="gen-extensions"
            type="text"
            bind:value={genExtensions}
            class="w-full px-2.5 py-1.5 text-xs bg-[var(--color-canvas)] border border-[var(--color-hairline)] font-mono text-[var(--color-text-headline)] focus:border-amber-500 focus:outline-none"
          />
        </div>
      </div>

      <div class="flex items-center gap-6 flex-wrap text-xs">
        <label class="flex items-center gap-2 cursor-pointer select-none">
          <input
            type="checkbox"
            bind:checked={genIncludeDotfiles}
            class="rounded-none accent-amber-500"
          />
          <span class="text-[var(--color-text-body)]">Include dotfile variants (e.g., <code>/.env</code>, <code>/.git/</code>)</span>
        </label>

        <label class="flex items-center gap-2 cursor-pointer select-none">
          <input
            type="checkbox"
            bind:checked={genIncludeBackups}
            class="rounded-none accent-amber-500"
          />
          <span class="text-[var(--color-text-body)]">Include backup suffixes (e.g., <code>.bak</code>, <code>.old</code>, <code>~</code>)</span>
        </label>
      </div>

      <div class="flex items-center justify-between pt-2 border-t border-[var(--color-hairline)] flex-wrap gap-2">
        <div class="flex items-center gap-2">
          <button
            type="button"
            onclick={handleGenerateDynamic}
            disabled={isGenerating}
            class="px-3 py-1.5 bg-amber-500 hover:bg-amber-600 text-black text-xs font-bold uppercase flex items-center gap-1.5 transition-colors cursor-pointer disabled:opacity-50"
          >
            {#if isGenerating}
              <Loader2 class="w-3 h-3 animate-spin" />
              <span>GENERATING...</span>
            {:else}
              <Wand2 class="w-3 h-3" />
              <span>RUN GENERATOR</span>
            {/if}
          </button>

          {#if genResults.length > 0}
            <span class="text-xs text-[var(--color-text-muted)]">
              Output: <strong class="text-amber-400">{genResults.length}</strong> permutation paths
            </span>
          {/if}
        </div>

        {#if genResults.length > 0}
          <div class="flex items-center gap-2">
            <button
              type="button"
              onclick={applyGeneratedToCustom}
              class="px-2.5 py-1.5 bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] border border-[var(--color-hairline)] text-xs text-[var(--color-text-headline)] font-bold uppercase flex items-center gap-1 cursor-pointer"
              title="Append generated paths into active custom targets"
            >
              <Plus class="w-3 h-3" />
              <span>ADD TO ACTIVE TARGETS</span>
            </button>

            <button
              type="button"
              onclick={() => handleDownloadWordlist("generated_wordlist", genListName, genResults)}
              class="px-2.5 py-1.5 bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] border border-[var(--color-hairline)] text-xs text-emerald-500 font-bold uppercase flex items-center gap-1 cursor-pointer"
              title="Download generated wordlist via HTTP server"
            >
              <Download class="w-3 h-3 text-emerald-500" />
              <span>DOWNLOAD .TXT</span>
            </button>

            <button
              type="button"
              onclick={saveGeneratedAsWordlist}
              disabled={isSavingToDb}
              class="px-2.5 py-1.5 bg-[var(--color-text-headline)] text-[var(--color-canvas)] text-xs font-bold uppercase flex items-center gap-1 cursor-pointer hover:opacity-90 disabled:opacity-50"
              title="Persist this list to SQLite database and file system (.txt)"
            >
              <Save class="w-3 h-3" />
              <span>SAVE TO DB & DISK</span>
            </button>
          </div>
        {/if}
      </div>

      <!-- Preview strip of generated paths -->
      {#if genResults.length > 0}
        <div class="max-h-28 overflow-y-auto bg-[var(--color-canvas)] border border-[var(--color-hairline)] p-2 grid grid-cols-2 sm:grid-cols-4 gap-1 text-[10px]">
          {#each genResults.slice(0, 40) as path}
            <span class="truncate text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] font-mono">{path}</span>
          {/each}
          {#if genResults.length > 40}
            <span class="text-amber-500 font-bold">+{genResults.length - 40} more...</span>
          {/if}
        </div>
      {/if}
    </div>
  {/if}

  <!-- Custom Wordlist Accordion Panel -->
  {#if showCustomPanel}
    <div class="p-4 bg-[var(--color-surface)] border border-[var(--color-hairline)] border-l-2 border-l-[var(--color-signal-red)] flex flex-col gap-3 animate-fade-in">
      <div class="flex items-center justify-between">
        <div>
          <h3 class="text-xs font-bold uppercase tracking-tight text-[var(--color-text-headline)] flex items-center gap-2">
            <FileText class="w-3.5 h-3.5 text-[var(--color-signal-red)]" />
            <span>Custom Target Paths & Wordlist Editor</span>
          </h3>
          <p class="text-[11px] text-[var(--color-text-muted)] mt-0.5">
            Enter one path per line or load a .txt file. Stored dynamically in memory and optionally persisted to database.
          </p>
        </div>
        <button
          type="button"
          onclick={() => {
            customInputText = "";
            handleCustomTextChange();
          }}
          class="px-2 py-1 text-[11px] font-bold text-red-500 hover:bg-red-500/10 border border-red-500/20 flex items-center gap-1 cursor-pointer"
        >
          <Trash2 class="w-3 h-3" />
          <span>CLEAR</span>
        </button>
      </div>

      <textarea
        bind:value={customInputText}
        oninput={handleCustomTextChange}
        rows={5}
        placeholder="/internal-dashboard&#10;/backup/db_2026.sql&#10;/auth/v2/keys&#10;/.private.key"
        class="w-full p-2.5 text-xs font-mono bg-[var(--color-canvas)] border border-[var(--color-hairline)] text-[var(--color-text-headline)] rounded-none focus:border-[var(--color-signal-red)] focus:outline-none"
      ></textarea>

      <!-- Save custom paths to SQLite & File System -->
      <div class="p-3 bg-[var(--color-canvas)] border border-[var(--color-hairline)] flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-3">
        <div class="flex items-center gap-2 flex-1">
          <input
            type="text"
            bind:value={customSaveName}
            placeholder="Wordlist Title (e.g. Corporate Admin Endpoints)..."
            class="px-2.5 py-1 text-xs bg-[var(--color-surface)] border border-[var(--color-hairline)] text-[var(--color-text-headline)] font-mono flex-1 focus:outline-none focus:border-[var(--color-signal-red)]"
          />
          <select
            bind:value={customSaveCategory}
            class="appearance-none px-2 py-1 pr-7 text-xs bg-[var(--color-surface)] border border-[var(--color-hairline)] text-[var(--color-text-headline)] font-mono cursor-pointer"
          >
            <option value="custom">CATEGORY: CUSTOM</option>
            <option value="paths">CATEGORY: PATHS</option>
            <option value="files">CATEGORY: FILES</option>
            <option value="directories">CATEGORY: DIRECTORIES</option>
          </select>
        </div>

        <button
          type="button"
          onclick={saveCustomTextAsWordlist}
          disabled={!customSaveName.trim() || config.customPaths.length === 0 || isSavingToDb}
          class="px-3 py-1 bg-[var(--color-text-headline)] text-[var(--color-canvas)] text-xs font-bold uppercase flex items-center gap-1.5 transition-opacity cursor-pointer disabled:opacity-40"
        >
          {#if isSavingToDb}
            <Loader2 class="w-3 h-3 animate-spin" />
            <span>SAVING...</span>
          {:else}
            <Save class="w-3 h-3" />
            <span>SAVE TO DATABASE & DISK</span>
          {/if}
        </button>
      </div>

      <div class="flex items-center justify-between text-[11px] text-[var(--color-text-muted)]">
        <span>Parsed: <strong class="text-[var(--color-text-headline)]">{config.customPaths.length}</strong> unique targets</span>
        <span>Storage: SQLite WAL + Filesystem <code>wordlists/</code> directory</span>
      </div>
    </div>
  {/if}

  <!-- Wordlists Checkbox Selection Grid -->
  <div class="flex flex-col gap-2.5">
    {#if filteredWordlists.length === 0}
      <div class="p-8 text-center bg-[var(--color-surface)] border border-[var(--color-hairline)] text-[var(--color-text-muted)]">
        <p class="text-xs font-mono uppercase">No wordlists match the current category or search criteria.</p>
        <button
          type="button"
          onclick={() => {
            selectedCategory = "all";
            searchQuery = "";
          }}
          class="mt-2 px-3 py-1 text-xs font-bold uppercase bg-[var(--color-canvas)] border border-[var(--color-hairline)] text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)] cursor-pointer"
        >
          CLEAR FILTERS
        </button>
      </div>
    {:else}
      {#each filteredWordlists as wordlist (wordlist.id)}
        {@const selected = isSelected(wordlist.id)}
        {@const Icon = getCategoryIcon(wordlist.category)}
        {@const isExpanded = expandedWordlistId === wordlist.id}
        {@const colorClass = getCategoryColor(wordlist.category)}

        <div
          class="border transition-colors bg-[var(--color-surface)] {selected ? 'border-[var(--color-hairline-strong)] bg-[var(--color-surface)]' : 'border-[var(--color-hairline)] opacity-80 hover:opacity-100'}"
        >
          <!-- Item Card Header / Main Row -->
          <div class="p-3.5 flex flex-col md:flex-row md:items-center justify-between gap-3">
            <!-- Left: Checkbox + Title + Description -->
            <div class="flex items-start gap-3 flex-1">
              <!-- Custom Accessible Checkbox -->
              <button
                type="button"
                role="checkbox"
                aria-checked={selected}
                aria-label={`Select ${wordlist.name}`}
                onclick={() => toggleWordlist(wordlist.id)}
                class="mt-0.5 p-1 rounded-none border transition-colors cursor-pointer flex-shrink-0 {selected ? 'bg-[var(--color-signal-red)] border-[var(--color-signal-red)] text-white' : 'bg-[var(--color-canvas)] border-[var(--color-hairline)] text-transparent hover:border-[var(--color-text-muted)]'}"
              >
                <Check class="w-3.5 h-3.5" />
              </button>

              <div class="flex flex-col gap-1">
                <!-- Title & Tags -->
                <div class="flex items-center gap-2 flex-wrap">
                  <button
                    type="button"
                    class="text-xs font-black uppercase tracking-tight text-[var(--color-text-headline)] hover:underline cursor-pointer select-none text-left bg-transparent p-0 border-none"
                    onclick={() => toggleWordlist(wordlist.id)}
                  >
                    {wordlist.name}
                  </button>

                  <!-- Category Badge -->
                  <span class="px-1.5 py-0.2 text-[9px] font-bold uppercase border {colorClass}">
                    {wordlist.category}
                  </span>

                  <!-- Path count badge -->
                  <span class="px-1.5 py-0.2 text-[9px] font-bold uppercase bg-[var(--color-canvas)] border border-[var(--color-hairline)] text-[var(--color-text-muted)]">
                    {wordlist.paths.length} PATHS
                  </span>

                  <!-- Custom User List indicator -->
                  {#if wordlist.isCustom}
                    <span class="px-1.5 py-0.2 text-[9px] font-bold uppercase bg-amber-500/10 text-amber-500 border border-amber-500/30">
                      SAVED IN DB
                    </span>
                  {/if}
                </div>

                <!-- Description -->
                <p class="text-[11px] text-[var(--color-text-muted)] line-clamp-1">
                  {wordlist.description}
                </p>

                <!-- Sample Preview Pills -->
                <div class="flex items-center gap-1.5 flex-wrap mt-1">
                  {#each wordlist.paths.slice(0, 4) as path}
                    <code class="px-1.5 py-0.5 text-[10px] bg-[var(--color-canvas)] border border-[var(--color-hairline)] text-[var(--color-text-headline)]">
                      {path}
                    </code>
                  {/each}
                  {#if wordlist.paths.length > 4}
                    <span class="text-[10px] text-[var(--color-text-muted)]">
                      +{wordlist.paths.length - 4} more
                    </span>
                  {/if}
                </div>
              </div>
            </div>

            <!-- Right: Actions (Inspect / Copy / Delete if custom) -->
            <div class="flex items-center gap-2 self-end md:self-center flex-shrink-0">
              {#if wordlist.isCustom}
                <button
                  type="button"
                  onclick={() => handleDeleteCustomWordlist(wordlist.id, wordlist.name)}
                  class="p-1.5 text-red-500 hover:bg-red-500/10 border border-red-500/20 rounded-none cursor-pointer transition-colors"
                  title="Delete custom wordlist from SQLite and file system"
                >
                  <Trash2 class="w-3 h-3" />
                </button>
              {/if}

              <!-- Download as Text File via HTTP Server -->
              <button
                type="button"
                onclick={() => handleDownloadWordlist(wordlist.id, wordlist.name, wordlist.paths)}
                class="px-2 py-1 text-[11px] font-bold uppercase bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] border border-[var(--color-hairline)] text-emerald-500 hover:text-emerald-400 flex items-center gap-1 transition-colors cursor-pointer"
                title={`Download ${wordlist.name} as text file via HTTP server`}
              >
                <Download class="w-3 h-3 text-emerald-500" />
                <span class="hidden sm:inline">DOWNLOAD</span>
              </button>

              <button
                type="button"
                onclick={() => copyPathsToClipboard(wordlist.id, wordlist.paths)}
                class="px-2 py-1 text-[11px] font-bold uppercase bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] border border-[var(--color-hairline)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] flex items-center gap-1 transition-colors cursor-pointer"
                title="Copy all paths in this wordlist to clipboard"
              >
                {#if copiedWordlistId === wordlist.id}
                  <Check class="w-3 h-3 text-emerald-500" />
                  <span class="text-emerald-500">COPIED</span>
                {:else}
                  <Copy class="w-3 h-3" />
                  <span>COPY</span>
                {/if}
              </button>

              <button
                type="button"
                onclick={() => {
                  expandedWordlistId = isExpanded ? null : wordlist.id;
                }}
                class="px-2 py-1 text-[11px] font-bold uppercase bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] border border-[var(--color-hairline)] text-[var(--color-text-headline)] flex items-center gap-1 transition-colors cursor-pointer"
                title="Preview and inspect all paths in this wordlist"
              >
                <span>{isExpanded ? "HIDE" : "INSPECT"}</span>
                {#if isExpanded}
                  <ChevronUp class="w-3 h-3" />
                {:else}
                  <ChevronDown class="w-3 h-3" />
                {/if}
              </button>
            </div>
          </div>

          <!-- Expanded Inline Inspector / Path Viewer -->
          {#if isExpanded}
            <div class="border-t border-[var(--color-hairline)] bg-[var(--color-canvas)] p-3 text-xs animate-fade-in">
              <div class="flex items-center justify-between mb-2">
                <span class="text-[10px] uppercase font-bold text-[var(--color-text-muted)]">
                  Complete Manifest ({wordlist.paths.length} entries)
                </span>
                <span class="text-[10px] text-[var(--color-text-muted)]">
                  Disk sync: <code>wordlists/{wordlist.id}.txt</code>
                </span>
              </div>
              <div class="max-h-48 overflow-y-auto border border-[var(--color-hairline)] bg-[var(--color-surface)] p-2 grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-1">
                {#each wordlist.paths as p, idx}
                  <div class="flex items-center gap-2 text-[11px] font-mono py-0.5 px-1 hover:bg-[var(--color-canvas)]">
                    <span class="text-[9px] text-[var(--color-text-muted)] w-5 text-right select-none">{idx + 1}</span>
                    <span class="text-[var(--color-text-headline)] truncate">{p}</span>
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        </div>
      {/each}
    {/if}
  </div>

  <!-- Bottom Summary & Status Bar -->
  <div class="p-3 bg-[var(--color-surface)] border border-[var(--color-hairline)] flex flex-col sm:flex-row items-center justify-between gap-3 text-xs">
    <div class="flex items-center gap-3 flex-wrap">
      <div class="flex items-center gap-1.5">
        <span class="w-2 h-2 rounded-none bg-[var(--color-signal-red)]"></span>
        <span class="font-bold uppercase text-[var(--color-text-headline)]">
          {config.selectedIds.length} of {wordlists.length} WORDLISTS ACTIVE
        </span>
      </div>
      <span class="text-[var(--color-text-muted)]">/</span>
      <div class="text-[var(--color-text-muted)]">
        TOTAL UNIQUE TARGETS: <strong class="text-[var(--color-text-headline)]">{totalUniquePaths.length}</strong> PATHS
      </div>
      {#if config.customPaths.length > 0}
        <span class="text-[var(--color-text-muted)]">/</span>
        <div class="text-amber-500 font-bold">
          +{config.customPaths.length} CUSTOM
        </div>
      {/if}
    </div>

    <div class="flex items-center gap-2 text-[11px] text-[var(--color-text-muted)] uppercase tracking-wider">
      <Database class="w-3 h-3 text-[var(--color-signal-red)]" />
      <span>STORAGE: SQLITE WAL + DISK .TXT</span>
    </div>
  </div>
</div>

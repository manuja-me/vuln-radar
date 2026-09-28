<script lang="ts">
  import { onMount } from "svelte";
  import {
    X,
    Sparkles,
    Download,
    RotateCw,
    CheckCircle2,
    AlertCircle,
    ArrowUpCircle,
    ExternalLink,
    ShieldCheck,
    Loader2,
  } from "lucide-svelte";
  import { check, type Update, type DownloadEvent } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";

  let {
    isOpen = $bindable(false),
    currentVersion = "1.2.0",
    onUpdateStatusChange,
    onClose,
  }: {
    isOpen: boolean;
    currentVersion?: string;
    onUpdateStatusChange?: (hasUpdate: boolean, newVersion?: string) => void;
    onClose: () => void;
  } = $props();

  type UpdatePhase =
    | "idle"
    | "checking"
    | "available"
    | "up_to_date"
    | "downloading"
    | "installing"
    | "ready_to_restart"
    | "error";

  let phase: UpdatePhase = $state("idle");
  let activeUpdate: Update | null = $state(null);
  let errorMessage: string = $state("");
  let newVersion: string = $state("");
  let releaseNotes: string = $state("");
  let releaseDate: string = $state("");
  let downloadedBytes: number = $state(0);
  let totalBytes: number = $state(0);

  let progressPercent = $derived(
    totalBytes > 0 ? Math.min(100, Math.round((downloadedBytes / totalBytes) * 100)) : 0
  );

  function formatBytes(bytes: number): string {
    if (bytes <= 0) return "0 MB";
    const mb = bytes / (1024 * 1024);
    return `${mb.toFixed(1)} MB`;
  }

  export async function checkForUpdates(manual = false): Promise<boolean> {
    if (phase === "downloading" || phase === "installing") return false;

    phase = "checking";
    errorMessage = "";

    if (manual) {
      isOpen = true;
    }

    try {
      const update = await check();
      if (update) {
        activeUpdate = update;
        newVersion = update.version;
        releaseDate = update.date ? new Date(update.date).toLocaleDateString() : "";
        releaseNotes = update.body || "A new version of VulnRadar is available with security enhancements and bug fixes.";
        phase = "available";
        isOpen = true;
        onUpdateStatusChange?.(true, update.version);
        return true;
      } else {
        activeUpdate = null;
        if (manual) {
          phase = "up_to_date";
        } else {
          phase = "idle";
        }
        onUpdateStatusChange?.(false);
        return false;
      }
    } catch (err: unknown) {
      console.warn("Update check failed:", err);
      const msg = err instanceof Error ? err.message : String(err);
      errorMessage = msg;
      if (manual) {
        phase = "error";
      } else {
        phase = "idle";
      }
      onUpdateStatusChange?.(false);
      return false;
    }
  }

  async function startDownloadAndInstall() {
    if (!activeUpdate) return;

    phase = "downloading";
    downloadedBytes = 0;
    totalBytes = 0;
    errorMessage = "";

    try {
      await activeUpdate.downloadAndInstall((event: DownloadEvent) => {
        if (event.event === "Started") {
          totalBytes = event.data.contentLength || 0;
        } else if (event.event === "Progress") {
          downloadedBytes += event.data.chunkLength;
        } else if (event.event === "Finished") {
          phase = "installing";
        }
      });

      phase = "ready_to_restart";
    } catch (err: unknown) {
      console.error("Download or install error:", err);
      errorMessage = err instanceof Error ? err.message : "Failed to download and verify update installer.";
      phase = "error";
    }
  }

  async function handleRelaunch() {
    try {
      await relaunch();
    } catch (err: unknown) {
      console.error("Relaunch error:", err);
      errorMessage = "Could not automatically restart. Please restart VulnRadar manually.";
      phase = "error";
    }
  }

  function handleCloseModal() {
    if (phase === "downloading" || phase === "installing") {
      // Don't interrupt active installation
      return;
    }
    isOpen = false;
    onClose();
  }

  // Automatic check silently on launch after initial load delay
  onMount(() => {
    const timer = setTimeout(() => {
      checkForUpdates(false).catch(() => {});
    }, 4000);

    return () => clearTimeout(timer);
  });
</script>

{#if isOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-xs p-4 animate-fade-in text-[var(--color-text-body)]"
    onclick={(e) => {
      if (e.target === e.currentTarget && phase !== "downloading" && phase !== "installing") {
        handleCloseModal();
      }
    }}
    onkeydown={(e) => {
      if (e.key === "Escape" && phase !== "downloading" && phase !== "installing") {
        handleCloseModal();
      }
    }}
    tabindex="-1"
    role="dialog"
    aria-modal="true"
  >
    <div
      class="bg-[var(--color-surface)] border border-[var(--color-hairline-strong)] rounded-none w-full max-w-lg shadow-2xl overflow-hidden flex flex-col my-auto"
    >
      <!-- Header -->
      <div class="px-5 py-4 border-b border-[var(--color-hairline)] flex items-center justify-between bg-[var(--color-canvas)]">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded bg-primary/10 border border-primary/20 flex items-center justify-center text-primary">
            <Sparkles class="w-4 h-4" />
          </div>
          <div>
            <h3 class="font-mono text-xs font-black uppercase tracking-wider text-[var(--color-text-strong)]">
              VulnRadar Software Update
            </h3>
            <p class="text-[11px] font-mono text-[var(--color-text-muted)]">
              Current Version: <span class="text-[var(--color-text-strong)]">v{currentVersion}</span>
            </p>
          </div>
        </div>

        {#if phase !== "downloading" && phase !== "installing"}
          <button
            type="button"
            onclick={handleCloseModal}
            class="text-[var(--color-text-muted)] hover:text-[var(--color-text-strong)] transition-colors cursor-pointer p-1 rounded hover:bg-[var(--color-surface-hover)]"
            title="Close"
          >
            <X class="w-4 h-4" />
          </button>
        {/if}
      </div>

      <!-- Body Content According to State -->
      <div class="p-6 flex flex-col gap-4">
        {#if phase === "checking"}
          <div class="flex flex-col items-center justify-center py-8 gap-3">
            <Loader2 class="w-8 h-8 text-primary animate-spin" />
            <p class="font-mono text-xs text-[var(--color-text-strong)]">
              Connecting to GitHub Releases...
            </p>
            <p class="text-[11px] font-mono text-[var(--color-text-muted)]">
              Checking cryptographic manifest and signatures
            </p>
          </div>

        {:else if phase === "up_to_date"}
          <div class="flex flex-col items-center justify-center py-6 text-center gap-3">
            <div class="w-12 h-12 rounded-full bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center text-emerald-400">
              <CheckCircle2 class="w-6 h-6" />
            </div>
            <div>
              <h4 class="font-mono text-sm font-bold text-[var(--color-text-strong)]">
                You're completely up to date!
              </h4>
              <p class="text-xs font-mono text-[var(--color-text-muted)] mt-1">
                VulnRadar v{currentVersion} is the latest released version.
              </p>
            </div>
          </div>

        {:else if phase === "available"}
          <div class="flex flex-col gap-3.5">
            <div class="flex items-start justify-between bg-primary/5 border border-primary/20 p-3 rounded">
              <div class="flex items-center gap-2.5">
                <ArrowUpCircle class="w-5 h-5 text-primary shrink-0" />
                <div>
                  <span class="font-mono text-xs font-bold text-[var(--color-text-strong)]">
                    New Release Available: v{newVersion}
                  </span>
                  {#if releaseDate}
                    <p class="text-[10px] font-mono text-[var(--color-text-muted)]">Released on {releaseDate}</p>
                  {/if}
                </div>
              </div>
              <span class="px-2 py-0.5 text-[10px] font-mono font-bold bg-primary text-black rounded uppercase">
                New
              </span>
            </div>

            <!-- Release Notes Snippet -->
            <div class="flex flex-col gap-1.5">
              <span class="font-mono text-[10px] uppercase font-bold text-[var(--color-text-muted)] tracking-wider">
                Release Information
              </span>
              <div class="p-3 bg-[var(--color-canvas)] border border-[var(--color-hairline)] rounded max-h-36 overflow-y-auto text-xs font-mono whitespace-pre-wrap leading-relaxed text-[var(--color-text-body)]">
                {releaseNotes}
              </div>
            </div>

            <div class="flex items-center gap-2 text-[11px] font-mono text-[var(--color-text-muted)]">
              <ShieldCheck class="w-4 h-4 text-emerald-400 shrink-0" />
              <span>Signed with Minisign Ed25519 cryptographic signature.</span>
            </div>
          </div>

        {:else if phase === "downloading" || phase === "installing"}
          <div class="flex flex-col gap-4 py-2">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <Loader2 class="w-4 h-4 text-primary animate-spin" />
                <span class="font-mono text-xs font-bold text-[var(--color-text-strong)]">
                  {phase === "installing" ? "Verifying signature and applying update..." : `Downloading VulnRadar v${newVersion}...`}
                </span>
              </div>
              <span class="font-mono text-xs text-primary font-bold">
                {phase === "installing" ? "Installing" : `${progressPercent}%`}
              </span>
            </div>

            <!-- Progress Bar -->
            <div class="w-full bg-[var(--color-canvas)] border border-[var(--color-hairline)] h-3 rounded overflow-hidden p-0.5">
              <div
                class="bg-primary h-full transition-all duration-200 ease-out rounded-xs {phase === 'installing' ? 'animate-pulse' : ''}"
                style="width: {phase === 'installing' ? 100 : progressPercent}%"
              ></div>
            </div>

            <div class="flex justify-between items-center text-[10px] font-mono text-[var(--color-text-muted)]">
              <span>{phase === "installing" ? "Writing verified binaries..." : "Streaming package over secure HTTPS"}</span>
              {#if totalBytes > 0 && phase === "downloading"}
                <span>{formatBytes(downloadedBytes)} / {formatBytes(totalBytes)}</span>
              {/if}
            </div>
          </div>

        {:else if phase === "ready_to_restart"}
          <div class="flex flex-col items-center justify-center py-4 text-center gap-3">
            <div class="w-12 h-12 rounded-full bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center text-emerald-400">
              <CheckCircle2 class="w-6 h-6" />
            </div>
            <div>
              <h4 class="font-mono text-sm font-bold text-[var(--color-text-strong)]">
                Update Ready to Apply!
              </h4>
              <p class="text-xs font-mono text-[var(--color-text-muted)] mt-1">
                VulnRadar v{newVersion} was verified and installed. Relaunch now to start using the new version.
              </p>
            </div>
          </div>

        {:else if phase === "error"}
          <div class="flex flex-col gap-3 bg-red-500/10 border border-red-500/20 p-4 rounded text-left">
            <div class="flex items-center gap-2 text-red-400">
              <AlertCircle class="w-4 h-4 shrink-0" />
              <span class="font-mono text-xs font-bold">Update Check or Installation Failed</span>
            </div>
            <p class="text-[11px] font-mono text-[var(--color-text-muted)] break-all leading-normal">
              {errorMessage || "Unable to reach update server or verify signature."}
            </p>
          </div>
        {/if}
      </div>

      <!-- Footer Buttons -->
      <div class="px-5 py-3.5 border-t border-[var(--color-hairline)] bg-[var(--color-canvas)] flex items-center justify-between">
        <a
          href="https://github.com/manuja-me/vuln-radar/releases"
          target="_blank"
          rel="noreferrer"
          class="text-[11px] font-mono text-[var(--color-text-muted)] hover:text-primary flex items-center gap-1 transition-colors"
        >
          <span>GitHub Releases</span>
          <ExternalLink class="w-3 h-3" />
        </a>

        <div class="flex items-center gap-2">
          {#if phase === "available"}
            <button
              type="button"
              onclick={handleCloseModal}
              class="px-3 py-1.5 font-mono text-xs text-[var(--color-text-muted)] hover:text-[var(--color-text-strong)] hover:bg-[var(--color-surface-hover)] rounded transition-colors cursor-pointer"
            >
              Later
            </button>
            <button
              type="button"
              onclick={startDownloadAndInstall}
              class="px-3.5 py-1.5 font-mono text-xs font-bold uppercase bg-primary hover:bg-primary/90 text-black rounded flex items-center gap-1.5 transition-colors cursor-pointer"
            >
              <Download class="w-3.5 h-3.5" />
              <span>Download & Install</span>
            </button>

          {:else if phase === "ready_to_restart"}
            <button
              type="button"
              onclick={handleRelaunch}
              class="px-4 py-1.5 font-mono text-xs font-bold uppercase bg-emerald-500 hover:bg-emerald-400 text-black rounded flex items-center gap-1.5 transition-colors cursor-pointer"
            >
              <RotateCw class="w-3.5 h-3.5" />
              <span>Restart VulnRadar</span>
            </button>

          {:else if phase === "error"}
            <button
              type="button"
              onclick={() => checkForUpdates(true)}
              class="px-3.5 py-1.5 font-mono text-xs font-bold uppercase bg-[var(--color-surface)] border border-[var(--color-hairline-strong)] text-[var(--color-text-strong)] hover:border-primary rounded flex items-center gap-1.5 transition-colors cursor-pointer"
            >
              <RotateCw class="w-3.5 h-3.5" />
              <span>Retry Check</span>
            </button>
            <button
              type="button"
              onclick={handleCloseModal}
              class="px-3.5 py-1.5 font-mono text-xs font-bold uppercase bg-primary text-black rounded transition-colors cursor-pointer"
            >
              Done
            </button>

          {:else if phase === "up_to_date"}
            <button
              type="button"
              onclick={handleCloseModal}
              class="px-4 py-1.5 font-mono text-xs font-bold uppercase bg-primary text-black rounded transition-colors cursor-pointer"
            >
              Close
            </button>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}

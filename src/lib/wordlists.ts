import type {
  WordlistItem,
  WordlistCategory,
  WordlistRecord,
  DynamicWordlistParams,
} from "$lib/types";

export const BUILTIN_WORDLISTS: WordlistItem[] = [
  {
    id: "common_paths",
    name: "Common Web Paths & Administration",
    category: "paths",
    description: "High-value administrative routes, login portals, dashboards, and control consoles.",
    tags: ["Auth", "Admin", "Portals", "Common"],
    paths: [
      "/admin",
      "/admin/login",
      "/administrator",
      "/login",
      "/signin",
      "/dashboard",
      "/portal",
      "/console",
      "/auth",
      "/user/login",
      "/account/login",
      "/manage",
      "/backend",
      "/cpanel",
      "/private",
      "/secure",
      "/control",
      "/member",
      "/panel",
      "/system",
      "/superadmin",
      "/root",
      "/app",
      "/internal"
    ]
  },
  {
    id: "directory_names",
    name: "Directory & Folder Names",
    category: "directories",
    description: "Typical server subdirectories, static asset storage, test folders, and staging paths.",
    tags: ["Directories", "Folders", "Recon"],
    paths: [
      "/backup/",
      "/config/",
      "/database/",
      "/uploads/",
      "/files/",
      "/static/",
      "/assets/",
      "/server/",
      "/includes/",
      "/logs/",
      "/temp/",
      "/tmp/",
      "/test/",
      "/testing/",
      "/dev/",
      "/development/",
      "/staging/",
      "/internal/",
      "/private/",
      "/src/",
      "/data/",
      "/media/",
      "/docs/",
      "/vendor/"
    ]
  },
  {
    id: "sensitive_files",
    name: "Sensitive Files & Dotfiles",
    category: "files",
    description: "Environment secret files, version control metadata, credentials, and config dotfiles.",
    tags: ["Secrets", "Dotfiles", "Credentials", "Critical"],
    paths: [
      "/.env",
      "/.env.local",
      "/.env.production",
      "/.env.development",
      "/.env.backup",
      "/.git/HEAD",
      "/.git/config",
      "/.gitignore",
      "/.svn/entries",
      "/.gitlab-ci.yml",
      "/.travis.yml",
      "/docker-compose.yml",
      "/Dockerfile",
      "/config.json",
      "/settings.json",
      "/web.config",
      "/id_rsa",
      "/id_rsa.pub",
      "/server.key",
      "/database.sqlite",
      "/.bash_history",
      "/.htpasswd",
      "/.htaccess"
    ]
  },
  {
    id: "backup_archives",
    name: "Database Dumps & Backup Archives",
    category: "backups",
    description: "SQL database dumps, compressed site backups, and old configuration files.",
    tags: ["Backups", "SQL", "Archives", "High"],
    paths: [
      "/backup.sql",
      "/dump.sql",
      "/db.sql",
      "/data.sql",
      "/backup.tar.gz",
      "/backup.zip",
      "/site.tar",
      "/archive.zip",
      "/db_backup.sql",
      "/wp-config.php.bak",
      "/wp-config.php.old",
      "/index.php.old",
      "/config.php~",
      "/settings.php.bak",
      "/web.config.bak"
    ]
  },
  {
    id: "api_documentation",
    name: "API & Specification Endpoints",
    category: "api",
    description: "OpenAPI, Swagger specifications, GraphQL explorer endpoints, and developer API routes.",
    tags: ["API", "Swagger", "OpenAPI", "GraphQL"],
    paths: [
      "/swagger.json",
      "/swagger/v1/swagger.json",
      "/swagger-ui.html",
      "/swagger-ui/",
      "/openapi.json",
      "/openapi.yaml",
      "/v2/api-docs",
      "/v3/api-docs",
      "/api/swagger",
      "/api/docs",
      "/graphql",
      "/graphiql",
      "/api/v1",
      "/api/v2",
      "/api/health"
    ]
  },
  {
    id: "debuggers_telemetry",
    name: "Debuggers, Profilers & Telemetry",
    category: "debug",
    description: "PHP diagnostic files, Symfony profilers, Laravel debugbars, and Spring Actuator metrics.",
    tags: ["Diagnostics", "Profilers", "Metrics", "Spring"],
    paths: [
      "/phpinfo.php",
      "/_profiler/",
      "/_debugbar/assets/stylesheets",
      "/telescope/requests",
      "/elmah.axd",
      "/metrics",
      "/actuator",
      "/actuator/health",
      "/actuator/env",
      "/actuator/beans",
      "/actuator/info",
      "/actuator/gateway/routes",
      "/debug",
      "/trace"
    ]
  }
];

export interface WordlistPreset {
  id: string;
  name: string;
  description: string;
  selectedIds: string[];
}

export const WORDLIST_PRESETS: WordlistPreset[] = [
  {
    id: "balanced",
    name: "Default Balanced (Essential Paths)",
    description: "Standard reconnaissance covering critical secrets, common portals, and API specs.",
    selectedIds: ["common_paths", "sensitive_files", "api_documentation"]
  },
  {
    id: "quick",
    name: "Quick Recon (Top Paths)",
    description: "Fastest pass targeting admin panels, authentication portals, and exposed dotfiles.",
    selectedIds: ["common_paths", "sensitive_files"]
  },
  {
    id: "directories",
    name: "Directory & Structure Audit",
    description: "Audits folder structures, static assets, and subdirectories.",
    selectedIds: ["common_paths", "directory_names"]
  },
  {
    id: "sensitive_leaks",
    name: "Sensitive Files & Backup Hunt",
    description: "Focused scan targeting SQL dumps, git repositories, .env files, and backups.",
    selectedIds: ["sensitive_files", "backup_archives"]
  },
  {
    id: "full",
    name: "Full Comprehensive Audit",
    description: "Scans all built-in wordlists (over 100+ high-fidelity paths & probes).",
    selectedIds: ["common_paths", "directory_names", "sensitive_files", "backup_archives", "api_documentation", "debuggers_telemetry"]
  },
  {
    id: "custom",
    name: "Custom Wordlist Only",
    description: "Scans exclusively with user-provided paths from a .txt file or manual input.",
    selectedIds: []
  }
];

export function computeUniquePaths(selectedIds: string[], customPaths: string[] = []): string[] {
  const set = new Set<string>();

  for (const wl of BUILTIN_WORDLISTS) {
    if (selectedIds.includes(wl.id)) {
      for (const p of wl.paths) {
        const clean = p.trim();
        if (clean) set.add(clean.startsWith("/") ? clean : `/${clean}`);
      }
    }
  }

  for (const p of customPaths) {
    const clean = p.trim();
    if (clean) set.add(clean.startsWith("/") ? clean : `/${clean}`);
  }

  return Array.from(set).sort();
}

export function exportWordlistTxt(paths: string[], filename: string = "vulnradar-wordlist.txt") {
  const content = paths.join("\n");
  const blob = new Blob([content], { type: "text/plain;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

async function invokeTauri<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return await invoke<T>(cmd, args);
}

export async function fetchWordlistsFromBackend(): Promise<WordlistRecord[]> {
  try {
    return await invokeTauri<WordlistRecord[]>("get_wordlists");
  } catch (e) {
    console.warn("Backend get_wordlists not available, using builtins:", e);
    return BUILTIN_WORDLISTS.map((w) => ({
      id: w.id,
      name: w.name,
      category: w.category,
      description: w.description,
      paths: w.paths,
      item_count: w.paths.length,
      is_custom: false,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }));
  }
}

export async function saveWordlistToBackend(item: WordlistRecord): Promise<WordlistRecord> {
  try {
    return await invokeTauri<WordlistRecord>("save_wordlist", { item });
  } catch (e) {
    console.warn("Backend save_wordlist not available:", e);
    return item;
  }
}

export async function deleteWordlistFromBackend(id: string): Promise<void> {
  try {
    await invokeTauri<void>("delete_wordlist", { id });
  } catch (e) {
    console.warn("Backend delete_wordlist not available:", e);
  }
}

export async function generateDynamicWordlistBackend(params: DynamicWordlistParams): Promise<string[]> {
  try {
    return await invokeTauri<string[]>("generate_dynamic_wordlist", { params });
  } catch (e) {
    console.warn("Backend generate_dynamic_wordlist not available, using local generator:", e);
    return generateDynamicWordlistClient(params);
  }
}

export async function exportWordlistToDisk(filename: string, paths: string[]): Promise<string> {
  try {
    return await invokeTauri<string>("export_wordlist_file", { filename, paths });
  } catch (e) {
    console.warn("Backend export_wordlist_file not available, falling back to browser download:", e);
    exportWordlistTxt(paths, filename);
    return filename;
  }
}


export function generateDynamicWordlistClient(params: DynamicWordlistParams): string[] {
  const set = new Set<string>();
  const baseWords =
    params.base_words && params.base_words.length > 0
      ? params.base_words
      : ["admin", "api", "config", "backup", "login", "dashboard", "portal", "user", "auth", "dev", "test", "server"];
  const directories = params.directories && params.directories.length > 0 ? params.directories : [""];
  const prefixes = params.prefixes && params.prefixes.length > 0 ? params.prefixes : [""];
  const extensions = params.extensions || [];

  for (const word of baseWords) {
    const cleanWord = word.trim().replace(/^\/+|\/+$/g, "");
    if (!cleanWord) continue;

    for (const dir of directories) {
      const cleanDir = dir.trim().replace(/^\/+|\/+$/g, "");
      const dirPfx = cleanDir ? `${cleanDir}/` : "";

      for (const pfx of prefixes) {
        const cleanPfx = pfx.trim();
        const stem = `${dirPfx}${cleanPfx}${cleanWord}`;

        set.add(`/${stem}`);

        for (const ext of extensions) {
          const cleanExt = ext.trim().replace(/^\.+/, "");
          if (cleanExt) {
            set.add(`/${stem}.${cleanExt}`);
            if (params.include_backups) {
              set.add(`/${stem}.${cleanExt}.bak`);
              set.add(`/${stem}.${cleanExt}.old`);
              set.add(`/${stem}.${cleanExt}~`);
            }
          }
        }

        if (params.include_dotfiles) {
          set.add(`/.${stem}`);
          for (const ext of extensions) {
            const cleanExt = ext.trim().replace(/^\.+/, "");
            if (cleanExt) {
              set.add(`/.${stem}.${cleanExt}`);
            }
          }
        }
      }
    }
  }

  return Array.from(set).sort();
}

export async function getServerPort(): Promise<number> {
  try {
    return await invokeTauri<number>("get_server_port");
  } catch (e) {
    console.warn("Could not get server port:", e);
    return 0;
  }
}

export async function downloadWordlistFromServer(
  id: string,
  filename?: string,
  fallbackPaths: string[] = []
): Promise<void> {
  const safeFilename = filename || `${id}.txt`;
  try {
    const port = await getServerPort();
    if (port > 0) {
      const downloadUrl = `http://127.0.0.1:${port}/api/wordlist/download?id=${encodeURIComponent(id)}`;
      const response = await fetch(downloadUrl);
      if (response.ok) {
        const blob = await response.blob();
        const disposition = response.headers.get("Content-Disposition");
        let resolvedFilename = safeFilename;
        if (disposition && disposition.includes("filename=")) {
          const match = disposition.match(/filename="?([^";]+)"?/);
          if (match && match[1]) resolvedFilename = match[1];
        }
        const blobUrl = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = blobUrl;
        a.download = resolvedFilename;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(blobUrl);
        return;
      }
    }
  } catch (e) {
    console.warn("Server HTTP download failed, using client-side fallback:", e);
  }

  // Fallback if local HTTP server is unavailable
  exportWordlistTxt(fallbackPaths, safeFilename);
}

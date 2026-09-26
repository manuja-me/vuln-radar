use crate::models::{DynamicWordlistParams, WordlistRecord};
use chrono::Utc;
use std::collections::BTreeSet;
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

#[allow(clippy::vec_init_then_push)]
pub fn get_predefined_records() -> Vec<WordlistRecord> {
    let now = Utc::now().to_rfc3339();
    let mut records = Vec::with_capacity(6);

    records.push(WordlistRecord {
        id: "common_paths".to_string(),
        name: "Common Web Paths & Administration".to_string(),
        category: "paths".to_string(),
        description: "High-value administrative routes, login portals, dashboards, and control consoles.".to_string(),
        paths: vec![
            "/admin".to_string(),
            "/admin/login".to_string(),
            "/administrator".to_string(),
            "/login".to_string(),
            "/signin".to_string(),
            "/dashboard".to_string(),
            "/portal".to_string(),
            "/console".to_string(),
            "/auth".to_string(),
            "/user/login".to_string(),
            "/account/login".to_string(),
            "/manage".to_string(),
            "/backend".to_string(),
            "/cpanel".to_string(),
            "/private".to_string(),
            "/secure".to_string(),
            "/control".to_string(),
            "/member".to_string(),
            "/panel".to_string(),
            "/system".to_string(),
            "/superadmin".to_string(),
            "/root".to_string(),
            "/app".to_string(),
            "/internal".to_string(),
        ],
        item_count: 24,
        is_custom: false,
        created_at: now.clone(),
        updated_at: now.clone(),
    });

    records.push(WordlistRecord {
        id: "directory_names".to_string(),
        name: "Directory & Folder Names".to_string(),
        category: "directories".to_string(),
        description: "Typical server subdirectories, static asset storage, test folders, and staging paths.".to_string(),
        paths: vec![
            "/backup/".to_string(),
            "/config/".to_string(),
            "/database/".to_string(),
            "/uploads/".to_string(),
            "/files/".to_string(),
            "/static/".to_string(),
            "/assets/".to_string(),
            "/server/".to_string(),
            "/includes/".to_string(),
            "/logs/".to_string(),
            "/temp/".to_string(),
            "/tmp/".to_string(),
            "/test/".to_string(),
            "/testing/".to_string(),
            "/dev/".to_string(),
            "/development/".to_string(),
            "/staging/".to_string(),
            "/internal/".to_string(),
            "/private/".to_string(),
            "/src/".to_string(),
            "/data/".to_string(),
            "/media/".to_string(),
            "/docs/".to_string(),
            "/vendor/".to_string(),
        ],
        item_count: 24,
        is_custom: false,
        created_at: now.clone(),
        updated_at: now.clone(),
    });

    records.push(WordlistRecord {
        id: "sensitive_files".to_string(),
        name: "Sensitive Files & Dotfiles".to_string(),
        category: "files".to_string(),
        description: "Environment secret files, version control metadata, credentials, and config dotfiles.".to_string(),
        paths: vec![
            "/.env".to_string(),
            "/.env.local".to_string(),
            "/.env.production".to_string(),
            "/.env.development".to_string(),
            "/.env.backup".to_string(),
            "/.git/HEAD".to_string(),
            "/.git/config".to_string(),
            "/.gitignore".to_string(),
            "/.svn/entries".to_string(),
            "/.gitlab-ci.yml".to_string(),
            "/.travis.yml".to_string(),
            "/docker-compose.yml".to_string(),
            "/Dockerfile".to_string(),
            "/config.json".to_string(),
            "/settings.json".to_string(),
            "/web.config".to_string(),
            "/id_rsa".to_string(),
            "/id_rsa.pub".to_string(),
            "/server.key".to_string(),
            "/database.sqlite".to_string(),
            "/.bash_history".to_string(),
            "/.htpasswd".to_string(),
            "/.htaccess".to_string(),
        ],
        item_count: 23,
        is_custom: false,
        created_at: now.clone(),
        updated_at: now.clone(),
    });

    records.push(WordlistRecord {
        id: "backup_archives".to_string(),
        name: "Database Dumps & Backup Archives".to_string(),
        category: "backups".to_string(),
        description: "SQL database dumps, compressed site backups, and old configuration files.".to_string(),
        paths: vec![
            "/backup.sql".to_string(),
            "/dump.sql".to_string(),
            "/db.sql".to_string(),
            "/data.sql".to_string(),
            "/backup.tar.gz".to_string(),
            "/backup.zip".to_string(),
            "/site.tar".to_string(),
            "/archive.zip".to_string(),
            "/db_backup.sql".to_string(),
            "/wp-config.php.bak".to_string(),
            "/wp-config.php.old".to_string(),
            "/index.php.old".to_string(),
            "/config.php~".to_string(),
            "/settings.php.bak".to_string(),
            "/web.config.bak".to_string(),
        ],
        item_count: 15,
        is_custom: false,
        created_at: now.clone(),
        updated_at: now.clone(),
    });

    records.push(WordlistRecord {
        id: "api_documentation".to_string(),
        name: "API & Specification Endpoints".to_string(),
        category: "api".to_string(),
        description: "OpenAPI, Swagger specifications, GraphQL explorer endpoints, and developer API routes.".to_string(),
        paths: vec![
            "/swagger.json".to_string(),
            "/swagger/v1/swagger.json".to_string(),
            "/swagger-ui.html".to_string(),
            "/swagger-ui/".to_string(),
            "/openapi.json".to_string(),
            "/openapi.yaml".to_string(),
            "/v2/api-docs".to_string(),
            "/v3/api-docs".to_string(),
            "/api/swagger".to_string(),
            "/api/docs".to_string(),
            "/graphql".to_string(),
            "/graphiql".to_string(),
            "/api/v1".to_string(),
            "/api/v2".to_string(),
            "/api/health".to_string(),
        ],
        item_count: 15,
        is_custom: false,
        created_at: now.clone(),
        updated_at: now.clone(),
    });

    records.push(WordlistRecord {
        id: "debuggers_telemetry".to_string(),
        name: "Debuggers, Profilers & Telemetry".to_string(),
        category: "debug".to_string(),
        description: "PHP diagnostic files, Symfony profilers, Laravel debugbars, and Spring Actuator metrics.".to_string(),
        paths: vec![
            "/phpinfo.php".to_string(),
            "/_profiler/".to_string(),
            "/_debugbar/assets/stylesheets".to_string(),
            "/telescope/requests".to_string(),
            "/elmah.axd".to_string(),
            "/metrics".to_string(),
            "/actuator".to_string(),
            "/actuator/health".to_string(),
            "/actuator/env".to_string(),
            "/actuator/beans".to_string(),
            "/actuator/info".to_string(),
            "/actuator/gateway/routes".to_string(),
            "/debug".to_string(),
            "/trace".to_string(),
        ],
        item_count: 14,
        is_custom: false,
        created_at: now.clone(),
        updated_at: now,
    });

    records
}

/// Dynamically generates word lists based on user input parameters
pub fn generate_dynamic_wordlist(params: &DynamicWordlistParams) -> Vec<String> {
    let mut results = BTreeSet::new();

    let default_words = vec![
        "admin".to_string(),
        "api".to_string(),
        "config".to_string(),
        "backup".to_string(),
        "login".to_string(),
        "dashboard".to_string(),
        "portal".to_string(),
        "user".to_string(),
        "auth".to_string(),
        "dev".to_string(),
        "test".to_string(),
        "server".to_string(),
    ];

    let base_words = if params.base_words.is_empty() {
        &default_words
    } else {
        &params.base_words
    };

    let mut prefixes = params.prefixes.clone();
    if prefixes.is_empty() {
        prefixes.push(String::new());
    }

    let mut directories = params.directories.clone();
    if directories.is_empty() {
        directories.push(String::new());
    }

    let extensions = &params.extensions;

    for word in base_words {
        let clean_word = word.trim().trim_matches('/');
        if clean_word.is_empty() {
            continue;
        }

        for dir in &directories {
            let clean_dir = dir.trim().trim_matches('/');
            let dir_prefix = match clean_dir.is_empty() {
                true => String::new(),
                false => format!("{}/", clean_dir),
            };

            for pfx in &prefixes {
                let clean_pfx = pfx.trim();
                let stem = format!("{}{}{}", dir_prefix, clean_pfx, clean_word);

                // Base path
                results.insert(format!("/{}", stem));

                // Extensions
                for ext in extensions {
                    let clean_ext = ext.trim().trim_start_matches('.');
                    if !clean_ext.is_empty() {
                        results.insert(format!("/{}.{}", stem, clean_ext));

                        if params.include_backups {
                            results.insert(format!("/{}.{}.bak", stem, clean_ext));
                            results.insert(format!("/{}.{}.old", stem, clean_ext));
                            results.insert(format!("/{}.{}~", stem, clean_ext));
                        }
                    }
                }

                // Dotfile variants if requested
                if params.include_dotfiles {
                    results.insert(format!("/.{}", stem));
                    for ext in extensions {
                        let clean_ext = ext.trim().trim_start_matches('.');
                        if !clean_ext.is_empty() {
                            results.insert(format!("/.{}.{}", stem, clean_ext));
                        }
                    }
                }
            }
        }
    }

    results.into_iter().collect()
}

// --- Filesystem Storage Utilities ---

/// Ensures wordlists storage directory exists and returns its PathBuf
pub fn ensure_wordlist_dir(app_data_dir: &Path) -> io::Result<PathBuf> {
    let dir = app_data_dir.join("wordlists");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Saves wordlist paths to a .txt file on the filesystem
pub fn save_to_filesystem(dir: &Path, id: &str, paths: &[String]) -> io::Result<PathBuf> {
    let filename = format!("{}.txt", id.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_"));
    let target_path = dir.join(filename);
    let content = paths.join("\n");
    fs::write(&target_path, content)?;
    Ok(target_path)
}

/// Reads paths from a .txt file on the filesystem
pub fn read_from_filesystem(file_path: &Path) -> io::Result<Vec<String>> {
    let file = fs::File::open(file_path)?;
    let reader = io::BufReader::new(file);
    let mut paths = Vec::new();

    for line in reader.lines() {
        let l = line?;
        let trimmed = l.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            let normalized = format!("/{}", trimmed.trim_start_matches('/'));
            paths.push(normalized);
        }
    }

    Ok(paths)
}

/// Deletes a wordlist file from the filesystem directory
pub fn delete_from_filesystem(dir: &Path, id: &str) -> io::Result<()> {
    let filename = format!("{}.txt", id.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_"));
    let target_path = dir.join(filename);
    if target_path.exists() {
        fs::remove_file(target_path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predefined_records_structure() {
        let records = get_predefined_records();
        assert_eq!(records.len(), 6);
        assert_eq!(records[0].id, "common_paths");
        assert!(!records[0].paths.is_empty());
    }

    #[test]
    fn test_generate_dynamic_wordlist() {
        let params = DynamicWordlistParams {
            base_words: vec!["api".to_string()],
            directories: vec!["v1".to_string()],
            extensions: vec!["json".to_string()],
            prefixes: vec![],
            include_dotfiles: false,
            include_backups: false,
        };
        let list = generate_dynamic_wordlist(&params);
        assert!(!list.is_empty());
        assert!(list.iter().any(|p| p.ends_with(".json") || p.contains("api")));
    }
}


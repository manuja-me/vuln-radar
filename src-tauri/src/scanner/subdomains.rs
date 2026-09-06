use reqwest::Client;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::time::Duration;

#[derive(Deserialize)]
struct CrtShEntry {
    name_value: Option<String>,
}

pub async fn discover_subdomains(client: &Client, domain: &str) -> Vec<String> {
    let clean_domain = domain.trim_start_matches("www.").to_lowercase();
    if clean_domain.is_empty() || clean_domain == "localhost" || clean_domain.parse::<std::net::IpAddr>().is_ok() {
        return Vec::new();
    }

    let mut subdomains: BTreeSet<String> = BTreeSet::new();

    // 1. Query HackerTarget Hostsearch API (Fast & highly reliable)
    let ht_fut = async {
        let url = format!("https://api.hackertarget.com/hostsearch/?q={}", clean_domain);
        if let Ok(resp) = client.get(&url).timeout(Duration::from_secs(5)).send().await {
            if resp.status().is_success() {
                if let Ok(text) = resp.text().await {
                    let mut found = Vec::new();
                    for line in text.lines() {
                        if let Some((sub, _)) = line.split_once(',') {
                            let sub_clean = sub.trim().to_lowercase();
                            if sub_clean.ends_with(&clean_domain) && !sub_clean.is_empty() {
                                found.push(sub_clean);
                            }
                        }
                    }
                    return found;
                }
            }
        }
        Vec::new()
    };

    // 2. Query crt.sh Certificate Transparency with short timeout
    let crt_fut = async {
        let url = format!("https://crt.sh/?q=%.{}&output=json", clean_domain);
        if let Ok(resp) = client.get(&url).timeout(Duration::from_secs(4)).send().await {
            if resp.status().is_success() {
                if let Ok(entries) = resp.json::<Vec<CrtShEntry>>().await {
                    let mut found = Vec::new();
                    for entry in entries {
                        if let Some(names) = entry.name_value {
                            for line in names.lines() {
                                let name = line.trim().trim_start_matches("*.").to_lowercase();
                                if name.ends_with(&clean_domain) && !name.is_empty() {
                                    found.push(name);
                                }
                            }
                        }
                    }
                    return found;
                }
            }
        }
        Vec::new()
    };

    // 3. Fast Bounded Concurrent DNS Active Wordlist Probing
    let dns_probe_fut = async {
        const WORDLIST: &[&str] = &[
            "www", "api", "app", "dev", "staging", "mail", "admin", "portal", "auth", "blog",
            "cdn", "status", "test", "beta", "vpn", "shop", "docs", "git", "dashboard", "m",
            "login", "secure", "static", "support", "cloud", "ws", "backend", "demo", "stage",
            "preview", "v1", "v2", "grafana", "assets", "media", "internal", "hub", "campusnest",
        ];

        let mut set = tokio::task::JoinSet::new();
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(12));

        for &sub_prefix in WORDLIST {
            let candidate = format!("{}.{}", sub_prefix, clean_domain);
            let sem = semaphore.clone();
            set.spawn(async move {
                let _permit = sem.acquire().await.ok()?;
                let host_port = format!("{}:80", candidate);
                let is_alive = match tokio::net::lookup_host(&host_port).await {
                    Ok(mut addrs) => addrs.next().is_some(),
                    Err(_) => false,
                };
                if is_alive {
                    Some(candidate)
                } else {
                    None
                }
            });
        }

        let mut found = Vec::new();
        while let Some(res) = set.join_next().await {
            if let Ok(Some(sub)) = res {
                found.push(sub);
            }
        }
        found
    };

    // Run all three discovery strategies concurrently
    let (ht_res, crt_res, dns_res) = tokio::join!(ht_fut, crt_fut, dns_probe_fut);

    for s in ht_res {
        subdomains.insert(s);
    }
    for s in crt_res {
        subdomains.insert(s);
    }
    for s in dns_res {
        subdomains.insert(s);
    }

    subdomains.into_iter().take(60).collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_clean_domain_parsing() {
        let name = "*.sub.example.com";
        let clean = name.trim().trim_start_matches("*.").to_lowercase();
        assert_eq!(clean, "sub.example.com");
    }
}

use reqwest::Client;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::time::Duration;

#[derive(Deserialize)]
struct CrtShEntry {
    name_value: Option<String>,
}

pub async fn discover_subdomains(domain: &str) -> Vec<String> {
    let clean_domain = domain.trim_start_matches("www.").to_lowercase();
    if clean_domain.is_empty() || clean_domain == "localhost" || clean_domain.parse::<std::net::IpAddr>().is_ok() {
        return Vec::new();
    }

    let url = format!("https://crt.sh/?q=%.{}&output=json", clean_domain);

    let client = match Client::builder()
        .timeout(Duration::from_secs(8))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) VulnRadar/1.0")
        .build()
    {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    if !resp.status().is_success() {
        return Vec::new();
    }

    let entries: Vec<CrtShEntry> = match resp.json().await {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut subdomains: BTreeSet<String> = BTreeSet::new();

    for entry in entries {
        if let Some(names) = entry.name_value {
            for line in names.lines() {
                let name = line.trim().trim_start_matches("*.").to_lowercase();
                if name.ends_with(&clean_domain) && !name.is_empty() {
                    subdomains.insert(name);
                }
            }
        }
    }

    subdomains.into_iter().take(50).collect()
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

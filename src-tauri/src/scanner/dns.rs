use crate::models::{Category, DnsRecord, DnsSecurityReport, Finding, Severity};
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct DohAnswer {
    #[serde(default)]
    name: String,
    #[serde(rename = "type", default)]
    record_type: u16,
    #[serde(rename = "TTL", default)]
    ttl: u32,
    #[serde(default)]
    data: String,
}

#[derive(Deserialize)]
struct DohResponse {
    #[serde(rename = "Status")]
    _status: Option<u32>,
    #[serde(rename = "AD")]
    ad: Option<bool>,
    #[serde(rename = "Answer")]
    answer: Option<Vec<DohAnswer>>,
}

pub fn type_num_to_str(t: u16) -> &'static str {
    match t {
        1 => "A",
        28 => "AAAA",
        2 => "NS",
        5 => "CNAME",
        6 => "SOA",
        15 => "MX",
        16 => "TXT",
        257 => "CAA",
        _ => "OTHER",
    }
}

pub fn record_note(rec_type: &str, data: &str) -> &'static str {
    match rec_type {
        "A" => "IPv4 Host Address",
        "AAAA" => "IPv6 Host Address",
        "NS" => "Authoritative Nameserver",
        "MX" => "Mail Exchange",
        "TXT" if data.to_lowercase().starts_with("v=spf1") => "SPF Verification Directive",
        "TXT" if data.to_lowercase().starts_with("v=dmarc1") => "DMARC Conformance Policy",
        "TXT" => "Text / Verification Record",
        "SOA" => "Start of Authority",
        "CNAME" => "Canonical Name Alias",
        "CAA" => "Certificate Authority Authorization",
        _ => "DNS Resource Record",
    }
}

async fn query_doh(client: &Client, name: &str, record_type: &str) -> (Vec<DohAnswer>, bool, u64) {
    let start = std::time::Instant::now();
    let cf_url = format!("https://cloudflare-dns.com/dns-query?name={}&type={}", name, record_type);

    if let Ok(resp) = client
        .get(&cf_url)
        .header("accept", "application/dns-json")
        .send()
        .await
    {
        if resp.status().is_success() {
            if let Ok(doh) = resp.json::<DohResponse>().await {
                let elapsed = start.elapsed().as_millis() as u64;
                let dnssec = doh.ad.unwrap_or(false);
                let answers = doh.answer.unwrap_or_default();
                return (answers, dnssec, elapsed);
            }
        }
    }

    // Secondary fallback: Google Public DNS DoH (RFC 8484 JSON)
    let google_url = format!("https://dns.google/resolve?name={}&type={}", name, record_type);
    if let Ok(resp) = client
        .get(&google_url)
        .header("accept", "application/dns-json")
        .send()
        .await
    {
        let elapsed = start.elapsed().as_millis() as u64;
        if resp.status().is_success() {
            if let Ok(doh) = resp.json::<DohResponse>().await {
                let dnssec = doh.ad.unwrap_or(false);
                let answers = doh.answer.unwrap_or_default();
                return (answers, dnssec, elapsed);
            }
        }
        return (Vec::new(), false, elapsed);
    }

    (Vec::new(), false, start.elapsed().as_millis() as u64)
}

fn dns_finding(
    id: &str,
    title: &str,
    severity: Severity,
    desc: &str,
    impact: &str,
    remediation: &str,
    refs: &[&str],
) -> Finding {
    Finding::new(
        id,
        title,
        severity,
        Category::DnsEmailSecurity,
        desc,
        impact,
        remediation,
        "A05:2021-Security Misconfiguration",
    )
    .with_refs(refs)
}

pub async fn audit_dns_and_email_security(client: &Client, domain: &str) -> (DnsSecurityReport, Vec<Finding>) {
    let clean_domain = domain.trim_start_matches("www.").to_lowercase();
    let mut report = DnsSecurityReport {
        domain: clean_domain.clone(),
        spf_record: None,
        spf_valid: false,
        dmarc_record: None,
        dmarc_valid: false,
        dmarc_policy: None,
        dnssec_enabled: false,
        nameservers: Vec::new(),
        latency_ms: None,
        records: Vec::new(),
    };
    let mut findings = Vec::new();

    if clean_domain.is_empty() || clean_domain == "localhost" || clean_domain.parse::<std::net::IpAddr>().is_ok() {
        return (report, findings);
    }

    let dmarc_query_name = format!("_dmarc.{}", clean_domain);

    let doh_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(6))
        .build()
        .unwrap_or_else(|_| client.clone());

    let (a_res, aaaa_res, ns_res, mx_res, txt_res, dmarc_res, soa_res) = tokio::join!(
        query_doh(&doh_client, &clean_domain, "A"),
        query_doh(&doh_client, &clean_domain, "AAAA"),
        query_doh(&doh_client, &clean_domain, "NS"),
        query_doh(&doh_client, &clean_domain, "MX"),
        query_doh(&doh_client, &clean_domain, "TXT"),
        query_doh(&doh_client, &dmarc_query_name, "TXT"),
        query_doh(&doh_client, &clean_domain, "SOA"),
    );

    let (a_answers, a_dnssec, a_lat) = a_res;
    let (aaaa_answers, aaaa_dnssec, aaaa_lat) = aaaa_res;
    let (ns_answers, ns_dnssec, ns_lat) = ns_res;
    let (mx_answers, mx_dnssec, mx_lat) = mx_res;
    let (txt_answers, txt_dnssec, txt_lat) = txt_res;
    let (dmarc_answers, dmarc_dnssec, dmarc_lat) = dmarc_res;
    let (soa_answers, soa_dnssec, soa_lat) = soa_res;

    report.dnssec_enabled = a_dnssec || aaaa_dnssec || ns_dnssec || mx_dnssec || txt_dnssec || dmarc_dnssec || soa_dnssec;

    let mut latencies: Vec<u64> = vec![a_lat, aaaa_lat, ns_lat, mx_lat, txt_lat, dmarc_lat, soa_lat]
        .into_iter()
        .filter(|&l| l > 0)
        .collect();
    if !latencies.is_empty() {
        latencies.sort_unstable();
        report.latency_ms = Some(latencies[latencies.len() / 2]);
    }

    // Nameservers
    for ns in &ns_answers {
        let clean_ns = ns.data.trim().trim_end_matches('.').to_string();
        if !clean_ns.is_empty() && !report.nameservers.contains(&clean_ns) {
            report.nameservers.push(clean_ns);
        }
    }

    // Collect all RFC 1035 records
    let mut all_records = Vec::new();
    let mut root_txts: Vec<String> = Vec::new();
    let mut dmarc_txts: Vec<String> = Vec::new();

    let groups: Vec<(Vec<DohAnswer>, &str)> = vec![
        (a_answers, "A"),
        (aaaa_answers, "AAAA"),
        (ns_answers, "NS"),
        (mx_answers, "MX"),
        (soa_answers, "SOA"),
        (txt_answers, "TXT"),
        (dmarc_answers, "TXT"),
    ];

    for (answers, fallback_type) in groups {
        for ans in answers {
            let clean_data = ans.data.trim().trim_matches('"').replace("\\\"", "\"");
            if clean_data.is_empty() {
                continue;
            }

            let type_str = if ans.record_type > 0 {
                type_num_to_str(ans.record_type).to_string()
            } else {
                fallback_type.to_string()
            };

            let name_str = if ans.name.is_empty() {
                format!("{}.", clean_domain)
            } else {
                ans.name.clone()
            };

            let note = record_note(&type_str, &clean_data).to_string();

            if fallback_type == "TXT" {
                if name_str.starts_with("_dmarc") {
                    dmarc_txts.push(clean_data.clone());
                } else {
                    root_txts.push(clean_data.clone());
                }
            }

            if !all_records.iter().any(|r: &DnsRecord| r.name == name_str && r.record_type == type_str && r.data == clean_data) {
                all_records.push(DnsRecord {
                    name: name_str,
                    ttl: if ans.ttl > 0 { ans.ttl } else { 300 },
                    class: "IN".to_string(),
                    record_type: type_str,
                    data: clean_data,
                    note: Some(note),
                });
            }
        }
    }

    report.records = all_records;

    let spf_record = root_txts
        .into_iter()
        .find(|txt| txt.to_lowercase().starts_with("v=spf1"));

    if let Some(spf) = spf_record {
        report.spf_record = Some(spf.clone());
        let spf_lower = spf.to_lowercase();

        if spf_lower.contains("+all") {
            findings.push(
                dns_finding(
                    "dns-spf-permissive-plus-all",
                    "Insecure SPF Record (+all Directive)",
                    Severity::High,
                    "The SPF record includes the '+all' directive, explicitly permitting ANY mail server in the world to send authorized emails on behalf of this domain.",
                    "Attackers can trivially spoof emails from this domain, leading to high-credibility CEO fraud, business email compromise (BEC), and phishing campaigns.",
                    "Change '+all' to '~all' (SoftFail) or '-all' (HardFail) in your DNS TXT record.",
                    &["https://www.rfc-editor.org/rfc/rfc7208"],
                )
                .with_evidence(spf.clone()),
            );
        } else if spf_lower.contains("?all") {
            findings.push(
                dns_finding(
                    "dns-spf-neutral-all",
                    "Neutral SPF Policy (?all Directive)",
                    Severity::Medium,
                    "The SPF record ends with '?all' (Neutral), meaning receiving servers will treat unauthorized sender IPs as neutral without taking defensive action.",
                    "Provides little to no protection against phishing and email forgery.",
                    "Update SPF directive from '?all' to '-all' (HardFail) or '~all' (SoftFail).",
                    &["https://www.rfc-editor.org/rfc/rfc7208"],
                )
                .with_evidence(spf.clone()),
            );
            report.spf_valid = true;
        } else {
            report.spf_valid = true;
        }
    } else {
        findings.push(dns_finding(
            "dns-missing-spf",
            "Missing SPF (Sender Policy Framework) Record",
            Severity::High,
            "No SPF TXT record was detected on this domain. SPF allows domain owners to publish a list of authorized IP addresses or subnets permitted to send emails.",
            "Threat actors can easily forge email senders using your domain name to conduct phishing and identity impersonation.",
            "Add a DNS TXT record for your domain with a valid SPF policy, e.g., 'v=spf1 include:_spf.google.com ~all'.",
            &[
                "https://www.rfc-editor.org/rfc/rfc7208",
                "https://owasp.org/www-community/attacks/Spamming",
            ],
        ));
    }

    let dmarc_record = dmarc_txts
        .into_iter()
        .find(|txt| txt.to_lowercase().starts_with("v=dmarc1"));

    if let Some(dmarc) = dmarc_record {
        report.dmarc_record = Some(dmarc.clone());
        let dmarc_lower = dmarc.to_lowercase();

        let policy = if dmarc_lower.contains("p=reject") {
            "reject"
        } else if dmarc_lower.contains("p=quarantine") {
            "quarantine"
        } else if dmarc_lower.contains("p=none") {
            "none"
        } else {
            "unknown"
        };
        report.dmarc_policy = Some(policy.to_string());

        if policy == "none" {
            findings.push(
                dns_finding(
                    "dns-dmarc-policy-none",
                    "DMARC Policy Set to 'none' (Monitoring Only)",
                    Severity::Low,
                    "The DMARC record specifies 'p=none', which instructs receiving mail servers to deliver fraudulent or unaligned emails without quarantine or rejection.",
                    "While helpful for initial setup monitoring, 'p=none' provides no active defense against phishing emails spoofing your domain.",
                    "Graduate your DMARC policy from 'p=none' to 'p=quarantine' and ultimately 'p=reject'.",
                    &["https://www.rfc-editor.org/rfc/rfc7489"],
                )
                .with_evidence(dmarc.clone()),
            );
            report.dmarc_valid = true;
        } else {
            report.dmarc_valid = true;
        }
    } else {
        findings.push(dns_finding(
            "dns-missing-dmarc",
            "Missing DMARC Record",
            Severity::High,
            "No DMARC TXT record was found at _dmarc.<domain>. DMARC validates SPF and DKIM alignment to prevent email address spoofing.",
            "Without DMARC enforcement, email providers have no authoritative instructions to reject or quarantine fraudulent emails impersonating this domain.",
            "Create a DNS TXT record at '_dmarc.<domain>' with a policy such as 'v=DMARC1; p=reject; rua=mailto:dmarc-reports@example.com;'.",
            &["https://dmarc.org/", "https://www.rfc-editor.org/rfc/rfc7489"],
        ));
    }

    (report, findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doh_record_cleanup() {
        let raw = "\"v=spf1 include:_spf.google.com ~all\"";
        let clean = raw.trim().trim_matches('"');
        assert_eq!(clean, "v=spf1 include:_spf.google.com ~all");
    }

    #[test]
    fn test_dns_type_num_to_str() {
        assert_eq!(type_num_to_str(1), "A");
        assert_eq!(type_num_to_str(28), "AAAA");
        assert_eq!(type_num_to_str(2), "NS");
        assert_eq!(type_num_to_str(15), "MX");
        assert_eq!(type_num_to_str(16), "TXT");
        assert_eq!(type_num_to_str(6), "SOA");
        assert_eq!(type_num_to_str(999), "OTHER");
    }

    #[test]
    fn test_record_note() {
        assert_eq!(record_note("A", "1.2.3.4"), "IPv4 Host Address");
        assert_eq!(record_note("TXT", "v=spf1 -all"), "SPF Verification Directive");
        assert_eq!(record_note("TXT", "v=DMARC1; p=reject"), "DMARC Conformance Policy");
    }
}

use crate::models::{Category, DnsSecurityReport, Finding, Severity};
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct DohAnswer {
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

async fn query_doh_txt(client: &Client, name: &str) -> (Vec<String>, bool) {
    let url = format!("https://cloudflare-dns.com/dns-query?name={}&type=TXT", name);
    let resp = match client
        .get(&url)
        .header("accept", "application/dns-json")
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return (Vec::new(), false),
    };

    if !resp.status().is_success() {
        return (Vec::new(), false);
    }

    let doh: DohResponse = match resp.json().await {
        Ok(d) => d,
        Err(_) => return (Vec::new(), false),
    };

    let dnssec = doh.ad.unwrap_or(false);
    let records = doh.answer.map(|answers| {
        answers.into_iter().map(|a| a.data.trim().trim_matches('"').replace("\\\"", "\"")).collect()
    }).unwrap_or_default();

    (records, dnssec)
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
    };
    let mut findings = Vec::new();

    if clean_domain.is_empty() || clean_domain == "localhost" || clean_domain.parse::<std::net::IpAddr>().is_ok() {
        return (report, findings);
    }

    // Query SPF and DMARC concurrently
    let dmarc_query_name = format!("_dmarc.{}", clean_domain);
    let (root_res, dmarc_res) = tokio::join!(
        query_doh_txt(client, &clean_domain),
        query_doh_txt(client, &dmarc_query_name)
    );
    let (root_txts, dnssec) = root_res;
    let (dmarc_txts, _) = dmarc_res;
    report.dnssec_enabled = dnssec;

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
    #[test]
    fn test_doh_record_cleanup() {
        let raw = "\"v=spf1 include:_spf.google.com ~all\"";
        let clean = raw.trim().trim_matches('"');
        assert_eq!(clean, "v=spf1 include:_spf.google.com ~all");
    }
}

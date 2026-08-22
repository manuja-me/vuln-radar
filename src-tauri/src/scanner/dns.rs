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
    let mut records = Vec::new();

    if let Some(answers) = doh.answer {
        for a in answers {
            let clean = a.data.trim().trim_matches('"').replace("\\\"", "\"");
            records.push(clean);
        }
    }

    (records, dnssec)
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

    // 1. Query SPF (TXT records on root domain)
    let (root_txts, dnssec) = query_doh_txt(client, &clean_domain).await;
    report.dnssec_enabled = dnssec;

    let spf_record = root_txts
        .into_iter()
        .find(|txt| txt.to_lowercase().starts_with("v=spf1"));

    if let Some(spf) = spf_record {
        report.spf_record = Some(spf.clone());
        let spf_lower = spf.to_lowercase();

        if spf_lower.contains("+all") {
            findings.push(Finding {
                id: "dns-spf-permissive-plus-all".to_string(),
                title: "Insecure SPF Record (+all Directive)".to_string(),
                severity: Severity::High,
                category: Category::DnsEmailSecurity,
                description: "The SPF record includes the '+all' directive, explicitly permitting ANY mail server in the world to send authorized emails on behalf of this domain.".to_string(),
                impact: "Attackers can trivially spoof emails from this domain, leading to high-credibility CEO fraud, business email compromise (BEC), and phishing campaigns.".to_string(),
                remediation: "Change '+all' to '~all' (SoftFail) or '-all' (HardFail) in your DNS TXT record.".to_string(),
                evidence: Some(spf.clone()),
                owasp_category: "A05:2021-Security Misconfiguration".to_string(),
                cve_id: None,
                references: vec!["https://www.rfc-editor.org/rfc/rfc7208".to_string()],
            });
        } else if spf_lower.contains("?all") {
            findings.push(Finding {
                id: "dns-spf-neutral-all".to_string(),
                title: "Neutral SPF Policy (?all Directive)".to_string(),
                severity: Severity::Medium,
                category: Category::DnsEmailSecurity,
                description: "The SPF record ends with '?all' (Neutral), meaning receiving servers will treat unauthorized sender IPs as neutral without taking defensive action.".to_string(),
                impact: "Provides little to no protection against phishing and email forgery.".to_string(),
                remediation: "Update SPF directive from '?all' to '-all' (HardFail) or '~all' (SoftFail).".to_string(),
                evidence: Some(spf.clone()),
                owasp_category: "A05:2021-Security Misconfiguration".to_string(),
                cve_id: None,
                references: vec!["https://www.rfc-editor.org/rfc/rfc7208".to_string()],
            });
            report.spf_valid = true;
        } else {
            report.spf_valid = true;
        }
    } else {
        findings.push(Finding {
            id: "dns-missing-spf".to_string(),
            title: "Missing SPF (Sender Policy Framework) Record".to_string(),
            severity: Severity::High,
            category: Category::DnsEmailSecurity,
            description: "No SPF TXT record was detected on this domain. SPF allows domain owners to publish a list of authorized IP addresses or subnets permitted to send emails.".to_string(),
            impact: "Threat actors can easily forge email senders using your domain name to conduct phishing and identity impersonation.".to_string(),
            remediation: "Add a DNS TXT record for your domain with a valid SPF policy, e.g., 'v=spf1 include:_spf.google.com ~all'.".to_string(),
            evidence: None,
            owasp_category: "A05:2021-Security Misconfiguration".to_string(),
            cve_id: None,
            references: vec![
                "https://www.rfc-editor.org/rfc/rfc7208".to_string(),
                "https://owasp.org/www-community/attacks/Spamming".to_string(),
            ],
        });
    }

    // 2. Query DMARC (TXT records on _dmarc.{domain})
    let dmarc_query_name = format!("_dmarc.{}", clean_domain);
    let (dmarc_txts, _) = query_doh_txt(client, &dmarc_query_name).await;

    let dmarc_record = dmarc_txts
        .into_iter()
        .find(|txt| txt.to_lowercase().starts_with("v=dmarc1"));

    if let Some(dmarc) = dmarc_record {
        report.dmarc_record = Some(dmarc.clone());
        let dmarc_lower = dmarc.to_lowercase();

        // Extract policy p=...
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
            findings.push(Finding {
                id: "dns-dmarc-policy-none".to_string(),
                title: "DMARC Policy Set to 'none' (Monitoring Only)".to_string(),
                severity: Severity::Low,
                category: Category::DnsEmailSecurity,
                description: "The DMARC record specifies 'p=none', which instructs receiving mail servers to deliver fraudulent or unaligned emails without quarantine or rejection.".to_string(),
                impact: "While helpful for initial setup monitoring, 'p=none' provides no active defense against phishing emails spoofing your domain.".to_string(),
                remediation: "Graduate your DMARC policy from 'p=none' to 'p=quarantine' and ultimately 'p=reject'.".to_string(),
                evidence: Some(dmarc.clone()),
                owasp_category: "A05:2021-Security Misconfiguration".to_string(),
                cve_id: None,
                references: vec!["https://www.rfc-editor.org/rfc/rfc7489".to_string()],
            });
            report.dmarc_valid = true;
        } else {
            report.dmarc_valid = true;
        }
    } else {
        findings.push(Finding {
            id: "dns-missing-dmarc".to_string(),
            title: "Missing DMARC Record".to_string(),
            severity: Severity::High,
            category: Category::DnsEmailSecurity,
            description: "No DMARC TXT record was found at _dmarc.<domain>. DMARC validates SPF and DKIM alignment to prevent email address spoofing.".to_string(),
            impact: "Without DMARC enforcement, email providers have no authoritative instructions to reject or quarantine fraudulent emails impersonating this domain.".to_string(),
            remediation: "Create a DNS TXT record at '_dmarc.<domain>' with a policy such as 'v=DMARC1; p=reject; rua=mailto:dmarc-reports@example.com;'.".to_string(),
            evidence: None,
            owasp_category: "A05:2021-Security Misconfiguration".to_string(),
            cve_id: None,
            references: vec!["https://dmarc.org/".to_string(), "https://www.rfc-editor.org/rfc/rfc7489".to_string()],
        });
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

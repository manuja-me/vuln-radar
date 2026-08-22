use crate::models::{Category, Finding, Severity};
use regex::Regex;
use scraper::{Html, Selector};

struct KnownLibRule {
    name: &'static str,
    pattern: &'static str,
    vulnerable_check: fn(major: u32, minor: u32, patch: u32) -> Option<(&'static str, Severity, &'static str, &'static str, &'static str)>,
}

fn parse_semver(ver_str: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = ver_str.split('.').collect();
    if parts.is_empty() {
        return None;
    }
    let major = parts[0].parse::<u32>().ok()?;
    let minor = if parts.len() > 1 { parts[1].parse::<u32>().unwrap_or(0) } else { 0 };
    let patch = if parts.len() > 2 {
        parts[2].chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse::<u32>().unwrap_or(0)
    } else {
        0
    };
    Some((major, minor, patch))
}

pub fn analyze_dependencies(html_content: &str, detected_tech: &mut Vec<String>) -> Vec<Finding> {
    let mut findings = Vec::new();
    let document = Html::parse_document(html_content);
    let script_selector = match Selector::parse("script") {
        Ok(s) => s,
        Err(_) => return findings,
    };

    // Extract script src attributes and inline script contents
    let mut script_sources = Vec::new();
    for element in document.select(&script_selector) {
        if let Some(src) = element.value().attr("src") {
            script_sources.push(src.to_string());
        }
    }

    // Rules for popular libraries and CVEs
    let rules: Vec<KnownLibRule> = vec![
        // jQuery
        KnownLibRule {
            name: "jQuery",
            pattern: r"(?:jquery[.-]|jquery/)([0-9]+\.[0-9]+(?:\.[0-9]+)?)",
            vulnerable_check: |major, minor, patch| {
                if (major == 1 || major == 2) || (major == 3 && (minor < 5 || (minor == 5 && patch == 0))) {
                    Some((
                        "CVE-2020-11022 / CVE-2020-11023",
                        Severity::High,
                        "Passing HTML from untrusted sources to jQuery's DOM manipulation methods (e.g. $.htmlPrefilter) leads to Cross-Site Scripting (XSS).",
                        "Upgrade jQuery to version 3.5.0 or later (currently 3.7.1+ recommended).",
                        "https://nvd.nist.gov/vuln/detail/CVE-2020-11022",
                    ))
                } else if major < 3 {
                    Some((
                        "CVE-2015-9251",
                        Severity::Medium,
                        "3rd-party CORS requests may execute unintended scripts.",
                        "Upgrade jQuery to version 3.5.0+",
                        "https://nvd.nist.gov/vuln/detail/CVE-2015-9251",
                    ))
                } else {
                    None
                }
            },
        },
        // AngularJS 1.x
        KnownLibRule {
            name: "AngularJS",
            pattern: r"(?:angular[.-]|angular/)([0-9]+\.[0-9]+(?:\.[0-9]+)?)",
            vulnerable_check: |major, _minor, _patch| {
                if major == 1 {
                    Some((
                        "CVE-2020-7674 / EOL",
                        Severity::High,
                        "AngularJS 1.x reached End-of-Life (EOL) and contains multiple unpatched client-side template injection and XSS vulnerabilities.",
                        "Migrate to modern Angular (v18+) or a supported framework (React, Vue, Svelte).",
                        "https://blog.angular.io/discontinued-long-term-support-for-angularjs-cc066b82e65a",
                    ))
                } else {
                    None
                }
            },
        },
        // Bootstrap
        KnownLibRule {
            name: "Bootstrap",
            pattern: r"(?:bootstrap[.-]|bootstrap/)([0-9]+\.[0-9]+(?:\.[0-9]+)?)",
            vulnerable_check: |major, minor, patch| {
                if major < 4 || (major == 4 && (minor < 3 || (minor == 3 && patch < 1))) {
                    Some((
                        "CVE-2019-8331 / CVE-2018-14041",
                        Severity::Medium,
                        "Cross-Site Scripting (XSS) in Bootstrap tooltip, popover, and scrollspy components via data-template / data-container attributes.",
                        "Upgrade Bootstrap to 4.3.1 or Bootstrap 5.3+.",
                        "https://nvd.nist.gov/vuln/detail/CVE-2019-8331",
                    ))
                } else {
                    None
                }
            },
        },
        // Lodash
        KnownLibRule {
            name: "Lodash",
            pattern: r"(?:lodash[.-]|lodash/)([0-9]+\.[0-9]+(?:\.[0-9]+)?)",
            vulnerable_check: |major, minor, patch| {
                if major < 4 || (major == 4 && (minor < 17 || (minor == 17 && patch < 21))) {
                    Some((
                        "CVE-2021-23337 / CVE-2020-8203",
                        Severity::High,
                        "Prototype pollution and command injection vulnerabilities via template / zipObjectDeep methods.",
                        "Upgrade Lodash to version 4.17.21 or later.",
                        "https://nvd.nist.gov/vuln/detail/CVE-2021-23337",
                    ))
                } else {
                    None
                }
            },
        },
        // Moment.js
        KnownLibRule {
            name: "Moment.js",
            pattern: r"(?:moment[.-]|moment/)([0-9]+\.[0-9]+(?:\.[0-9]+)?)",
            vulnerable_check: |major, minor, patch| {
                if major < 2 || (major == 2 && (minor < 29 || (minor == 29 && patch < 4))) {
                    Some((
                        "CVE-2022-31129 / CVE-2022-24785",
                        Severity::High,
                        "Path traversal and Regular Expression Denial of Service (ReDoS) when parsing crafted user date inputs.",
                        "Upgrade Moment.js to version 2.29.4 or migrate to Luxon / date-fns.",
                        "https://nvd.nist.gov/vuln/detail/CVE-2022-31129",
                    ))
                } else {
                    None
                }
            },
        },
    ];

    for rule in &rules {
        let re = match Regex::new(rule.pattern) {
            Ok(r) => r,
            Err(_) => continue,
        };

        for src in &script_sources {
            if let Some(caps) = re.captures(src) {
                if let Some(ver_match) = caps.get(1) {
                    let version_str = ver_match.as_str();
                    let tech_entry = format!("{} v{}", rule.name, version_str);
                    if !detected_tech.contains(&tech_entry) {
                        detected_tech.push(tech_entry);
                    }

                    if let Some((major, minor, patch)) = parse_semver(version_str) {
                        if let Some((cve, severity, impact, remediation, ref_url)) = (rule.vulnerable_check)(major, minor, patch) {
                            findings.push(Finding {
                                id: format!("vulnerable-lib-{}-{}", rule.name.to_lowercase(), version_str),
                                title: format!("Outdated & Vulnerable Library: {} v{} ({})", rule.name, version_str, cve),
                                severity,
                                category: Category::VulnerableDependency,
                                description: format!("The application is loading {} version {}, which has known public security vulnerabilities.", rule.name, version_str),
                                impact: impact.to_string(),
                                remediation: remediation.to_string(),
                                evidence: Some(format!("Script source: {}", src)),
                                owasp_category: "A06:2021-Vulnerable and Outdated Components".to_string(),
                                cve_id: Some(cve.to_string()),
                                references: vec![ref_url.to_string()],
                            });
                        }
                    }
                }
            }
        }
    }

    findings
}

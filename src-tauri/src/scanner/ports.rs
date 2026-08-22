use crate::models::{Category, Finding, OpenPort, PortScanReport, Severity};
use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Semaphore;

/// Parse custom port list or range string (e.g., "80, 443, 3000-3005, 8080")
pub fn parse_port_input(input: &str) -> Vec<u16> {
    let mut ports = BTreeSet::new();

    for segment in input.split(',') {
        let trimmed = segment.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some((start_str, end_str)) = trimmed.split_once('-') {
            if let (Ok(start), Ok(end)) = (start_str.trim().parse::<u16>(), end_str.trim().parse::<u16>()) {
                let min_p = start.min(end);
                let max_p = start.max(end);
                for p in min_p..=max_p {
                    if p > 0 {
                        ports.insert(p);
                    }
                }
            }
        } else if let Ok(p) = trimmed.parse::<u16>() {
            if p > 0 {
                ports.insert(p);
            }
        }
    }

    ports.into_iter().collect()
}

pub fn get_preset_ports(profile: &str, custom_input: Option<&str>) -> Vec<u16> {
    match profile.to_lowercase().as_str() {
        "top20" => vec![
            21, 22, 23, 25, 53, 80, 110, 143, 443, 465, 587, 993, 995, 3000, 3306, 3389, 5432,
            6379, 8080, 8443,
        ],
        "databases" => vec![
            1433, 1521, 2375, 2376, 3306, 5432, 6379, 8086, 9200, 9300, 11211, 27017, 27018,
            28017,
        ],
        "top100" => vec![
            20, 21, 22, 23, 25, 53, 67, 68, 69, 80, 110, 111, 123, 135, 137, 138, 139, 143, 161,
            162, 179, 389, 443, 445, 465, 514, 515, 587, 636, 873, 902, 993, 995, 1025, 1080,
            1194, 1433, 1434, 1521, 1723, 2049, 2082, 2083, 2086, 2087, 2181, 2222, 2375, 2376,
            2483, 2484, 3000, 3128, 3306, 3389, 3690, 4000, 4443, 5000, 5001, 5432, 5672, 5900,
            5985, 5986, 6379, 6667, 7000, 7001, 8000, 8008, 8080, 8081, 8086, 8088, 8443, 8888,
            9000, 9090, 9200, 9300, 9418, 9999, 10000, 11211, 27017, 27018, 28017,
        ],
        "custom" => {
            if let Some(custom) = custom_input {
                let parsed = parse_port_input(custom);
                if !parsed.is_empty() {
                    return parsed;
                }
            }
            // fallback if empty
            vec![21, 22, 80, 443, 3000, 3306, 5432, 8080, 8443]
        }
        _ => vec![
            21, 22, 23, 25, 53, 80, 110, 143, 443, 465, 587, 993, 995, 3000, 3306, 3389, 5432,
            6379, 8080, 8443,
        ],
    }
}

pub struct PortMetadata {
    pub service: &'static str,
    pub description: &'static str,
    pub is_risky: bool,
}

pub fn get_port_metadata(port: u16) -> PortMetadata {
    match port {
        20 => PortMetadata {
            service: "FTP-DATA",
            description: "File Transfer Protocol (Data Channel)",
            is_risky: false,
        },
        21 => PortMetadata {
            service: "FTP",
            description: "File Transfer Protocol (Cleartext authentication)",
            is_risky: true,
        },
        22 => PortMetadata {
            service: "SSH",
            description: "Secure Shell Remote Administration",
            is_risky: false,
        },
        23 => PortMetadata {
            service: "Telnet",
            description: "Unencrypted legacy remote terminal access",
            is_risky: true,
        },
        25 => PortMetadata {
            service: "SMTP",
            description: "Simple Mail Transfer Protocol",
            is_risky: false,
        },
        53 => PortMetadata {
            service: "DNS",
            description: "Domain Name System Server",
            is_risky: false,
        },
        67 | 68 => PortMetadata {
            service: "DHCP",
            description: "Dynamic Host Configuration Protocol",
            is_risky: false,
        },
        69 => PortMetadata {
            service: "TFTP",
            description: "Trivial File Transfer Protocol",
            is_risky: true,
        },
        80 => PortMetadata {
            service: "HTTP",
            description: "World Wide Web HTTP Server",
            is_risky: false,
        },
        110 => PortMetadata {
            service: "POP3",
            description: "Post Office Protocol v3 (Cleartext)",
            is_risky: true,
        },
        111 => PortMetadata {
            service: "RPCBind",
            description: "ONC RPC Portmapper",
            is_risky: true,
        },
        123 => PortMetadata {
            service: "NTP",
            description: "Network Time Protocol",
            is_risky: false,
        },
        135 => PortMetadata {
            service: "MSRPC",
            description: "Microsoft Windows RPC Endpoint Mapper",
            is_risky: true,
        },
        137 | 138 => PortMetadata {
            service: "NetBIOS",
            description: "NetBIOS Name & Datagram Service",
            is_risky: true,
        },
        139 => PortMetadata {
            service: "NetBIOS-SSN",
            description: "NetBIOS Session Service (SMB over NetBIOS)",
            is_risky: true,
        },
        143 => PortMetadata {
            service: "IMAP",
            description: "Internet Message Access Protocol (Cleartext)",
            is_risky: true,
        },
        161 | 162 => PortMetadata {
            service: "SNMP",
            description: "Simple Network Management Protocol",
            is_risky: true,
        },
        389 => PortMetadata {
            service: "LDAP",
            description: "Lightweight Directory Access Protocol (Cleartext)",
            is_risky: true,
        },
        443 => PortMetadata {
            service: "HTTPS",
            description: "HTTP over TLS/SSL Secure Web Server",
            is_risky: false,
        },
        445 => PortMetadata {
            service: "SMB",
            description: "Microsoft-DS Active Directory / SMB File Sharing",
            is_risky: true,
        },
        465 => PortMetadata {
            service: "SMTPS",
            description: "Secure SMTP over TLS",
            is_risky: false,
        },
        587 => PortMetadata {
            service: "SMTP-Submission",
            description: "Mail Message Submission Protocol",
            is_risky: false,
        },
        636 => PortMetadata {
            service: "LDAPS",
            description: "Secure LDAP over TLS",
            is_risky: false,
        },
        873 => PortMetadata {
            service: "rsync",
            description: "rsync Remote File Synchronization Daemon",
            is_risky: true,
        },
        993 => PortMetadata {
            service: "IMAPS",
            description: "Secure IMAP over TLS",
            is_risky: false,
        },
        995 => PortMetadata {
            service: "POP3S",
            description: "Secure POP3 over TLS",
            is_risky: false,
        },
        1080 => PortMetadata {
            service: "SOCKS",
            description: "SOCKS Proxy Server",
            is_risky: true,
        },
        1194 => PortMetadata {
            service: "OpenVPN",
            description: "OpenVPN Tunneling Daemon",
            is_risky: false,
        },
        1433 => PortMetadata {
            service: "MSSQL",
            description: "Microsoft SQL Server Database Engine",
            is_risky: true,
        },
        1521 => PortMetadata {
            service: "Oracle",
            description: "Oracle Database Listener",
            is_risky: true,
        },
        2049 => PortMetadata {
            service: "NFS",
            description: "Network File System Daemon",
            is_risky: true,
        },
        2082 | 2083 => PortMetadata {
            service: "cPanel",
            description: "cPanel Web Management Interface",
            is_risky: false,
        },
        2086 | 2087 => PortMetadata {
            service: "WHM",
            description: "WebHost Manager Interface",
            is_risky: true,
        },
        2181 => PortMetadata {
            service: "ZooKeeper",
            description: "Apache ZooKeeper Coordination Service",
            is_risky: true,
        },
        2375 | 2376 => PortMetadata {
            service: "Docker",
            description: "Docker Daemon REST API (Unauthenticated/TLS)",
            is_risky: true,
        },
        3000 => PortMetadata {
            service: "Node.js/Dev",
            description: "Node.js / React / Next.js Development Server",
            is_risky: false,
        },
        3128 => PortMetadata {
            service: "Squid",
            description: "Squid HTTP Proxy Caching Server",
            is_risky: true,
        },
        3306 => PortMetadata {
            service: "MySQL",
            description: "MySQL / MariaDB Relational Database",
            is_risky: true,
        },
        3389 => PortMetadata {
            service: "RDP",
            description: "Microsoft Remote Desktop Protocol",
            is_risky: true,
        },
        5000 => PortMetadata {
            service: "Flask/Dev",
            description: "Python Flask / Docker Registry / Dev Server",
            is_risky: false,
        },
        5432 => PortMetadata {
            service: "PostgreSQL",
            description: "PostgreSQL Relational Database Engine",
            is_risky: true,
        },
        5672 => PortMetadata {
            service: "RabbitMQ",
            description: "RabbitMQ AMQP Message Broker",
            is_risky: true,
        },
        5900..=5905 => PortMetadata {
            service: "VNC",
            description: "Virtual Network Computing Remote Display",
            is_risky: true,
        },
        5985 | 5986 => PortMetadata {
            service: "WinRM",
            description: "Windows Remote Management (HTTP/HTTPS)",
            is_risky: true,
        },
        6379 => PortMetadata {
            service: "Redis",
            description: "Redis In-Memory Key-Value Data Store",
            is_risky: true,
        },
        7000 | 7001 => PortMetadata {
            service: "Cassandra",
            description: "Apache Cassandra Cluster / Storage Service",
            is_risky: true,
        },
        8000 => PortMetadata {
            service: "HTTP-Alt",
            description: "Alternate HTTP / Django / Python Server",
            is_risky: false,
        },
        8080 => PortMetadata {
            service: "HTTP-Proxy",
            description: "HTTP Alternate / Apache Tomcat / Spring Boot",
            is_risky: false,
        },
        8081 => PortMetadata {
            service: "HTTP-Alt2",
            description: "Alternate HTTP Application Server",
            is_risky: false,
        },
        8086 => PortMetadata {
            service: "InfluxDB",
            description: "InfluxDB Time Series Database HTTP API",
            is_risky: true,
        },
        8443 => PortMetadata {
            service: "HTTPS-Alt",
            description: "Alternate HTTPS SSL/TLS Web Service",
            is_risky: false,
        },
        8888 => PortMetadata {
            service: "Jupyter/Admin",
            description: "Jupyter Notebook / Web Administration Console",
            is_risky: false,
        },
        9000 => PortMetadata {
            service: "Portainer/PHP",
            description: "Portainer Docker Management / PHP-FPM / MinIO",
            is_risky: true,
        },
        9090 => PortMetadata {
            service: "Prometheus",
            description: "Prometheus Monitoring Metrics Server",
            is_risky: false,
        },
        9200 | 9300 => PortMetadata {
            service: "Elasticsearch",
            description: "Elasticsearch REST API / Cluster Node Communication",
            is_risky: true,
        },
        11211 => PortMetadata {
            service: "Memcached",
            description: "Memcached In-Memory Distributed Cache",
            is_risky: true,
        },
        27017 | 27018 => PortMetadata {
            service: "MongoDB",
            description: "MongoDB NoSQL Database Server",
            is_risky: true,
        },
        28017 => PortMetadata {
            service: "MongoDB-Web",
            description: "MongoDB Legacy Web Status Interface",
            is_risky: true,
        },
        _ => PortMetadata {
            service: "Unknown",
            description: "Custom or unmapped TCP network service",
            is_risky: false,
        },
    }
}

/// Attempt to grab lightweight service banner from open TCP stream
async fn grab_banner(stream: &mut TcpStream, port: u16) -> Option<String> {
    let mut buf = [0u8; 512];

    // For HTTP/HTTPS/Web ports, send a quick HTTP probe
    if port == 80 || port == 8000 || port == 8080 || port == 8081 || port == 8888 {
        let probe = b"HEAD / HTTP/1.0\r\nUser-Agent: VulnRadar/1.0\r\n\r\n";
        let _ = stream.write_all(probe).await;
    } else if port == 6379 {
        // Redis PING
        let _ = stream.write_all(b"PING\r\n").await;
    }

    // Wait for response banner with short timeout
    let read_result = tokio::time::timeout(Duration::from_millis(350), stream.read(&mut buf)).await;

    match read_result {
        Ok(Ok(n)) if n > 0 => {
            let raw_str = String::from_utf8_lossy(&buf[..n]);
            // Take first line or up to 120 chars, sanitize non-printable
            let first_line = raw_str.lines().next().unwrap_or("").trim();
            let sanitized: String = first_line
                .chars()
                .filter(|c| c.is_ascii_graphic() || *c == ' ')
                .take(120)
                .collect();
            if !sanitized.is_empty() {
                Some(sanitized)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Core port scan engine
pub async fn audit_ports(
    target_host: &str,
    profile: &str,
    custom_ports: Option<&str>,
    timeout_ms: Option<u64>,
) -> (PortScanReport, Vec<Finding>) {
    let start_time = Instant::now();
    let clean_host = target_host
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .split('/')
        .next()
        .unwrap_or(target_host)
        .split(':')
        .next()
        .unwrap_or(target_host)
        .trim()
        .to_string();

    let mut report = PortScanReport {
        host: clean_host.clone(),
        ip_address: None,
        scanned_ports_count: 0,
        open_ports_count: 0,
        open_ports: Vec::new(),
        scan_duration_ms: 0,
    };
    let mut findings = Vec::new();

    if clean_host.is_empty() {
        return (report, findings);
    }

    // 1. Resolve host to IP address
    let socket_addrs: Vec<SocketAddr> = match tokio::net::lookup_host(format!("{}:80", clean_host)).await {
        Ok(iter) => iter.collect(),
        Err(_) => match tokio::net::lookup_host(format!("{}:443", clean_host)).await {
            Ok(iter) => iter.collect(),
            Err(_) => Vec::new(),
        },
    };

    if socket_addrs.is_empty() {
        report.scan_duration_ms = start_time.elapsed().as_millis() as u64;
        return (report, findings);
    }

    let target_ip = socket_addrs[0].ip();
    report.ip_address = Some(target_ip.to_string());

    let ports_to_scan = get_preset_ports(profile, custom_ports);
    report.scanned_ports_count = ports_to_scan.len();

    let probe_timeout = Duration::from_millis(timeout_ms.unwrap_or(800).clamp(200, 5000));
    let semaphore = Arc::new(Semaphore::new(45)); // Concurrency limiter

    let mut tasks = Vec::new();

    for port in ports_to_scan {
        let sem = semaphore.clone();
        let target_sock = SocketAddr::new(target_ip, port);

        tasks.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.ok();
            let connect_fut = TcpStream::connect(target_sock);

            match tokio::time::timeout(probe_timeout, connect_fut).await {
                Ok(Ok(mut stream)) => {
                    let banner = grab_banner(&mut stream, port).await;
                    let meta = get_port_metadata(port);
                    Some(OpenPort {
                        port,
                        protocol: "tcp".to_string(),
                        service: meta.service.to_string(),
                        state: "open".to_string(),
                        banner,
                        is_risky: meta.is_risky,
                        description: meta.description.to_string(),
                    })
                }
                _ => None,
            }
        }));
    }

    let mut open_ports = Vec::new();
    for task in tasks {
        if let Ok(Some(open_p)) = task.await {
            open_ports.push(open_p);
        }
    }

    // Sort by port number
    open_ports.sort_by_key(|p| p.port);
    report.open_ports_count = open_ports.len();

    // 2. Generate Security Findings for risky open ports
    for op in &open_ports {
        match op.port {
            23 => {
                findings.push(Finding {
                    id: "port-telnet-exposed".to_string(),
                    title: "Exposed Telnet Remote Terminal (Port 23)".to_string(),
                    severity: Severity::Critical,
                    category: Category::PortExposure,
                    description: "An unencrypted Telnet service is publicly accessible on port 23. Telnet transmits credentials and commands in cleartext across the network.".to_string(),
                    impact: "Adversaries can intercept administrative login credentials via network sniffing or brute-force remote terminal access.".to_string(),
                    remediation: "Disable the Telnet daemon immediately. Transition all remote administration to SSH (Port 22) with public key authentication.".to_string(),
                    evidence: Some(format!("Port 23/TCP open. Banner: {}", op.banner.as_deref().unwrap_or("N/A"))),
                    owasp_category: "A05:2021-Security Misconfiguration".to_string(),
                    cve_id: None,
                    references: vec![
                        "https://cwe.mitre.org/data/definitions/319.html".to_string(),
                        "https://owasp.org/www-project-web-security-testing-guide/latest/4-Web_Application_Security_Testing/01-Information_Gathering/02-Fingerprint_Web_Server".to_string(),
                    ],
                });
            }
            21 => {
                findings.push(Finding {
                    id: "port-ftp-exposed".to_string(),
                    title: "Exposed Cleartext FTP Service (Port 21)".to_string(),
                    severity: Severity::Medium,
                    category: Category::PortExposure,
                    description: "An unencrypted File Transfer Protocol (FTP) service was discovered open on port 21. Standard FTP sends user credentials in plaintext.".to_string(),
                    impact: "Network eavesdroppers can capture FTP authentication credentials and gain unauthorized file system read/write access.".to_string(),
                    remediation: "Enforce SFTP (over SSH on port 22) or FTPS (FTP over TLS/SSL) and restrict port 21 with firewall rules.".to_string(),
                    evidence: Some(format!("Port 21/TCP open. Banner: {}", op.banner.as_deref().unwrap_or("N/A"))),
                    owasp_category: "A05:2021-Security Misconfiguration".to_string(),
                    cve_id: None,
                    references: vec!["https://cwe.mitre.org/data/definitions/319.html".to_string()],
                });
            }
            135 | 139 | 445 => {
                findings.push(Finding {
                    id: format!("port-smb-netbios-exposed-{}", op.port),
                    title: format!("Exposed Windows SMB/NetBIOS Service (Port {})", op.port),
                    severity: Severity::High,
                    category: Category::PortExposure,
                    description: format!("Port {} ({}) is exposed to the public network. SMB/MSRPC ports are prime vectors for lateral movement, ransomware, and remote exploits.", op.port, op.service),
                    impact: "Attackers can exploit known SMB vulnerabilities (e.g. EternalBlue) or enumerate network shares and domain accounts.".to_string(),
                    remediation: "Block ports 135, 137-139, and 445 at the perimeter edge firewall. Require VPN for internal network share access.".to_string(),
                    evidence: Some(format!("Port {}/TCP open ({})", op.port, op.service)),
                    owasp_category: "A05:2021-Security Misconfiguration".to_string(),
                    cve_id: None,
                    references: vec!["https://www.cisa.gov/news-events/alerts/2017/01/16/risks-associated-smb-and-open-ports".to_string()],
                });
            }
            3389 => {
                findings.push(Finding {
                    id: "port-rdp-exposed".to_string(),
                    title: "Exposed Remote Desktop Protocol (Port 3389)".to_string(),
                    severity: Severity::High,
                    category: Category::PortExposure,
                    description: "Microsoft Remote Desktop Protocol (RDP) is accessible on port 3389 directly over the public internet.".to_string(),
                    impact: "Publicly exposed RDP services are targeted by automated credential stuffing, brute-force bots, and remote code execution vulnerabilities (e.g. BlueKeep).".to_string(),
                    remediation: "Do not expose RDP directly to the internet. Protect desktop access behind an enterprise VPN or Zero-Trust Network Access (ZTNA) with MFA.".to_string(),
                    evidence: Some(format!("Port 3389/TCP open. Host: {}", clean_host)),
                    owasp_category: "A05:2021-Security Misconfiguration".to_string(),
                    cve_id: None,
                    references: vec!["https://www.cisa.gov/news-events/analysis-reports/ar19-133a".to_string()],
                });
            }
            5900..=5905 => {
                findings.push(Finding {
                    id: format!("port-vnc-exposed-{}", op.port),
                    title: format!("Exposed VNC Remote Desktop (Port {})", op.port),
                    severity: Severity::High,
                    category: Category::PortExposure,
                    description: format!("Virtual Network Computing (VNC) is exposed on port {}. Many VNC servers lack strong brute-force protections or TLS encryption.", op.port),
                    impact: "Allows unauthorized remote GUI desktop access if weak or default passwords are configured.".to_string(),
                    remediation: "Tunnel VNC sessions over SSH or VPN, or disable VNC if not strictly needed.".to_string(),
                    evidence: Some(format!("Port {}/TCP open ({})", op.port, op.service)),
                    owasp_category: "A05:2021-Security Misconfiguration".to_string(),
                    cve_id: None,
                    references: vec!["https://owasp.org/".to_string()],
                });
            }
            3306 | 5432 | 1433 | 1521 | 6379 | 27017 | 9200 | 11211 => {
                findings.push(Finding {
                    id: format!("port-database-exposed-{}", op.port),
                    title: format!("Exposed Database Engine ({}, Port {})", op.service, op.port),
                    severity: Severity::High,
                    category: Category::PortExposure,
                    description: format!(
                        "The {} database service is listening and accessible on port {}. Database ports should never be exposed to the public internet.",
                        op.service, op.port
                    ),
                    impact: "Attackers can perform automated credential brute-forcing, exploit unauthenticated configurations (e.g., Redis/MongoDB/Elasticsearch default setups), or exfiltrate sensitive data.".to_string(),
                    remediation: format!(
                        "Bind {} to localhost (127.0.0.1) or private VPC subnets. Enforce firewall rules restricting inbound traffic on port {}.",
                        op.service, op.port
                    ),
                    evidence: Some(format!("Port {}/TCP open. Service: {}. Banner: {}", op.port, op.service, op.banner.as_deref().unwrap_or("N/A"))),
                    owasp_category: "A05:2021-Security Misconfiguration".to_string(),
                    cve_id: None,
                    references: vec![
                        "https://owasp.org/www-project-top-ten/2017/A6_2017-Security_Misconfiguration".to_string(),
                    ],
                });
            }
            2375 | 2376 => {
                findings.push(Finding {
                    id: format!("port-docker-exposed-{}", op.port),
                    title: format!("Exposed Docker Daemon API (Port {})", op.port),
                    severity: Severity::Critical,
                    category: Category::PortExposure,
                    description: format!("Docker Daemon API port {} was found open. An exposed unauthenticated Docker API allows full root-level container creation and host takeover.", op.port),
                    impact: "Attackers can run privileged containers with mounted host root filesystems, achieving complete remote server takeover.".to_string(),
                    remediation: "Never expose Docker socket or API over public TCP. Use Unix socket or require mutual TLS client certificate authentication.".to_string(),
                    evidence: Some(format!("Port {}/TCP open. Service: Docker", op.port)),
                    owasp_category: "A05:2021-Security Misconfiguration".to_string(),
                    cve_id: None,
                    references: vec!["https://docs.docker.com/engine/security/protect-access/".to_string()],
                });
            }
            _ => {}
        }
    }

    report.open_ports = open_ports;
    report.scan_duration_ms = start_time.elapsed().as_millis() as u64;

    (report, findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_port_input_single_and_ranges() {
        let input = "80, 443, 3000-3003, 8080";
        let ports = parse_port_input(input);
        assert_eq!(ports, vec![80, 443, 3000, 3001, 3002, 3003, 8080]);
    }

    #[test]
    fn test_preset_ports() {
        let top20 = get_preset_ports("top20", None);
        assert_eq!(top20.len(), 20);
        assert!(top20.contains(&80));
        assert!(top20.contains(&443));
        assert!(top20.contains(&22));

        let db = get_preset_ports("databases", None);
        assert!(db.contains(&3306));
        assert!(db.contains(&5432));
        assert!(db.contains(&6379));

        let custom = get_preset_ports("custom", Some("2222, 8443, 9000-9002"));
        assert_eq!(custom, vec![2222, 8443, 9000, 9001, 9002]);
    }

    #[test]
    fn test_port_metadata() {
        let meta_ssh = get_port_metadata(22);
        assert_eq!(meta_ssh.service, "SSH");
        assert!(!meta_ssh.is_risky);

        let meta_telnet = get_port_metadata(23);
        assert_eq!(meta_telnet.service, "Telnet");
        assert!(meta_telnet.is_risky);

        let meta_redis = get_port_metadata(6379);
        assert_eq!(meta_redis.service, "Redis");
        assert!(meta_redis.is_risky);
    }
}

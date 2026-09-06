use crate::models::{Category, Finding, OpenPort, PortScanReport, Severity};
use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::task::JoinSet;

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
                        if ports.len() >= 1000 {
                            break;
                        }
                    }
                }
            }
        } else if let Ok(p) = trimmed.parse::<u16>() {
            if p > 0 {
                ports.insert(p);
            }
        }
        if ports.len() >= 1000 {
            break;
        }
    }

    ports.into_iter().take(1000).collect()
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
    let (service, description, is_risky) = match port {
        20 => ("FTP-DATA", "File Transfer Protocol (Data Channel)", false),
        21 => ("FTP", "File Transfer Protocol (Cleartext authentication)", true),
        22 => ("SSH", "Secure Shell Remote Administration", false),
        23 => ("Telnet", "Unencrypted legacy remote terminal access", true),
        25 => ("SMTP", "Simple Mail Transfer Protocol", false),
        53 => ("DNS", "Domain Name System Server", false),
        67 | 68 => ("DHCP", "Dynamic Host Configuration Protocol", false),
        69 => ("TFTP", "Trivial File Transfer Protocol", true),
        80 => ("HTTP", "World Wide Web HTTP Server", false),
        110 => ("POP3", "Post Office Protocol v3 (Cleartext)", true),
        111 => ("RPCBind", "ONC RPC Portmapper", true),
        123 => ("NTP", "Network Time Protocol", false),
        135 => ("MSRPC", "Microsoft Windows RPC Endpoint Mapper", true),
        137 | 138 => ("NetBIOS", "NetBIOS Name & Datagram Service", true),
        139 => ("NetBIOS-SSN", "NetBIOS Session Service (SMB over NetBIOS)", true),
        143 => ("IMAP", "Internet Message Access Protocol (Cleartext)", true),
        161 | 162 => ("SNMP", "Simple Network Management Protocol", true),
        389 => ("LDAP", "Lightweight Directory Access Protocol (Cleartext)", true),
        443 => ("HTTPS", "HTTP over TLS/SSL Secure Web Server", false),
        445 => ("SMB", "Microsoft-DS Active Directory / SMB File Sharing", true),
        465 => ("SMTPS", "Secure SMTP over TLS", false),
        587 => ("SMTP-Submission", "Mail Message Submission Protocol", false),
        636 => ("LDAPS", "Secure LDAP over TLS", false),
        873 => ("rsync", "rsync Remote File Synchronization Daemon", true),
        993 => ("IMAPS", "Secure IMAP over TLS", false),
        995 => ("POP3S", "Secure POP3 over TLS", false),
        1080 => ("SOCKS", "SOCKS Proxy Server", true),
        1194 => ("OpenVPN", "OpenVPN Tunneling Daemon", false),
        1433 => ("MSSQL", "Microsoft SQL Server Database Engine", true),
        1521 => ("Oracle", "Oracle Database Listener", true),
        2049 => ("NFS", "Network File System Daemon", true),
        2082 | 2083 => ("cPanel", "cPanel Web Management Interface", false),
        2086 | 2087 => ("WHM", "WebHost Manager Interface", true),
        2181 => ("ZooKeeper", "Apache ZooKeeper Coordination Service", true),
        2375 | 2376 => ("Docker", "Docker Daemon REST API (Unauthenticated/TLS)", true),
        3000 => ("Node.js/Dev", "Node.js / React / Next.js Development Server", false),
        3128 => ("Squid", "Squid HTTP Proxy Caching Server", true),
        3306 => ("MySQL", "MySQL / MariaDB Relational Database", true),
        3389 => ("RDP", "Microsoft Remote Desktop Protocol", true),
        5000 => ("Flask/Dev", "Python Flask / Docker Registry / Dev Server", false),
        5432 => ("PostgreSQL", "PostgreSQL Relational Database Engine", true),
        5672 => ("RabbitMQ", "RabbitMQ AMQP Message Broker", true),
        5900..=5905 => ("VNC", "Virtual Network Computing Remote Display", true),
        5985 | 5986 => ("WinRM", "Windows Remote Management (HTTP/HTTPS)", true),
        6379 => ("Redis", "Redis In-Memory Key-Value Data Store", true),
        7000 | 7001 => ("Cassandra", "Apache Cassandra Cluster / Storage Service", true),
        8000 => ("HTTP-Alt", "Alternate HTTP / Django / Python Server", false),
        8080 => ("HTTP-Proxy", "HTTP Alternate / Apache Tomcat / Spring Boot", false),
        8081 => ("HTTP-Alt2", "Alternate HTTP Application Server", false),
        8086 => ("InfluxDB", "InfluxDB Time Series Database HTTP API", true),
        8443 => ("HTTPS-Alt", "Alternate HTTPS SSL/TLS Web Service", false),
        8888 => ("Jupyter/Admin", "Jupyter Notebook / Web Administration Console", false),
        9000 => ("Portainer/PHP", "Portainer Docker Management / PHP-FPM / MinIO", true),
        9090 => ("Prometheus", "Prometheus Monitoring Metrics Server", false),
        9200 | 9300 => ("Elasticsearch", "Elasticsearch REST API / Cluster Node Communication", true),
        11211 => ("Memcached", "Memcached In-Memory Distributed Cache", true),
        27017 | 27018 => ("MongoDB", "MongoDB NoSQL Database Server", true),
        28017 => ("MongoDB-Web", "MongoDB Legacy Web Status Interface", true),
        _ => ("Unknown", "Custom or unmapped TCP network service", false),
    };
    PortMetadata { service, description, is_risky }
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
    const MAX_CONCURRENT_PORT_TASKS: usize = 45;
    let mut set = JoinSet::new();
    let mut open_ports = Vec::new();

    for port in ports_to_scan {
        // Enforce maximum concurrent active tasks in memory
        while set.len() >= MAX_CONCURRENT_PORT_TASKS {
            if let Some(res) = set.join_next().await {
                if let Ok(Some(open_p)) = res {
                    open_ports.push(open_p);
                }
            }
        }

        let target_sock = SocketAddr::new(target_ip, port);
        set.spawn(async move {
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
        });
    }

    // Drain remaining active tasks
    while let Some(res) = set.join_next().await {
        if let Ok(Some(open_p)) = res {
            open_ports.push(open_p);
        }
    }

    // Sort by port number
    open_ports.sort_by_key(|p| p.port);
    report.open_ports_count = open_ports.len();

    // 2. Generate Security Findings for risky open ports
    for op in &open_ports {
        match op.port {
            23 => findings.push(
                Finding::new(
                    "port-telnet-exposed",
                    "Exposed Telnet Remote Terminal (Port 23)",
                    Severity::Critical,
                    Category::PortExposure,
                    "An unencrypted Telnet service is publicly accessible on port 23. Telnet transmits credentials and commands in cleartext across the network.",
                    "Adversaries can intercept administrative login credentials via network sniffing or brute-force remote terminal access.",
                    "Disable the Telnet daemon immediately. Transition all remote administration to SSH (Port 22) with public key authentication.",
                    "A05:2021-Security Misconfiguration",
                )
                .with_evidence(format!("Port 23/TCP open. Banner: {}", op.banner.as_deref().unwrap_or("N/A")))
                .with_refs(&[
                    "https://cwe.mitre.org/data/definitions/319.html",
                    "https://owasp.org/www-project-web-security-testing-guide/latest/4-Web_Application_Security_Testing/01-Information_Gathering/02-Fingerprint_Web_Server",
                ]),
            ),
            21 => findings.push(
                Finding::new(
                    "port-ftp-exposed",
                    "Exposed Cleartext FTP Service (Port 21)",
                    Severity::Medium,
                    Category::PortExposure,
                    "An unencrypted File Transfer Protocol (FTP) service was discovered open on port 21. Standard FTP sends user credentials in plaintext.",
                    "Network eavesdroppers can capture FTP authentication credentials and gain unauthorized file system read/write access.",
                    "Enforce SFTP (over SSH on port 22) or FTPS (FTP over TLS/SSL) and restrict port 21 with firewall rules.",
                    "A05:2021-Security Misconfiguration",
                )
                .with_evidence(format!("Port 21/TCP open. Banner: {}", op.banner.as_deref().unwrap_or("N/A")))
                .with_refs(&["https://cwe.mitre.org/data/definitions/319.html"]),
            ),
            135 | 139 | 445 => findings.push(
                Finding::new(
                    format!("port-smb-netbios-exposed-{}", op.port),
                    format!("Exposed Windows SMB/NetBIOS Service (Port {})", op.port),
                    Severity::High,
                    Category::PortExposure,
                    format!("Port {} ({}) is exposed to the public network. SMB/MSRPC ports are prime vectors for lateral movement, ransomware, and remote exploits.", op.port, op.service),
                    "Attackers can exploit known SMB vulnerabilities (e.g. EternalBlue) or enumerate network shares and domain accounts.",
                    "Block ports 135, 137-139, and 445 at the perimeter edge firewall. Require VPN for internal network share access.",
                    "A05:2021-Security Misconfiguration",
                )
                .with_evidence(format!("Port {}/TCP open ({})", op.port, op.service))
                .with_refs(&["https://www.cisa.gov/news-events/alerts/2017/01/16/risks-associated-smb-and-open-ports"]),
            ),
            3389 => findings.push(
                Finding::new(
                    "port-rdp-exposed",
                    "Exposed Remote Desktop Protocol (Port 3389)",
                    Severity::High,
                    Category::PortExposure,
                    "Microsoft Remote Desktop Protocol (RDP) is accessible on port 3389 directly over the public internet.",
                    "Publicly exposed RDP services are targeted by automated credential stuffing, brute-force bots, and remote code execution vulnerabilities (e.g. BlueKeep).",
                    "Do not expose RDP directly to the internet. Protect desktop access behind an enterprise VPN or Zero-Trust Network Access (ZTNA) with MFA.",
                    "A05:2021-Security Misconfiguration",
                )
                .with_evidence(format!("Port 3389/TCP open. Host: {}", clean_host))
                .with_refs(&["https://www.cisa.gov/news-events/analysis-reports/ar19-133a"]),
            ),
            5900..=5905 => findings.push(
                Finding::new(
                    format!("port-vnc-exposed-{}", op.port),
                    format!("Exposed VNC Remote Desktop (Port {})", op.port),
                    Severity::High,
                    Category::PortExposure,
                    format!("Virtual Network Computing (VNC) is exposed on port {}. Many VNC servers lack strong brute-force protections or TLS encryption.", op.port),
                    "Allows unauthorized remote GUI desktop access if weak or default passwords are configured.",
                    "Tunnel VNC sessions over SSH or VPN, or disable VNC if not strictly needed.",
                    "A05:2021-Security Misconfiguration",
                )
                .with_evidence(format!("Port {}/TCP open ({})", op.port, op.service))
                .with_refs(&["https://owasp.org/"]),
            ),
            3306 | 5432 | 1433 | 1521 | 6379 | 27017 | 9200 | 11211 => findings.push(
                Finding::new(
                    format!("port-database-exposed-{}", op.port),
                    format!("Exposed Database Engine ({}, Port {})", op.service, op.port),
                    Severity::High,
                    Category::PortExposure,
                    format!(
                        "The {} database service is listening and accessible on port {}. Database ports should never be exposed to the public internet.",
                        op.service, op.port
                    ),
                    "Attackers can perform automated credential brute-forcing, exploit unauthenticated configurations (e.g., Redis/MongoDB/Elasticsearch default setups), or exfiltrate sensitive data.",
                    format!(
                        "Bind {} to localhost (127.0.0.1) or private VPC subnets. Enforce firewall rules restricting inbound traffic on port {}.",
                        op.service, op.port
                    ),
                    "A05:2021-Security Misconfiguration",
                )
                .with_evidence(format!("Port {}/TCP open. Service: {}. Banner: {}", op.port, op.service, op.banner.as_deref().unwrap_or("N/A")))
                .with_refs(&["https://owasp.org/www-project-top-ten/2017/A6_2017-Security_Misconfiguration"]),
            ),
            2375 | 2376 => findings.push(
                Finding::new(
                    format!("port-docker-exposed-{}", op.port),
                    format!("Exposed Docker Daemon API (Port {})", op.port),
                    Severity::Critical,
                    Category::PortExposure,
                    format!("Docker Daemon API port {} was found open. An exposed unauthenticated Docker API allows full root-level container creation and host takeover.", op.port),
                    "Attackers can run privileged containers with mounted host root filesystems, achieving complete remote server takeover.",
                    "Never expose Docker socket or API over public TCP. Use Unix socket or require mutual TLS client certificate authentication.",
                    "A05:2021-Security Misconfiguration",
                )
                .with_evidence(format!("Port {}/TCP open. Service: Docker", op.port))
                .with_refs(&["https://docs.docker.com/engine/security/protect-access/"]),
            ),
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

//! End-to-end process tests for startup TCP port binding and banner ordering.
//!
//! Verifies that `daypress` never prints the `Listening` banner before the TCP
//! listener is bound, and exits with a clear error naming the port and `--port`
//! when the requested port is already in use.

use std::io::{BufRead, BufReader};
use std::net::{SocketAddr, TcpListener};
use std::process::{Command, Stdio};

#[test]
fn second_process_on_occupied_port_exits_without_printing_listening_banner() {
    let occupied =
        TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))).expect("bind ephemeral port");
    let port = occupied.local_addr().expect("local addr").port();

    let temp_dir =
        std::env::temp_dir().join(format!("daypress_bind_conflict_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).expect("create temp dir");
    let db_path = temp_dir.join("daypress.sqlite");

    let output = Command::new(env!("CARGO_BIN_EXE_daypress"))
        .arg("--port")
        .arg(port.to_string())
        .arg("--db")
        .arg(&db_path)
        .env("DAYPRESS_DISABLE_UPDATE_CHECKER", "1")
        .env("DAYPRESS_NO_BROWSER", "1")
        .output()
        .expect("run daypress binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "daypress should exit with non-zero status when port {port} is in use; stdout={stdout}, stderr={stderr}"
    );
    assert!(
        !stdout.contains("Listening"),
        "daypress must not print 'Listening' before binding succeeds; stdout={stdout}"
    );
    assert!(
        stderr.contains(&port.to_string()),
        "stderr should identify the busy port {port}; stderr={stderr}"
    );
    assert!(
        stderr.contains("--port"),
        "stderr should suggest choosing another port with `--port`; stderr={stderr}"
    );

    drop(occupied);
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn startup_prints_listening_banner_after_binding_ephemeral_port() {
    let temp_dir = std::env::temp_dir().join(format!("daypress_bind_ok_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).expect("create temp dir");
    let db_path = temp_dir.join("daypress.sqlite");

    let mut child = Command::new(env!("CARGO_BIN_EXE_daypress"))
        .arg("--port")
        .arg("0")
        .arg("--db")
        .arg(&db_path)
        .env("DAYPRESS_DISABLE_UPDATE_CHECKER", "1")
        .env("DAYPRESS_NO_BROWSER", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn daypress binary");

    let stdout = child.stdout.take().expect("child stdout");
    let reader = BufReader::new(stdout);
    let mut bound_port: Option<u16> = None;

    for line in reader.lines().take(25) {
        let Ok(line) = line else { break };
        if let Some(rest) = line.split("Listening on http://localhost:").nth(1) {
            let port_str = rest.trim_end_matches('/');
            bound_port = port_str.parse::<u16>().ok();
        }
        if line.contains("[SERVER] TcpListener bound to") {
            break;
        }
    }

    let port = bound_port.expect("banner should report actual bound port");
    assert_ne!(port, 0, "ephemeral port 0 should resolve to actual OS port");

    // Because the banner is printed after bind, a TCP connection to the reported port must succeed immediately.
    let stream = std::net::TcpStream::connect_timeout(
        &SocketAddr::from(([127, 0, 0, 1], port)),
        std::time::Duration::from_secs(2),
    );
    assert!(
        stream.is_ok(),
        "port {port} reported in Listening banner must already be bound and accepting connections: {stream:?}"
    );

    let _ = child.kill();
    let _ = child.wait();
    let _ = std::fs::remove_dir_all(&temp_dir);
}

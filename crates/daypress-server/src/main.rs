//! # Composition Root (`main.rs`)
//!
//! Initializes concrete infrastructure adapters (`SqliteDatabase`), wires them into
//! the pure application orchestrator (`CmsService`), and drives either the Axum HTTP server
//! or the built-in Model Context Protocol (`--mcp-stdio`) server for Claude / AI tools.

use clap::{Parser, Subcommand};
use daypress_server::{
    adapters::mcp::run_stdio_mcp_loop, create_app, legacy, password_reset, AppError, CmsService,
    Database, SqliteDatabase,
};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(name = "daypress")]
#[command(about = "Daypress: Self-Hosted Rust CMS with 1-Click WP Migration & Built-in MCP Server")]
pub struct Cli {
    /// Port to listen on (overridable via PORT env var)
    #[arg(short, long, env = "PORT", default_value_t = 8090)]
    pub port: u16,

    /// Host IP to bind (defaults to 127.0.0.1 on desktop to avoid OS firewall popups, or 0.0.0.0 when LAN mode / Docker is active)
    #[arg(long, env = "HOST")]
    pub host: Option<String>,

    /// Path to SQLite database file (overridable via DATABASE_PATH env var)
    #[arg(
        long,
        env = "DATABASE_PATH",
        default_value = "data/daypress.sqlite",
        global = true
    )]
    pub db: PathBuf,

    /// Run as a stdio Model Context Protocol (MCP) server for Claude Desktop / Claude Code
    #[arg(long, default_value_t = false)]
    pub mcp_stdio: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Set a new password for an account, e.g. when you are locked out of /d-admin.
    ///
    /// Takes the password from DAYPRESS_NEW_PASSWORD when it is set; otherwise generates
    /// one and prints it once on stdout. Works on an existing database only.
    ResetPassword {
        /// Login name or email address of the account
        username: String,

        /// Also sign out every user by replacing the session signing key
        #[arg(long)]
        sign_out_everyone: bool,
    },
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    // stderr, so `--mcp-stdio` keeps a clean JSON-RPC stdout.
    match legacy::adopt_legacy_database_file(&args.db) {
        Ok(Some(old)) => eprintln!(
            "✓ Neopress is now Daypress: moved {} to {}",
            old.display(),
            args.db.display()
        ),
        Ok(None) => {}
        Err(err) => {
            eprintln!(
                "Could not move the Neopress database to {}: {err}",
                args.db.display()
            );
            std::process::exit(1);
        }
    }

    if let Some(Command::ResetPassword {
        username,
        sign_out_everyone,
    }) = &args.command
    {
        std::process::exit(reset_password(&args.db, username, *sign_out_everyone));
    }

    if args.mcp_stdio {
        if let Err(err) = run_mcp_stdio(&args) {
            eprintln!("MCP STDIO ERROR: {err:?}");
            std::process::exit(1);
        }
        return;
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "daypress_server=info,tower_http=info".into()),
        )
        .init();

    if let Err(err) = run(args).await {
        eprintln!("CRITICAL SERVER ERROR: {err}");
        std::process::exit(1);
    }
}

/// `daypress reset-password`; returns the process exit code.
fn reset_password(db_path: &Path, username: &str, sign_out_everyone: bool) -> i32 {
    let from_env = legacy::env_var(password_reset::NEW_PASSWORD_ENV).filter(|p| !p.is_empty());
    let result = password_reset::reset_password_from_cli(
        db_path,
        username,
        from_env.as_deref(),
        sign_out_everyone,
    );
    let report = match result {
        Ok(report) => report,
        Err(AppError::NotFound(message) | AppError::BadRequest(message)) => {
            eprintln!("reset-password: {message}");
            return 1;
        }
        Err(err) => {
            eprintln!("reset-password failed: {err}");
            return 1;
        }
    };
    println!("Password for '{}' has been reset.", report.login);
    match &report.generated_password {
        Some(password) => println!("New password (shown only once): {password}"),
        None => println!(
            "New password: the value of {}.",
            password_reset::NEW_PASSWORD_ENV
        ),
    }
    println!(
        "Cancelled {} unused password reset link(s).",
        report.cancelled_reset_links
    );
    if sign_out_everyone {
        println!("Every user has been signed out.");
    } else {
        println!("Existing login sessions stay valid until they expire (at most 14 days).");
        println!("To end them too, run this again with --sign-out-everyone (it sets a new password again).");
    }
    0
}

fn run_mcp_stdio(args: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SqliteDatabase::new(&args.db)?;
    let repo = Arc::new(adapter);
    let cms_service = CmsService::new(repo.clone(), repo.clone(), repo.clone(), repo.clone(), repo);
    run_stdio_mcp_loop(&cms_service)?;
    Ok(())
}

fn format_bind_error(port: u16, err: &std::io::Error) -> String {
    if err.kind() == std::io::ErrorKind::AddrInUse {
        format!(
            "Port {port} is already in use ({err}). Stop the process using port {port} or choose another port with `--port <PORT>`."
        )
    } else {
        format!(
            "Failed to bind TCP listener on port {port} (0.0.0.0:{port}): {err}. Try choosing another port with `--port <PORT>`."
        )
    }
}

async fn bind_http_listener(addr: SocketAddr) -> Result<tokio::net::TcpListener, String> {
    // Only retry binding when spawned by an in-app self-update handoff while the old process exits.
    let restart_handoff = std::env::var("DAYPRESS_RESTART_HANDOFF")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let max_retries = if restart_handoff { 12u8 } else { 0u8 };
    let mut bind_attempt = 0u8;
    loop {
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => return Ok(listener),
            Err(_) if bind_attempt < max_retries => {
                bind_attempt += 1;
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            }
            Err(e) => return Err(format_bind_error(addr.port(), &e)),
        }
    }
}

fn print_startup_banner(
    db_path: &Path,
    bound_port: u16,
    bound_addr: SocketAddr,
    needs_onboarding: bool,
) {
    println!("============================================================");
    println!("           🦀 Daypress: The All-in-One Rust CMS             ");
    println!("============================================================");
    println!("✓ Architecture:       Hexagonal (Domain / Ports / Application / Adapters)");
    println!("✓ License:            AGPLv3 + Commercial Dual-License");
    println!("✓ Engine:             Native Rust (Axum + Tokio)");
    println!("✓ Storage Adapter:    Embedded Rusqlite (Zero SQL Server Required)");
    println!("✓ Database Path:      {}", db_path.display());
    println!(
        "✓ Status:             Listening on http://localhost:{}/",
        bound_port
    );
    println!("------------------------------------------------------------");
    if needs_onboarding {
        println!(
            "✨ First-Time Setup:  http://localhost:{}/setup",
            bound_port
        );
    }
    println!("🌐 Public Website:    http://localhost:{}/", bound_port);
    println!(
        "⚙️ Admin Dashboard:   http://localhost:{}/d-admin/",
        bound_port
    );
    println!(
        "📡 REST API Root:     http://localhost:{}/wp-json/",
        bound_port
    );
    println!(
        "🤖 Claude MCP RPC:    http://localhost:{}/mcp (or `daypress --mcp-stdio`)",
        bound_port
    );
    println!("============================================================");
    println!("[SERVER] TcpListener bound to {bound_addr}");
}

fn maybe_open_browser_on_first_run(port: u16) {
    if std::env::var_os("CI").is_some() || std::env::var_os("DAYPRESS_NO_BROWSER").is_some() {
        return;
    }
    let url = format!("http://localhost:{port}/setup");
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", &url])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(&url)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(&url)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
}

async fn run(args: Cli) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize Concrete Storage Adapter first so we know onboarding state and saved network bind preference.
    let database = Database::new(&args.db)?;
    let needs_onboarding = database.needs_onboarding();

    // Resolve bind address:
    // - Explicit `--host` / `HOST` env var wins.
    // - Inside Docker (`/.dockerenv`) or when the user enabled LAN mode (`daypress_network_bind_mode = "lan"`), bind `0.0.0.0`.
    // - Otherwise default to `127.0.0.1` (loopback) so Windows/macOS never show unexpected Public/Private network firewall popups on launch.
    let bind_ip: std::net::IpAddr = if let Some(ref h) = args.host {
        h.trim()
            .parse()
            .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST))
    } else if Path::new("/.dockerenv").exists()
        || std::env::var_os("KUBERNETES_SERVICE_HOST").is_some()
    {
        std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)
    } else {
        let saved_mode = database
            .get_option("daypress_network_bind_mode")
            .ok()
            .flatten()
            .unwrap_or_else(|| "localhost".to_string());
        if saved_mode.eq_ignore_ascii_case("lan") || saved_mode == "0.0.0.0" {
            std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)
        } else {
            std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
        }
    };

    // 2. Bind the TCP port before printing the listening banner
    let addr = SocketAddr::new(bind_ip, args.port);
    let listener = bind_http_listener(addr).await?;
    let bound_addr = listener.local_addr().unwrap_or(addr);
    let bound_port = bound_addr.port();

    // 3. Wire Output Ports into Application Service via Arc<dyn Trait>
    let repo = Arc::new(database.clone());
    let _cms_service =
        CmsService::new(repo.clone(), repo.clone(), repo.clone(), repo.clone(), repo);

    // 4. Spawn non-blocking 24-hour release update checker (persisted in SQLite)
    daypress_server::updater::spawn_background_update_checker(database.clone());

    // 5. Hand wired state to Primary Web & MCP Adapter
    let app = create_app(database);

    // 6. Print listening banner only after TCP listener and application state are ready
    print_startup_banner(&args.db, bound_port, bound_addr, needs_onboarding);
    if needs_onboarding {
        maybe_open_browser_on_first_run(bound_port);
    }

    // Connect info gives handlers the client socket address (used by the login throttle).
    let res = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await;

    match res {
        Ok(()) => {
            println!("[SERVER] Axum serve completed successfully");
            Ok(())
        }
        Err(e) => {
            eprintln!("[SERVER] Axum serve error: {e:?}");
            Err(e.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_bind_error_identifies_port_and_suggests_port_flag() {
        let err = std::io::Error::new(std::io::ErrorKind::AddrInUse, "address in use");
        let msg = format_bind_error(8090, &err);
        assert!(msg.contains("8090"), "expected port in error: {msg}");
        assert!(
            msg.contains("--port"),
            "expected --port hint in error: {msg}"
        );
        assert!(
            msg.contains("already in use"),
            "expected 'already in use' in error: {msg}"
        );
    }

    #[tokio::test]
    async fn bind_http_listener_fails_with_clear_message_when_port_is_in_use() {
        let occupied = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
            .await
            .expect("bind ephemeral port");
        let port = occupied.local_addr().expect("local addr").port();

        let err = bind_http_listener(SocketAddr::from(([127, 0, 0, 1], port)))
            .await
            .expect_err("second bind on same port must fail");
        assert!(
            err.contains(&port.to_string()),
            "error must name port {port}: {err}"
        );
        assert!(err.contains("--port"), "error must suggest --port: {err}");
    }
}

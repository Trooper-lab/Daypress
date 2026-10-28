//! # Composition Root (`main.rs`)
//!
//! Initializes concrete infrastructure adapters (`SqliteDatabase`), wires them into
//! the pure application orchestrator (`CmsService`), and drives either the Axum HTTP server
//! or the built-in Model Context Protocol (`--mcp-stdio`) server for Claude / AI tools.

use clap::{Parser, Subcommand};
use neopress_server::{
    adapters::mcp::run_stdio_mcp_loop, create_app, password_reset, AppError, CmsService, Database,
    SqliteDatabase,
};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(name = "neopress")]
#[command(about = "Neopress: Self-Hosted Rust CMS with 1-Click WP Migration & Built-in MCP Server")]
pub struct Cli {
    /// Port to listen on (overridable via PORT env var)
    #[arg(short, long, env = "PORT", default_value_t = 8090)]
    pub port: u16,

    /// Path to SQLite database file (overridable via DATABASE_PATH env var)
    #[arg(
        long,
        env = "DATABASE_PATH",
        default_value = "data/neopress.sqlite",
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
    /// Set a new password for an account, e.g. when you are locked out of /n-admin.
    ///
    /// Takes the password from NEOPRESS_NEW_PASSWORD when it is set; otherwise generates
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
                .unwrap_or_else(|_| "neopress_server=info,tower_http=info".into()),
        )
        .init();

    if let Err(err) = run(args).await {
        eprintln!("CRITICAL SERVER ERROR: {err:?}");
        std::process::exit(1);
    }
}

/// `neopress reset-password`; returns the process exit code.
fn reset_password(db_path: &Path, username: &str, sign_out_everyone: bool) -> i32 {
    let from_env = std::env::var(password_reset::NEW_PASSWORD_ENV)
        .ok()
        .filter(|p| !p.is_empty());
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

async fn run(args: Cli) -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!("           🦀 Neopress: The All-in-One Rust CMS             ");
    println!("============================================================");
    println!("✓ Architecture:       Hexagonal (Domain / Ports / Application / Adapters)");
    println!("✓ License:            AGPLv3 + Commercial Dual-License");
    println!("✓ Engine:             Native Rust (Axum + Tokio)");
    println!("✓ Storage Adapter:    Embedded Rusqlite (Zero SQL Server Required)");
    println!("✓ Database Path:      {}", args.db.display());

    // 1. Initialize Concrete Storage Adapter
    let database = Database::new(&args.db)?;

    // 2. Wire Output Ports into Application Service via Arc<dyn Trait>
    let repo = Arc::new(database.clone());
    let _cms_service =
        CmsService::new(repo.clone(), repo.clone(), repo.clone(), repo.clone(), repo);

    // 3. Hand wired state to Primary Web & MCP Adapter
    let app = create_app(database);

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    println!(
        "✓ Status:             Listening on http://localhost:{}/",
        args.port
    );
    println!("------------------------------------------------------------");
    println!("🌐 Public Website:    http://localhost:{}/", args.port);
    println!(
        "⚙️ Admin Dashboard:   http://localhost:{}/n-admin/",
        args.port
    );
    println!(
        "📡 REST API Root:     http://localhost:{}/wp-json/",
        args.port
    );
    println!(
        "🤖 Claude MCP RPC:    http://localhost:{}/mcp (or `neopress --mcp-stdio`)",
        args.port
    );
    println!("============================================================");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("[SERVER] TcpListener bound to {addr}");

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

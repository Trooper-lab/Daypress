//! # Composition Root (`main.rs`)
//!
//! Initializes concrete infrastructure adapters (`SqliteDatabase`), wires them into
//! the pure application orchestrator (`CmsService`), and drives either the Axum HTTP server
//! or the built-in Model Context Protocol (`--mcp-stdio`) server for Claude / AI tools.

use clap::Parser;
use neopress_server::{
    adapters::mcp::run_stdio_mcp_loop, create_app, CmsService, Database, SqliteDatabase,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(name = "neopress")]
#[command(about = "Neopress: Self-Hosted Rust CMS with 1-Click WP Migration & Built-in MCP Server")]
pub struct Cli {
    /// Port to listen on (overridable via PORT env var)
    #[arg(short, long, env = "PORT", default_value_t = 8090)]
    pub port: u16,

    /// Path to SQLite database file (overridable via DATABASE_PATH env var)
    #[arg(long, env = "DATABASE_PATH", default_value = "data/neopress.sqlite")]
    pub db: PathBuf,

    /// Run as a stdio Model Context Protocol (MCP) server for Claude Desktop / Claude Code
    #[arg(long, default_value_t = false)]
    pub mcp_stdio: bool,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

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

fn run_mcp_stdio(args: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SqliteDatabase::new(&args.db)?;
    let repo = Arc::new(adapter);
    let cms_service = CmsService::new(
        repo.clone(),
        repo.clone(),
        repo.clone(),
        repo.clone(),
        repo,
    );
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
    let _cms_service = CmsService::new(
        repo.clone(),
        repo.clone(),
        repo.clone(),
        repo.clone(),
        repo,
    );

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

    let res = axum::serve(listener, app).await;

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

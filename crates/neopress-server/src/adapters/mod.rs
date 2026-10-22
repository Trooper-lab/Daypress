//! # Adapters Layer (Infrastructure & Primary/Secondary Edges)
//!
//! * [`database`]: Embedded Rusqlite implementation of the Hexagonal Output Ports, split into focused domain repositories.
//! * [`web`]: Axum HTTP REST API (`/wp-json/*`), Admin Portal (`/n-admin/*`), and Public SSR handlers.
//! * [`mcp`]: Model Context Protocol (MCP) JSON-RPC 2.0 server for Claude Desktop, Claude Code, and AI tools.

pub mod database;
pub mod mcp;
pub mod web;

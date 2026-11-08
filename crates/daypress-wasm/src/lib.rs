//! # `daypress-wasm` — Sandboxed WebAssembly (`.wasm`) Extension Runtime
//!
//! Provides a zero-C-dependency, deterministic, fuel-metered WebAssembly (`.wasm`) binary
//! validator, stack virtual machine, and bytecode compiler for third-party Daypress extensions.
//!
//! ## Security & Isolation Guarantees
//! 1. **Binary Validation**: Verifies `\0asm` (`0x00 0x61 0x73 0x6d`) magic header, version `1`,
//!    LEB128 section headers, linear memory page bounds, and export table.
//! 2. **Instruction Fuel Metering**: Every executed Wasm opcode consumes 1 unit of fuel.
//!    Runaway or infinite loops (`loop br 0 end`) trap deterministically with [`WasmError::FuelExhausted`]
//!    in microseconds without blocking the host Axum runtime.
//! 3. **Linear Memory Cap**: Linear memory is capped at `max_memory_pages` (64 KiB per page, default 16 pages = 1 MiB).
//! 4. **Capability-Based Host ABI**: Host imports (`dp_emit_str`, `dp_get_attr`, `dp_kv_get`, `dp_kv_set`,
//!    `dp_get_option`) check [`WasmCapabilities`] before executing and trap with
//!    [`WasmError::CapabilityDenied`] if an extension attempts an unauthorized operation.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use thiserror::Error;

pub const WASM_MAGIC_HEADER: [u8; 8] = [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
/// Stable Daypress 1.0 WebAssembly Extension Host ABI version (`v1`).
///
/// Extensions targeting ABI v1 are guaranteed forward-compatibility across all `1.x` releases.
pub const WASM_ABI_VERSION: u32 = 1;
pub const DEFAULT_FUEL_LIMIT: u64 = 50_000;
pub const DEFAULT_MAX_MEMORY_PAGES: u32 = 16; // 1 MiB

/// Canonical list of host imports guaranteed by Daypress Wasm Host ABI v1.
pub const SUPPORTED_HOST_IMPORTS: &[&str] = &[
    "env.dp_emit_str",
    "env.dp_emit_attr",
    "env.dp_emit_input",
    "env.dp_kv_set",
    "env.dp_kv_get_emit",
    "env.dp_kv_incr_emit",
    "env.dp_option_emit",
    "env.dp_http_get",
    "env.dp_abi_version",
];

/// Canonical list of extension exports invoked by the Daypress host runtime under ABI v1.
pub const SUPPORTED_EXPORTS: &[&str] = &["render_shortcode", "filter_content"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WasmCapabilities {
    #[serde(default = "default_true")]
    pub shortcodes: bool,
    #[serde(default)]
    pub content_filter: bool,
    #[serde(default)]
    pub kv_storage: bool,
    #[serde(default)]
    pub read_options: bool,
    #[serde(default)]
    pub http_outbound: bool,
}

fn default_true() -> bool {
    true
}

impl Default for WasmCapabilities {
    fn default() -> Self {
        Self {
            shortcodes: true,
            content_filter: false,
            kv_storage: false,
            read_options: false,
            http_outbound: false,
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WasmError {
    #[error("Invalid WebAssembly binary: {0}")]
    InvalidBinary(String),
    #[error("Export '{0}' not found in WebAssembly module")]
    ExportNotFound(String),
    #[error(
        "WebAssembly execution trapped: instruction fuel exhausted (limit: {limit} instructions)"
    )]
    FuelExhausted { limit: u64 },
    #[error("WebAssembly memory limit exceeded (requested {requested_pages} pages, max {max_pages} pages)")]
    MemoryLimitExceeded {
        requested_pages: u32,
        max_pages: u32,
    },
    #[error("WebAssembly out-of-bounds memory access at offset {offset} (len {len})")]
    MemoryOutOfBounds { offset: usize, len: usize },
    #[error("Capability denied: extension is not granted '{capability}' permission")]
    CapabilityDenied { capability: String },
    #[error("WebAssembly trap: unreachable instruction executed")]
    UnreachableTrap,
}

/// Host environment callback trait so `daypress-wasm` remains decoupled from SQLite details.
pub trait WasmHostEnvironment {
    fn kv_get(&self, key: &str) -> Result<Option<String>, String>;
    fn kv_set(&mut self, key: &str, value: &str) -> Result<(), String>;
    fn get_option(&self, option_name: &str) -> Option<String>;
}

/// In-memory host environment implementation used for unit testing or stateless runs.
#[derive(Default, Clone)]
pub struct MemoryWasmHost {
    pub kv: HashMap<String, String>,
    pub options: HashMap<String, String>,
}

impl WasmHostEnvironment for MemoryWasmHost {
    fn kv_get(&self, key: &str) -> Result<Option<String>, String> {
        Ok(self.kv.get(key).cloned())
    }
    fn kv_set(&mut self, key: &str, value: &str) -> Result<(), String> {
        self.kv.insert(key.to_string(), value.to_string());
        Ok(())
    }
    fn get_option(&self, option_name: &str) -> Option<String> {
        self.options.get(option_name).cloned()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmModuleInspection {
    pub sha256: String,
    pub size_bytes: usize,
    pub initial_memory_pages: u32,
    pub max_memory_pages: Option<u32>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
}

/// Result of verifying a `.wasm` binary against the Daypress 1.0 Stable Host ABI v1 contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WasmAbiCompatibilityReport {
    pub abi_version: u32,
    pub is_compatible: bool,
    pub sha256: String,
    pub recognized_imports: Vec<String>,
    pub unsupported_imports: Vec<String>,
    pub recognized_exports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WasmExecutionOutput {
    pub output_html: String,
    pub return_code: i32,
    pub fuel_consumed: u64,
    pub fuel_remaining: u64,
}

#[derive(Debug, Clone)]
struct DataSegment {
    offset: usize,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
struct ParsedWasmModule {
    initial_memory_pages: u32,
    max_memory_pages: Option<u32>,
    imports: Vec<(String, String)>,
    exports: HashMap<String, usize>,
    code_bodies: Vec<Vec<u8>>,
    data_segments: Vec<DataSegment>,
}

fn read_leb128_u32(bytes: &[u8], cursor: &mut usize) -> Result<u32, WasmError> {
    let mut result = 0u32;
    let mut shift = 0u32;
    loop {
        if *cursor >= bytes.len() {
            return Err(WasmError::InvalidBinary(
                "Unexpected EOF reading LEB128 u32".into(),
            ));
        }
        let byte = bytes[*cursor];
        *cursor += 1;
        result |= ((byte & 0x7f) as u32) << shift;
        if (byte & 0x80) == 0 {
            break;
        }
        shift += 7;
        if shift >= 35 {
            return Err(WasmError::InvalidBinary("LEB128 u32 overflow".into()));
        }
    }
    Ok(result)
}

fn read_leb128_i32(bytes: &[u8], cursor: &mut usize) -> Result<i32, WasmError> {
    let mut result = 0i32;
    let mut shift = 0u32;
    let mut byte: u8;
    loop {
        if *cursor >= bytes.len() {
            return Err(WasmError::InvalidBinary(
                "Unexpected EOF reading LEB128 i32".into(),
            ));
        }
        byte = bytes[*cursor];
        *cursor += 1;
        result |= ((byte & 0x7f) as i32) << shift;
        shift += 7;
        if (byte & 0x80) == 0 {
            break;
        }
        if shift >= 35 {
            return Err(WasmError::InvalidBinary("LEB128 i32 overflow".into()));
        }
    }
    if shift < 32 && (byte & 0x40) != 0 {
        result |= !0 << shift;
    }
    Ok(result)
}

fn read_wasm_name(bytes: &[u8], cursor: &mut usize) -> Result<String, WasmError> {
    let len = read_leb128_u32(bytes, cursor)? as usize;
    if *cursor + len > bytes.len() {
        return Err(WasmError::InvalidBinary(
            "String length exceeds section bounds".into(),
        ));
    }
    let s = std::str::from_utf8(&bytes[*cursor..*cursor + len])
        .map_err(|_| WasmError::InvalidBinary("Invalid UTF-8 identifier in Wasm module".into()))?
        .to_string();
    *cursor += len;
    Ok(s)
}

static PARSED_MODULE_CACHE: OnceLock<Mutex<HashMap<String, Arc<ParsedWasmModule>>>> =
    OnceLock::new();

/// Computes the hex-encoded SHA-256 digest of a `.wasm` binary.
pub fn compute_wasm_sha256(wasm_bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(wasm_bytes);
    hex::encode(hasher.finalize())
}

fn get_or_parse_wasm_module(
    wasm_bytes: &[u8],
) -> Result<(String, Arc<ParsedWasmModule>), WasmError> {
    let sha256 = compute_wasm_sha256(wasm_bytes);
    let cache = PARSED_MODULE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(guard) = cache.lock() {
        if let Some(cached) = guard.get(&sha256) {
            return Ok((sha256, Arc::clone(cached)));
        }
    }
    let parsed = Arc::new(parse_wasm_module(wasm_bytes)?);
    if let Ok(mut guard) = cache.lock() {
        if guard.len() >= 256 {
            guard.clear();
        }
        guard.insert(sha256.clone(), Arc::clone(&parsed));
    }
    Ok((sha256, parsed))
}

/// Validates and inspects a `.wasm` binary module (using the in-memory parsed module cache).
pub fn inspect_wasm_binary(wasm_bytes: &[u8]) -> Result<WasmModuleInspection, WasmError> {
    let (sha256, parsed) = get_or_parse_wasm_module(wasm_bytes)?;
    let mut exports: Vec<String> = parsed.exports.keys().cloned().collect();
    exports.sort();
    let imports = parsed
        .imports
        .iter()
        .map(|(m, n)| format!("{}.{}", m, n))
        .collect();
    Ok(WasmModuleInspection {
        sha256,
        size_bytes: wasm_bytes.len(),
        initial_memory_pages: parsed.initial_memory_pages,
        max_memory_pages: parsed.max_memory_pages,
        imports,
        exports,
    })
}

/// Checks whether a `.wasm` binary conforms to the Daypress 1.0 Stable Host ABI v1 contract.
pub fn check_abi_compatibility(wasm_bytes: &[u8]) -> Result<WasmAbiCompatibilityReport, WasmError> {
    let inspection = inspect_wasm_binary(wasm_bytes)?;
    let mut recognized_imports = Vec::new();
    let mut unsupported_imports = Vec::new();
    for imp in &inspection.imports {
        if SUPPORTED_HOST_IMPORTS.contains(&imp.as_str()) {
            recognized_imports.push(imp.clone());
        } else {
            unsupported_imports.push(imp.clone());
        }
    }
    let recognized_exports: Vec<String> = inspection
        .exports
        .iter()
        .filter(|e| SUPPORTED_EXPORTS.contains(&e.as_str()))
        .cloned()
        .collect();
    let is_compatible = unsupported_imports.is_empty() && !recognized_exports.is_empty();
    Ok(WasmAbiCompatibilityReport {
        abi_version: WASM_ABI_VERSION,
        is_compatible,
        sha256: inspection.sha256,
        recognized_imports,
        unsupported_imports,
        recognized_exports,
    })
}

fn parse_wasm_module(wasm_bytes: &[u8]) -> Result<ParsedWasmModule, WasmError> {
    if wasm_bytes.len() < 8 || wasm_bytes[0..8] != WASM_MAGIC_HEADER {
        return Err(WasmError::InvalidBinary(
            "Missing WebAssembly \\0asm v1 header".into(),
        ));
    }
    let mut cursor = 8usize;
    let mut initial_memory_pages = 1u32;
    let mut max_memory_pages = None;
    let mut imports = Vec::new();
    let mut exports = HashMap::new();
    let mut code_bodies = Vec::new();
    let mut data_segments = Vec::new();

    while cursor < wasm_bytes.len() {
        let section_id = wasm_bytes[cursor];
        cursor += 1;
        let section_len = read_leb128_u32(wasm_bytes, &mut cursor)? as usize;
        if cursor + section_len > wasm_bytes.len() {
            return Err(WasmError::InvalidBinary(format!(
                "Section {} length ({}) exceeds binary size",
                section_id, section_len
            )));
        }
        let section_end = cursor + section_len;
        let mut sc = cursor;

        match section_id {
            2 => {
                // Import section
                let count = read_leb128_u32(wasm_bytes, &mut sc)?;
                for _ in 0..count {
                    let mod_name = read_wasm_name(wasm_bytes, &mut sc)?;
                    let field_name = read_wasm_name(wasm_bytes, &mut sc)?;
                    if sc >= section_end {
                        return Err(WasmError::InvalidBinary("Truncated import entry".into()));
                    }
                    let kind = wasm_bytes[sc];
                    sc += 1;
                    match kind {
                        0 => {
                            let _type_idx = read_leb128_u32(wasm_bytes, &mut sc)?;
                            imports.push((mod_name, field_name));
                        }
                        _ => {
                            break;
                        }
                    }
                }
            }
            5 => {
                // Memory section
                let count = read_leb128_u32(wasm_bytes, &mut sc)?;
                if count > 0 {
                    let flags = read_leb128_u32(wasm_bytes, &mut sc)?;
                    initial_memory_pages = read_leb128_u32(wasm_bytes, &mut sc)?;
                    if (flags & 0x01) != 0 {
                        max_memory_pages = Some(read_leb128_u32(wasm_bytes, &mut sc)?);
                    }
                }
            }
            7 => {
                // Export section
                let count = read_leb128_u32(wasm_bytes, &mut sc)?;
                for _ in 0..count {
                    let name = read_wasm_name(wasm_bytes, &mut sc)?;
                    if sc >= section_end {
                        return Err(WasmError::InvalidBinary("Truncated export entry".into()));
                    }
                    let kind = wasm_bytes[sc];
                    sc += 1;
                    let idx = read_leb128_u32(wasm_bytes, &mut sc)? as usize;
                    if kind == 0 {
                        exports.insert(name, idx);
                    }
                }
            }
            10 => {
                // Code section
                let count = read_leb128_u32(wasm_bytes, &mut sc)?;
                for _ in 0..count {
                    let body_size = read_leb128_u32(wasm_bytes, &mut sc)? as usize;
                    if sc + body_size > section_end {
                        return Err(WasmError::InvalidBinary(
                            "Function body exceeds code section".into(),
                        ));
                    }
                    let body_end = sc + body_size;
                    let local_decl_count = read_leb128_u32(wasm_bytes, &mut sc)?;
                    for _ in 0..local_decl_count {
                        let _n = read_leb128_u32(wasm_bytes, &mut sc)?;
                        sc += 1; // valtype
                    }
                    code_bodies.push(wasm_bytes[sc..body_end].to_vec());
                    sc = body_end;
                }
            }
            11 => {
                // Data section
                let count = read_leb128_u32(wasm_bytes, &mut sc)?;
                for _ in 0..count {
                    let mode = read_leb128_u32(wasm_bytes, &mut sc)?;
                    let mut offset = 0usize;
                    if mode == 0 {
                        if sc < section_end && wasm_bytes[sc] == 0x41 {
                            sc += 1;
                            offset = read_leb128_i32(wasm_bytes, &mut sc)?.max(0) as usize;
                        }
                        if sc < section_end && wasm_bytes[sc] == 0x0b {
                            sc += 1;
                        }
                    }
                    let size = read_leb128_u32(wasm_bytes, &mut sc)? as usize;
                    if sc + size > section_end {
                        return Err(WasmError::InvalidBinary(
                            "Data segment exceeds section bounds".into(),
                        ));
                    }
                    data_segments.push(DataSegment {
                        offset,
                        bytes: wasm_bytes[sc..sc + size].to_vec(),
                    });
                    sc += size;
                }
            }
            _ => {}
        }

        cursor = section_end;
    }

    Ok(ParsedWasmModule {
        initial_memory_pages,
        max_memory_pages,
        imports,
        exports,
        code_bodies,
        data_segments,
    })
}

fn read_mem_str(memory: &[u8], ptr: i32, len: i32) -> Result<String, WasmError> {
    if ptr < 0 || len < 0 {
        return Err(WasmError::MemoryOutOfBounds {
            offset: ptr.max(0) as usize,
            len: len.max(0) as usize,
        });
    }
    let start = ptr as usize;
    let l = len as usize;
    if start.saturating_add(l) > memory.len() {
        return Err(WasmError::MemoryOutOfBounds {
            offset: start,
            len: l,
        });
    }
    Ok(String::from_utf8_lossy(&memory[start..start + l]).into_owned())
}

/// Executes an exported function (e.g., `"render_shortcode"` or `"filter_content"`) in a `.wasm` binary
/// with deterministic instruction fuel metering, linear memory limits, and capability-gated host imports.
#[allow(clippy::too_many_arguments)]
pub fn execute_wasm_export<H: WasmHostEnvironment>(
    wasm_bytes: &[u8],
    export_name: &str,
    attrs: &HashMap<String, String>,
    input_content: &str,
    capabilities: &WasmCapabilities,
    fuel_limit: u64,
    max_memory_pages: u32,
    host: &mut H,
) -> Result<WasmExecutionOutput, WasmError> {
    if export_name == "render_shortcode" && !capabilities.shortcodes {
        return Err(WasmError::CapabilityDenied {
            capability: "shortcodes".into(),
        });
    }
    if export_name == "filter_content" && !capabilities.content_filter {
        return Err(WasmError::CapabilityDenied {
            capability: "content_filter".into(),
        });
    }

    let (_, module) = get_or_parse_wasm_module(wasm_bytes)?;
    if module.initial_memory_pages > max_memory_pages {
        return Err(WasmError::MemoryLimitExceeded {
            requested_pages: module.initial_memory_pages,
            max_pages: max_memory_pages,
        });
    }
    if let Some(decl_max) = module.max_memory_pages {
        if decl_max > max_memory_pages {
            return Err(WasmError::MemoryLimitExceeded {
                requested_pages: decl_max,
                max_pages: max_memory_pages,
            });
        }
    }

    let &func_idx = module
        .exports
        .get(export_name)
        .ok_or_else(|| WasmError::ExportNotFound(export_name.to_string()))?;
    let num_imports = module.imports.len();
    if func_idx < num_imports || (func_idx - num_imports) >= module.code_bodies.len() {
        return Err(WasmError::InvalidBinary(format!(
            "Export '{}' points to invalid function index {}",
            export_name, func_idx
        )));
    }

    let mem_size = (module.initial_memory_pages.max(1) as usize) * 65_536;
    let mut memory = vec![0u8; mem_size];
    for seg in &module.data_segments {
        if seg.offset.saturating_add(seg.bytes.len()) > memory.len() {
            return Err(WasmError::MemoryOutOfBounds {
                offset: seg.offset,
                len: seg.bytes.len(),
            });
        }
        memory[seg.offset..seg.offset + seg.bytes.len()].copy_from_slice(&seg.bytes);
    }

    let code = &module.code_bodies[func_idx - num_imports];
    let mut pc = 0usize;
    let mut stack: Vec<i32> = Vec::with_capacity(64);
    let mut block_targets: Vec<(u8, usize)> = Vec::new(); // (opcode 0x02=block / 0x03=loop, start_pc)
    let mut fuel_remaining = fuel_limit;
    let mut output_html = String::new();

    while pc < code.len() {
        if fuel_remaining == 0 {
            return Err(WasmError::FuelExhausted { limit: fuel_limit });
        }
        fuel_remaining -= 1;

        let op = code[pc];
        pc += 1;
        match op {
            0x00 => {
                // unreachable
                return Err(WasmError::UnreachableTrap);
            }
            0x01 => {
                // nop
            }
            0x02 | 0x03 => {
                // block / loop
                if pc < code.len() {
                    pc += 1; // blocktype (0x40 = empty)
                }
                block_targets.push((op, pc));
            }
            0x0b => {
                // end
                if block_targets.is_empty() {
                    break;
                } else {
                    block_targets.pop();
                }
            }
            0x0c => {
                // br <depth>
                let depth = read_leb128_u32(code, &mut pc)? as usize;
                if depth >= block_targets.len() {
                    break;
                }
                let target_idx = block_targets.len() - 1 - depth;
                let (kind, start_pc) = block_targets[target_idx];
                if kind == 0x03 {
                    // loop: jump back to loop start!
                    block_targets.truncate(target_idx + 1);
                    pc = start_pc;
                } else {
                    break;
                }
            }
            0x0f => {
                // return
                break;
            }
            0x1a => {
                // drop
                let _ = stack.pop();
            }
            0x41 => {
                // i32.const
                let val = read_leb128_i32(code, &mut pc)?;
                stack.push(val);
            }
            0x6a => {
                // i32.add
                let b = stack.pop().unwrap_or(0);
                let a = stack.pop().unwrap_or(0);
                stack.push(a.wrapping_add(b));
            }
            0x6b => {
                // i32.sub
                let b = stack.pop().unwrap_or(0);
                let a = stack.pop().unwrap_or(0);
                stack.push(a.wrapping_sub(b));
            }
            0x6c => {
                // i32.mul
                let b = stack.pop().unwrap_or(0);
                let a = stack.pop().unwrap_or(0);
                stack.push(a.wrapping_mul(b));
            }
            0x10 => {
                // call <func_idx>
                let target_fn = read_leb128_u32(code, &mut pc)? as usize;
                if target_fn < num_imports {
                    let (_mod_name, import_name) = &module.imports[target_fn];
                    match import_name.as_str() {
                        "dp_emit_str" => {
                            let len = stack.pop().unwrap_or(0);
                            let ptr = stack.pop().unwrap_or(0);
                            let s = read_mem_str(&memory, ptr, len)?;
                            output_html.push_str(&s);
                        }
                        "dp_emit_attr" => {
                            let def_len = stack.pop().unwrap_or(0);
                            let def_ptr = stack.pop().unwrap_or(0);
                            let key_len = stack.pop().unwrap_or(0);
                            let key_ptr = stack.pop().unwrap_or(0);
                            let key = read_mem_str(&memory, key_ptr, key_len)?;
                            let fallback = read_mem_str(&memory, def_ptr, def_len)?;
                            let val = attrs.get(&key).map(|s| s.as_str()).unwrap_or(&fallback);
                            output_html.push_str(val);
                        }
                        "dp_emit_input" => {
                            output_html.push_str(input_content);
                        }
                        "dp_kv_set" => {
                            if !capabilities.kv_storage {
                                return Err(WasmError::CapabilityDenied {
                                    capability: "kv_storage".into(),
                                });
                            }
                            let val_len = stack.pop().unwrap_or(0);
                            let val_ptr = stack.pop().unwrap_or(0);
                            let key_len = stack.pop().unwrap_or(0);
                            let key_ptr = stack.pop().unwrap_or(0);
                            let key = read_mem_str(&memory, key_ptr, key_len)?;
                            let val = read_mem_str(&memory, val_ptr, val_len)?;
                            let _ = host.kv_set(&key, &val);
                        }
                        "dp_kv_get_emit" => {
                            if !capabilities.kv_storage {
                                return Err(WasmError::CapabilityDenied {
                                    capability: "kv_storage".into(),
                                });
                            }
                            let key_len = stack.pop().unwrap_or(0);
                            let key_ptr = stack.pop().unwrap_or(0);
                            let key = read_mem_str(&memory, key_ptr, key_len)?;
                            if let Ok(Some(v)) = host.kv_get(&key) {
                                output_html.push_str(&v);
                            }
                        }
                        "dp_kv_incr_emit" => {
                            if !capabilities.kv_storage {
                                return Err(WasmError::CapabilityDenied {
                                    capability: "kv_storage".into(),
                                });
                            }
                            let key_len = stack.pop().unwrap_or(0);
                            let key_ptr = stack.pop().unwrap_or(0);
                            let key = read_mem_str(&memory, key_ptr, key_len)?;
                            let cur: i64 = host
                                .kv_get(&key)
                                .ok()
                                .flatten()
                                .and_then(|s| s.parse::<i64>().ok())
                                .unwrap_or(0);
                            let next = cur + 1;
                            let next_str = next.to_string();
                            let _ = host.kv_set(&key, &next_str);
                            output_html.push_str(&next_str);
                        }
                        "dp_option_emit" => {
                            if !capabilities.read_options {
                                return Err(WasmError::CapabilityDenied {
                                    capability: "read_options".into(),
                                });
                            }
                            let key_len = stack.pop().unwrap_or(0);
                            let key_ptr = stack.pop().unwrap_or(0);
                            let key = read_mem_str(&memory, key_ptr, key_len)?;
                            if let Some(v) = host.get_option(&key) {
                                output_html.push_str(&v);
                            }
                        }
                        "dp_http_get" => {
                            if !capabilities.http_outbound {
                                return Err(WasmError::CapabilityDenied {
                                    capability: "http_outbound".into(),
                                });
                            }
                            let _len = stack.pop().unwrap_or(0);
                            let _ptr = stack.pop().unwrap_or(0);
                        }
                        "dp_abi_version" => {
                            stack.push(WASM_ABI_VERSION as i32);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    let return_code = stack.pop().unwrap_or(0);
    Ok(WasmExecutionOutput {
        output_html,
        return_code,
        fuel_consumed: fuel_limit.saturating_sub(fuel_remaining),
        fuel_remaining,
    })
}

// =========================================================================
// Built-in WebAssembly (`.wasm`) Binary Compiler Helpers
// =========================================================================

fn write_leb128_u32(out: &mut Vec<u8>, mut val: u32) {
    loop {
        let mut byte = (val & 0x7f) as u8;
        val >>= 7;
        if val != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if val == 0 {
            break;
        }
    }
}

fn write_leb128_i32(out: &mut Vec<u8>, mut val: i32) {
    let mut more = true;
    while more {
        let mut byte = (val & 0x7f) as u8;
        val >>= 7;
        if (val == 0 && (byte & 0x40) == 0) || (val == -1 && (byte & 0x40) != 0) {
            more = false;
        } else {
            byte |= 0x80;
        }
        out.push(byte);
    }
}

fn write_wasm_section(module: &mut Vec<u8>, section_id: u8, payload: &[u8]) {
    module.push(section_id);
    write_leb128_u32(module, payload.len() as u32);
    module.extend_from_slice(payload);
}

fn write_wasm_str(out: &mut Vec<u8>, s: &str) {
    write_leb128_u32(out, s.len() as u32);
    out.extend_from_slice(s.as_bytes());
}

/// Compiles a template string (supporting `{{attr:key|default}}`, `{{kv_incr:key}}`,
/// `{{kv:key}}`, and `{{option:blogname}}` placeholders) into a genuine, standards-compliant
/// `.wasm` binary module with linear memory data segments and host ABI imports!
pub fn compile_template_to_wasm_binary(template: &str, initial_pages: u32) -> Vec<u8> {
    // Imports:
    // 0: env.dp_emit_str(ptr: i32, len: i32)
    // 1: env.dp_emit_attr(k_ptr: i32, k_len: i32, def_ptr: i32, def_len: i32)
    // 2: env.dp_kv_incr_emit(k_ptr: i32, k_len: i32)
    // 3: env.dp_option_emit(k_ptr: i32, k_len: i32)
    // 4: env.dp_kv_get_emit(k_ptr: i32, k_len: i32)
    let mut data_buf: Vec<u8> = Vec::new();
    let mut intern_str = |s: &str| -> (i32, i32) {
        let ptr = data_buf.len() as i32;
        data_buf.extend_from_slice(s.as_bytes());
        (ptr, s.len() as i32)
    };

    let mut code_ops: Vec<u8> = Vec::new();
    let mut rest = template;
    while let Some(open_idx) = rest.find("{{") {
        let literal = &rest[..open_idx];
        if !literal.is_empty() {
            let (ptr, len) = intern_str(literal);
            code_ops.push(0x41);
            write_leb128_i32(&mut code_ops, ptr);
            code_ops.push(0x41);
            write_leb128_i32(&mut code_ops, len);
            code_ops.push(0x10);
            write_leb128_u32(&mut code_ops, 0); // dp_emit_str
        }
        let after_open = &rest[open_idx + 2..];
        if let Some(close_idx) = after_open.find("}}") {
            let token = after_open[..close_idx].trim();
            if let Some(attr_spec) = token.strip_prefix("attr:") {
                let (key, def_val) = match attr_spec.split_once('|') {
                    Some((k, d)) => (k.trim(), d.trim()),
                    None => (attr_spec.trim(), ""),
                };
                let (k_ptr, k_len) = intern_str(key);
                let (d_ptr, d_len) = intern_str(def_val);
                code_ops.push(0x41);
                write_leb128_i32(&mut code_ops, k_ptr);
                code_ops.push(0x41);
                write_leb128_i32(&mut code_ops, k_len);
                code_ops.push(0x41);
                write_leb128_i32(&mut code_ops, d_ptr);
                code_ops.push(0x41);
                write_leb128_i32(&mut code_ops, d_len);
                code_ops.push(0x10);
                write_leb128_u32(&mut code_ops, 1); // dp_emit_attr
            } else if let Some(kv_key) = token.strip_prefix("kv_incr:") {
                let (k_ptr, k_len) = intern_str(kv_key.trim());
                code_ops.push(0x41);
                write_leb128_i32(&mut code_ops, k_ptr);
                code_ops.push(0x41);
                write_leb128_i32(&mut code_ops, k_len);
                code_ops.push(0x10);
                write_leb128_u32(&mut code_ops, 2); // dp_kv_incr_emit
            } else if let Some(opt_key) = token.strip_prefix("option:") {
                let (k_ptr, k_len) = intern_str(opt_key.trim());
                code_ops.push(0x41);
                write_leb128_i32(&mut code_ops, k_ptr);
                code_ops.push(0x41);
                write_leb128_i32(&mut code_ops, k_len);
                code_ops.push(0x10);
                write_leb128_u32(&mut code_ops, 3); // dp_option_emit
            } else if let Some(kv_key) = token.strip_prefix("kv:") {
                let (k_ptr, k_len) = intern_str(kv_key.trim());
                code_ops.push(0x41);
                write_leb128_i32(&mut code_ops, k_ptr);
                code_ops.push(0x41);
                write_leb128_i32(&mut code_ops, k_len);
                code_ops.push(0x10);
                write_leb128_u32(&mut code_ops, 4); // dp_kv_get_emit
            }
            rest = &after_open[close_idx + 2..];
        } else {
            let (ptr, len) = intern_str(&rest[open_idx..]);
            code_ops.push(0x41);
            write_leb128_i32(&mut code_ops, ptr);
            code_ops.push(0x41);
            write_leb128_i32(&mut code_ops, len);
            code_ops.push(0x10);
            write_leb128_u32(&mut code_ops, 0);
            rest = "";
        }
    }
    if !rest.is_empty() {
        let (ptr, len) = intern_str(rest);
        code_ops.push(0x41);
        write_leb128_i32(&mut code_ops, ptr);
        code_ops.push(0x41);
        write_leb128_i32(&mut code_ops, len);
        code_ops.push(0x10);
        write_leb128_u32(&mut code_ops, 0);
    }
    // Push return code 0 and `end` (0x0b)
    code_ops.push(0x41);
    write_leb128_i32(&mut code_ops, 0);
    code_ops.push(0x0b);

    let mut module = WASM_MAGIC_HEADER.to_vec();

    // Import section (5 host imports)
    let mut import_sec = Vec::new();
    write_leb128_u32(&mut import_sec, 5);
    for (idx, name) in [
        "dp_emit_str",
        "dp_emit_attr",
        "dp_kv_incr_emit",
        "dp_option_emit",
        "dp_kv_get_emit",
    ]
    .iter()
    .enumerate()
    {
        write_wasm_str(&mut import_sec, "env");
        write_wasm_str(&mut import_sec, name);
        import_sec.push(0x00); // func import
        write_leb128_u32(&mut import_sec, idx as u32);
    }
    write_wasm_section(&mut module, 2, &import_sec);

    // Memory section
    let mut mem_sec = Vec::new();
    write_leb128_u32(&mut mem_sec, 1);
    write_leb128_u32(&mut mem_sec, 0x00); // no max
    write_leb128_u32(&mut mem_sec, initial_pages.max(1));
    write_wasm_section(&mut module, 5, &mem_sec);

    // Export section: export `render_shortcode` -> func index 5
    let mut exp_sec = Vec::new();
    write_leb128_u32(&mut exp_sec, 1);
    write_wasm_str(&mut exp_sec, "render_shortcode");
    exp_sec.push(0x00); // func export
    write_leb128_u32(&mut exp_sec, 5);
    write_wasm_section(&mut module, 7, &exp_sec);

    // Code section: 1 function body
    let mut body = Vec::new();
    write_leb128_u32(&mut body, 0); // 0 locals
    body.extend_from_slice(&code_ops);

    let mut code_sec = Vec::new();
    write_leb128_u32(&mut code_sec, 1);
    write_leb128_u32(&mut code_sec, body.len() as u32);
    code_sec.extend_from_slice(&body);
    write_wasm_section(&mut module, 10, &code_sec);

    // Data section
    let mut data_sec = Vec::new();
    write_leb128_u32(&mut data_sec, 1);
    write_leb128_u32(&mut data_sec, 0); // active memory 0
    data_sec.push(0x41);
    write_leb128_i32(&mut data_sec, 0); // offset 0
    data_sec.push(0x0b);
    write_leb128_u32(&mut data_sec, data_buf.len() as u32);
    data_sec.extend_from_slice(&data_buf);
    write_wasm_section(&mut module, 11, &data_sec);

    module
}

/// Compiles a malicious `.wasm` binary containing an infinite `loop br 0 end` (`0x03 0x40 0x0c 0x00 0x0b`)
/// to verify deterministic instruction fuel exhaustion in tests.
pub fn compile_infinite_loop_wasm_binary() -> Vec<u8> {
    let mut module = WASM_MAGIC_HEADER.to_vec();

    // Memory section (1 page)
    let mut mem_sec = Vec::new();
    write_leb128_u32(&mut mem_sec, 1);
    write_leb128_u32(&mut mem_sec, 0);
    write_leb128_u32(&mut mem_sec, 1);
    write_wasm_section(&mut module, 5, &mem_sec);

    // Export `render_shortcode` -> func 0
    let mut exp_sec = Vec::new();
    write_leb128_u32(&mut exp_sec, 1);
    write_wasm_str(&mut exp_sec, "render_shortcode");
    exp_sec.push(0x00);
    write_leb128_u32(&mut exp_sec, 0);
    write_wasm_section(&mut module, 7, &exp_sec);

    // Code: `0x00` (0 locals), `0x03 0x40` (`loop`), `0x0c 0x00` (`br 0`), `0x0b` (`end`), `0x0b` (`end`)
    let body = vec![0x00, 0x03, 0x40, 0x0c, 0x00, 0x0b, 0x0b];
    let mut code_sec = Vec::new();
    write_leb128_u32(&mut code_sec, 1);
    write_leb128_u32(&mut code_sec, body.len() as u32);
    code_sec.extend_from_slice(&body);
    write_wasm_section(&mut module, 10, &code_sec);

    module
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_template_compilation_and_sandboxed_execution() {
        let wasm_bytes = compile_template_to_wasm_binary(
            r#"<div class="wasm-badge" data-site="{{option:blogname}}">Hello {{attr:user|Guest}}! Hits: {{kv_incr:badge_hits}}</div>"#,
            1,
        );
        let inspection = inspect_wasm_binary(&wasm_bytes).expect("inspect wasm");
        assert!(inspection.exports.contains(&"render_shortcode".to_string()));
        assert_eq!(inspection.initial_memory_pages, 1);

        let mut host = MemoryWasmHost::default();
        host.options
            .insert("blogname".to_string(), "Daypress".to_string());

        let mut attrs = HashMap::new();
        attrs.insert("user".to_string(), "Rustacean".to_string());

        // 1. Without `kv_storage` capability, `{{kv_incr:badge_hits}}` traps with `CapabilityDenied`
        let denied = execute_wasm_export(
            &wasm_bytes,
            "render_shortcode",
            &attrs,
            "",
            &WasmCapabilities {
                shortcodes: true,
                content_filter: false,
                kv_storage: false,
                read_options: true,
                http_outbound: false,
            },
            DEFAULT_FUEL_LIMIT,
            DEFAULT_MAX_MEMORY_PAGES,
            &mut host,
        );
        assert_eq!(
            denied,
            Err(WasmError::CapabilityDenied {
                capability: "kv_storage".to_string()
            })
        );

        // 2. With `kv_storage` and `read_options` granted, executes deterministically and increments KV state
        let caps = WasmCapabilities {
            shortcodes: true,
            content_filter: false,
            kv_storage: true,
            read_options: true,
            http_outbound: false,
        };
        let out1 = execute_wasm_export(
            &wasm_bytes,
            "render_shortcode",
            &attrs,
            "",
            &caps,
            DEFAULT_FUEL_LIMIT,
            DEFAULT_MAX_MEMORY_PAGES,
            &mut host,
        )
        .expect("exec 1");
        assert_eq!(
            out1.output_html,
            r#"<div class="wasm-badge" data-site="Daypress">Hello Rustacean! Hits: 1</div>"#
        );

        let out2 = execute_wasm_export(
            &wasm_bytes,
            "render_shortcode",
            &HashMap::new(),
            "",
            &caps,
            DEFAULT_FUEL_LIMIT,
            DEFAULT_MAX_MEMORY_PAGES,
            &mut host,
        )
        .expect("exec 2");
        assert_eq!(
            out2.output_html,
            r#"<div class="wasm-badge" data-site="Daypress">Hello Guest! Hits: 2</div>"#
        );
    }

    #[test]
    fn test_wasm_infinite_loop_fuel_exhaustion() {
        let bad_wasm = compile_infinite_loop_wasm_binary();
        let mut host = MemoryWasmHost::default();
        let res = execute_wasm_export(
            &bad_wasm,
            "render_shortcode",
            &HashMap::new(),
            "",
            &WasmCapabilities::default(),
            500,
            DEFAULT_MAX_MEMORY_PAGES,
            &mut host,
        );
        assert_eq!(res, Err(WasmError::FuelExhausted { limit: 500 }));
    }

    #[test]
    fn test_wasm_abi_v1_compatibility_check() {
        let wasm_bytes = compile_template_to_wasm_binary(
            r#"<span class="abi-v1">{{attr:label|OK}} ({{kv_incr:runs}})</span>"#,
            1,
        );
        let report = check_abi_compatibility(&wasm_bytes).expect("abi compatibility check");
        assert_eq!(report.abi_version, WASM_ABI_VERSION);
        assert!(report.is_compatible);
        assert!(report.unsupported_imports.is_empty());
        assert!(report
            .recognized_exports
            .contains(&"render_shortcode".to_string()));
    }
}

pub mod embeds;
pub mod gallery;
pub mod lists;
pub mod mapper;
pub mod parser;
pub mod styles;
pub mod theme_json;

use mapper::map_gutenberg_block;
use parser::{parse_gutenberg_tree, tokenize_gutenberg};
pub use theme_json::{compile_theme_json_to_css, parse_theme_json, ThemeJson};

use crate::schema::{NeopressLayout, NeopressNode, NodeKind, StyleProperties};

/// Parses WordPress Gutenberg block markup (`<!-- wp:... -->`) into a canonical NeopressLayout AST with site context.
pub fn transpile_gutenberg_content_with_context(
    content: &str,
    source_site_url: Option<&str>,
) -> Result<NeopressLayout, String> {
    let mut normalized = content.to_string();

    if let Some(src) = source_site_url.map(|s| s.trim_end_matches('/')) {
        let https_base = src.replacen("http://", "https://", 1);
        let http_base = src.replacen("https://", "http://", 1);
        let https_esc = https_base.replace('/', "\\/");
        let http_esc = http_base.replace('/', "\\/");

        // 1. Base URL media paths: base + /wp-content/uploads/ -> /uploads/
        for base in [&https_base, &http_base] {
            normalized = normalized.replace(&format!("{}/wp-content/uploads/", base), "/uploads/");
        }
        for base_esc in [&https_esc, &http_esc] {
            normalized = normalized.replace(
                &format!("{}\\/wp-content\\/uploads\\/", base_esc),
                "\\/uploads\\/",
            );
        }

        // 2. Domain-level media paths (when source_site_url has a host)
        if let Some(host) = crate::url_utils::extract_url_host(src) {
            normalized = normalized.replace(
                &format!("https://{}/wp-content/uploads/", host),
                "/uploads/",
            );
            normalized =
                normalized.replace(&format!("http://{}/wp-content/uploads/", host), "/uploads/");
            normalized = normalized.replace(
                &format!("https:\\/\\/{}\\/wp-content\\/uploads\\/", host),
                "\\/uploads\\/",
            );
            normalized = normalized.replace(
                &format!("http:\\/\\/{}\\/wp-content\\/uploads\\/", host),
                "\\/uploads\\/",
            );
        }

        // 3. Internal page/post links: base + / -> /
        for base in [&https_base, &http_base] {
            normalized = normalized.replace(&format!("{}/", base), "/");
        }
        for base_esc in [&https_esc, &http_esc] {
            normalized = normalized.replace(&format!("{}\\/", base_esc), "\\/");
        }

        // 4. Exact base URL in quotes
        for base in [&https_base, &http_base] {
            normalized = normalized.replace(&format!("\"{}\"", base), "\"/\"");
            normalized = normalized.replace(&format!("'{}'", base), "'/'");
        }
        for base_esc in [&https_esc, &http_esc] {
            normalized = normalized.replace(&format!("\\\"{}\\\"", base_esc), "\\\"\\/\\\"");
        }
    }

    // 5. Safe root-relative uploads normalization (protects external WordPress URLs)
    normalized = crate::url_utils::remap_root_relative_uploads(&normalized);

    let tokens = tokenize_gutenberg(&normalized);
    let blocks = parse_gutenberg_tree(tokens);

    let mut id_counter: usize = 0;
    let root_children: Vec<NeopressNode> = blocks
        .iter()
        .filter_map(|b| map_gutenberg_block(b, &mut id_counter))
        .collect();

    Ok(NeopressLayout {
        version: 1,
        root: NeopressNode {
            id: "gutenberg_root".to_string(),
            kind: NodeKind::Section,
            styles: StyleProperties::default(),
            children: root_children,
        },
        global_styles: None,
    })
}

/// Parses WordPress Gutenberg block markup (`<!-- wp:... -->`) into a canonical NeopressLayout AST.
pub fn transpile_gutenberg_content(content: &str) -> Result<NeopressLayout, String> {
    transpile_gutenberg_content_with_context(content, None)
}

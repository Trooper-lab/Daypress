//! WordPress-style archive pagination (`/page/N/` links and slicing) for public listings.

/// Returns the items on 1-based `page` and the total page count (at least 1).
pub fn paginate<T>(items: &[T], page: usize, per_page: usize) -> (&[T], usize) {
    let per_page = per_page.max(1);
    let total_pages = items.len().div_ceil(per_page).max(1);
    let start = (page.max(1) - 1).saturating_mul(per_page).min(items.len());
    let end = (start + per_page).min(items.len());
    (&items[start..end], total_pages)
}

/// URL of `page` under `base_path` (which must end with `/`): page 1 is the base itself.
pub fn page_url(base_path: &str, page: usize) -> String {
    if page <= 1 {
        base_path.to_string()
    } else {
        format!("{}page/{}/", base_path, page)
    }
}

/// Previous / next navigation with a "Page X of Y" label; empty for single-page listings.
pub fn render_pagination(base_path: &str, page: usize, total_pages: usize) -> String {
    if total_pages <= 1 {
        return String::new();
    }
    let page = page.clamp(1, total_pages);
    let prev = if page > 1 {
        format!(
            r#"<a class="dp-button dp-page-prev" rel="prev" href="{}">&larr; Newer posts</a>"#,
            page_url(base_path, page - 1)
        )
    } else {
        String::new()
    };
    let next = if page < total_pages {
        format!(
            r#"<a class="dp-button dp-page-next" rel="next" href="{}">Older posts &rarr;</a>"#,
            page_url(base_path, page + 1)
        )
    } else {
        String::new()
    };
    format!(
        r#"<nav class="dp-pagination" aria-label="Posts navigation" style="display: flex; justify-content: space-between; align-items: center; gap: 16px; margin-top: 40px;"><span>{prev}</span><span class="dp-pagination-status" style="color: var(--text-dim); font-size: 14px;">Page {page} of {total_pages}</span><span>{next}</span></nav>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paginates_and_links_pages() {
        let items: Vec<u32> = (1..=25).collect();
        let (page2, total) = paginate(&items, 2, 10);
        assert_eq!(page2, &items[10..20]);
        assert_eq!(total, 3);
        assert!(paginate(&items, 9, 10).0.is_empty());
        assert_eq!(page_url("/blog/", 1), "/blog/");
        assert_eq!(page_url("/blog/", 3), "/blog/page/3/");
        let nav = render_pagination("/", 2, 3);
        assert!(nav.contains(r#"href="/""#) && nav.contains(r#"href="/page/3/""#));
        assert!(render_pagination("/", 1, 1).is_empty());
    }
}

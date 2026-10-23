# Archetype 4: Custom PHP + ACF + CPT (NORTHWIND)

## Overview
This archetype represents a complex custom PHP + ACF site:
- WXR: `site.WordPress.xml` (8.4MB, 95k lines)
- Theme: `crates/neopress-server/static/themes/northwind/`
- 7+ Custom Post Types dynamically registered and discovered
- ACF custom fields in `wp_postmeta`
- Dynamic CPT archive routes (`/trainingen/`, `/vacatures/`, `/cases/`)
- Public SSR single routes (`/trainingen/:slug/`)
- Diagonal section styles and Swiper carousel markup
- Zero site-specific hardcoding across Rust logic

<?php
require_once __DIR__ . '/wp-load.php';
require_once ABSPATH . 'wp-admin/includes/image.php';
require_once ABSPATH . 'wp-admin/includes/file.php';
require_once ABSPATH . 'wp-admin/includes/media.php';

echo "=== Cleaning existing WordPress test content for deterministic migration test ===\n";

// 0. Clean all existing posts, pages, attachments, nav_menu_items, custom_css
$_GET['force_delete_kit'] = '1';
$all_posts = get_posts([
    'post_type' => ['post', 'page', 'attachment', 'nav_menu_item', 'custom_css'],
    'post_status' => 'any',
    'numberposts' => -1,
]);
foreach ($all_posts as $old_post) {
    wp_delete_post($old_post->ID, true);
}

// Clean all existing comments
$all_comments = get_comments(['status' => 'all', 'number' => 0]);
foreach ($all_comments as $old_c) {
    wp_delete_comment($old_c->comment_ID, true);
}

// Clean existing nav menus
$all_menus = wp_get_nav_menus();
foreach ($all_menus as $old_m) {
    wp_delete_nav_menu($old_m->term_id);
}

// Clean non-default categories and tags
$all_cats = get_terms(['taxonomy' => 'category', 'hide_empty' => false]);
foreach ($all_cats as $c) {
    if ($c->term_id > 1) {
        wp_delete_term($c->term_id, 'category');
    }
}
$all_tags = get_terms(['taxonomy' => 'post_tag', 'hide_empty' => false]);
foreach ($all_tags as $t) {
    wp_delete_term($t->term_id, 'post_tag');
}

echo "✓ Cleaned old WordPress content.\n";
echo "=== Seeding WordPress with comprehensive migration test suite ===\n";

// 1. Update Site Settings & Identity
update_option('blogname', 'WordPress Production Reference');
update_option('blogdescription', 'Enterprise WordPress Instance Ready for Daypress 1-Click Migration');
update_option('posts_per_page', 10);
update_option('date_format', 'F j, Y');
update_option('time_format', 'g:i a');
update_option('show_on_front', 'posts');
update_option('page_on_front', 0);

// 1b. Create Additional Author User
$editor_user_id = username_exists('alex_chen');
if (!$editor_user_id) {
    $editor_user_id = wp_insert_user([
        'user_login' => 'alex_chen',
        'user_pass' => 'DaypressTestPass2026!',
        'user_email' => 'alex.chen@daypress.io',
        'display_name' => 'Alex Chen (Systems Lead)',
        'user_nicename' => 'alex-chen',
        'role' => 'editor',
    ]);
    echo "Created user: Alex Chen (ID: $editor_user_id)\n";
}

// 1c. Create Real Media Attachments (JPEG, PNG, and SVG) in wp-content/uploads
$upload_dir = wp_upload_dir();
wp_mkdir_p($upload_dir['path']);

function create_test_media_attachment($filename, $mime_type, $title, $alt_text, $remote_url, $gd_generator) {
    global $upload_dir;
    $filepath = $upload_dir['path'] . '/' . $filename;
    $downloaded = false;

    if ($remote_url) {
        $resp = wp_remote_get($remote_url, ['timeout' => 8, 'redirection' => 5]);
        if (!is_wp_error($resp) && wp_remote_retrieve_response_code($resp) === 200) {
            $body = wp_remote_retrieve_body($resp);
            if (strlen($body) > 1024) {
                file_put_contents($filepath, $body);
                $downloaded = true;
            }
        }
    }

    if (!$downloaded) {
        $gd_generator($filepath);
    }

    $file_url = $upload_dir['url'] . '/' . $filename;
    $attach_id = wp_insert_attachment([
        'guid' => $file_url,
        'post_mime_type' => $mime_type,
        'post_title' => $title,
        'post_content' => '',
        'post_status' => 'inherit',
    ], $filepath);

    if ($mime_type !== 'image/svg+xml') {
        $attach_data = wp_generate_attachment_metadata($attach_id, $filepath);
        wp_update_attachment_metadata($attach_id, $attach_data);
    }
    update_post_meta($attach_id, '_wp_attachment_image_alt', $alt_text);
    echo "Created media attachment [$mime_type]: $filename (ID: $attach_id, Size: " . filesize($filepath) . " bytes, URL: $file_url)\n";
    return [$attach_id, $file_url];
}

// 1) Real JPEG #1: Cloud Datacenter Photo (1200x800)
list($jpg1_id, $jpg1_url) = create_test_media_attachment(
    'daypress-cloud-datacenter.jpg',
    'image/jpeg',
    'Daypress Global Edge Datacenter',
    'High-density server racks running Daypress Rust edge nodes',
    'https://picsum.photos/seed/daypress-datacenter/1200/800.jpg',
    function($filepath) {
        $im = imagecreatetruecolor(1200, 800);
        for ($y = 0; $y < 800; $y++) {
            $col = imagecolorallocate($im, (int)(10 + ($y / 800) * 20), (int)(15 + ($y / 800) * 40), (int)(35 + ($y / 800) * 80));
            imageline($im, 0, $y, 1200, $y, $col);
        }
        $accent = imagecolorallocate($im, 255, 79, 0);
        $white = imagecolorallocate($im, 255, 255, 255);
        imagefilledrectangle($im, 80, 80, 1120, 720, imagecolorallocatealpha($im, 15, 23, 42, 40));
        imagerectangle($im, 80, 80, 1120, 720, $accent);
        imagestring($im, 5, 120, 140, 'DAYPRESS CLOUD DATACENTER // 1200x800 JPEG', $white);
        imagejpeg($im, $filepath, 90);
        imagedestroy($im);
    }
);

// 2) Real JPEG #2: Edge Network Topology Photo (1200x800)
list($jpg2_id, $jpg2_url) = create_test_media_attachment(
    'daypress-edge-network.jpg',
    'image/jpeg',
    'Daypress Global Edge Network',
    'Global low-latency routing across Munich, Tokyo, and Sao Paulo',
    'https://picsum.photos/seed/daypress-edge/1200/800.jpg',
    function($filepath) {
        $im = imagecreatetruecolor(1200, 800);
        $bg = imagecolorallocate($im, 9, 13, 22);
        imagefill($im, 0, 0, $bg);
        $cyan = imagecolorallocate($im, 6, 182, 212);
        for ($i = 0; $i < 12; $i++) {
            imageellipse($im, 600, 400, 150 + $i * 80, 100 + $i * 50, $cyan);
        }
        imagejpeg($im, $filepath, 90);
        imagedestroy($im);
    }
);

// 3) Real PNG: Editorial Dashboard UI Snapshot (1200x800)
list($png_id, $png_url) = create_test_media_attachment(
    'daypress-editorial-dashboard.png',
    'image/png',
    'Daypress Obsidian Editorial UI',
    'Distraction-free Daypress Visual Builder and Admin Portal in Rust',
    'https://picsum.photos/seed/daypress-ui/1200/800.jpg',
    function($filepath) {
        $im = imagecreatetruecolor(1200, 800);
        $bg = imagecolorallocate($im, 15, 23, 42);
        imagefill($im, 0, 0, $bg);
        $card = imagecolorallocate($im, 30, 41, 59);
        $accent = imagecolorallocate($im, 37, 99, 235);
        $white = imagecolorallocate($im, 255, 255, 255);
        imagefilledrectangle($im, 60, 60, 1140, 740, $card);
        imagefilledrectangle($im, 60, 60, 1140, 120, $accent);
        imagestring($im, 5, 90, 82, 'DAYPRESS VISUAL BUILDER // 1200x800 LOSSLESS PNG', $white);
        imagepng($im, $filepath, 6);
        imagedestroy($im);
    }
);
// Ensure the PNG file is true PNG format even if downloaded from Picsum JPEG endpoint
$png_filepath = $upload_dir['path'] . '/daypress-editorial-dashboard.png';
$im_loaded = @imagecreatefromjpeg($png_filepath);
if ($im_loaded) {
    imagepng($im_loaded, $png_filepath, 6);
    imagedestroy($im_loaded);
    $attach_data = wp_generate_attachment_metadata($png_id, $png_filepath);
    wp_update_attachment_metadata($png_id, $attach_data);
}

// 4) Vector SVG: Architecture Diagram (1200x630)
list($attachment_id, $svg_url) = create_test_media_attachment(
    'daypress-hero-architecture.svg',
    'image/svg+xml',
    'Daypress Hero Architecture Diagram',
    'Daypress Sub-Millisecond Rust CMS Architecture Diagram',
    null,
    function($filepath) {
        $svg_content = <<<SVG
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1200 630" width="1200" height="630">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#05050a"/>
      <stop offset="100%" stop-color="#0f172a"/>
    </linearGradient>
  </defs>
  <rect width="1200" height="630" fill="url(#bg)"/>
  <circle cx="950" cy="200" r="220" fill="#ff4f00" opacity="0.15"/>
  <rect x="80" y="120" width="140" height="40" rx="6" fill="#ff4f00"/>
  <text x="150" y="146" fill="#ffffff" font-family="sans-serif" font-size="18" font-weight="bold" text-anchor="middle">DAYPRESS</text>
  <text x="80" y="250" fill="#ffffff" font-family="sans-serif" font-size="54" font-weight="bold">Sub-Millisecond Rust CMS</text>
  <text x="80" y="320" fill="#94a3b8" font-family="sans-serif" font-size="28">1-Click WordPress Migration • Axum • Embedded SQLite WAL</text>
</svg>
SVG;
        file_put_contents($filepath, $svg_content);
    }
);

// 2. Create Categories (Including Parent -> Child Hierarchy)
$cat_ids = [];
$categories = [
    'Engineering' => ['desc' => 'Deep dive into system programming, concurrency, and high-performance backend systems.', 'parent' => 0],
    'Architecture' => ['desc' => 'Modern software architecture, modular monoliths, and database engine designs.', 'parent' => 0],
    'Design Systems' => ['desc' => '2026 UX patterns, typographic scales, and distraction-free editorial workflows.', 'parent' => 0],
    'Tutorials' => ['desc' => 'Step-by-step guides for modern web engineering and data migrations.', 'parent' => 0],
];

foreach ($categories as $name => $info) {
    $term = wp_insert_term($name, 'category', ['description' => $info['desc'], 'parent' => $info['parent']]);
    if (!is_wp_error($term)) {
        $cat_ids[$name] = $term['term_id'];
    } else {
        $existing = get_term_by('name', $name, 'category');
        $cat_ids[$name] = $existing->term_id;
    }
}

// Create a child category under Engineering to test parent category ID remapping
$child_cat = wp_insert_term('Rust Internals', 'category', [
    'description' => 'Zero-cost abstractions, borrow checking, and FFI SQLite internals.',
    'parent' => $cat_ids['Engineering'],
]);
if (!is_wp_error($child_cat)) {
    $cat_ids['Rust Internals'] = $child_cat['term_id'];
}

// 3. Create Tags
$tags = ['rust', 'axum', 'sqlite', 'cms', 'performance', 'tokio', 'wal-mode', 'zero-sql', 'i18n', 'elementor', 'divi'];
foreach ($tags as $tag) {
    wp_insert_term($tag, 'post_tag');
}

// 4. Create Rich Blog Posts (Including UTF-8 International / Multibyte & Embedded Media URL)
$posts_data = [
    [
        'title' => 'Why In-Process Storage Outperforms External SQL in 2026',
        'slug' => 'in-process-storage-outperforms-external-sql',
        'category' => 'Engineering',
        'tags' => ['rust', 'sqlite', 'performance', 'wal-mode'],
        'featured_media' => $attachment_id,
        'content' => <<<HTML
<!-- wp:paragraph -->
<p>Modern web infrastructure has reached a tipping point where traditional three-tier client-server architectures introduce unnecessary latency. When an application server communicates over network TCP sockets with an external database daemon, every query incurs serialization, network traversal, and context switching overhead.</p>
<!-- /wp:paragraph -->

<!-- wp:image -->
<figure class="wp-block-image"><img src="{$svg_url}" alt="Daypress Architecture"/></figure>
<!-- /wp:image -->

<!-- wp:heading {"level":2} -->
<h2>The Latency Trap of Legacy SQL Networks</h2>
<!-- /wp:heading -->

<!-- wp:paragraph -->
<p>In standard WordPress installations running MySQL or MariaDB, a typical page render fires between <strong>25 and 70 individual SQL queries</strong>. Even when the database is hosted on the same physical server via localhost socket, kernel IPC overhead adds between 1ms to 5ms per transaction.</p>
<!-- /wp:paragraph -->

<!-- wp:quote -->
<blockquote class="wp-block-quote"><p>By replacing external database daemons with embedded SQLite memory-mapped engines in WAL mode, median query latency plummets from <strong>4.2ms to 0.08ms</strong> — a 50x throughput amplification.</p><cite>Daypress Systems Benchmark, 2026</cite></blockquote>
<!-- /wp:quote -->

<!-- wp:heading {"level":2} -->
<h2>Rust C-FFI Zero-Allocation Execution</h2>
<!-- /wp:heading -->

<!-- wp:paragraph -->
<p>Here is how the in-process execution model communicates with SQLite directly through raw pointers without any intermediate network stack:</p>
<!-- /wp:paragraph -->

<!-- wp:code -->
<pre><code class="language-rust">// Embedded query execution in Rusqlite
let mut stmt = conn.prepare_cached("SELECT id, title, content FROM wp_posts WHERE post_status = ?1")?;
let post_iter = stmt.query_map([ "publish" ], |row| {
    Ok(Post {
        id: row.get(0)?,
        title: row.get(1)?,
        content: row.get(2)?,
    })
})?;
</code></pre>
<!-- /wp:code -->
HTML
    ],
    [
        'title' => 'The Architecture of High-Throughput Tokio Asynchronous CMS Engines',
        'slug' => 'architecture-tokio-async-cms-engines',
        'category' => 'Architecture',
        'tags' => ['rust', 'axum', 'tokio', 'performance'],
        'author' => $editor_user_id,
        'featured_media' => $jpg1_id,
        'content' => <<<HTML
<!-- wp:paragraph -->
<p>Traditional PHP web applications rely on synchronous, process-per-request execution models like PHP-FPM or Apache mod_php. While simple to reason about, this paradigm struggles under concurrent load due to memory bloat and operating system context switching limits.</p>
<!-- /wp:paragraph -->

<!-- wp:image -->
<figure class="wp-block-image"><img src="{$jpg1_url}" alt="Daypress Global Edge Datacenter"/></figure>
<!-- /wp:image -->

<!-- wp:heading {"level":2} -->
<h2>Work-Stealing Thread Pools vs Worker Pools</h2>
<!-- /wp:heading -->

<!-- wp:paragraph -->
<p>The Tokio runtime utilizes a cooperative multi-threaded work-stealing scheduler. Lightweight async tasks (green threads) consume mere bytes of memory compared to the 2MB to 8MB stack allocation of native operating system threads.</p>
<!-- /wp:paragraph -->

<!-- wp:list -->
<ul>
    <li><strong>Non-blocking I/O:</strong> Epoll/IOCP event loops handle tens of thousands of idle connections simultaneously.</li>
    <li><strong>Zero-Copy Buffers:</strong> Hyper and Axum stream HTTP payloads without unnecessary heap reallocations.</li>
    <li><strong>Predictable Tail Latency:</strong> P99 latency remains bounded under sustained concurrency spikes.</li>
</ul>
<!-- /wp:list -->
HTML
    ],
    [
        'title' => 'Designing Editorial Systems for Focus: The 2026 UI/UX Shift',
        'slug' => 'designing-editorial-systems-focus-2026',
        'category' => 'Design Systems',
        'tags' => ['cms'],
        'featured_media' => $png_id,
        'content' => <<<HTML
<!-- wp:paragraph -->
<p>For more than two decades, content management admin panels have accumulated visual clutter: redundant metaboxes, promotional banners, nested sidebars, and distracting dashboard widgets that fragment author attention.</p>
<!-- /wp:paragraph -->

<!-- wp:image -->
<figure class="wp-block-image"><img src="{$png_url}" alt="Daypress Obsidian Editorial UI"/></figure>
<!-- /wp:image -->

<!-- wp:heading {"level":2} -->
<h2>Principles of the Obsidian Workspace</h2>
<!-- /wp:heading -->

<!-- wp:list {"ordered":true} -->
<ol>
    <li><strong>Monochromatic Foundation:</strong> Obsidian slate navigation with electric cyan speed indicators.</li>
    <li><strong>Distraction-Free Canvas:</strong> Direct article typing without modal overlays or popup interruptions.</li>
    <li><strong>Tabular Density:</strong> Data tables with hairline dividers and hover-activated row micro-actions.</li>
    <li><strong>Instant Micro-Interactions:</strong> Immediate optimistic UI updates paired with sub-millisecond server acknowledgments.</li>
</ol>
<!-- /wp:list -->
HTML
    ],
    [
        'title' => 'Step-by-Step Guide: 1-Click Migration from WordPress to Daypress',
        'slug' => 'guide-1-click-migration-wordpress-to-daypress',
        'category' => 'Tutorials',
        'tags' => ['rust', 'cms', 'zero-sql', 'sqlite'],
        'content' => <<<HTML
<!-- wp:paragraph -->
<p>Transitioning from an existing WordPress website to Daypress requires zero manual database export commands or complex SQL conversion scripts. With the <strong>Daypress Migrator</strong> plugin, the entire schema, media assets, terms, and posts are transferred in a single atomic transaction.</p>
<!-- /wp:paragraph -->
HTML
    ],
    [
        'title' => 'Memory Safety and Zero-Cost Abstractions in Web Publishing',
        'slug' => 'memory-safety-zero-cost-abstractions',
        'category' => 'Rust Internals',
        'tags' => ['rust', 'performance'],
        'content' => <<<HTML
<!-- wp:paragraph -->
<p>Memory corruption vulnerabilities (buffer overflows, use-after-free, null pointer dereferences) represent over 70% of historical CVE security advisories in legacy C and PHP extensions. Rust’s ownership and borrow checker guarantee memory safety at compile time without requiring a tracing garbage collector.</p>
<!-- /wp:paragraph -->
HTML
    ],
    [
        'title' => 'Global Edge Deployment: München, 東京, São Paulo & 🚀 Sub-Millisecond UTF-8 Safety',
        'slug' => 'global-edge-deployment-munchen-tokyo-sao-paulo',
        'category' => 'Rust Internals',
        'tags' => ['rust', 'i18n', 'performance'],
        'featured_media' => $jpg2_id,
        'content' => <<<HTML
<!-- wp:paragraph -->
<p>Deploying across global edge regions — from München (äöüß) and Zürich to 東京 (日本語のマルチバイト文字列テスト) and São Paulo (atenção, coração) — requires strict UTF-8 character boundary handling 🚀⚡🦀 so multi-byte characters at index 140 never cause slice panics in Rust.</p>
<!-- /wp:paragraph -->
HTML
    ]
];

$first_post_id = 0;
foreach ($posts_data as $p) {
    $cat_id = isset($cat_ids[$p['category']]) ? [$cat_ids[$p['category']]] : [];
    $post_id = wp_insert_post([
        'post_title' => $p['title'],
        'post_name' => $p['slug'],
        'post_content' => $p['content'],
        'post_status' => 'publish',
        'post_type' => 'post',
        'post_author' => $p['author'] ?? 1,
        'post_category' => $cat_id,
        'tags_input' => $p['tags']
    ]);
    if (!empty($p['featured_media'])) {
        set_post_thumbnail($post_id, $p['featured_media']);
    }
    if ($first_post_id === 0) $first_post_id = $post_id;
    echo "Created post: {$p['title']} (ID: $post_id)\n";
}

// Seed threaded comments on the first post
if ($first_post_id > 0) {
    $parent_comment_id = wp_insert_comment([
        'comment_post_ID' => $first_post_id,
        'comment_author' => 'Sarah Jenkins (Principal Systems Architect)',
        'comment_author_email' => 'sarah@systems.example.com',
        'comment_author_url' => 'https://daypress.io',
        'comment_content' => 'Migrating our 400-page Elementor & Gutenberg publication to Daypress dropped our P99 TTFB from 420ms to 0.3ms.',
        'comment_approved' => 1,
    ]);
    wp_insert_comment([
        'comment_post_ID' => $first_post_id,
        'comment_parent' => $parent_comment_id,
        'comment_author' => 'Alex Chen (Systems Lead)',
        'comment_author_email' => 'alex.chen@daypress.io',
        'comment_author_url' => 'https://daypress.io',
        'comment_content' => 'Confirmed! And the zero-allocation SQLite WAL queries eliminated our Redis cache cluster entirely.',
        'comment_approved' => 1,
        'user_id' => $editor_user_id,
    ]);
}

// 4b. Create or Update Elementor Global Kit (`elementor_active_kit`)
$kit_id = (int) get_option('elementor_active_kit', 0);
if ($kit_id <= 0 || !get_post($kit_id)) {
    $kit_id = wp_insert_post([
        'post_title' => 'Default Kit',
        'post_name' => 'default-kit',
        'post_status' => 'publish',
        'post_type' => 'elementor_library',
    ]);
    update_option('elementor_active_kit', $kit_id);
}
update_post_meta($kit_id, '_elementor_page_settings', [
    'system_colors' => [
        ['_id' => 'primary', 'title' => 'Primary', 'color' => '#2563eb'],
        ['_id' => 'secondary', 'title' => 'Secondary', 'color' => '#0f172a'],
        ['_id' => 'text', 'title' => 'Text', 'color' => '#e2e8f0'],
        ['_id' => 'accent', 'title' => 'Accent', 'color' => '#ff4f00'],
    ],
    'system_typography' => [
        ['_id' => 'primary', 'title' => 'Primary', 'typography_font_family' => 'Inter', 'typography_font_weight' => '800'],
        ['_id' => 'text', 'title' => 'Text', 'typography_font_family' => 'Inter', 'typography_font_weight' => '400'],
    ],
]);

// 5. Create Rich Pages (Elementor 3.x Flexbox + Section/Columns, Divi Builder, Gutenberg Blocks, and Parent->Child Hierarchy)
$elementor_json = wp_json_encode([
    [
        'id' => 'elem_hero_sec',
        'elType' => 'container',
        'settings' => [
            'flex_direction' => 'column',
            'background_background' => 'classic',
            'background_color' => '#0f172a',
            'padding' => ['top' => '64', 'right' => '24', 'bottom' => '64', 'left' => '24', 'unit' => 'px']
        ],
        'elements' => [
            [
                'id' => 'elem_h1',
                'elType' => 'widget',
                'widgetType' => 'heading',
                'settings' => [
                    'title' => 'Elementor 3.x Flexbox Container Transpiled to Rust',
                    'header_size' => 'h1',
                    'title_color' => '#ffffff',
                    'align' => 'center'
                ],
                'elements' => []
            ],
            [
                'id' => 'elem_txt',
                'elType' => 'widget',
                'widgetType' => 'text-editor',
                'settings' => [
                    'editor' => '<p>This page was built in Elementor using Flexbox Containers, Icon Boxes, Counters, Pricing Tables, and Accordions, and automatically transpiled to Daypress AST.</p>',
                    'text_color' => '#cbd5e1',
                    'align' => 'center'
                ],
                'elements' => []
            ],
            [
                'id' => 'elem_btn',
                'elType' => 'widget',
                'widgetType' => 'button',
                'settings' => [
                    'text' => 'Explore Rust AST Engine',
                    'link' => ['url' => '/about-us/'],
                    'background_color' => '#2563eb'
                ],
                'elements' => []
            ],
            [
                'id' => 'elem_cnt',
                'elType' => 'widget',
                'widgetType' => 'counter',
                'settings' => [
                    'ending_number' => 15000,
                    'suffix' => ' req/s',
                    'title' => 'Throughput per Single Node'
                ],
                'elements' => []
            ]
        ]
    ],
    [
        'id' => 'elem_features_sec',
        'elType' => 'section',
        'settings' => [
            'background_background' => 'classic',
            'background_color' => '#111827',
            'padding' => ['top' => '48', 'right' => '24', 'bottom' => '48', 'left' => '24', 'unit' => 'px']
        ],
        'elements' => [
            [
                'id' => 'elem_col_1',
                'elType' => 'column',
                'settings' => ['_column_size' => 50],
                'elements' => [
                    [
                        'id' => 'elem_icon_box',
                        'elType' => 'widget',
                        'widgetType' => 'icon-box',
                        'settings' => [
                            'selected_icon' => ['value' => 'fa-solid fa-bolt'],
                            'title_text' => 'Zero-Copy AST Rendering',
                            'description_text' => 'Elementor JSON is transpiled once at migration time into native Rust structs.'
                        ],
                        'elements' => []
                    ],
                    [
                        'id' => 'elem_accordion',
                        'elType' => 'widget',
                        'widgetType' => 'accordion',
                        'settings' => [
                            'tabs' => [
                                ['tab_title' => 'Does it support Elementor 3.x Containers?', 'tab_content' => 'Yes, both Flexbox Containers and legacy Sections/Columns are supported.'],
                                ['tab_title' => 'Are Global Colors preserved?', 'tab_content' => 'Yes, ElementorKit global colors and typography compile to CSS custom properties.']
                            ]
                        ],
                        'elements' => []
                    ]
                ]
            ],
            [
                'id' => 'elem_col_2',
                'elType' => 'column',
                'settings' => ['_column_size' => 50],
                'elements' => [
                    [
                        'id' => 'elem_price',
                        'elType' => 'widget',
                        'widgetType' => 'price-table',
                        'settings' => [
                            'heading' => 'Enterprise Edge Node',
                            'price' => '49',
                            'currency_symbol' => '$',
                            'period' => 'month',
                            'features_list' => [
                                ['item_text' => 'Unlimited 1-Click WP Migrations'],
                                ['item_text' => 'Sub-Millisecond SQLite WAL'],
                                ['item_text' => 'Zero PHP Runtime Vulnerabilities']
                            ],
                            'button_text' => 'Deploy Now',
                            'link' => ['url' => '/contact/']
                        ],
                        'elements' => []
                    ]
                ]
            ]
        ]
    ]
]);

$divi_content = <<<DIVI
[et_pb_section fb_built="1" background_color="#0f172a" custom_padding="60px|24px|60px|24px"][et_pb_row][et_pb_column type="1_2"][et_pb_text]<h2>Divi Visual Builder Page in Native Rust</h2><p>Deeply nested Divi shortcodes are tokenized and transpiled into canonical Daypress Layout AST nodes during 1-Click Migration.</p>[/et_pb_text][et_pb_divider color="#334155"][et_pb_button button_text="Read Benchmarks" button_url="/architecture-and-benchmarks/"][/et_pb_column][et_pb_column type="1_2"][et_pb_blurb title="Zero Shortcode Runtime Overhead" use_icon="on" font_icon="fa-solid fa-bolt"]Divi shortcodes are pre-compiled into semantic HTML & CSS Grid at import time.[/et_pb_blurb][et_pb_counters][et_pb_counter percent="99"]Visual Parity Score[/et_pb_counter][/et_pb_counters][et_pb_accordion][et_pb_accordion_item title="Divi Shortcode Parser"]Handles unclosed leaf tags like et_pb_divider and et_pb_button without stack corruption.[/et_pb_accordion_item][/et_pb_accordion][/et_pb_column][/et_pb_row][/et_pb_section]
DIVI;

$gutenberg_showcase_content = <<<HTML
<!-- wp:heading {"level":1} -->
<h1>Gutenberg Block Editor Showcase</h1>
<!-- /wp:heading -->

<!-- wp:paragraph -->
<p>This page verifies that structured Gutenberg block comments (columns, buttons, quotes, code blocks, and separators) are cleanly parsed into the unified Daypress AST and rendered with full semantic fidelity.</p>
<!-- /wp:paragraph -->

<!-- wp:columns -->
<div class="wp-block-columns">
<!-- wp:column -->
<div class="wp-block-column">
<!-- wp:heading {"level":3} -->
<h3>Column One: Block Grammar Parser</h3>
<!-- /wp:heading -->
<!-- wp:paragraph -->
<p>Parses nested <code>wp:columns</code> and <code>wp:group</code> containers into <code>DaypressNode</code> trees.</p>
<!-- /wp:paragraph -->
</div>
<!-- /wp:column -->

<!-- wp:column -->
<div class="wp-block-column">
<!-- wp:heading {"level":3} -->
<h3>Column Two: Zero Plugin Tax</h3>
<!-- /wp:heading -->
<!-- wp:paragraph -->
<p>Native SEO, forms, and analytics are compiled directly into the Rust server binary.</p>
<!-- /wp:paragraph -->
</div>
<!-- /wp:column -->
</div>
<!-- /wp:columns -->

<!-- wp:separator -->
<hr class="wp-block-separator"/>
<!-- /wp:separator -->

<!-- wp:buttons -->
<div class="wp-block-buttons">
<!-- wp:button -->
<div class="wp-block-button"><a class="wp-block-button__link" href="/contact/">Contact Engineering</a></div>
<!-- /wp:button -->
</div>
<!-- /wp:buttons -->
HTML;

$pages_data = [
    [
        'title' => 'About Daypress Foundation',
        'slug' => 'about-us',
        'menu_order' => 1,
        'content' => <<<HTML
<!-- wp:paragraph -->
<p>The Daypress Project was created to liberate web publishing from the twin burdens of legacy PHP execution overhead and complex distributed database management. By compiling the entire content management lifecycle into a single native binary, we provide developers and content creators with unprecedented speed, stability, and peace of mind.</p>
<!-- /wp:paragraph -->

<!-- wp:heading {"level":2} -->
<h2>Our Mission</h2>
<!-- /wp:heading -->

<!-- wp:paragraph -->
<p>To power the next 100 million websites with zero external dependencies, embedded memory-mapped storage, and instant headless REST API compatibility.</p>
<!-- /wp:paragraph -->
HTML
    ],
    [
        'title' => 'Architecture & Performance Benchmark',
        'slug' => 'architecture-and-benchmarks',
        'menu_order' => 2,
        'content' => <<<HTML
<!-- wp:paragraph -->
<p>In independent load tests conducted on 4-vCPU 8GB RAM instances, Daypress demonstrated a <strong>14x reduction in P99 response time</strong> and served <strong>82,000 requests per second</strong> with 0 dropped frames compared to standard WordPress + Apache + MariaDB configurations.</p>
<!-- /wp:paragraph -->
HTML
    ],
    [
        'title' => 'Enterprise Services & Cloud Consulting',
        'slug' => 'services',
        'menu_order' => 3,
        'content' => <<<HTML
<!-- wp:paragraph -->
<p>We provide full-lifecycle migration assistance, custom Rust module engineering, high-availability replication topology design, and 24/7 mission-critical production support for enterprises adopting Daypress.</p>
<!-- /wp:paragraph -->
HTML
    ],
    [
        'title' => 'Elementor Visual Showcase',
        'slug' => 'elementor-showcase',
        'menu_order' => 4,
        'content' => '<p>Elementor Fallback Content</p>',
        'meta' => [
            '_elementor_edit_mode' => 'builder',
            '_elementor_data' => $elementor_json,
        ]
    ],
    [
        'title' => 'Divi Builder Showcase',
        'slug' => 'divi-showcase',
        'menu_order' => 5,
        'content' => $divi_content,
    ],
    [
        'title' => 'Gutenberg Block Showcase',
        'slug' => 'gutenberg-showcase',
        'menu_order' => 6,
        'content' => $gutenberg_showcase_content,
    ],
    [
        'title' => 'Contact the Core Team',
        'slug' => 'contact',
        'menu_order' => 7,
        'content' => <<<HTML
<!-- wp:paragraph -->
<p>Have questions about migrating your WordPress publication or implementing headless integrations with Daypress? Reach out to our core architecture group at <a href="mailto:hello@daypress.io">hello@daypress.io</a>.</p>
<!-- /wp:paragraph -->
HTML
    ]
];

$services_page_id = 0;
foreach ($pages_data as $p) {
    $page_id = wp_insert_post([
        'post_title' => $p['title'],
        'post_name' => $p['slug'],
        'post_content' => $p['content'],
        'post_status' => 'publish',
        'post_type' => 'page',
        'menu_order' => $p['menu_order'] ?? 0,
    ]);
    if ($p['slug'] === 'services') {
        $services_page_id = $page_id;
    }
    echo "Created page: {$p['title']} (ID: $page_id)\n";
    if (!empty($p['meta'])) {
        foreach ($p['meta'] as $mk => $mv) {
            update_post_meta($page_id, $mk, wp_slash($mv));
        }
    }
}

// Create a Child Page under "Enterprise Services & Cloud Consulting" to test post_parent ID remapping
if ($services_page_id > 0) {
    $child_page_id = wp_insert_post([
        'post_title' => 'Cloud SLA & Security Architecture',
        'post_name' => 'cloud-sla',
        'post_parent' => $services_page_id,
        'menu_order' => 1,
        'post_status' => 'publish',
        'post_type' => 'page',
        'post_content' => '<!-- wp:paragraph --><p>Dedicated 99.999% uptime SLA backed by memory-safe Rust binaries and automated SQLite WAL snapshots.</p><!-- /wp:paragraph -->',
    ]);
    echo "Created child page: Cloud SLA & Security Architecture (ID: $child_page_id, Parent ID: $services_page_id)\n";
}

// 6. Navigation Menu Setup
$menu_name = 'Primary Navigation';
$menu_id = wp_create_nav_menu($menu_name);

wp_update_nav_menu_item($menu_id, 0, [
    'menu-item-title' => 'Home',
    'menu-item-url' => home_url('/'),
    'menu-item-status' => 'publish'
]);

foreach ($pages_data as $p) {
    $page = get_page_by_path($p['slug'], OBJECT, 'page');
    if ($page) {
        wp_update_nav_menu_item($menu_id, 0, [
            'menu-item-title' => $p['title'],
            'menu-item-object' => 'page',
            'menu-item-object-id' => $page->ID,
            'menu-item-type' => 'post_type',
            'menu-item-status' => 'publish'
        ]);
    }
}

$locations = [
    'primary' => $menu_id,
    'primary-menu' => $menu_id,
    'header_menu' => $menu_id,
];
set_theme_mod('nav_menu_locations', $locations);
echo "Primary Navigation Menu created and assigned (Menu ID: $menu_id).\n";

// 7. Theme Customizations & Custom CSS
set_theme_mod('custom_logo', $attachment_id);
set_theme_mod('site_title', 'WordPress Production Reference');
set_theme_mod('site_tagline', 'Enterprise WordPress Instance Ready for Daypress 1-Click Migration');

$custom_css = "
/* Daypress Migration Reference Styling */
:root {
    --daypress-accent: #2563eb;
    --daypress-dark: #090d16;
}
.site-header {
    border-bottom: 2px solid #2563eb !important;
}
";
wp_update_custom_css_post($custom_css);

echo "✓ WordPress comprehensive migration test suite seeded!\n";

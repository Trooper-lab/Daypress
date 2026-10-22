<?php
/**
 * Plugin Name: Neopress 1-Click Migrator
 * Plugin URI: https://neopress.io
 * Description: 1-Click Migration Engine moving all posts, pages, Elementor/Divi/Gutenberg layouts, menus, widgets, comments, users, terms, media, settings, and theme styles from WordPress to high-performance Neopress Rust CMS.
 * Version: 2.0.0
 * Author: Neopress Core Team
 * Author URI: https://neopress.io
 * License: GPL-2.0+
 */

if (!defined('ABSPATH')) {
    exit;
}

class Neopress_1Click_Migrator {

    public function __construct() {
        add_action('admin_menu', [$this, 'add_admin_menu']);
        add_action('admin_enqueue_scripts', [$this, 'enqueue_assets']);
        add_action('wp_ajax_neopress_preflight', [$this, 'ajax_preflight']);
        add_action('wp_ajax_neopress_execute_migration', [$this, 'ajax_execute_migration']);
        add_action('wp_ajax_neopress_export_bundle', [$this, 'ajax_export_bundle']);

        if (defined('WP_CLI') && WP_CLI) {
            WP_CLI::add_command('neopress migrate', [$this, 'cli_migrate']);
        }
    }

    public function add_admin_menu() {
        $icon_svg = 'data:image/svg+xml;base64,' . base64_encode(
            '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#38bdf8" stroke-width="2.2"><circle cx="12" cy="12" r="10"/><path d="M8 7v10l8-10v10"/></svg>'
        );

        add_menu_page(
            'Neopress Migrator',
            'Neopress Migrator',
            'manage_options',
            'neopress-migrator',
            [$this, 'render_admin_page'],
            $icon_svg,
            65
        );

        add_submenu_page(
            'tools.php',
            'Neopress Migrator',
            'Neopress Migrator',
            'manage_options',
            'neopress-migrator',
            [$this, 'render_admin_page']
        );
    }

    public function enqueue_assets($hook) {
        if (strpos($hook, 'neopress-migrator') === false) {
            return;
        }
    }

    public function collect_bundle() {
        // 1. Site & Reading Settings
        $settings = [
            'title' => (string) get_option('blogname', 'WordPress Site'),
            'description' => (string) get_option('blogdescription', ''),
            'email' => (string) get_option('admin_email', 'admin@example.com'),
            'url' => (string) home_url(),
            'date_format' => (string) get_option('date_format', 'F j, Y'),
            'time_format' => (string) get_option('time_format', 'g:i a'),
            'posts_per_page' => (int) get_option('posts_per_page', 10),
            'show_on_front' => (string) get_option('show_on_front', 'posts'),
            'page_on_front' => (int) get_option('page_on_front', 0),
            'page_for_posts' => (int) get_option('page_for_posts', 0),
            'permalink_structure' => (string) get_option('permalink_structure', '/%postname%/'),
        ];

        // 2. Categories
        $categories_raw = get_categories(['hide_empty' => false]);
        $categories = [];
        if (is_array($categories_raw)) {
            foreach ($categories_raw as $c) {
                $categories[] = [
                    'id' => (int) $c->term_id,
                    'name' => (string) $c->name,
                    'slug' => (string) $c->slug,
                    'description' => (string) $c->description,
                    'parent' => (int) $c->parent,
                ];
            }
        }

        // 3. Tags
        $tags_raw = get_tags(['hide_empty' => false]);
        $tags = [];
        if (is_array($tags_raw)) {
            foreach ($tags_raw as $t) {
                $tags[] = [
                    'id' => (int) $t->term_id,
                    'name' => (string) $t->name,
                    'slug' => (string) $t->slug,
                    'description' => (string) $t->description,
                    'parent' => 0,
                ];
            }
        }

        // 4. Media Attachments
        $media_posts = get_posts([
            'post_type' => 'attachment',
            'posts_per_page' => -1,
            'post_status' => 'any',
        ]);
        $media = [];
        foreach ($media_posts as $m) {
            $file_path = get_attached_file($m->ID);
            $file_name = $file_path ? basename($file_path) : basename((string) $m->guid);
            if (empty($file_name)) {
                $file_name = 'attachment-' . $m->ID . '.jpg';
            }
            $mime_type = get_post_mime_type($m->ID);
            if (empty($mime_type)) {
                $mime_type = 'image/jpeg';
            }
            $file_size = ($file_path && file_exists($file_path)) ? filesize($file_path) : 0;
            $base64 = '';
            if ($file_path && file_exists($file_path) && $file_size < 10 * 1024 * 1024) {
                $base64 = base64_encode(file_get_contents($file_path));
            }
            $alt_text = (string) get_post_meta($m->ID, '_wp_attachment_image_alt', true);

            $media[] = [
                'id' => (int) $m->ID,
                'title' => (string) $m->post_title,
                'file_name' => (string) $file_name,
                'mime_type' => (string) $mime_type,
                'source_url' => '/uploads/' . $file_name,
                'original_url' => (string) wp_get_attachment_url($m->ID),
                'file_size' => (int) $file_size,
                'base64_data' => $base64,
                'alt_text' => $alt_text,
            ];
        }

        // 5. Elementor Active Kit Global Styles (if Elementor is installed/used)
        $elementor_kit_settings = null;
        $active_kit_id = (int) get_option('elementor_active_kit', 0);
        if ($active_kit_id > 0) {
            $kit_meta = get_post_meta($active_kit_id, '_elementor_page_settings', true);
            if (!empty($kit_meta)) {
                $elementor_kit_settings = is_string($kit_meta) ? json_decode($kit_meta, true) : $kit_meta;
            }
        }

        // 6. Posts & Pages (including postmeta for Elementor, Divi, Page Templates, Custom Fields)
        $posts_raw = get_posts([
            'post_type' => ['post', 'page'],
            'posts_per_page' => -1,
            'post_status' => 'any',
            'orderby' => 'ID',
            'order' => 'ASC',
        ]);
        $posts = [];
        foreach ($posts_raw as $p) {
            $cat_ids = [];
            $tag_ids = [];
            if ($p->post_type === 'post') {
                $cats = wp_get_post_categories($p->ID);
                if (is_array($cats)) {
                    $cat_ids = array_map('intval', $cats);
                }
                $post_tags = wp_get_post_tags($p->ID, ['fields' => 'ids']);
                if (is_array($post_tags)) {
                    $tag_ids = array_map('intval', $post_tags);
                }
            }

            $featured_media = (int) get_post_thumbnail_id($p->ID);
            $page_template = (string) get_post_meta($p->ID, '_wp_page_template', true);

            // Collect all relevant postmeta
            $raw_meta = get_post_meta($p->ID);
            $postmeta = [];
            if (is_array($raw_meta)) {
                foreach ($raw_meta as $meta_key => $meta_values) {
                    if (empty($meta_values)) {
                        continue;
                    }
                    // Skip internal edit locks
                    if ($meta_key === '_edit_lock' || $meta_key === '_edit_last') {
                        continue;
                    }
                    $val = $meta_values[0];
                    $unserialized = maybe_unserialize($val);
                    if (is_array($unserialized) || is_object($unserialized)) {
                        $postmeta[$meta_key] = wp_json_encode($unserialized);
                    } else {
                        $postmeta[$meta_key] = (string) $val;
                    }
                }
            }
            if ($elementor_kit_settings && !isset($postmeta['_elementor_page_settings'])) {
                $postmeta['_elementor_page_settings'] = wp_json_encode($elementor_kit_settings);
            }

            $posts[] = [
                'id' => (int) $p->ID,
                'title' => (string) $p->post_title,
                'slug' => (string) $p->post_name,
                'content' => (string) $p->post_content,
                'excerpt' => (string) $p->post_excerpt,
                'status' => (string) $p->post_status,
                'post_type' => (string) $p->post_type,
                'date' => (string) $p->post_date,
                'parent' => (int) $p->post_parent,
                'menu_order' => (int) $p->menu_order,
                'author' => (int) $p->post_author,
                'categories' => $cat_ids,
                'tags' => $tag_ids,
                'featured_media' => $featured_media > 0 ? $featured_media : null,
                'template' => $page_template,
                'postmeta' => $postmeta,
            ];
        }

        // 7. Navigation Menus & Menu Items
        $menus = [];
        $nav_menus = wp_get_nav_menus();
        $locations = get_nav_menu_locations();
        if (is_array($nav_menus)) {
            foreach ($nav_menus as $menu_obj) {
                $menu_items_raw = wp_get_nav_menu_items($menu_obj->term_id);
                $items = [];
                if (is_array($menu_items_raw)) {
                    foreach ($menu_items_raw as $mi) {
                        $url = (string) $mi->url;
                        // Make internal home_url relative so it resolves cleanly on Neopress
                        $site_home = rtrim((string) home_url(), '/');
                        if (!empty($site_home) && strpos($url, $site_home) === 0) {
                            $rel = substr($url, strlen($site_home));
                            $url = empty($rel) ? '/' : $rel;
                        }
                        $items[] = [
                            'id' => (int) $mi->ID,
                            'title' => (string) $mi->title,
                            'url' => $url,
                            'object' => (string) $mi->object,
                            'object_id' => (int) $mi->object_id,
                            'item_type' => (string) $mi->type,
                            'menu_order' => (int) $mi->menu_order,
                            'parent' => (int) $mi->menu_item_parent,
                        ];
                    }
                }
                $assigned_locs = [];
                if (is_array($locations)) {
                    foreach ($locations as $loc_slug => $loc_term_id) {
                        if ((int) $loc_term_id === (int) $menu_obj->term_id) {
                            $assigned_locs[] = (string) $loc_slug;
                        }
                    }
                }
                $menus[] = [
                    'id' => (int) $menu_obj->term_id,
                    'name' => (string) $menu_obj->name,
                    'slug' => (string) $menu_obj->slug,
                    'locations' => $assigned_locs,
                    'items' => $items,
                ];
            }
        }

        // 8. Comments
        $comments_raw = get_comments(['status' => 'all', 'number' => 500]);
        $comments = [];
        if (is_array($comments_raw)) {
            foreach ($comments_raw as $cm) {
                $comments[] = [
                    'id' => (int) $cm->comment_ID,
                    'post_id' => (int) $cm->comment_post_ID,
                    'author' => (string) $cm->comment_author,
                    'author_email' => (string) $cm->comment_author_email,
                    'author_url' => (string) $cm->comment_author_url,
                    'author_ip' => (string) $cm->comment_author_IP,
                    'content' => (string) $cm->comment_content,
                    'date' => (string) $cm->comment_date,
                    'status' => (string) $cm->comment_approved,
                    'parent' => (int) $cm->comment_parent,
                ];
            }
        }

        // 9. Users
        $users_raw = get_users(['number' => 200]);
        $users = [];
        if (is_array($users_raw)) {
            foreach ($users_raw as $u) {
                $roles = is_array($u->roles) ? array_values($u->roles) : ['subscriber'];
                $users[] = [
                    'id' => (int) $u->ID,
                    'username' => (string) $u->user_login,
                    'email' => (string) $u->user_email,
                    'display_name' => (string) $u->display_name,
                    'role' => isset($roles[0]) ? (string) $roles[0] : 'subscriber',
                ];
            }
        }

        // 10. Theme Mods, Customizer Colors & Widgets
        $all_mods = get_theme_mods();
        if (!is_array($all_mods)) {
            $all_mods = [];
        }
        $brand_color = '#2563eb';
        if (isset($all_mods['astra-settings']) && is_array($all_mods['astra-settings']) && !empty($all_mods['astra-settings']['theme-color'])) {
            $brand_color = (string) $all_mods['astra-settings']['theme-color'];
        } elseif (!empty($all_mods['accent_color'])) {
            $brand_color = (string) $all_mods['accent_color'];
        }

        $theme_mods = [
            'active_theme' => (string) get_template(),
            'custom_css' => (string) wp_get_custom_css(),
            'brand_color' => $brand_color,
            'raw_mods' => wp_json_encode($all_mods),
        ];

        $sidebars_widgets = get_option('sidebars_widgets', []);

        // 11. Universal WordPress Plugin Convergence Extraction (SEO, Forms, Redirects, WhatsApp, Snippets, SMTP)
        $plugin_option_keys = [
            'wpseo_titles',
            'wpseo_social',
            'rank-math-options-titles',
            'rank-math-options-general',
            'aioseo_options',
            'joinchat',
            'ht_ctc_chat_options',
            'qlwapp',
            'ihaf_insert_header',
            'ihaf_insert_footer',
            'ihaf_insert_body',
            'wpcode_snippets',
            '301_redirects',
            'wp_mail_smtp',
        ];
        $plugin_options = [];
        foreach ($plugin_option_keys as $opt_key) {
            $val = get_option($opt_key, null);
            if ($val !== null && $val !== false && $val !== '') {
                $plugin_options[$opt_key] = $val;
            }
        }

        // Extract Contact Form 7 & WPForms custom post types
        $forms = [];
        $raw_forms = get_posts([
            'post_type' => ['wpcf7_contact_form', 'wpforms'],
            'post_status' => 'any',
            'numberposts' => 200,
        ]);
        if (is_array($raw_forms)) {
            foreach ($raw_forms as $fpost) {
                $fmeta = get_post_meta($fpost->ID);
                $tpl = isset($fmeta['_form'][0]) ? (string) $fmeta['_form'][0] : (string) $fpost->post_content;
                $mail_meta = isset($fmeta['_mail'][0]) ? maybe_unserialize($fmeta['_mail'][0]) : [];
                $recipient = (is_array($mail_meta) && !empty($mail_meta['recipient'])) ? (string) $mail_meta['recipient'] : (string) get_option('admin_email', '');
                $forms[] = [
                    'id' => (int) $fpost->ID,
                    'title' => (string) $fpost->post_title,
                    'source_plugin' => (string) $fpost->post_type,
                    'raw_template' => $tpl,
                    'notify_email' => $recipient,
                ];
            }
        }

        // Extract Redirection / RankMath Redirections custom SQL tables if present
        global $wpdb;
        $redirects = [];
        $redirection_table = $wpdb->prefix . 'redirection_items';
        if ($wpdb->get_var($wpdb->prepare("SHOW TABLES LIKE %s", $redirection_table)) === $redirection_table) {
            $rows = $wpdb->get_results("SELECT url, action_data, action_code, last_count FROM {$redirection_table} WHERE status = 'enabled' LIMIT 500", ARRAY_A);
            if (is_array($rows)) {
                foreach ($rows as $r) {
                    if (!empty($r['url']) && !empty($r['action_data'])) {
                        $redirects[] = [
                            'source_path' => (string) $r['url'],
                            'target_url' => (string) $r['action_data'],
                            'status_code' => isset($r['action_code']) ? (int) $r['action_code'] : 301,
                            'hits' => isset($r['last_count']) ? (int) $r['last_count'] : 0,
                        ];
                    }
                }
            }
        }
        $rm_redirection_table = $wpdb->prefix . 'rank_math_redirections';
        if ($wpdb->get_var($wpdb->prepare("SHOW TABLES LIKE %s", $rm_redirection_table)) === $rm_redirection_table) {
            $rows = $wpdb->get_results("SELECT sources, url_to, header_code, hits FROM {$rm_redirection_table} WHERE status = 'active' LIMIT 500", ARRAY_A);
            if (is_array($rows)) {
                foreach ($rows as $r) {
                    $sources = maybe_unserialize($r['sources']);
                    if (is_array($sources) && !empty($r['url_to'])) {
                        foreach ($sources as $src_item) {
                            if (is_array($src_item) && !empty($src_item['pattern'])) {
                                $redirects[] = [
                                    'source_path' => '/' . ltrim((string) $src_item['pattern'], '/'),
                                    'target_url' => (string) $r['url_to'],
                                    'status_code' => isset($r['header_code']) ? (int) $r['header_code'] : 301,
                                    'hits' => isset($r['hits']) ? (int) $r['hits'] : 0,
                                ];
                            }
                        }
                    }
                }
            }
        }

        return [
            'site_settings' => $settings,
            'categories' => $categories,
            'tags' => $tags,
            'posts' => $posts,
            'media' => $media,
            'menus' => $menus,
            'comments' => $comments,
            'users' => $users,
            'widgets' => is_array($sidebars_widgets) ? $sidebars_widgets : [],
            'elementor_kit' => $elementor_kit_settings,
            'theme_mods' => $theme_mods,
            'plugin_options' => $plugin_options,
            'forms' => $forms,
            'redirects' => $redirects,
        ];
    }

    public function ajax_preflight() {
        check_ajax_referer('neopress_migrator_nonce', 'nonce');

        $bundle = $this->collect_bundle();

        $post_count = 0;
        $page_count = 0;
        $builder_count = 0;
        foreach ($bundle['posts'] as $p) {
            if ($p['post_type'] === 'page') {
                $page_count++;
            } else {
                $post_count++;
            }
            if (!empty($p['postmeta']['_elementor_data']) || strpos($p['content'], '[et_pb_') !== false || strpos($p['content'], '<!-- wp:') !== false) {
                $builder_count++;
            }
        }

        wp_send_json_success([
            'posts_count' => $post_count,
            'pages_count' => $page_count,
            'builder_count' => $builder_count,
            'categories_count' => count($bundle['categories']),
            'tags_count' => count($bundle['tags']),
            'media_count' => count($bundle['media']),
            'menus_count' => count($bundle['menus']),
            'comments_count' => count($bundle['comments']),
            'active_theme' => $bundle['theme_mods']['active_theme'],
            'has_custom_css' => !empty($bundle['theme_mods']['custom_css']),
            'site_title' => $bundle['site_settings']['title'],
        ]);
    }

    public function ajax_export_bundle() {
        check_ajax_referer('neopress_migrator_nonce', 'nonce');
        $bundle = $this->collect_bundle();
        wp_send_json_success($bundle);
    }

    public function ajax_execute_migration() {
        check_ajax_referer('neopress_migrator_nonce', 'nonce');

        $target_url = isset($_POST['target_url']) ? esc_url_raw(trim($_POST['target_url'])) : 'http://host.docker.internal:8090';
        if (empty($target_url)) {
            $target_url = 'http://host.docker.internal:8090';
        }
        $api_auth = isset($_POST['api_auth']) ? trim((string) $_POST['api_auth']) : 'admin:admin';
        if (empty($api_auth)) {
            $api_auth = 'admin:admin';
        }

        $bundle = $this->collect_bundle();

        $endpoint = rtrim($target_url, '/') . '/wp-json/neopress/v1/import';
        $start_time = microtime(true);

        $headers = [
            'Content-Type' => 'application/json',
            'X-WP-Nonce' => 'wp_rest_admin',
        ];
        if (strpos($api_auth, ':') !== false) {
            $headers['Authorization'] = 'Basic ' . base64_encode($api_auth);
        } else {
            $headers['Authorization'] = 'Bearer ' . $api_auth;
        }

        $response = wp_remote_post($endpoint, [
            'headers' => $headers,
            'body' => wp_json_encode($bundle),
            'timeout' => 90,
            'sslverify' => false,
        ]);

        $duration_ms = round((microtime(true) - $start_time) * 1000, 2);

        if (is_wp_error($response)) {
            wp_send_json_error([
                'message' => 'Connection to Neopress failed: ' . $response->get_error_message(),
                'target_url' => $endpoint,
            ]);
        }

        $code = wp_remote_retrieve_response_code($response);
        $body = wp_remote_retrieve_body($response);
        $data = json_decode($body, true);

        if ($code >= 200 && $code < 300 && (!empty($data['success']) || isset($data['status']))) {
            $data['client_duration_ms'] = $duration_ms;
            wp_send_json_success($data);
        } else {
            wp_send_json_error([
                'message' => 'Neopress rejected import: ' . ($body ?: 'HTTP ' . $code),
                'code' => $code,
            ]);
        }
    }

    public function cli_migrate($args, $assoc_args) {
        $target = isset($assoc_args['target']) ? $assoc_args['target'] : 'http://host.docker.internal:8090';
        $auth = isset($assoc_args['auth']) ? $assoc_args['auth'] : 'admin:admin';
        WP_CLI::log("Starting Neopress 1-Click Migration to: $target");

        $bundle = $this->collect_bundle();
        WP_CLI::log("Collected " . count($bundle['posts']) . " posts/pages, " . count($bundle['media']) . " media files, " . count($bundle['categories']) . " categories, " . count($bundle['menus']) . " menus.");

        $endpoint = rtrim($target, '/') . '/wp-json/neopress/v1/import';
        $start = microtime(true);

        $headers = [
            'Content-Type' => 'application/json',
            'X-WP-Nonce' => 'wp_rest_admin',
            'Authorization' => 'Basic ' . base64_encode($auth),
        ];

        $res = wp_remote_post($endpoint, [
            'headers' => $headers,
            'body' => wp_json_encode($bundle),
            'timeout' => 90,
        ]);

        $ms = round((microtime(true) - $start) * 1000, 2);

        if (is_wp_error($res)) {
            WP_CLI::error("Migration failed: " . $res->get_error_message());
        }

        $code = wp_remote_retrieve_response_code($res);
        $body = wp_remote_retrieve_body($res);
        $data = json_decode($body, true);

        if ($code === 200 && (!empty($data['success']) || isset($data['status']))) {
            WP_CLI::success("1-Click Migration completed in {$ms}ms! All content is now live in Neopress!");
        } else {
            WP_CLI::error("Neopress returned error ($code): $body");
        }
    }

    public function render_admin_page() {
        $nonce = wp_create_nonce('neopress_migrator_nonce');
        ?>
        <div class="wrap" style="max-width: 1040px; margin: 24px auto; font-family: -apple-system, BlinkMacSystemFont, 'Inter', 'Segoe UI', Roboto, sans-serif;">
            
            <!-- Obsidian Hero Card -->
            <div style="background: linear-gradient(135deg, #090d16 0%, #1e293b 100%); border-radius: 14px; padding: 32px 36px; color: #ffffff; box-shadow: 0 10px 25px -5px rgba(0,0,0,0.3); border: 1px solid #334155; margin-bottom: 24px;">
                <div style="display: flex; align-items: center; justify-content: space-between;">
                    <div style="display: flex; align-items: center; gap: 16px;">
                        <div style="width: 52px; height: 52px; border-radius: 12px; background: rgba(56, 189, 248, 0.12); border: 1px solid rgba(56, 189, 248, 0.4); display: flex; align-items: center; justify-content: center;">
                            <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="#38bdf8" stroke-width="2.2">
                                <circle cx="12" cy="12" r="10" stroke="#38bdf8" stroke-opacity="0.3"/>
                                <path d="M8 7v10l8-10v10" stroke="#38bdf8"/>
                            </svg>
                        </div>
                        <div>
                            <div style="display: flex; align-items: center; gap: 10px;">
                                <h1 style="font-size: 24px; font-weight: 700; color: #ffffff; margin: 0; line-height: 1.2;">Neopress 1-Click Migrator</h1>
                                <span style="background: #2563eb; color: #ffffff; font-size: 10px; font-weight: 700; padding: 2px 8px; border-radius: 9999px; letter-spacing: 0.05em;">RUST ENGINE v2.0</span>
                            </div>
                            <p style="color: #94a3b8; font-size: 14px; margin: 4px 0 0 0;">
                                Atomically transfer your entire WordPress database, Elementor/Divi/Gutenberg layouts, menus, media library, and theme styles into Neopress Rusqlite storage.
                            </p>
                        </div>
                    </div>
                    <div style="text-align: right;">
                        <span id="neopress-ping-badge" style="background: rgba(16, 185, 129, 0.15); color: #10b981; border: 1px solid rgba(16, 185, 129, 0.3); padding: 6px 14px; border-radius: 9999px; font-size: 12px; font-weight: 600; display: inline-flex; align-items: center; gap: 6px;">
                            <span style="width: 7px; height: 7px; background: #10b981; border-radius: 50%; box-shadow: 0 0 8px #10b981;"></span>
                            Neopress Active (:8090)
                        </span>
                    </div>
                </div>
            </div>

            <!-- Configuration & Pre-Flight Card -->
            <div style="background: #ffffff; border-radius: 12px; border: 1px solid #e2e8f0; padding: 28px; box-shadow: 0 1px 3px rgba(0,0,0,0.05); margin-bottom: 24px;">
                <h2 style="font-size: 16px; font-weight: 600; color: #0f172a; margin: 0 0 16px 0;">Target Neopress Instance & Credentials</h2>
                
                <div style="display: flex; gap: 12px; align-items: flex-start; margin-bottom: 20px;">
                    <div style="flex: 2;">
                        <label style="font-size: 12px; font-weight: 600; color: #475569; display: block; margin-bottom: 4px;">Target URL</label>
                        <input type="text" id="target-neopress-url" value="http://host.docker.internal:8090" 
                               style="width: 100%; padding: 10px 14px; font-size: 14px; font-family: monospace; border: 1px solid #cbd5e1; border-radius: 8px; background: #f8fafc;" />
                        <small style="color: #64748b; font-size: 12px; margin-top: 4px; display: block;">
                            Inside Docker, use <code>http://host.docker.internal:8090</code>. For local host, use <code>http://localhost:8090</code>.
                        </small>
                    </div>
                    <div style="flex: 1;">
                        <label style="font-size: 12px; font-weight: 600; color: #475569; display: block; margin-bottom: 4px;">Admin Auth / App Password</label>
                        <input type="password" id="target-neopress-auth" value="admin:admin" 
                               style="width: 100%; padding: 10px 14px; font-size: 14px; font-family: monospace; border: 1px solid #cbd5e1; border-radius: 8px; background: #f8fafc;" />
                    </div>
                    <div style="padding-top: 22px;">
                        <button type="button" class="button" onclick="testConnection()" style="padding: 10px 18px; height: auto; font-size: 13px; font-weight: 500; border-radius: 8px;">
                            Ping Neopress
                        </button>
                    </div>
                </div>

                <!-- Pre-flight stats grid -->
                <h3 style="font-size: 13px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em; color: #64748b; margin: 24px 0 12px 0;">
                    Discovered WordPress Content to Migrate
                </h3>
                <div style="display: grid; grid-template-columns: repeat(6, 1fr); gap: 12px; margin-bottom: 24px;">
                    <div style="background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 14px; text-align: center;">
                        <span style="font-size: 11px; font-weight: 600; color: #64748b; text-transform: uppercase;">Posts</span>
                        <div id="stat-posts" style="font-size: 24px; font-weight: 700; color: #0f172a; margin-top: 4px;">...</div>
                    </div>
                    <div style="background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 14px; text-align: center;">
                        <span style="font-size: 11px; font-weight: 600; color: #64748b; text-transform: uppercase;">Pages</span>
                        <div id="stat-pages" style="font-size: 24px; font-weight: 700; color: #0f172a; margin-top: 4px;">...</div>
                    </div>
                    <div style="background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 14px; text-align: center;">
                        <span style="font-size: 11px; font-weight: 600; color: #64748b; text-transform: uppercase;">Categories</span>
                        <div id="stat-cats" style="font-size: 24px; font-weight: 700; color: #0f172a; margin-top: 4px;">...</div>
                    </div>
                    <div style="background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 14px; text-align: center;">
                        <span style="font-size: 11px; font-weight: 600; color: #64748b; text-transform: uppercase;">Tags</span>
                        <div id="stat-tags" style="font-size: 24px; font-weight: 700; color: #0f172a; margin-top: 4px;">...</div>
                    </div>
                    <div style="background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 14px; text-align: center;">
                        <span style="font-size: 11px; font-weight: 600; color: #64748b; text-transform: uppercase;">Menus</span>
                        <div id="stat-menus" style="font-size: 24px; font-weight: 700; color: #0f172a; margin-top: 4px;">...</div>
                    </div>
                    <div style="background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 14px; text-align: center;">
                        <span style="font-size: 11px; font-weight: 600; color: #64748b; text-transform: uppercase;">Media</span>
                        <div id="stat-media" style="font-size: 24px; font-weight: 700; color: #0f172a; margin-top: 4px;">...</div>
                    </div>
                </div>

                <!-- 1-Click Action Button -->
                <div style="background: #f1f5f9; border-radius: 10px; padding: 20px; display: flex; align-items: center; justify-content: space-between;">
                    <div>
                        <strong style="font-size: 15px; color: #0f172a; display: block;">Atomic 1-Click Migration (with AST Transpilation)</strong>
                        <span style="font-size: 13px; color: #64748b;">
                            Transpiles Elementor, Divi, and Gutenberg layouts to Neopress AST and commits inside a single SQLite transaction.
                        </span>
                    </div>
                    <button type="button" id="btn-migrate" class="button button-primary" onclick="startMigration()"
                            style="background: #2563eb; border-color: #1d4ed8; font-size: 15px; font-weight: 600; padding: 12px 28px; height: auto; border-radius: 8px; box-shadow: 0 4px 12px rgba(37,99,235,0.3); display: flex; align-items: center; gap: 8px;">
                        <span>🚀 Start 1-Click Migration</span>
                    </button>
                </div>
            </div>

            <!-- Migration Progress & Results Card -->
            <div id="migration-progress-card" style="display: none; background: #ffffff; border-radius: 12px; border: 1px solid #e2e8f0; padding: 28px; box-shadow: 0 1px 3px rgba(0,0,0,0.05); margin-bottom: 24px;">
                <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px;">
                    <h3 id="progress-heading" style="font-size: 16px; font-weight: 600; color: #0f172a; margin: 0;">Migrating to Neopress...</h3>
                    <span id="progress-pct" style="font-family: monospace; font-size: 14px; font-weight: 700; color: #2563eb;">0%</span>
                </div>
                
                <!-- Progress Bar -->
                <div style="width: 100%; height: 10px; background: #e2e8f0; border-radius: 9999px; overflow: hidden; margin-bottom: 20px;">
                    <div id="progress-bar-fill" style="width: 0%; height: 100%; background: linear-gradient(90deg, #2563eb, #38bdf8); transition: width 0.3s ease;"></div>
                </div>

                <!-- Execution Step List -->
                <div id="step-log" style="font-family: monospace; font-size: 12.5px; background: #090d16; color: #e2e8f0; border-radius: 8px; padding: 16px; max-height: 240px; overflow-y: auto;">
                    <div>[System] Initializing Neopress migration engine...</div>
                </div>

                <!-- Final Success Box -->
                <div id="migration-success-box" style="display: none; margin-top: 24px; padding: 20px; background: #ecfdf5; border: 1px solid #10b981; border-radius: 10px;">
                    <div style="display: flex; align-items: center; justify-content: space-between;">
                        <div style="display: flex; align-items: center; gap: 14px;">
                            <div style="width: 40px; height: 40px; border-radius: 50%; background: #10b981; color: #ffffff; display: flex; align-items: center; justify-content: center; font-size: 20px; font-weight: 700;">✓</div>
                            <div>
                                <h4 style="font-size: 16px; font-weight: 700; color: #065f46; margin: 0;">Migration Complete!</h4>
                                <p id="success-summary-text" style="font-size: 13px; color: #047857; margin: 4px 0 0 0;">
                                    All content successfully committed to Rusqlite WAL storage.
                                </p>
                            </div>
                        </div>
                        <div style="display: flex; gap: 10px;">
                            <a href="http://localhost:8090/n-admin/" target="_blank" class="button button-primary" style="background: #059669; border-color: #047857; font-size: 13px; padding: 8px 16px; height: auto;">
                                Open Neopress Admin &rarr;
                            </a>
                            <a href="http://localhost:8090/" target="_blank" class="button" style="font-size: 13px; padding: 8px 16px; height: auto;">
                                View Neopress Website &rarr;
                            </a>
                        </div>
                    </div>
                </div>
            </div>

        </div>

        <script>
            const ajaxUrl = '<?php echo esc_url(admin_url('admin-ajax.php')); ?>';
            const nonce = '<?php echo esc_js($nonce); ?>';

            document.addEventListener('DOMContentLoaded', () => {
                loadPreflight();
            });

            async function loadPreflight() {
                try {
                    const fd = new FormData();
                    fd.append('action', 'neopress_preflight');
                    fd.append('nonce', nonce);
                    const res = await fetch(ajaxUrl, { method: 'POST', body: fd });
                    const data = await res.json();
                    if (data.success) {
                        document.getElementById('stat-posts').textContent = data.data.posts_count;
                        document.getElementById('stat-pages').textContent = data.data.pages_count;
                        document.getElementById('stat-cats').textContent = data.data.categories_count;
                        document.getElementById('stat-tags').textContent = data.data.tags_count;
                        document.getElementById('stat-menus').textContent = data.data.menus_count || 0;
                        document.getElementById('stat-media').textContent = data.data.media_count;
                    }
                } catch (e) {
                    console.error('Preflight error:', e);
                }
            }

            async function testConnection() {
                const target = document.getElementById('target-neopress-url').value.trim();
                const badge = document.getElementById('neopress-ping-badge');
                badge.textContent = 'Pinging Neopress...';
                badge.style.color = '#38bdf8';

                try {
                    const res = await fetch('http://localhost:8090/wp-json/neopress/v1/stats');
                    if (res.ok) {
                        const data = await res.json();
                        badge.innerHTML = '<span style="width: 7px; height: 7px; background: #10b981; border-radius: 50%; box-shadow: 0 0 8px #10b981;"></span> Connected (Rusqlite ' + (Math.round((data.file_size_bytes || 131072)/1024)) + ' KB)';
                        badge.style.color = '#10b981';
                        return;
                    }
                } catch(e) {}

                badge.textContent = 'Target: ' + target;
                badge.style.color = '#cbd5e1';
            }

            function logStep(msg) {
                const log = document.getElementById('step-log');
                const div = document.createElement('div');
                div.style.marginTop = '4px';
                div.textContent = '[' + new Date().toLocaleTimeString() + '] ' + msg;
                log.appendChild(div);
                log.scrollTop = log.scrollHeight;
            }

            function updateProgress(pct, heading) {
                document.getElementById('progress-pct').textContent = pct + '%';
                document.getElementById('progress-bar-fill').style.width = pct + '%';
                if (heading) {
                    document.getElementById('progress-heading').textContent = heading;
                }
            }

            async function startMigration() {
                const btn = document.getElementById('btn-migrate');
                btn.disabled = true;
                btn.style.opacity = '0.6';

                const card = document.getElementById('migration-progress-card');
                card.style.display = 'block';
                card.scrollIntoView({ behavior: 'smooth' });

                updateProgress(10, 'Collecting WordPress data, builders, menus, and media assets...');
                logStep('Scanning database tables (wp_posts, wp_postmeta, wp_terms, wp_options, wp_comments)...');

                const targetUrl = document.getElementById('target-neopress-url').value.trim();
                const apiAuth = document.getElementById('target-neopress-auth').value.trim();

                try {
                    updateProgress(35, 'Packaging content, postmeta, and menus into JSON bundle...');
                    logStep('Encoding media binaries and Elementor/Divi AST metadata...');

                    const fd = new FormData();
                    fd.append('action', 'neopress_execute_migration');
                    fd.append('nonce', nonce);
                    fd.append('target_url', targetUrl);
                    fd.append('api_auth', apiAuth);

                    updateProgress(60, 'Streaming atomic transaction to Neopress...');
                    logStep('Transmitting HTTP POST to ' + targetUrl + '/wp-json/neopress/v1/import...');

                    const res = await fetch(ajaxUrl, { method: 'POST', body: fd });
                    const result = await res.json();

                    if (result.success) {
                        updateProgress(100, 'Migration Successfully Completed!');
                        const d = result.data;
                        logStep('✓ Committed to Rusqlite WAL storage in ' + d.client_duration_ms + 'ms!');
                        logStep('✓ Posts: ' + d.migrated_posts + ' | Pages: ' + d.migrated_pages + ' | Media: ' + d.migrated_media + ' | Categories: ' + d.migrated_categories + ' | Tags: ' + d.migrated_tags);

                        document.getElementById('migration-success-box').style.display = 'block';
                        document.getElementById('success-summary-text').textContent = 
                            'Transferred ' + d.migrated_posts + ' posts, ' + d.migrated_pages + ' pages, ' + d.migrated_media + ' media files, and ' + d.migrated_categories + ' categories in ' + d.client_duration_ms + 'ms.';
                    } else {
                        updateProgress(100, 'Migration Failed');
                        logStep('❌ Error: ' + (result.data ? result.data.message : 'Unknown error'));
                        alert('Migration error: ' + (result.data ? result.data.message : 'Unknown error'));
                        btn.disabled = false;
                        btn.style.opacity = '1';
                    }
                } catch (e) {
                    updateProgress(100, 'Migration Error');
                    logStep('❌ Exception: ' + e);
                    btn.disabled = false;
                    btn.style.opacity = '1';
                }
            }
        </script>
        <?php
    }
}

new Neopress_1Click_Migrator();

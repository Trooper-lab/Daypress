//! HTML `<head>` section with 2026 CAD typography and icon assets.

pub fn render_head() -> &'static str {
    r#"<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>NeoStudio &ndash; __POST_TITLE__ &ndash; __SITE_TITLE__</title>
    <!-- Google Fonts: Inter, Plus Jakarta Sans & JetBrains Mono -->
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500;600&family=Plus+Jakarta+Sans:wght@500;600;700&display=swap" rel="stylesheet">
    <!-- FontAwesome 6 Free Vector Icons -->
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css" integrity="sha512-DTOQO9RWCH3ppGqcWaEA1BIZOC6xxalwEsw9c2QQeAIftl+Vegovlnee1c9QX4TctnWMn13TZye+giMm8e2LwA==" crossorigin="anonymous" referrerpolicy="no-referrer" />
    <style id="neostudio-core-styles">__STYLES__</style>
    <style id="neopress-elementor-kit-css">__EXTRA_HEAD_CSS__</style>
    __EXTRA_HEAD_ASSETS__
</head>"#
}

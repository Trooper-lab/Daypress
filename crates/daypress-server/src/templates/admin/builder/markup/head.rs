//! HTML `<head>` section with icon assets; the brand typefaces are self-hosted (`DESK_FONT_FACES`).

pub fn render_head() -> &'static str {
    r#"<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>DayStudio &ndash; __POST_TITLE__ &ndash; __SITE_TITLE__</title>
    <!-- FontAwesome 6 Free Vector Icons -->
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css" integrity="sha512-DTOQO9RWCH3ppGqcWaEA1BIZOC6xxalwEsw9c2QQeAIftl+Vegovlnee1c9QX4TctnWMn13TZye+giMm8e2LwA==" crossorigin="anonymous" referrerpolicy="no-referrer" />
    <script>window.wpApiSettings = { root: "/wp-json/", nonce: "__REST_NONCE__", versionString: "wp/v2/" };</script>
    <style id="daystudio-core-styles">__STYLES__</style>
    <style id="daystudio-kit-css">__EXTRA_HEAD_CSS__</style>
    __EXTRA_HEAD_ASSETS__
</head>"#
}

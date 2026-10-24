//! 2026 CAD Design Tokens, CSS variables, typography and reset (`styles/tokens.rs`).

pub const CSS_TOKENS: &str = r#"
:root {
    /* -----------------------------------------------------------------
       5 NAMED HEX TOKENS + CORE ACCENTS
       ----------------------------------------------------------------- */
    --neo-chassis:         #0D0F12; /* Obsidian Chassis: primary studio shell, sidebars, rail */
    --neo-chassis-subtle:  #080A0D; /* Deep CAD canvas background underneath artboard */
    --neo-chassis-hover:   #12151A; /* Subtle hover state on chassis rail/topbar buttons */
    
    --neo-surface:         #15181E; /* Graphite Surface: inspector panels, card headers, inputs */
    --neo-surface-hover:   #1A1F26; /* Hover state for surface buttons and interactive pills */
    --neo-surface-active:  #202630; /* Active / pressed state for tools */
    --neo-surface-inset:   #0F1216; /* Recessed inputs, scrubber wells, code blocks */
    
    --neo-rule:            #242933; /* Hairline Rule: crisp 1px structural grid dividers */
    --neo-rule-subtle:     #1C2028; /* Subdued inner borders within control rows */
    --neo-rule-focus:      #3B4455; /* Focus border for inputs */
    
    --neo-ink:             #F2F4F8; /* Alabaster Ink: primary studio text, high-contrast labels */
    --neo-ink-bright:      #FFFFFF; /* Pure white highlights */
    --neo-ink-contrast:    #0D0F12; /* Contrast text on signal / amber badges */
    
    --neo-muted:           #8A93A6; /* Slate Muted: secondary studio labels, inactive states */
    --neo-muted-dark:      #636D80; /* Disabled text, placeholder hints */
    --neo-muted-light:     #A5ADC0; /* Secondary hover labels */

    /* -----------------------------------------------------------------
       SIGNAL & INHERITANCE TOKENS
       ----------------------------------------------------------------- */
    --neo-signal:          #2563EB; /* Cobalt Signal: active node selection, local overrides */
    --neo-signal-hover:    #1D4ED8;
    --neo-signal-light:    #3B82F6;
    --neo-signal-tint:     rgba(37, 99, 235, 0.14); /* Box-model padding hover fill */
    --neo-signal-border:   rgba(37, 99, 235, 0.40);

    --neo-token:           #F59E0B; /* Amber Token: inherited classes, design tokens, margin guides */
    --neo-token-hover:     #D97706;
    --neo-token-tint:      rgba(245, 158, 11, 0.16); /* Box-model margin hover fill */
    --neo-token-border:    rgba(245, 158, 11, 0.40);

    /* -----------------------------------------------------------------
       FUNCTIONAL STATUS ACCENTS
       ----------------------------------------------------------------- */
    --neo-emerald:         #10B981; /* Saved to SQLite status, success validation */
    --neo-emerald-tint:    rgba(16, 185, 129, 0.15);
    --neo-crimson:         #EF4444; /* Delete actions, destructive alerts, error state */
    --neo-crimson-tint:    rgba(239, 68, 68, 0.15);
    --neo-violet:          #8B5CF6; /* Master symbols, theme builder templates */
    --neo-violet-tint:     rgba(139, 92, 246, 0.15);
    --neo-cyan:            #06B6D4; /* X-Ray wireframe outline mode, grid crosshairs */

    /* -----------------------------------------------------------------
       TYPOGRAPHY
       ----------------------------------------------------------------- */
    --neo-font-ui:         "Inter", "Plus Jakarta Sans", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    --neo-font-mono:       "JetBrains Mono", "SF Mono", Consolas, monospace;

    --neo-text-micro:      10px;    /* Status pills, tag badges */
    --neo-text-sub:        11px;    /* Section titles, shortcut keys */
    --neo-text-base:       12px;    /* Inspector property labels, dropdown items */
    --neo-text-ui:         12.5px;  /* Topbar navigation, buttons */
    --neo-text-title:      13px;    /* Panel headers, brand text */

    /* -----------------------------------------------------------------
       HIGH-DENSITY CAD METRICS
       ----------------------------------------------------------------- */
    --neo-topbar-h:        40px;
    --neo-rail-w:          48px;
    --neo-drawer-w:        280px;
    --neo-inspector-w:     300px;
    --neo-breadcrumb-h:    28px;
    --neo-control-h:       26px;    /* Standard CAD control row height */
    --neo-control-sm-h:    22px;    /* Micro pills, unit buttons */
    --neo-radius-sm:       3px;     /* Crisp industrial radius */
    --neo-radius-md:       4px;     /* Container corner radius */

    /* -----------------------------------------------------------------
       Z-INDEX ELEVATION HIERARCHY
       ----------------------------------------------------------------- */
    --neo-z-canvas:        1;
    --neo-z-artboard-handles: 10;
    --neo-z-overlays:      20;
    --neo-z-badge:         30;
    --neo-z-dock:          100;
    --neo-z-dropdown:      200;
    --neo-z-modal:         500;
    --neo-z-palette:       1000;
    --neo-z-toast:         2000;
}

*, *::before, *::after {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

/* Root stays at the browser default (16px) so `rem` values on the canvas resolve exactly like on
   the public site; the studio UI itself is sized in px via body. */
html {
    font-size: 16px;
}

html, body {
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    background: var(--neo-chassis);
    color: var(--neo-ink);
    font-family: var(--neo-font-ui);
    line-height: 1.4;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
}

body {
    font-size: var(--neo-text-ui);
}

/* Crisp CAD dark scrollbars */
::-webkit-scrollbar {
    width: 6px;
    height: 6px;
}
::-webkit-scrollbar-track {
    background: var(--neo-chassis);
}
::-webkit-scrollbar-thumb {
    background: var(--neo-rule);
    border-radius: 3px;
}
::-webkit-scrollbar-thumb:hover {
    background: var(--neo-muted-dark);
}
"#;

//! DayStudio design tokens in the Daypress Night edition, typography and reset (`styles/tokens.rs`).

pub const CSS_TOKENS: &str = r#"
:root {
    /* -----------------------------------------------------------------
       5 NAMED HEX TOKENS + CORE ACCENTS
       ----------------------------------------------------------------- */
    --day-chassis:         #111214; /* Night paper: primary studio shell, sidebars, rail */
    --day-chassis-subtle:  #0A0B0C; /* Night paper-sunk: canvas underneath the artboard */
    --day-chassis-hover:   #1A1C1F; /* Hover state on chassis rail/topbar buttons */
    
    --day-surface:         #1A1C1F; /* Night paper-raised: inspector panels, card headers, inputs */
    --day-surface-hover:   #222428; /* Hover state for surface buttons and interactive pills */
    --day-surface-active:  #2A2D31; /* Active / pressed state for tools */
    --day-surface-inset:   #0A0B0C; /* Recessed inputs, scrubber wells, code blocks */
    
    --day-rule:            #393836; /* Hairline rule: 1px structural grid dividers */
    --day-rule-subtle:     #26272A; /* Subdued inner borders within control rows */
    --day-rule-focus:      #5C5A55; /* Focus border for inputs */
    
    --day-ink:             #EFE9DE; /* Night ink: primary studio text, high-contrast labels */
    --day-ink-bright:      #FBF8F2; /* Brightest highlights */
    --day-ink-contrast:    #16171A; /* Text on signal / ochre fills */
    
    --day-muted:           #A7A297; /* Night ink-muted: secondary labels, inactive states */
    --day-muted-dark:      #7C7870; /* Disabled text, placeholder hints */
    --day-muted-light:     #C4BFB4; /* Secondary hover labels */

    /* -----------------------------------------------------------------
       SIGNAL & INHERITANCE TOKENS
       ----------------------------------------------------------------- */
    --day-signal:          #93A1FF; /* Night press: active node selection, local overrides */
    --day-signal-hover:    #B3BDFF;
    --day-signal-light:    #B3BDFF;
    --day-signal-tint:     rgba(147, 161, 255, 0.16); /* Box-model padding hover fill */
    --day-signal-border:   rgba(147, 161, 255, 0.45);

    --day-token:           #E6B84A; /* Night ochre: inherited classes, design tokens, margin guides */
    --day-token-hover:     #D9A42A;
    --day-token-tint:      rgba(230, 184, 74, 0.16); /* Box-model margin hover fill */
    --day-token-border:    rgba(230, 184, 74, 0.42);

    /* -----------------------------------------------------------------
       FUNCTIONAL STATUS ACCENTS
       ----------------------------------------------------------------- */
    --day-emerald:         #93B98A; /* Night moss: saved to SQLite status, success validation */
    --day-emerald-tint:    rgba(147, 185, 138, 0.15);
    --day-crimson:         #E8744A; /* Night oxide: delete actions, destructive alerts, error state */
    --day-crimson-tint:    rgba(232, 116, 74, 0.15);
    --day-oxide:           #E8744A; /* Night oxide: the one primary action (Publish) */
    --day-violet:          #C2C9FF; /* Light press: master symbols, theme builder templates */
    --day-violet-tint:     rgba(194, 201, 255, 0.15);
    --day-cyan:            #06B6D4; /* X-Ray wireframe outline mode, grid crosshairs */

    /* -----------------------------------------------------------------
       TYPOGRAPHY
       ----------------------------------------------------------------- */
    --day-font-ui:         "Schibsted Grotesk", "Helvetica Neue", Arial, sans-serif;
    --day-font-mono:       "IBM Plex Mono", ui-monospace, Menlo, monospace;

    --day-text-micro:      10px;    /* Status pills, tag badges */
    --day-text-sub:        11px;    /* Section titles, shortcut keys */
    --day-text-base:       12px;    /* Inspector property labels, dropdown items */
    --day-text-ui:         12.5px;  /* Topbar navigation, buttons */
    --day-text-title:      13px;    /* Panel headers, brand text */

    /* -----------------------------------------------------------------
       HIGH-DENSITY CAD METRICS
       ----------------------------------------------------------------- */
    --day-topbar-h:        40px;
    --day-rail-w:          48px;
    --day-drawer-w:        280px;
    --day-inspector-w:     300px;
    --day-breadcrumb-h:    28px;
    --day-control-h:       26px;    /* Standard CAD control row height */
    --day-control-sm-h:    22px;    /* Micro pills, unit buttons */
    --day-radius-sm:       3px;     /* Crisp industrial radius */
    --day-radius-md:       4px;     /* Container corner radius */

    /* -----------------------------------------------------------------
       Z-INDEX ELEVATION HIERARCHY
       ----------------------------------------------------------------- */
    --day-z-canvas:        1;
    --day-z-artboard-handles: 10;
    --day-z-overlays:      20;
    --day-z-badge:         30;
    --day-z-dock:          100;
    --day-z-dropdown:      200;
    --day-z-modal:         500;
    --day-z-palette:       1000;
    --day-z-toast:         2000;
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
    background: var(--day-chassis);
    color: var(--day-ink);
    font-family: var(--day-font-ui);
    line-height: 1.4;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
}

body {
    font-size: var(--day-text-ui);
}

/* Crisp CAD dark scrollbars */
::-webkit-scrollbar {
    width: 6px;
    height: 6px;
}
::-webkit-scrollbar-track {
    background: var(--day-chassis);
}
::-webkit-scrollbar-thumb {
    background: var(--day-rule);
    border-radius: 3px;
}
::-webkit-scrollbar-thumb:hover {
    background: var(--day-muted-dark);
}
"#;

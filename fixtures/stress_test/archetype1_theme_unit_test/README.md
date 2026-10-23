# Archetype 1: WordPress Theme Unit Test Fixture

## Overview
This fixture exercises:
- 3+ level hierarchical navigation menu trees
- Core Gutenberg blocks (`core/columns`, `core/gallery`, `core/quote`, `core/list`, `core/embed`, `core/table`, `core/buttons`)
- Sticky post query ordering (older date prioritized over newer non-sticky posts)
- 2-pass threaded comments (`comment_parent` remapping)
- Unicode UTF-8 slugification (Cyrillic `привет-мир` and Japanese `こんにちは-世界`)

---
name: php-to-rust-transpiler
description: >-
  Run the Daypress Golden Master PHP-to-Rust transpilation and verification loop.
  Use this skill whenever translating WordPress core or plugin PHP functions/classes
  into verified Rust crates in crates/generated_* or crates/daypress-transpiler.
---

# Daypress PHP-to-Rust Golden Master Transpiler Skill

This skill defines the end-to-end agentic workflow for translating legacy WordPress PHP functions and classes into mathematically verified Rust code.

## Workflow Steps

### Step 1: Isolate the Target PHP Function or Class in `fixtures/`
Place the target PHP source file in `fixtures/` (e.g. `fixtures/slugify.php` or `fixtures/wp-includes/fonts/class-wp-font-utils.php`) and define an inputs vector JSON file (e.g. `fixtures/slugify_inputs.json`) covering edge cases:
- ASCII, Unicode / accented characters (`Café`, `Über`, `日本語`), HTML entities, tags, empty strings, nested arrays, and boundary numbers.

### Step 2: Generate the Golden Master Baseline via PHP 8.3
Generate the canonical input/output expectations by executing the PHP code in the reference PHP runtime (local `php` or the running `daypress-ref-wp` Docker container):

```powershell
# Using Node runner (auto-detects local PHP or docker exec daypress-ref-wp):
node generate_golden_master.js --file fixtures/slugify.php --func slugify --inputs fixtures/slugify_inputs.json --output crates/generated_slugify/golden_master.json
```

### Step 3: Run `daypress-transpiler` Verification Loop
Execute the Rust transpiler & verification harness:

```powershell
cargo run -p daypress-transpiler -- --php-file fixtures/slugify.php --golden-master crates/generated_slugify/golden_master.json --output-crate crates/generated_slugify
```

`daypress-transpiler` will:
1. Parse the PHP source and Golden Master test vectors.
2. Synthesize safe, idiomatic Rust code + a `#[cfg(test)]` Golden Master test module in the target crate.
3. Run `cargo test --manifest-path <output-crate>/Cargo.toml`.
4. If any assertion or compiler error occurs, iteratively self-repair up to `--max-retries` until 100% of Golden Master vectors pass.

### Step 4: Wire Verified Crate into `daypress-server` or `daypress-ast`
Import the verified function into `crates/daypress-server` or `crates/daypress-ast` and run `cargo test --workspace`.

<?php
/**
 * PHP Golden Master Generator
 *
 * Usage:
 *   generate_golden_master <target.php> <inputs.json> [golden_master.json]
 *   generate_golden_master --target <target.php> --inputs <inputs.json> [--output <golden_master.json>] [--func <callable>]
 *   generate_golden_master --file <target.php> --func <callable> --inputs <inputs.json> --output <golden_master.json>
 */

$target_file = null;
$inputs_arg = null;
$output_file = 'golden_master.json';
$func_arg = null;

$args = array_slice($argv, 1);
$positional = [];

for ($i = 0; $i < count($args); $i++) {
    $arg = $args[$i];
    if ($arg === '--target' || $arg === '-t' || $arg === '--file' || $arg === '-f') {
        $target_file = $args[++$i] ?? null;
    } elseif ($arg === '--inputs' || $arg === '-i') {
        $inputs_arg = $args[++$i] ?? null;
    } elseif ($arg === '--output' || $arg === '-o') {
        $output_file = $args[++$i] ?? 'golden_master.json';
    } elseif ($arg === '--func') {
        $func_arg = $args[++$i] ?? null;
    } elseif ($arg === '--help' || $arg === '-h') {
        echo "Usage: generate_golden_master <target.php> <inputs.json> [golden_master.json]\n";
        echo "Options:\n";
        echo "  -t, --target, -f, --file  Path to the target PHP file\n";
        echo "  --func                    Function or static method (e.g. slugify or WP_Font_Utils::sanitize_font_family)\n";
        echo "  -i, --inputs              Path to JSON file of inputs (or raw JSON array string)\n";
        echo "  -o, --output              Path to output golden master JSON (default: golden_master.json)\n";
        exit(0);
    } else {
        $positional[] = $arg;
    }
}

if (!$target_file && isset($positional[0])) {
    $target_file = $positional[0];
}
if (!$inputs_arg && isset($positional[1])) {
    $inputs_arg = $positional[1];
}
if ($output_file === 'golden_master.json' && isset($positional[2])) {
    $output_file = $positional[2];
}
if (!$func_arg && isset($positional[3])) {
    $func_arg = $positional[3];
}

if (!$target_file || !$inputs_arg) {
    fwrite(STDERR, "Error: Target PHP file and inputs are required.\n");
    fwrite(STDERR, "Usage: generate_golden_master <target.php> <inputs.json> [golden_master.json]\n");
    exit(1);
}

if (!file_exists($target_file)) {
    fwrite(STDERR, "Error: Target PHP file not found at: {$target_file}\n");
    exit(1);
}

// Load inputs
$inputs_raw = file_exists($inputs_arg) ? file_get_contents($inputs_arg) : $inputs_arg;
$inputs = json_decode($inputs_raw, true);
if (!is_array($inputs)) {
    fwrite(STDERR, "Error: Inputs must be a valid JSON array.\n");
    exit(1);
}

// Optionally load WordPress environment if available
if (file_exists('/var/www/html/wp-load.php')) {
    @require_once '/var/www/html/wp-load.php';
}

// Inspect target file for already-loaded core classes/functions to prevent redeclaration fatal error
$target_source = file_get_contents($target_file);
$already_loaded = false;
if (preg_match('/^\s*(?:final\s+|abstract\s+)?class\s+([A-Za-z_][A-Za-z0-9_]*)/m', $target_source, $class_match)) {
    if (class_exists($class_match[1], false)) {
        $already_loaded = true;
    }
}

$funcs_before = get_defined_functions()['user'];
$classes_before = get_declared_classes();
$ret = null;
if (!$already_loaded) {
    $ret = require_once $target_file;
}
$funcs_after = get_defined_functions()['user'];
$classes_after = get_declared_classes();
$new_funcs = array_values(array_diff($funcs_after, $funcs_before));
$new_classes = array_values(array_diff($classes_after, $classes_before));

// Determine callable function, static class method, or closure
$callable = null;
if ($func_arg !== null && $func_arg !== '') {
    if (str_contains($func_arg, '::')) {
        $parts = explode('::', $func_arg, 2);
        $callable = [$parts[0], $parts[1]];
    } elseif (is_callable($func_arg)) {
        $callable = $func_arg;
    } elseif (!empty($new_classes) && is_callable([end($new_classes), $func_arg])) {
        $callable = [end($new_classes), $func_arg];
    } elseif (!empty($class_match[1]) && is_callable([$class_match[1], $func_arg])) {
        $callable = [$class_match[1], $func_arg];
    }
}

if ($callable === null) {
    if (is_callable($ret)) {
        $callable = $ret;
    } else {
        $target_basename = pathinfo($target_file, PATHINFO_FILENAME);
        if (function_exists($target_basename)) {
            $callable = $target_basename;
        } elseif (!empty($new_funcs)) {
            $callable = end($new_funcs);
        } elseif (!empty($new_classes)) {
            $cls = new ReflectionClass(end($new_classes));
            foreach ($cls->getMethods(ReflectionMethod::IS_PUBLIC | ReflectionMethod::IS_STATIC) as $method) {
                $callable = [$cls->getName(), $method->getName()];
                break;
            }
        } elseif (!empty($class_match[1]) && class_exists($class_match[1], false)) {
            $cls = new ReflectionClass($class_match[1]);
            foreach ($cls->getMethods(ReflectionMethod::IS_PUBLIC | ReflectionMethod::IS_STATIC) as $method) {
                $callable = [$cls->getName(), $method->getName()];
                break;
            }
        }
    }
}

$ref = null;
if ($callable !== null) {
    if (is_array($callable)) {
        $ref = new ReflectionMethod($callable[0], $callable[1]);
    } elseif (is_string($callable) && str_contains($callable, '::')) {
        $parts = explode('::', $callable, 2);
        $ref = new ReflectionMethod($parts[0], $parts[1]);
    } else {
        $ref = new ReflectionFunction($callable);
    }
}

$golden_records = [];

foreach ($inputs as $input) {
    $expected_output = null;

    if ($callable !== null) {
        if (is_array($input) && !array_is_list($input) && array_key_exists('args', $input) && is_array($input['args'])) {
            $expected_output = call_user_func_array($callable, $input['args']);
        } elseif (is_array($input) && array_is_list($input) && $ref !== null && $ref->getNumberOfParameters() > 1 && count($input) <= $ref->getNumberOfParameters()) {
            $expected_output = call_user_func_array($callable, $input);
        } else {
            $expected_output = call_user_func($callable, $input);
        }
    } else {
        // Fallback: evaluate file per input with $input available in scope
        $executor = function ($__file, $input) {
            return include $__file;
        };
        $expected_output = $executor($target_file, $input);
    }

    $golden_records[] = [
        "input" => $input,
        "expected_output" => $expected_output
    ];
}

$json_output = json_encode($golden_records, JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE);

file_put_contents($output_file, $json_output);
echo "Successfully generated golden master with " . count($golden_records) . " records to: {$output_file}\n";

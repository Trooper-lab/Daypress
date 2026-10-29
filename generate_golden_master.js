#!/usr/bin/env node
const fs = require('fs');
const path = require('path');
const { execSync, spawnSync } = require('child_process');

function parseArgs() {
  const args = process.argv.slice(2);
  let target = null;
  let inputs = null;
  let output = 'golden_master.json';
  let func = null;
  const positional = [];

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === '--target' || arg === '-t' || arg === '--file' || arg === '-f') {
      target = args[++i];
    } else if (arg === '--inputs' || arg === '-i') {
      inputs = args[++i];
    } else if (arg === '--output' || arg === '-o') {
      output = args[++i];
    } else if (arg === '--func') {
      func = args[++i];
    } else if (arg === '--help' || arg === '-h') {
      console.log('Usage: generate_golden_master <target.php> <inputs.json> [golden_master.json]');
      console.log('Options:');
      console.log('  -t, --target, -f, --file  Path to the target PHP file');
      console.log('  --func                    Function or static method (e.g. slugify or WP_Font_Utils::sanitize_font_family)');
      console.log('  -i, --inputs              Path to JSON file of inputs (or raw JSON array string)');
      console.log('  -o, --output              Path to output golden master JSON (default: golden_master.json)');
      process.exit(0);
    } else {
      positional.push(arg);
    }
  }

  if (!target && positional[0]) target = positional[0];
  if (!inputs && positional[1]) inputs = positional[1];
  if (output === 'golden_master.json' && positional[2]) output = positional[2];
  if (!func && positional[3]) func = positional[3];

  return { target, inputs, output, func };
}

const { target, inputs, output, func } = parseArgs();

if (!target || !inputs) {
  console.error('Error: target PHP file and inputs JSON are required.');
  console.error('Usage: generate_golden_master <target.php> <inputs.json> [golden_master.json]');
  process.exit(1);
}

const resolvedTarget = path.resolve(target);
if (!fs.existsSync(resolvedTarget)) {
  console.error(`Error: target PHP file not found: ${resolvedTarget}`);
  process.exit(1);
}

let resolvedInputs = inputs;
let tmpInputsPath = null;
if (fs.existsSync(inputs)) {
  resolvedInputs = path.resolve(inputs);
} else {
  // If raw JSON was passed, write to a temp file
  tmpInputsPath = path.join(__dirname, '.tmp_inputs.json');
  fs.writeFileSync(tmpInputsPath, inputs);
  resolvedInputs = tmpInputsPath;
}

const resolvedOutput = path.resolve(output);
const phpScriptPath = path.join(__dirname, 'generate_golden_master.php');

let exitCode = 0;

try {
  // 1. Check if local php exists
  let hasLocalPhp = false;
  try {
    execSync('php -v', { stdio: 'ignore' });
    hasLocalPhp = true;
  } catch (e) {
    hasLocalPhp = false;
  }

  if (hasLocalPhp) {
    const phpArgs = [phpScriptPath, '--target', resolvedTarget, '--inputs', resolvedInputs, '--output', resolvedOutput];
    if (func) {
      phpArgs.push('--func', func);
    }
    const res = spawnSync('php', phpArgs, {
      stdio: 'inherit'
    });
    exitCode = res.status || 0;
  } else {
    // 2. Fallback: Execute inside Docker container daypress-ref-wp
    const container = 'daypress-ref-wp';
    const remoteTarget = '/tmp/target.php';
    const remoteInputs = '/tmp/inputs.json';
    const remoteRunner = '/tmp/generate_golden_master.php';
    const remoteOutput = '/tmp/golden_master.json';

    // Copy files to container
    execSync(`docker cp "${resolvedTarget}" ${container}:${remoteTarget}`, { stdio: 'pipe' });
    execSync(`docker cp "${resolvedInputs}" ${container}:${remoteInputs}`, { stdio: 'pipe' });
    execSync(`docker cp "${phpScriptPath}" ${container}:${remoteRunner}`, { stdio: 'pipe' });

    const dockerArgs = [
      'exec', container, 'php', remoteRunner,
      '--target', remoteTarget,
      '--inputs', remoteInputs,
      '--output', remoteOutput
    ];
    if (func) {
      dockerArgs.push('--func', func);
    }

    // Run PHP inside container
    const execRes = spawnSync('docker', dockerArgs, { stdio: 'inherit' });

    if (execRes.status !== 0) {
      exitCode = execRes.status || 1;
    } else {
      // Copy output back
      execSync(`docker cp ${container}:${remoteOutput} "${resolvedOutput}"`, { stdio: 'pipe' });
      console.log(`[generate_golden_master] Golden master written to ${resolvedOutput}`);
    }
  }
} catch (err) {
  console.error('Execution failed:', err.message);
  exitCode = 1;
} finally {
  if (tmpInputsPath && fs.existsSync(tmpInputsPath)) {
    try {
      fs.unlinkSync(tmpInputsPath);
    } catch (_) {
      // ignore cleanup error
    }
  }
}

process.exit(exitCode);

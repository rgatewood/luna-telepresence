#!/usr/bin/env node
/**
 * Auto-detect GPU and run Tauri with appropriate features
 */

const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');
const os = require('os');

// Get the command (dev or build)
const command = process.argv[2];
if (!command || !['dev', 'build'].includes(command)) {
  console.error('Usage: node tauri-auto.js [dev|build]');
  process.exit(1);
}

// Detect GPU feature
let feature = '';

// Check for environment variable override first
if (process.env.TAURI_GPU_FEATURE) {
  feature = process.env.TAURI_GPU_FEATURE;
  console.log(`🔧 Using forced GPU feature from environment: ${feature}`);
} else {
  try {
    const result = execSync('node scripts/auto-detect-gpu.js', {
      encoding: 'utf8',
      stdio: ['pipe', 'pipe', 'inherit']
    });
    feature = result.trim();
  } catch (err) {
    // If detection fails, continue with no features
  }
}

console.log(''); // Empty line for spacing

// Platform-specific environment variables
const platform = os.platform();
const env = { ...process.env };

if (platform === 'win32') {
  const llvmBin = 'C:\\Program Files\\LLVM\\bin';
  const cmakeBin = 'C:\\Program Files\\CMake\\bin';

  if (!env.LIBCLANG_PATH && fs.existsSync(path.join(llvmBin, 'libclang.dll'))) {
    env.LIBCLANG_PATH = llvmBin;
  }

  env.PATH = [llvmBin, cmakeBin, env.PATH].filter(Boolean).join(path.delimiter);
}

if (platform === 'linux' && feature === 'cuda') {
  console.log('🐧 Linux/CUDA detected: Setting CMAKE flags for NVIDIA GPU');
  env.CMAKE_CUDA_ARCHITECTURES = '75';
  env.CMAKE_CUDA_STANDARD = '17';
  env.CMAKE_POSITION_INDEPENDENT_CODE = 'ON';
}

function buildSidecar() {
  const repoRoot = path.resolve(__dirname, '..', '..');
  const frontendRoot = path.resolve(__dirname, '..');
  const profile = command === 'build' ? 'release' : 'debug';
  const extension = platform === 'win32' ? '.exe' : '';
  const rustInfo = execSync('rustc -vV', { encoding: 'utf8', env });
  const hostLine = rustInfo.split(/\r?\n/).find((line) => line.startsWith('host: '));

  if (!hostLine) {
    throw new Error('Unable to determine the Rust host target from rustc -vV');
  }

  const target = hostLine.slice('host: '.length).trim();
  const cargoArgs = ['build', '-p', 'llama-helper'];
  const sidecarFeatures = new Set(['metal', 'cuda', 'vulkan']);

  if (command === 'build') {
    cargoArgs.push('--release');
  }

  if (sidecarFeatures.has(feature)) {
    cargoArgs.push('--features', feature);
  }

  console.log(`Building llama-helper sidecar (${profile}, ${target})...`);
  execSync(`cargo ${cargoArgs.join(' ')}`, { cwd: repoRoot, stdio: 'inherit', env });

  const source = path.join(repoRoot, 'target', profile, `llama-helper${extension}`);
  const binariesDir = path.join(frontendRoot, 'src-tauri', 'binaries');
  const destination = path.join(binariesDir, `llama-helper-${target}${extension}`);

  if (!fs.existsSync(source)) {
    throw new Error(`llama-helper build completed without producing ${source}`);
  }

  fs.mkdirSync(binariesDir, { recursive: true });
  fs.copyFileSync(source, destination);
  console.log(`Sidecar ready: ${destination}`);
}

try {
  buildSidecar();
} catch (err) {
  console.error(`Failed to prepare llama-helper: ${err.message}`);
  process.exit(1);
}

// Build the tauri command
let tauriCmd = `tauri ${command}`;
if (feature && feature !== 'none') {
  tauriCmd += ` -- --features ${feature}`;
  console.log(`🚀 Running: tauri ${command} with features: ${feature}`);
} else {
  console.log(`🚀 Running: tauri ${command} (CPU-only mode)`);
}
console.log('');

// Execute the command
try {
  execSync(tauriCmd, { stdio: 'inherit', env });
} catch (err) {
  process.exit(err.status || 1);
}

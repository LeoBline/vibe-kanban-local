import { execSync, spawn } from 'child_process';
import path from 'path';
import fs from 'fs';
import { fileURLToPath } from 'url';
import dotenv from 'dotenv';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Load .env if it exists
if (fs.existsSync(path.join(__dirname, '..', '.env'))) {
  dotenv.config({ path: path.join(__dirname, '..', '.env') });
}

function getEnvValue(key) {
  try {
    return execSync(`node ${path.join(__dirname, 'setup-dev-environment.js')} ${key}`, { encoding: 'utf8' }).trim();
  } catch (e) {
    console.error(`Failed to get env value for ${key}:`, e.message);
    process.exit(1);
  }
}

const frontendPort = getEnvValue('frontend');
const backendPort = getEnvValue('backend');

const env = {
  ...process.env,
  FRONTEND_PORT: frontendPort,
  BACKEND_PORT: backendPort,
  VK_ALLOWED_ORIGINS: `http://localhost:${frontendPort}`,
  DISABLE_WORKTREE_CLEANUP: '1',
  RUST_LOG: 'debug',
  VITE_VK_SHARED_API_BASE: process.env.VITE_VK_SHARED_API_BASE || '',
};

console.log(`🚀 Starting Vibe Kanban (Tauri Dev)...`);
console.log(`   Frontend Port: ${frontendPort}`);
console.log(`   Backend Port: ${backendPort}`);

const cargoCmd = process.platform === 'win32' ? 'cargo.exe' : 'cargo';

const child = spawn(cargoCmd, ['tauri', 'dev'], {
  cwd: path.join(__dirname, '..', 'crates', 'tauri-app'),
  stdio: 'inherit',
  env,
  shell: true
});

child.on('exit', (code) => {
  process.exit(code || 0);
});

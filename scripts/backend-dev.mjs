import { execSync, spawn } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

function getEnvValue(key) {
  try {
    return execSync(`node ${path.join(__dirname, 'setup-dev-environment.js')} ${key}`, { encoding: 'utf8' }).trim();
  } catch (e) {
    console.error(`Failed to get env value for ${key}:`, e.message);
    process.exit(1);
  }
}

const backendPort = getEnvValue('backend');

const env = {
  ...process.env,
  BACKEND_PORT: backendPort,
  DISABLE_WORKTREE_CLEANUP: '1',
  RUST_LOG: 'debug',
};

console.log(`🚀 Starting Vibe Kanban Backend...`);
console.log(`   Backend Port: ${backendPort}`);

const pnpmCmd = process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm';

const child = spawn(pnpmCmd, ['run', 'backend:dev:watch'], {
  stdio: 'inherit',
  env,
  shell: true
});

child.on('exit', (code) => {
  process.exit(code || 0);
});

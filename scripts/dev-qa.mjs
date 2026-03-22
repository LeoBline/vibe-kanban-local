import { execSync } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';
import concurrently from 'concurrently';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

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
const previewProxyPort = getEnvValue('preview_proxy');

const env = {
  ...process.env,
  FRONTEND_PORT: frontendPort,
  BACKEND_PORT: backendPort,
  PREVIEW_PROXY_PORT: previewProxyPort,
  VK_ALLOWED_ORIGINS: `http://localhost:${frontendPort}`,
  VITE_VK_SHARED_API_BASE: process.env.VK_SHARED_API_BASE || '',
  DISABLE_WORKTREE_CLEANUP: '1',
  RUST_LOG: 'debug',
};

console.log(`🚀 Starting Vibe Kanban (QA Mode)...`);
console.log(`   Frontend Port: ${frontendPort}`);
console.log(`   Backend Port: ${backendPort}`);
console.log(`   Preview Proxy Port: ${previewProxyPort}`);

const pnpmCmd = process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm';

const { result } = concurrently(
  [
    { 
      command: `${pnpmCmd} run backend:dev:watch:qa`, 
      name: 'backend',
      prefixColor: 'blue',
      env
    },
    { 
      command: `${pnpmCmd} run local-web:dev -- --port ${frontendPort}`, 
      name: 'frontend',
      prefixColor: 'green',
      env
    },
  ],
  {
    prefix: 'name',
    killOthers: ['failure'],
    restartTries: 0,
  }
);

result.catch((e) => {
  // Concurrently handles logging errors, just ensure we exit with the right code
  process.exit(1);
});

#!/usr/bin/env node
// Orchestrates the web healthcheck: starts serve.mjs, runs playwright.mjs, stops the server.
import { spawn } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const PORT = Number(process.env.PORT || 8765);

function runScript(scriptPath, args = []) {
  return new Promise((resolve, reject) => {
    const child = spawn(process.execPath, [scriptPath, ...args], {
      stdio: 'inherit',
      env: { ...process.env },
    });
    child.on('close', (code) => {
      if (code === 0) resolve();
      else reject(new Error(`${scriptPath} exited with code ${code}`));
    });
    child.on('error', reject);
  });
}

function startServer() {
  return new Promise((resolve, reject) => {
    const server = spawn(process.execPath, [path.join(__dirname, 'serve.mjs')], {
      stdio: ['ignore', 'pipe', 'inherit'],
      env: { ...process.env, PORT: String(PORT) },
    });
    server.stdout.on('data', (chunk) => {
      const text = chunk.toString();
      process.stdout.write(text);
      if (text.includes(`http://localhost:${PORT}`)) {
        resolve(server);
      }
    });
    server.on('error', reject);
    server.on('close', (code) => {
      if (code !== null && code !== 0) reject(new Error(`Server exited with ${code}`));
    });
    // Fallback: if server doesn't log the ready message within 3 s, assume ready
    setTimeout(() => resolve(server), 3000);
  });
}

(async () => {
  const server = await startServer();
  let exitCode = 0;
  try {
    await runScript(path.join(__dirname, 'playwright.mjs'), [
      `http://localhost:${PORT}/healthcheck/`,
    ]);
    console.log('\nHealthcheck PASSED');
  } catch (e) {
    console.error('\nHealthcheck FAILED:', e.message);
    exitCode = 1;
  } finally {
    server.kill('SIGTERM');
  }
  process.exit(exitCode);
})();

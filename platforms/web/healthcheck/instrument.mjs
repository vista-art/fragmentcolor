// Instrumentation for FragmentColor Web healthcheck
// - Wraps selected methods to log BEGIN/OK/ERR with module/method context
// - Adds global error and unhandledrejection handlers
// - Uses globalThis.__HC.currentModule to annotate logs with originating example

function isPromise(v) {
  return v && (typeof v === 'object' || typeof v === 'function') && typeof v.then === 'function';
}

function argPreview(x) {
  try {
    if (x == null) return String(x);
    if (typeof x === 'string') return JSON.stringify(x);
    if (typeof x === 'number' || typeof x === 'boolean') return String(x);
    if (Array.isArray(x)) return `[${x.map(argPreview).join(', ')}]`;
    const name = x?.constructor?.name || typeof x;
    if (name === 'Uint8Array') return `Uint8Array(len=${x.length})`;
    if (name === 'Float32Array') return `Float32Array(len=${x.length})`;
    if (name && name !== 'Object') return `<${name}>`;
    return JSON.stringify(x);
  } catch {
    return '<unprintable>';
  }
}

function logCtx(className, method, phase, extra) {
  const mod = (globalThis.__HC && globalThis.__HC.currentModule) || 'healthcheck.main';
  const parts = [`[call] module=${mod}`, `class=${className}`, `method=${method}`, `phase=${phase}`];
  if (extra) parts.push(extra);
  // Keep Playwright console parsing simple
  console.log(parts.join(' '));
}

function wrap(proto, method, className) {
  const orig = proto?.[method];
  if (typeof orig !== 'function') return;
  Object.defineProperty(proto, method, {
    configurable: true,
    writable: true,
    value: function wrappedMethod(...args) {
      logCtx(className, method, 'BEGIN', `args=${args.map(argPreview).join('|')}`);
      try {
        const res = orig.apply(this, args);
        if (isPromise(res)) {
          return res.then((v) => {
            logCtx(className, method, 'OK');
            return v;
          }).catch((e) => {
            logCtx(className, method, 'ERR', `error=${e?.message || String(e)}`);
            throw e;
          });
        }
        logCtx(className, method, 'OK');
        return res;
      } catch (e) {
        logCtx(className, method, 'ERR', `error=${e?.message || String(e)}`);
        throw e;
      }
    }
  });
}

export function installInstrumentation(api) {
  globalThis.__HC = globalThis.__HC || { currentModule: 'healthcheck.main' };

  // Global diagnostics for errors that bypass our wrappers
  if (typeof addEventListener === 'function') {
    addEventListener('error', (e) => {
      const msg = e?.message || String(e?.error || e);
      console.error(`[pageerror-event] ${msg}`);
      if (e?.error?.stack) console.error(e.error.stack);
    });
    addEventListener('unhandledrejection', (e) => {
      const reason = e?.reason;
      const msg = reason?.message || String(reason);
      console.error(`[unhandledrejection] ${msg}`);
      if (reason?.stack) console.error(reason.stack);
    });
  }

  // Wrap frequently exercised API methods
  try {
    const { Renderer, Shader, Pass, TextureTarget, CanvasTarget } = api || {};

    // Renderer
    wrap(Renderer?.prototype, 'createTarget', 'Renderer');
    wrap(Renderer?.prototype, 'createTextureTarget', 'Renderer');
    wrap(Renderer?.prototype, 'render', 'Renderer');

    // Shader
    wrap(Shader?.prototype, 'set', 'Shader');
    wrap(Shader?.prototype, 'get', 'Shader');
    wrap(Shader?.prototype, 'listUniforms', 'Shader');
    wrap(Shader?.prototype, 'listKeys', 'Shader');

    // Pass
    wrap(Pass?.prototype, 'addShader', 'Pass');
    wrap(Pass?.constructor, 'compute', 'Pass'); // static method safeguard (no-op if absent)

    // Targets
    wrap(TextureTarget?.prototype, 'resize', 'TextureTarget');
    wrap(TextureTarget?.prototype, 'size', 'TextureTarget');
    wrap(TextureTarget?.prototype, 'getImage', 'TextureTarget');
    wrap(CanvasTarget?.prototype, 'resize', 'CanvasTarget');
    wrap(CanvasTarget?.prototype, 'size', 'CanvasTarget');
    wrap(CanvasTarget?.prototype, 'getImage', 'CanvasTarget');
  } catch (e) {
    console.warn('[instrument] failed to install wrappers', e);
  }
}


import { emptyMap, equalMap, validateMap, type Keymap } from './domain';
import type { Language } from './i18n';
export type Issue = { code: string; area: string; detail: string };
export type Runtime = { state: string; connected: boolean; online: string; paused: boolean; forwarding: boolean; synchronizing: boolean; resync_required: boolean; issues: Issue[]; queued: number; cleanup_pending: boolean; query_completed_ms: number | null; demo: boolean };
export type Model = { draft: Keymap; saved: Keymap | null; language: Language; saving: boolean; dirty: boolean; quit_prompt: boolean; exited: boolean; error: Issue | null; notice: string | null; runtime: Runtime };
export type Command = { type: 'edit'; keymap: Keymap } | { type: 'import'; content: string; discard: boolean } | { type: 'language'; language: Language } | { type: 'pause'; paused: boolean } | { type: 'save' | 'undo' | 'retry' | 'resync' | 'request_quit' | 'cancel_quit' | 'quit_without_saving' | 'save_and_quit' };
declare global { interface Window { __TAURI__?: { core: { invoke: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T> } } } }
export const native = Boolean(window.__TAURI__);
const demo: Model = { draft: emptyMap(), saved: null, language: navigator.language.startsWith('zh') ? 'zh' : 'en', saving: false, dirty: false, quit_prompt: false, exited: false, error: null, notice: null,
  runtime: { state: 'configuration', connected: true, online: 'online', paused: false, forwarding: false, synchronizing: false, resync_required: false, issues: [], queued: 0, cleanup_pending: false, query_completed_ms: null, demo: true } };
export async function getSnapshot(): Promise<Model> { return native ? window.__TAURI__!.core.invoke<Model>('snapshot') : structuredClone(demo); }
export async function sendCommand(command: Command): Promise<void> {
  if (native) return window.__TAURI__!.core.invoke('command', { command });
  switch (command.type) {
    case 'edit': if (!demo.saving) demo.draft = validateMap(JSON.stringify(command.keymap)); break;
    case 'undo': demo.draft = structuredClone(demo.saved ?? emptyMap()); demo.notice = null; break;
    case 'language': demo.language = command.language; break;
    case 'pause': demo.runtime.paused = command.paused; break;
    case 'import': { const map = validateMap(command.content); if (demo.dirty && !command.discard) throw Error('discard confirmation required'); demo.draft = map; demo.notice = 'imported'; break; }
    case 'save': case 'save_and_quit':
      demo.saving = true;
      await new Promise(resolve => setTimeout(resolve, 250));
      demo.saved = structuredClone(demo.draft); demo.saving = false; demo.notice = 'saved'; demo.quit_prompt = false;
      if (command.type === 'save_and_quit') demo.exited = true;
      break;
    case 'request_quit': if (demo.dirty) demo.quit_prompt = true; else demo.exited = true; break;
    case 'cancel_quit': demo.quit_prompt = false; break;
    case 'quit_without_saving': demo.quit_prompt = false; demo.exited = true; break;
    case 'retry': case 'resync': break;
  }
  demo.dirty = !equalMap(demo.draft, demo.saved ?? emptyMap());
  demo.runtime.state = demo.runtime.paused ? 'paused' : demo.saved ? 'forwarding' : 'configuration';
  demo.runtime.forwarding = demo.runtime.state === 'forwarding';
}
export async function chooseImport(): Promise<string | null> {
  if (native) return window.__TAURI__!.core.invoke('import_file');
  return new Promise(resolve => {
    const input = document.createElement('input'); input.type = 'file'; input.accept = '.json,application/json';
    input.oncancel = () => resolve(null);
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) { resolve(null); return; }
      if (file.size > 65536) { resolve('{"oversize":true}'); return; }
      resolve(await file.text());
    };
    input.click();
  });
}
export async function exportSaved(map: Keymap): Promise<boolean> {
  if (native) return window.__TAURI__!.core.invoke('export_file');
  const url = URL.createObjectURL(new Blob([JSON.stringify(map, null, 2)], { type: 'application/json' }));
  const anchor = document.createElement('a'); anchor.href = url; anchor.download = `olanzi-keymap-${new Date().toISOString().replace(/[:.]/g, '-')}.json`; anchor.click(); setTimeout(() => URL.revokeObjectURL(url), 1000); return true;
}

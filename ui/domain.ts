export const controls = ['key_top', 'key_middle', 'key_bottom', 'knob_press', 'knob_right', 'knob_left'] as const;
export type Control = typeof controls[number];
export type Modifier = 'ctrl' | 'shift' | 'alt' | 'super';
export type Action = { kind: 'none' } | { kind: 'key' | 'hold'; usage: number; modifiers: Modifier[] };
export type Keymap = { format: 'olanzi-cross-platform-keymap'; version: 1 | 2; device: 'AU05'; bindings: Record<Control, Action> };
export const modifierOrder: Modifier[] = ['ctrl', 'shift', 'alt', 'super'];
export function emptyMap(): Keymap {
  return { format: 'olanzi-cross-platform-keymap', version: 1, device: 'AU05', bindings: Object.fromEntries(controls.map(id => [id, { kind: 'none' }])) as Record<Control, Action> };
}
export type Key = { usage: number; name: string; zh: string; category: string; label: string };
const key = (usage: number, name: string, zh: string, category = 'common', label = name): Key => ({ usage, name, zh, category, label });
export const keys: Key[] = [
  ...Array.from({ length: 26 }, (_, i) => key(i + 4, String.fromCharCode(65 + i), String.fromCharCode(65 + i), 'letters')),
  ...Array.from({ length: 10 }, (_, i) => key(i + 30, String((i + 1) % 10), String((i + 1) % 10), 'numbers')),
  key(40, 'Enter', '回车', 'common', '↵'), key(41, 'Esc', '退出'), key(42, 'Backspace', '退格', 'common', '⌫'), key(43, 'Tab', '制表', 'common', '⇥'), key(44, 'Space', '空格', 'common', 'Space'),
  ...[['Minus', '减号', '−'], ['Equal', '等号', '='], ['LeftBracket', '左方括号', '['], ['RightBracket', '右方括号', ']'], ['Backslash', '反斜杠', '\\']].map(([name, zh, label], i) => key(45 + i, name, zh, 'symbols', label)),
  ...[['Semicolon', '分号', ';'], ['Quote', '引号', "'"], ['Backquote', '反引号', '`'], ['Comma', '逗号', ','], ['Period', '句点', '.'], ['Slash', '斜杠', '/']].map(([name, zh, label], i) => key(51 + i, name, zh, 'symbols', label)),
  ...Array.from({ length: 12 }, (_, i) => key(58 + i, `F${i + 1}`, `F${i + 1}`, 'function')),
  key(73, 'Insert', '插入'), key(74, 'Home', '行首'), key(75, 'PageUp', '上一页', 'common', 'PgUp'), key(76, 'Delete', '删除', 'common', 'Del'), key(77, 'End', '行尾'), key(78, 'PageDown', '下一页', 'common', 'PgDn'),
  key(79, 'RightArrow', '向右', 'common', '→'), key(80, 'LeftArrow', '向左', 'common', '←'), key(81, 'DownArrow', '向下', 'common', '↓'), key(82, 'UpArrow', '向上', 'common', '↑'),
];
export function combo(action: Action, language = 'en'): string {
  if (action.kind === 'none') return language === 'zh' ? '不执行动作' : 'No action';
  const selected = keys.find(k => k.usage === action.usage);
  return [...modifierOrder.filter(m => action.modifiers.includes(m)).map(m => ({ ctrl: 'Ctrl', shift: 'Shift', alt: 'Alt', super: 'Super' })[m]), selected ? (language === 'zh' && selected.category === 'common' ? selected.zh : selected.label) : '?'].join(' + ');
}
export function replaceKey(action: Action, usage: number): Action {
  return { kind: action.kind === 'hold' ? 'hold' : 'key', usage, modifiers: action.kind !== 'none' ? [...action.modifiers] : [] };
}
export function replaceBinding(map: Keymap, control: Control, action: Action): Keymap {
  const bindings = { ...map.bindings, [control]: action };
  return { ...map, version: controls.some(id => bindings[id].kind === 'hold') ? 2 : 1, bindings };
}
export function toggleModifier(action: Action, modifier: Modifier): Action {
  if (action.kind === 'none') return action;
  const active = new Set(action.modifiers);
  if (active.has(modifier)) active.delete(modifier); else active.add(modifier);
  return { ...action, modifiers: modifierOrder.filter(m => active.has(m)) };
}
export function matchingKeys(category: string, search: string): Key[] {
  const term = search.trim().toLowerCase();
  return keys.filter(k => term ? `${k.name} ${k.zh} ${k.label}`.toLowerCase().includes(term) : category === 'all' || k.category === category);
}
export function equalMap(a: Keymap, b: Keymap): boolean { return JSON.stringify(a) === JSON.stringify(b); }

// 浏览器预览也执行格式检查；原生端始终重新执行权威校验。
export function validateMap(content: string): Keymap {
  if (new TextEncoder().encode(content).length > 65536) throw Error('CONFIG_INVALID');
  // 用轻量词法扫描器拒绝重复 JSON 字段，JSON.parse 随后验证完整语法。
  const objects: (Set<string> | null)[] = [];
  for (let i = 0; i < content.length; i++) {
    const c = content[i];
    if (c === '{') objects.push(new Set());
    else if (c === '[') objects.push(null);
    else if (c === '}' || c === ']') objects.pop();
    else if (c === '"') {
      const start = i++;
      for (; i < content.length; i++) { if (content[i] === '\\') i++; else if (content[i] === '"') break; }
      const token = content.slice(start, i + 1);
      let next = i + 1; while (/\s/.test(content[next] ?? '') && next < content.length) next++;
      if (content[next] === ':') {
        const set = objects.at(-1); const name: string = JSON.parse(token);
        if (!set || set.has(name)) throw Error('CONFIG_INVALID'); set.add(name);
      }
    }
  }
  const value = JSON.parse(content);
  const fields = (v: unknown, expected: string[]) => v !== null && typeof v === 'object' && !Array.isArray(v) && Object.keys(v).sort().join(',') === [...expected].sort().join(',');
  if (!fields(value, ['format', 'version', 'device', 'bindings']) || value.format !== 'olanzi-cross-platform-keymap' || ![1, 2].includes(value.version) || value.device !== 'AU05' || !fields(value.bindings, [...controls])) throw Error('CONFIG_INVALID');
  for (const id of controls) {
    const a = value.bindings[id];
    if (a?.kind === 'none' && fields(a, ['kind'])) continue;
    if (!fields(a, ['kind', 'usage', 'modifiers']) || !['key', 'hold'].includes(a.kind) || !Number.isInteger(a.usage) || !keys.some(k => k.usage === a.usage) || !Array.isArray(a.modifiers) || a.modifiers.length > 4 || new Set(a.modifiers).size !== a.modifiers.length || a.modifiers.some((m: Modifier) => !modifierOrder.includes(m))) throw Error('CONFIG_INVALID');
    if (a.kind === 'hold' && (value.version !== 2 || id === 'knob_left' || id === 'knob_right')) throw Error('CONFIG_INVALID');
    a.modifiers = modifierOrder.filter(m => a.modifiers.includes(m));
  }
  return value as Keymap;
}

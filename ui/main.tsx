import React, { useCallback, useEffect, useRef, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { ArrowDownToLine, ArrowUpFromLine, ArrowUpRight, Check, ChevronRight, CircleSlash, Command as CommandIcon, Keyboard, LoaderCircle, Pause, Play, Power, RotateCcw, RotateCw, Search, Settings, ShieldCheck, SlidersHorizontal, Usb, X } from 'lucide-react';
import { combo, controls, emptyMap, equalMap, matchingKeys, modifierOrder, replaceBinding, replaceKey, toggleModifier, validateMap, type Action, type Control } from './domain';
import { chooseImport, exportSaved, getSnapshot, sendCommand, type Command, type Issue, type Model } from './bridge';
import { text, type Language } from './i18n';
import './style.css';

function Modal({ title, children, cancel }: { title: string; children: React.ReactNode; cancel: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => { const element = dialog.current!; element.showModal(); return () => element.close(); }, []);
  return <dialog ref={dialog} onCancel={e => { e.preventDefault(); cancel(); }} aria-labelledby="dialog-title"><div className="modal-head"><h2 id="dialog-title">{title}</h2><button className="icon-button" onClick={cancel} aria-label="Close"><X size={20}/></button></div>{children}</dialog>;
}

function App() {
  const [model, setModel] = useState<Model | null>(null);
  const [page, setPage] = useState<'keymap' | 'settings'>('keymap');
  const [selected, setSelected] = useState<Control>('key_top');
  const [category, setCategory] = useState('common');
  const [search, setSearch] = useState('');
  const [pendingImport, setPendingImport] = useState<string | null>(null);
  const [localError, setLocalError] = useState<Issue | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const modelRef = useRef(model); modelRef.current = model;
  const pending = useRef(0);
  const chain = useRef(Promise.resolve());
  const refresh = useCallback(async () => {
    try { const next = await getSnapshot(); if (!pending.current) setModel(next); }
    catch (error) { setLocalError({ code: 'OPERATION_FAILED', area: 'application', detail: String(error) }); }
  }, []);
  useEffect(() => { void refresh(); const timer = setInterval(() => void refresh(), 150); return () => clearInterval(timer); }, [refresh]);
  useEffect(() => { document.documentElement.lang = model?.language ?? 'en'; }, [model?.language]);
  const send = useCallback((command: Command) => {
    pending.current++;
    if (command.type === 'save' || command.type === 'save_and_quit') setModel(m => m && ({ ...m, saving: true }));
    chain.current = chain.current.then(async () => {
      try { await sendCommand(command); }
      catch (error) { setLocalError({ code: command.type === 'import' ? 'CONFIG_INVALID' : 'OPERATION_FAILED', area: 'application', detail: String(error) }); }
      finally { pending.current--; if (!pending.current) await refresh(); }
    });
  }, [refresh]);
  if (!model) return <div className="loading"><LoaderCircle className="spin"/><span>clanzi</span>{localError && <p role="alert">{localError.detail}</p>}</div>;
  const t = text[model.language];
  const tr = (key: string) => (t as Record<string, string>)[key] ?? key;
  const action = model.draft.bindings[selected];
  const runtime = model.runtime;
  const status = tr(`status_${runtime.state}`);
  const errorList = [localError, model.error, ...runtime.issues].filter((e): e is Issue => e !== null);
  const update = (next: Action) => {
    const current = modelRef.current!;
    if (current.saving) return;
    const draft = replaceBinding(current.draft, selected, next);
    const optimistic = { ...current, draft, dirty: !equalMap(draft, current.saved ?? emptyMap()), notice: null };
    modelRef.current = optimistic; setModel(optimistic); setNotice(null); send({ type: 'edit', keymap: draft });
  };
  const importMap = async () => {
    if (modelRef.current?.saving) return;
    try {
      const content = await chooseImport();
      if (content === null) return;
      validateMap(content);
      if (modelRef.current?.dirty) setPendingImport(content);
      else send({ type: 'import', content, discard: false });
      setLocalError(null);
    } catch (error) { setLocalError({ code: 'CONFIG_INVALID', area: 'import', detail: String(error) }); }
  };
  const exportMap = async () => {
    try { if (model.saved && await exportSaved(model.saved)) setNotice('exported'); }
    catch (error) { setLocalError({ code: 'OPERATION_FAILED', area: 'export', detail: String(error) }); }
  };
  const card = (id: Control, icon: React.ReactNode, extra = '') => <button key={id} className={`control-card ${selected === id ? 'selected' : ''} ${extra}`} onClick={() => setSelected(id)} aria-pressed={selected === id} aria-label={`${tr(id)}: ${combo(model.draft.bindings[id], model.language)}`}><span className="control-name">{icon}{tr(id)}</span><span className={`control-binding ${model.draft.bindings[id].kind === 'none' ? 'unassigned' : ''}`}>{combo(model.draft.bindings[id], model.language)}</span>{selected === id && <span className="selected-dot"/>}</button>;
  const visibleKeys = matchingKeys(category, search);
  const dirtyLabel = model.saving ? t.saving : model.dirty ? t.unsaved : !model.saved ? t.newMap : runtime.forwarding ? t.saved : t.savedWaiting;

  return <div className="app-shell">
    <header className="topbar"><a className="brand" href="#" onClick={e => { e.preventDefault(); setPage('keymap'); }} aria-label="clanzi"><span className="brand-mark"><span/><span/><span/></span>clanzi<span className="platform">LINUX</span></a>
      <nav className="page-nav" aria-label={model.language === 'zh' ? '主导航' : 'Main navigation'}><button onClick={() => setPage('keymap')} aria-current={page === 'keymap' ? 'page' : undefined}><Keyboard size={17}/>{t.keymap}</button><button onClick={() => setPage('settings')} aria-current={page === 'settings' ? 'page' : undefined}><Settings size={17}/>{t.settings}</button></nav>
      <button className={`status-pill ${runtime.forwarding ? 'active' : ''}`} onClick={() => setPage('settings')}><span className="status-dot"/>{status}<ChevronRight size={13}/></button>
    </header>
    {runtime.demo && <div className="demo-banner"><span>{t.demo}</span>{t.demoText}</div>}
    <main>
      {page === 'keymap' ? <>
        <section className="workspace-heading"><div><div className="eyebrow">VIBE KEY <span>/</span> AU05</div><h1>{t.deviceTitle}</h1><p>{t.deviceSubtitle}</p></div><span className="device-badge"><Usb size={15}/> USB RECEIVER</span></section>
        <section className="device-stage" aria-label={t.device}>
          <div className="stage-grid"/>
          <div className="rotary-controls">{card('knob_left', <RotateCcw size={15}/>)}{card('knob_press', <span className="press-icon">◎</span>)}{card('knob_right', <RotateCw size={15}/>)}</div>
          <div className="hardware" aria-label="AU05">
            <div className="hardware-top"><span className="hardware-brand">ULANZI</span><span className={`hardware-led ${runtime.forwarding ? 'on' : ''}`}/></div>
            <div className="hardware-face"><div className="knob-assembly"><button className={`knob ${selected.startsWith('knob') ? 'highlight' : ''}`} onClick={() => setSelected('knob_press')} aria-label={t.knob_press}><span className="knob-center"><span/></span></button><span className="hardware-label">VIBE KEY</span></div><div className="hardware-keys">{(['key_top', 'key_middle', 'key_bottom'] as Control[]).map((id, i) => <button key={id} className={selected === id ? 'highlight' : ''} aria-label={tr(id)} onClick={() => setSelected(id)}><span>{String(i + 1).padStart(2, '0')}</span><i/></button>)}</div></div>
            <div className="hardware-bottom"><span>AU05</span><span>•••</span></div>
          </div>
          <div className="button-controls">{card('key_top', <span className="key-number">01</span>)}{card('key_middle', <span className="key-number">02</span>)}{card('key_bottom', <span className="key-number">03</span>)}</div>
          <div className="stage-caption"><span className="tiny-dot"/>{t.draftHint}</div>
        </section>
        <section className="editor" aria-label={t.action}>
          <header className="editor-heading"><div><span className="eyebrow">{t.action}</span><h2>{tr(selected)}<ChevronRight size={17}/><span>{t.selectKey}</span></h2></div><label className="search"><Search size={17}/><input value={search} onChange={e => setSearch(e.target.value)} placeholder={t.search} aria-label={t.search}/>{search && <button onClick={() => setSearch('')} aria-label={t.cancel}><X size={14}/></button>}</label></header>
          <div className="editor-body"><nav className="categories" aria-label={t.selectKey}>{['common', 'letters', 'numbers', 'symbols', 'function', 'all'].map(c => <button key={c} className={category === c && !search ? 'current' : ''} onClick={() => { setCategory(c); setSearch(''); }} aria-pressed={category === c && !search}>{c === 'common' ? <ArrowUpRight size={15}/> : c === 'function' ? <CommandIcon size={15}/> : <span className="category-icon">{{ letters: 'Aa', numbers: '12', symbols: '#', all: '▦' }[c]}</span>}{tr(c)}</button>)}</nav>
            <div className="key-panel"><div className="key-grid">{visibleKeys.map(k => <button key={k.usage} className={`keycap ${action.kind !== 'none' && action.usage === k.usage ? 'chosen' : ''}`} onClick={() => update(replaceKey(action, k.usage))} disabled={model.saving} aria-label={model.language === 'zh' ? `${k.zh} ${k.name}` : k.name} aria-pressed={action.kind !== 'none' && action.usage === k.usage}><span>{k.label}</span><small>{model.language === 'zh' ? k.zh : k.name}</small></button>)}</div>{visibleKeys.length === 0 && <div className="empty-state"><Search size={26}/><strong>{t.noResults}</strong><p>{t.noResultsHelp}</p></div>}<p className="physical-hint">{t.physical}</p></div>
          </div>
          <div className="modifier-row"><div><h3>{t.modifiers}</h3><p>{action.kind === 'none' ? t.modifierHint : t.actionHint}</p></div><div className="modifier-buttons">{modifierOrder.map(m => <button key={m} disabled={model.saving || action.kind === 'none'} aria-pressed={action.kind !== 'none' && action.modifiers.includes(m)} onClick={() => update(toggleModifier(action, m))}>{m === 'super' && <CommandIcon size={13}/>}{{ ctrl: 'Ctrl', shift: 'Shift', alt: 'Alt', super: 'Super' }[m]}</button>)}</div></div>
          <div className="combination-row"><div className="combination"><span className="eyebrow">{t.preview}</span><span>{action.kind === 'none' ? <span className="none-preview">—</span> : combo(action, model.language).split(' + ').map((part, i) => <React.Fragment key={i}>{i > 0 && <b>+</b>}<kbd>{part}</kbd></React.Fragment>)}</span></div><button className={`no-action ${action.kind === 'none' ? 'is-none' : ''}`} disabled={model.saving} onClick={() => update({ kind: 'none' })} title={t.noActionHelp}><CircleSlash size={15}/>{t.noAction}</button></div>
        </section>
      </> : <>
        <section className="workspace-heading"><div><div className="eyebrow">clanzi <span>/</span> {t.settings.toUpperCase()}</div><h1>{t.settingsTitle}</h1><p>{t.settingsSubtitle}</p></div><SlidersHorizontal className="settings-glyph" size={30}/></section>
        <div className="settings-grid">
          <section className="settings-card"><h2><Usb size={19}/>{t.connection}</h2><dl><div><dt>{t.receiver}</dt><dd>{runtime.connected ? t.detected : t.missing}</dd></div><div><dt>{t.device}</dt><dd>{tr(runtime.online)}</dd></div><div><dt>{t.action}</dt><dd className={runtime.forwarding ? 'accent-text' : ''}>{status}</dd></div></dl><div className="button-row"><button className="secondary" onClick={() => send({ type: 'retry' })}><RotateCcw size={15}/>{t.retry}</button><button className="secondary" onClick={() => send({ type: 'pause', paused: !runtime.paused })}>{runtime.paused ? <Play size={15}/> : <Pause size={15}/>} {runtime.paused ? t.resume : t.pause}</button></div><p>{t.pausedHint}</p></section>
          <section className="settings-card"><h2><ShieldCheck size={19}/>{t.permissions}</h2><p>{t.permissionsText}</p><p>{t.permissionsHelp}</p></section>
          <section className="settings-card"><h2><Keyboard size={19}/>{t.files}</h2><p>{t.filesHint}</p><div className="button-row"><button className="secondary" disabled={model.saving} onClick={() => void importMap()}><ArrowDownToLine size={16}/>{t.import}</button><button className="secondary" disabled={!model.saved} onClick={() => void exportMap()}><ArrowUpFromLine size={16}/>{t.export}</button></div>{model.dirty && <p className="subtle-warning">{t.exportHint}</p>}</section>
          <section className="settings-card"><h2>{t.language}</h2><p>{t.languageHint}</p><div className="language-toggle">{(['en', 'zh'] as Language[]).map(language => <button key={language} aria-pressed={model.language === language} onClick={() => send({ type: 'language', language })}>{language === 'en' ? 'English' : '简体中文'}{model.language === language && <Check size={15}/>}</button>)}</div></section>
          <section className="settings-card background-card"><div><h2>{t.background}</h2><p>{t.backgroundHint}</p></div><button className="danger-button" onClick={() => send({ type: 'request_quit' })}><Power size={16}/>{t.quit}</button></section>
        </div>
      </>}
      {runtime.resync_required && <div className="alert"><p>{t.resyncHint}</p><button className="secondary" onClick={() => send({ type: 'resync' })}>{t.resync}</button></div>}
      {errorList.length > 0 && <section className="issues" aria-live="polite">{errorList.map((error, i) => <div className="issue" key={`${error.code}-${i}`}><span className="issue-mark">!</span><div><strong>{tr(error.code)}</strong><details><summary>{t.detail} · {error.code}</summary><code>{error.detail}</code></details></div></div>)}</section>}
      {(notice || model.notice === 'imported') && <div className="notice" role="status"><Check size={15}/>{tr(notice ?? model.notice!)}</div>}
    </main>
    <footer className="savebar"><div className="save-state" role="status">{model.saving ? <LoaderCircle size={17} className="spin"/> : model.dirty || !model.saved ? <span className="unsaved-dot"/> : <Check size={17}/>}<div><strong>{dirtyLabel}</strong><span>{t.draftHint}</span></div></div><div className="save-actions"><button className="secondary" disabled={!model.dirty || model.saving} onClick={() => { setLocalError(null); send({ type: 'undo' }); }}><RotateCcw size={15}/>{t.undo}</button><button className="primary" disabled={model.saving || (model.saved !== null && !model.dirty)} onClick={() => { setLocalError(null); send({ type: 'save' }); }}>{model.saving ? <LoaderCircle size={16} className="spin"/> : <Check size={17}/>} {model.saving ? t.saving : t.save}</button></div></footer>
    {pendingImport !== null && <Modal title={t.discardTitle} cancel={() => setPendingImport(null)}><p>{t.discardText}</p><div className="dialog-actions"><button className="secondary" onClick={() => setPendingImport(null)}>{t.cancel}</button><button className="primary" onClick={() => { send({ type: 'import', content: pendingImport, discard: true }); setPendingImport(null); }}>{t.discard}</button></div></Modal>}
    {model.quit_prompt && <Modal title={t.quitTitle} cancel={() => send({ type: 'cancel_quit' })}><p>{t.quitText}</p><div className="dialog-actions"><button className="secondary" onClick={() => send({ type: 'cancel_quit' })}>{t.cancel}</button><button className="secondary" onClick={() => send({ type: 'quit_without_saving' })}>{t.noSaveQuit}</button><button className="primary" onClick={() => send({ type: 'save_and_quit' })}>{t.saveQuit}</button></div></Modal>}
    {model.exited && <div className="demo-ended"><Power size={30}/><h2>{model.language === 'zh' ? '演示会话已结束' : 'Demo session ended'}</h2><p>{model.language === 'zh' ? '刷新页面开始新的隔离演示。' : 'Reload the page for a new isolated demo.'}</p></div>}
  </div>;
}

createRoot(document.getElementById('root')!).render(<App/>);

import test from 'node:test';
import assert from 'node:assert/strict';
import { emptyMap, combo, replaceKey, replaceBinding, toggleModifier, matchingKeys, keys, validateMap } from '../ui/domain.ts';

test('changing keys retains modifiers; clearing starts a fresh combination', () => {
  let action = replaceKey({kind:'none'}, 6);
  action = toggleModifier(action, 'super'); action = toggleModifier(action, 'ctrl');
  assert.equal(combo(action), 'Ctrl + Super + C');
  assert.equal(combo(replaceKey(action, 5)), 'Ctrl + Super + B');
  assert.deepEqual(replaceKey({kind:'none'}, 5).modifiers, []);
  assert.deepEqual(toggleModifier({kind:'none'}, 'ctrl'), {kind:'none'});
});
test('search covers English and Chinese across categories', () => {
  assert.equal(matchingKeys('letters', '回车')[0].usage, 40);
  assert.equal(matchingKeys('letters', 'RightArrow')[0].usage, 79);
  assert.equal(matchingKeys('all', 'unfindable').length, 0);
  assert.equal(new Set(keys.map(k=>k.usage)).size, keys.length);
});
test('legacy hold configurations remain editable without losing their shortcut', () => {
  let action = {kind:'hold',usage:6,modifiers:['ctrl']};
  action = replaceKey(action, 5);
  action = toggleModifier(action, 'shift');
  assert.deepEqual(action, {kind:'hold',usage:5,modifiers:['ctrl','shift']});
  assert.equal(combo(action), 'Ctrl + Shift + B');
  let map = replaceBinding(emptyMap(), 'key_top', action);
  assert.equal(map.version, 2);
  assert.deepEqual(validateMap(JSON.stringify(map)), map);
  map = replaceBinding(map, 'key_middle', action);
  map = replaceBinding(map, 'key_top', {kind:'none'});
  assert.equal(map.version, 2);
  map = replaceBinding(map, 'key_middle', {...action, kind:'key'});
  assert.equal(map.version, 1);
  assert.deepEqual(validateMap(JSON.stringify(map)), map);
});
test('ordinary button mappings need no mode field or configuration upgrade', () => {
  let map = emptyMap();
  for (const control of ['key_top', 'key_middle', 'key_bottom', 'knob_press']) {
    const action = toggleModifier(replaceKey({kind:'none'}, 6), 'ctrl');
    map = replaceBinding(map, control, action);
    assert.deepEqual(map.bindings[control], {kind:'key',usage:6,modifiers:['ctrl']});
  }
  assert.equal(map.version, 1);
  assert.deepEqual(validateMap(JSON.stringify(map)), map);
});
test('hold configuration rejects old versions, rotary holds and malformed actions', () => {
  const map = replaceBinding(emptyMap(), 'knob_press', {kind:'hold',usage:6,modifiers:['ctrl']});
  for (const version of [1, 3, true, '2']) {
    assert.throws(() => validateMap(JSON.stringify({...map, version})));
  }
  for (const control of ['knob_left', 'knob_right']) {
    assert.throws(() => validateMap(JSON.stringify(replaceBinding(map, control, map.bindings.knob_press))));
  }
  const valid = JSON.stringify(map);
  for (const bad of [
    valid.replace('"kind":"hold"', '"kind":"hold","kind":"key"'),
    valid.replace('"kind":"hold"', '"kind":"hold","extra":true'),
    valid.replace('"usage":6', '"usage":57'),
    valid.replace('["ctrl"]', '["ctrl","ctrl"]'),
  ]) assert.throws(() => validateMap(bad));
});
test('preview rejects invalid and duplicate fields like the native validator', () => {
  const map = emptyMap();
  assert.deepEqual(validateMap(JSON.stringify(map)), map);
  for (const bad of [
    '{"version":1,"version":1}',
    JSON.stringify(map).replace('"kind":"none"','"kind":"none","kind":"none"'),
    JSON.stringify(map).replace('"kind":"none"','"kind":"none","usage":4'),
    JSON.stringify(map).replace('"version":1','"version":true'),
    JSON.stringify(map).replace('"AU05"','"AU06"'),
    ' '.repeat(65537),
  ]) assert.throws(()=>validateMap(bad));
  map.bindings.key_top = {kind:'key',usage:6,modifiers:['super','ctrl']};
  assert.deepEqual(validateMap(JSON.stringify(map)).bindings.key_top.modifiers,['ctrl','super']);
});

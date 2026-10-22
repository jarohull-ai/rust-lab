import assert from 'node:assert/strict';
import test from 'node:test';
import { createStoredZip } from '../src/zip.ts';

test('creates a ZIP archive with local, central, and end records', () => {
  const archive = createStoredZip({
    'demo/Cargo.toml': '[package]\nname = "demo"\n',
    'demo/src/main.rs': 'fn main() {}\n',
  });
  const view = new DataView(archive.buffer);
  const text = new TextDecoder().decode(archive);

  assert.equal(view.getUint32(0, true), 0x04034b50);
  assert.ok(text.includes('demo/Cargo.toml'));
  assert.ok(text.includes('demo/src/main.rs'));
  assert.equal(view.getUint32(archive.length - 22, true), 0x06054b50);
  assert.equal(view.getUint16(archive.length - 12, true), 2);
});

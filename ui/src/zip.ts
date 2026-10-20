const encoder = new TextEncoder();

const crcTable = (() => {
  const table = new Uint32Array(256);
  for (let value = 0; value < 256; value += 1) {
    let crc = value;
    for (let bit = 0; bit < 8; bit += 1) {
      crc = (crc & 1) !== 0 ? 0xedb88320 ^ (crc >>> 1) : crc >>> 1;
    }
    table[value] = crc >>> 0;
  }
  return table;
})();

function crc32(bytes: Uint8Array): number {
  let crc = 0xffffffff;
  for (const byte of bytes) crc = crcTable[(crc ^ byte) & 0xff] ^ (crc >>> 8);
  return (crc ^ 0xffffffff) >>> 0;
}

function writeU16(view: DataView, offset: number, value: number): number {
  view.setUint16(offset, value, true);
  return offset + 2;
}

function writeU32(view: DataView, offset: number, value: number): number {
  view.setUint32(offset, value >>> 0, true);
  return offset + 4;
}

function concat(parts: Uint8Array[]): Uint8Array {
  const output = new Uint8Array(parts.reduce((total, part) => total + part.length, 0));
  let offset = 0;
  for (const part of parts) {
    output.set(part, offset);
    offset += part.length;
  }
  return output;
}

interface CentralEntry {
  name: Uint8Array;
  bytes: Uint8Array;
  crc: number;
  localOffset: number;
}

/** Creates a standards-compliant ZIP using the uncompressed STORE method. */
export function createStoredZip(files: Record<string, string>): Uint8Array {
  const localParts: Uint8Array[] = [];
  const entries: CentralEntry[] = [];
  let localOffset = 0;

  for (const [path, content] of Object.entries(files)) {
    const name = encoder.encode(path);
    const bytes = encoder.encode(content);
    const crc = crc32(bytes);
    const header = new Uint8Array(30 + name.length);
    const view = new DataView(header.buffer);
    let offset = 0;
    offset = writeU32(view, offset, 0x04034b50);
    offset = writeU16(view, offset, 20);
    offset = writeU16(view, offset, 0x0800);
    offset = writeU16(view, offset, 0);
    offset = writeU16(view, offset, 0);
    offset = writeU16(view, offset, 0);
    offset = writeU32(view, offset, crc);
    offset = writeU32(view, offset, bytes.length);
    offset = writeU32(view, offset, bytes.length);
    offset = writeU16(view, offset, name.length);
    writeU16(view, offset, 0);
    header.set(name, 30);

    entries.push({ name, bytes, crc, localOffset });
    localParts.push(header, bytes);
    localOffset += header.length + bytes.length;
  }

  const centralParts: Uint8Array[] = [];
  let centralSize = 0;
  for (const entry of entries) {
    const header = new Uint8Array(46 + entry.name.length);
    const view = new DataView(header.buffer);
    let offset = 0;
    offset = writeU32(view, offset, 0x02014b50);
    offset = writeU16(view, offset, 20);
    offset = writeU16(view, offset, 20);
    offset = writeU16(view, offset, 0x0800);
    offset = writeU16(view, offset, 0);
    offset = writeU16(view, offset, 0);
    offset = writeU16(view, offset, 0);
    offset = writeU32(view, offset, entry.crc);
    offset = writeU32(view, offset, entry.bytes.length);
    offset = writeU32(view, offset, entry.bytes.length);
    offset = writeU16(view, offset, entry.name.length);
    offset = writeU16(view, offset, 0);
    offset = writeU16(view, offset, 0);
    offset = writeU16(view, offset, 0);
    offset = writeU16(view, offset, 0);
    offset = writeU32(view, offset, 0);
    writeU32(view, offset, entry.localOffset);
    header.set(entry.name, 46);
    centralParts.push(header);
    centralSize += header.length;
  }

  const end = new Uint8Array(22);
  const endView = new DataView(end.buffer);
  let endOffset = 0;
  endOffset = writeU32(endView, endOffset, 0x06054b50);
  endOffset = writeU16(endView, endOffset, 0);
  endOffset = writeU16(endView, endOffset, 0);
  endOffset = writeU16(endView, endOffset, entries.length);
  endOffset = writeU16(endView, endOffset, entries.length);
  endOffset = writeU32(endView, endOffset, centralSize);
  endOffset = writeU32(endView, endOffset, localOffset);
  writeU16(endView, endOffset, 0);

  return concat([...localParts, ...centralParts, end]);
}

// Run after cargo leptos build. Only public build artifacts receive immutable names.
const fs = require('node:fs'), path = require('node:path'), crypto = require('node:crypto'), zlib = require('node:zlib');
const site = path.resolve(process.argv[2] || 'app/target/site'), pkg = path.join(site, 'pkg');
const manifest = {version: 1};
for (const [key, file] of [['js', 'living-trail.js'], ['wasm', 'living-trail.wasm']]) {
  const bytes = fs.readFileSync(path.join(pkg, file));
  const hash = crypto.createHash('sha256').update(bytes).digest('hex').slice(0, 16);
  const name = `living-trail.${hash}.${key}`;
  fs.writeFileSync(path.join(pkg, name), bytes);
  const br = zlib.brotliCompressSync(bytes, {params: {[zlib.constants.BROTLI_PARAM_QUALITY]: 11}});
  fs.writeFileSync(path.join(pkg, name + '.br'), br);
  fs.writeFileSync(path.join(pkg, name + '.gz'), zlib.gzipSync(bytes, {level: 9}));
  manifest[key] = `/pkg/${name}`;
  manifest[key + '_bytes'] = bytes.length;
  manifest[key + '_br_bytes'] = br.length;
}
if (manifest.wasm_br_bytes > 1100 * 1024) throw Error('WASM exceeds the 1,100 KiB Brotli budget');
if (manifest.js_br_bytes > 15 * 1024) throw Error('JS exceeds the 15 KiB Brotli budget');
fs.writeFileSync(path.join(pkg, 'runtime.json'), JSON.stringify(manifest, null, 2));
console.log(JSON.stringify(manifest));

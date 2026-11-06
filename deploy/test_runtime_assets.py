"""Immutable names must survive rebuilds and change when bytes change."""
import gzip, hashlib, json, shutil, subprocess, tempfile, unittest
from pathlib import Path

class RuntimeAssets(unittest.TestCase):
    @unittest.skipUnless(shutil.which('node'), 'Node is required by the release packager')
    def test_repeat_build_reuses_urls_and_changed_wasm_gets_a_new_url(self):
        with tempfile.TemporaryDirectory() as directory:
            site=Path(directory);pkg=site/'pkg';pkg.mkdir()
            (pkg/'living-trail.js').write_text('export function hydrate() {}')
            (pkg/'living-trail.wasm').write_bytes(b'\0asm'+bytes(range(256))*10)
            def build():
                subprocess.run(['node',str(Path(__file__).with_name('prepare-assets.cjs')),str(site)],check=True,capture_output=True)
                return json.loads((pkg/'runtime.json').read_text())
            first=build();self.assertEqual(first,build())
            original=(site/first['wasm'].lstrip('/')).read_bytes()
            self.assertIn(hashlib.sha256(original).hexdigest()[:16],first['wasm'])
            self.assertEqual(gzip.decompress((site/(first['wasm'].lstrip('/')+'.gz')).read_bytes()),original)
            (pkg/'living-trail.wasm').write_bytes(original+b'changed')
            second=build();self.assertNotEqual(first['wasm'],second['wasm']);self.assertEqual(first['js'],second['js'])
            self.assertEqual((site/first['wasm'].lstrip('/')).read_bytes(),original)

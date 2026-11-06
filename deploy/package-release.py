"""Package an already-built Linux binary and site; preserve only the current build's hashed assets."""
import argparse,json,re,shutil,subprocess,tarfile
from pathlib import Path

parser=argparse.ArgumentParser()
parser.add_argument('release_directory',type=Path)
parser.add_argument('--pass-number',required=True)
args=parser.parse_args()
repo=Path(__file__).resolve().parents[1];site=repo/'app/target/site';release=args.release_directory.resolve()
assert (release/'living-trail').is_file(),'Build and copy the Linux release binary first'
shutil.copytree(repo/'app/public',site,dirs_exist_ok=True)
shutil.copy2(site/'pkg/living-trail.wasm',site/'pkg/living-trail_bg.wasm')
subprocess.run(['node',str(repo/'deploy/prepare-assets.cjs'),str(site)],check=True)
manifest=json.loads((site/'pkg/runtime.json').read_text())
current={Path(manifest[k]).name for k in ['js','wasm']}
sha=subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()
(site/'design/release.json').write_text(json.dumps({'commit':sha,'pass':args.pass_number}),encoding='utf-8')
# Reconstruct only the generated site subtree inside the explicitly selected release.
target_site=release/'site'
if target_site.exists():
    assert not target_site.is_symlink() and target_site.resolve().parent==release,'Unexpected generated site location'
    shutil.rmtree(target_site)
def ignore(directory,names):
    return [n for n in names if n.endswith('.d.ts') or (re.fullmatch(r'living-trail\.[a-f0-9]{16}\.(js|wasm)(\.br|\.gz)?',n) and re.sub(r'\.(br|gz)$','',n) not in current)]
shutil.copytree(site,release/'site',ignore=ignore)
shutil.copy2(repo/'deploy/backup-review.py',release/'backup-review.py')
archive=release.with_suffix('.tar.gz')
with tarfile.open(archive,'w:gz') as tf:
    for name in ['living-trail','site','backup-review.py']+(['caddy'] if (release/'caddy').exists() else []):tf.add(release/name,arcname=name)
print(json.dumps({'archive':str(archive),'bytes':archive.stat().st_size,'commit':sha}))

#!/usr/bin/env bash
# Update application files while retaining live SQLite, media and accounts.
# Usage: sudo bash upgrade-review.sh /tmp/release.tar.gz release-id
set -euo pipefail
archive=$(realpath "$1")
release_id="$2"
[[ "$release_id" =~ ^[A-Za-z0-9._-]+$ ]] || { echo 'Invalid release identifier'; exit 1; }
previous=$(readlink -f /opt/living-trail/current)
next="/opt/living-trail/releases/$release_id"
[[ ! -e "$next" ]] || { echo 'Release already exists'; exit 1; }
systemctl start living-trail-backup.service
install -d -m 0755 "$next"
tar -xzf "$archive" -C "$next" --no-same-owner
cp "$previous/caddy" "$next/caddy"
chmod 0755 "$next/living-trail" "$next/caddy"
if [[ -f "$next/site/pkg/living-trail.wasm" && ! -e "$next/site/pkg/living-trail_bg.wasm" ]]; then
  ln -s living-trail.wasm "$next/site/pkg/living-trail_bg.wasm"
fi
if [[ -f "$next/backup-review.py" ]]; then
  install -m 0755 "$next/backup-review.py" /usr/local/bin/backup-living-trail
fi
cp /etc/caddy/Caddyfile /etc/caddy/Caddyfile.previous
python3 - "$next" <<'PYCFG'
from pathlib import Path
import hashlib,json,re,shutil,sys
site=Path(sys.argv[1])/'site'
# Keep content-addressed bundles across releases: an open old tab can finish
# its download after an atomic upgrade. Never copy HTML, uploads or CMS data.
manifest=json.loads((site/'pkg/runtime.json').read_text())
assets=Path('/opt/living-trail/assets/pkg');assets.mkdir(parents=True,exist_ok=True)
for kind in ('js','wasm'):
    name=Path(manifest[kind]).name
    match=re.fullmatch(r'living-trail\.([a-f0-9]{16})\.(js|wasm)',name)
    assert match and match[2]==kind,'Invalid bundle name'
    source=site/'pkg'/name
    assert hashlib.sha256(source.read_bytes()).hexdigest().startswith(match[1]),'Bundle hash mismatch'
    for suffix in ('','.br','.gz'):
        target=assets/(name+suffix)
        if target.exists():assert target.read_bytes()==Path(str(source)+suffix).read_bytes(),'Immutable asset changed'
        else:shutil.copyfile(str(source)+suffix,target)
p=Path('/etc/caddy/Caddyfile');s=p.read_text()
if '/design/*' not in s:s=s.replace('@assets path /pkg/*', '@assets path /design/* /maps/* /pkg/*')
if '@versioned_bundle' not in s:
    s=s.replace('  @assets path', '''  @versioned_bundle path_regexp bundle ^/pkg/living-trail\\.[a-f0-9]{16}\\.(js|wasm)$
  handle @versioned_bundle {
    root * /opt/living-trail/assets
    header Cache-Control "public, max-age=31536000, immutable"
    file_server {
      precompressed br gzip
    }
  }
  @assets path''')
p.write_text(s)
PYCFG
"$next/caddy" validate --config /etc/caddy/Caddyfile --adapter caddyfile >/dev/null
systemctl stop living-trail
ln -sfn "$next" /opt/living-trail/current
systemctl start living-trail
systemctl reload caddy
if ! python3 - <<'PYCHECK'
import time,urllib.request
for attempt in range(12):
    try:
        with urllib.request.urlopen('http://127.0.0.1:3107/', timeout=3) as r:
            body=r.read().decode()
            assert r.status==200 and 'field-journal' in body
        break
    except Exception:
        if attempt==11:raise
        time.sleep(1)
PYCHECK
then
  ln -sfn "$previous" /opt/living-trail/current
  cp /etc/caddy/Caddyfile.previous /etc/caddy/Caddyfile
  systemctl restart living-trail
  systemctl reload caddy
  echo 'Health check failed; restored previous release.'
  exit 1
fi
systemctl is-active living-trail caddy living-trail-backup.timer
echo "Activated release: $release_id"

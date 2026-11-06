"""Consistent live SQLite backup, plus immutable media."""
from contextlib import closing
import datetime
import json
import pathlib
import shutil
import sqlite3

root = pathlib.Path(__file__).resolve().parents[1]
source = root / '.cms/rust'
destination = root / '.cms/backups' / ('rust-' + datetime.datetime.now().strftime('%Y%m%d-%H%M%S-%f'))
if not (source / 'trail.sqlite3').exists():
    raise SystemExit('Start Trail Studio first; no native database exists yet.')
destination.mkdir(parents=True)
with closing(sqlite3.connect(f'file:{(source / "trail.sqlite3").as_posix()}?mode=ro', uri=True)) as db:
    with closing(sqlite3.connect(destination / 'trail.sqlite3')) as backup:
        db.backup(backup)
        result = backup.execute('PRAGMA integrity_check').fetchone()[0]
        if result != 'ok':
            raise RuntimeError(result)
        backup.execute('DELETE FROM sessions')
        backup.execute('DELETE FROM previews')
        backup.commit()
    items = [json.loads(row[0]) for row in db.execute('SELECT data FROM items')]
    (destination / 'content.json').write_text(json.dumps(items, indent=2, ensure_ascii=False), encoding='utf8')
for name in ('uploads', 'legacy-uploads'):
    if (source / name).exists():
        shutil.copytree(source / name, destination / name)
print(f'Verified database, {len(items)} content records, and media saved to {destination}')

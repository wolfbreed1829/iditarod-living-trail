#!/usr/bin/env python3
"""Daily consistent review backup; keep the latest 14 on the VM's persistent disk."""
from contextlib import closing
import os
import datetime
import pathlib
import sqlite3
import tarfile
import tempfile

source = pathlib.Path(os.environ.get('TRAIL_DATA_DIR', '/var/lib/living-trail'))
destination = pathlib.Path(os.environ.get('TRAIL_BACKUP_DIR', '/var/backups/living-trail'))
destination.mkdir(mode=0o700, exist_ok=True)
stamp = datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%SZ')
archive = destination / f'trail-{stamp}.tar.gz'
with tempfile.TemporaryDirectory(prefix='trail-backup-') as scratch:
    snapshot = pathlib.Path(scratch) / 'trail.sqlite3'
    with closing(sqlite3.connect(f'file:{(source / "trail.sqlite3").as_posix()}?mode=ro', uri=True)) as db:
        with closing(sqlite3.connect(snapshot)) as copy:
            db.backup(copy)
            assert copy.execute('PRAGMA integrity_check').fetchone()[0] == 'ok'
            copy.execute('DELETE FROM sessions')
            copy.execute('DELETE FROM previews')
            copy.commit()
    with tarfile.open(archive, 'w:gz') as tar:
        tar.add(snapshot, arcname='trail.sqlite3')
        for name in ('uploads', 'legacy-uploads'):
            if (source / name).exists():
                tar.add(source / name, arcname=name)
archive.chmod(0o600)
for old in sorted(destination.glob('trail-*.tar.gz'))[:-14]:
    old.unlink()
print(f'Verified backup: {archive.name}')

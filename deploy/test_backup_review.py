"""A live WAL database must produce a self-contained archive without sessions."""
from contextlib import closing
import os
import pathlib
import sqlite3
import subprocess
import sys
import tarfile
import tempfile
import unittest

class BackupTest(unittest.TestCase):
    def test_live_wal_backup_omits_sessions_and_preserves_content(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = pathlib.Path(scratch)
            source = root / 'data'
            source.mkdir()
            uploads = source / 'uploads'
            uploads.mkdir()
            (uploads / 'guide.pdf').write_bytes(b'%PDF-1.4 sample')
            db = sqlite3.connect(source / 'trail.sqlite3')
            try:
                db.execute('PRAGMA journal_mode=WAL')
                db.executescript("CREATE TABLE items (id INTEGER PRIMARY KEY, data TEXT); CREATE TABLE sessions (id TEXT); CREATE TABLE previews (id TEXT); INSERT INTO items VALUES (1, 'published content'); INSERT INTO sessions VALUES ('active session'); INSERT INTO previews VALUES ('private preview');")
                db.commit()
                env = dict(os.environ, TRAIL_DATA_DIR=str(source), TRAIL_BACKUP_DIR=str(root / 'backups'))
                subprocess.run([sys.executable, str(pathlib.Path(__file__).with_name('backup-review.py'))], env=env, check=True, capture_output=True)
                archive = next((root / 'backups').glob('*.tar.gz'))
                extracted = root / 'restore'
                with tarfile.open(archive) as tar:
                    self.assertFalse(any(n.endswith(('-wal', '-shm')) for n in tar.getnames()))
                    tar.extractall(extracted, filter='data')
                with closing(sqlite3.connect(f'file:{(extracted / "trail.sqlite3").as_posix()}?immutable=1', uri=True)) as copy:
                    self.assertEqual(copy.execute('PRAGMA integrity_check').fetchone()[0], 'ok')
                    self.assertEqual(copy.execute('SELECT data FROM items').fetchone()[0], 'published content')
                    self.assertEqual(copy.execute('SELECT count(*) FROM sessions').fetchone()[0], 0)
                    self.assertEqual(copy.execute('SELECT count(*) FROM previews').fetchone()[0], 0)
                self.assertEqual((extracted / 'uploads/guide.pdf').read_bytes(), b'%PDF-1.4 sample')
            finally:
                db.close()

if __name__ == '__main__':
    unittest.main()

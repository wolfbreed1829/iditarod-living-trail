"""Release gate for the credited photo library; no modification of licensed originals."""
import json
from pathlib import Path
root=Path(__file__).resolve().parents[1]/'app/public'
records=[]
for manifest in (root/'images').rglob('*manifest.json'):
    records.extend(json.loads(manifest.read_text(encoding='utf-8')))
for item in records:
    for field,limit in [('file',360*1024),('small_file',200*1024)]:
        size=(root/item[field].lstrip('/')).stat().st_size
        assert size<=limit,f'{item[field]}: {size:,} bytes exceeds {limit:,}'
print(f'{len(records)} credited photographs within 360 KiB full / 200 KiB small budgets')

"""Serve the exported preview locally under the same prefix as its public URL."""
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
from urllib.parse import urlsplit, unquote
import argparse

parser=argparse.ArgumentParser()
parser.add_argument('--port',type=int,default=3110)
parser.add_argument('--base',default='/iditarod-living-trail')
args=parser.parse_args()
root=Path(__file__).resolve().parents[1]/'dist'
class Handler(SimpleHTTPRequestHandler):
    def translate_path(self,path):
        path=unquote(urlsplit(path).path)
        if path.startswith(args.base):path=path[len(args.base):]
        target=(root/path.lstrip('/')).resolve()
        return str(target if target.is_relative_to(root.resolve()) else root/'404.html')
    def log_message(self,*args):pass
    def end_headers(self):
        self.send_header('Cache-Control','no-cache')
        super().end_headers()
print(f'Preview: http://localhost:{args.port}{args.base}/',flush=True)
ThreadingHTTPServer(('127.0.0.1',args.port),Handler).serve_forever()

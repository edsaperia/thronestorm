"""Build web/index.html: inline the engine and the network weights into app.html.

    python web/build.py [nets/h10-win.bin]
"""
import base64
import pathlib
import sys

here = pathlib.Path(__file__).parent
net = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else here.parent / "nets" / "h10-win.bin"
page = (here / "app.html").read_text()
page = page.replace("/*ENGINE*/", (here / "engine.js").read_text())
page = page.replace("WEIGHTS", base64.b64encode(net.read_bytes()).decode())
(here / "index.html").write_text(page)
print(f"wrote {here / 'index.html'} ({len(page) // 1024} KB) with {net.name}")

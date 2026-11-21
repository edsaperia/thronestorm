"""Build each skin of the game from app.html.

    python web/build.py [nets/h10-win.bin]

Writes web/index.html (Thronestorm) and web/fallen-leaves/index.html. A skin is
a folder under web/skins with its words (skin.js), look (skin.css), title
screen art (hero.html), rules text (rules.html) and settings (skin.json).
"""
import base64
import json
import pathlib
import sys

here = pathlib.Path(__file__).parent
net = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else here.parent / "nets" / "h10-win.bin"
template = (here / "app.html").read_text()
engine = (here / "engine.js").read_text()
weights = base64.b64encode(net.read_bytes()).decode()

for skin in sorted((here / "skins").iterdir()):
    cfg = json.loads((skin / "skin.json").read_text())
    fonts = ('<link href="https://fonts.googleapis.com/css2?family=%s&display=swap" rel="stylesheet">' % cfg["fonts"])
    page = template
    for key, value in {
        "{{TITLE}}": cfg["title"], "{{FONTS}}": fonts, "{{TAGLINE}}": cfg["tagline"],
        "{{S1}}": cfg["strength"][0], "{{S2}}": cfg["strength"][1], "{{S4}}": cfg["strength"][2],
        "{{LEVEL_HEAD}}": ("<h2>%s</h2>" % cfg["level_head"]) if cfg.get("level_head") else "",
        "{{PILES}}": cfg["piles"], "{{TABLE}}": cfg["table"],
        "{{HERO}}": (skin / "hero.html").read_text().strip(),
        "{{RULES}}": (skin / "rules.html").read_text().rstrip(),
        "/*SKINCSS*/": (skin / "skin.css").read_text(),
        "/*SKIN*/": (skin / "skin.js").read_text(),
        "/*ENGINE*/": engine, "WEIGHTS": weights,
    }.items():
        assert key in page, key
        page = page.replace(key, value)
    out = here / cfg["out"]
    out.parent.mkdir(exist_ok=True)
    out.write_text(page)
    print(f"wrote {out.relative_to(here.parent)} ({len(page) // 1024} KB), needs {', '.join(cfg['assets'])}")

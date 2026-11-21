# Thronestorm

Digital version of the Thronestorm card game, and an attempt to solve it.

- `RULES.md` — the rules as implemented, including rulings not on the printed sheet.
- `thronestorm/game.py` — rules engine (any number of houses >= 4).
- `thronestorm/agents.py` — baseline players (random, greedy).
- `thronestorm/sim.py` — self-play simulations.

```
python -m pytest
python -m thronestorm.sim --games 500 --houses 4 6 8 10
```

## Endgame search

`endgame/` (Rust) searches the deck-empty endgame for forced wins: random full
realms, both hands full, player 0 to move. The attacker may make a limited
number of quiet set-up moves; the defender gets every reply. Wins found are
proofs; positions left unresolved are not proven draws.

```
cd endgame && cargo build --release
# positions, max depth, seed, all-different-house hands (0|1), quiet moves, houses...
./target/release/endgame 500 3 1 0 1 5 6 7 8 9 10
```

## Trained player

`trainer/` (Rust, no dependencies) trains a value network by self-play and
plays it against baselines. Its rules are checked against the Python engine.

```
cd trainer && cargo build --release
# houses, games, output; rivalry=0 treats a draw as a loss, gamma<1 prefers quicker wins
./target/release/trainer train 10 200000 ../nets/h10-win.bin rivalry=0 gamma=0.98 resume=../nets/h10-win.bin
./target/release/trainer eval 10 2000 ../nets/h10-win.bin ../nets/h10-win.bin rivalry=0
./target/release/trainer eval 10 2000 ../nets/h10-win.bin ../nets/h10-half.bin rivalryA=0 rivalryB=1
./target/release/trainer eval 10 2000 ../nets/h10-win.bin greedy    # or random; FILE@N = lookahead of N moves (1 to 4, default 2), FILE@2d = always digs
./target/release/trainer transcript 10 200 1 | PYTHONPATH=.. python ../tests/verify_rust.py
```

`nets/h10-win.bin` was trained with a draw scored as a loss (rivalry 0, gamma 0.98);
`nets/h10-half.bin` with a draw scored as half a win (rivalry 1). Use the matching
`rivalry=` when playing them.

Other `train` options: `threads=`, `eps=` (exploration), `lambda=`, `lr=`, `seed=`, `depth=` (lookahead while training; 3 is a good trade-off).
Set `NOPASS=1` to play the variant where passing is only allowed with no other move.

## Web version and skins

`web/app.html` is the game page; `web/engine.js` is the rules and the trained
player in JavaScript, checked against the Rust trainer by `tests/web_check.js`.
`python web/build.py` writes one page per skin in `web/skins/`:

- `web/index.html`: Thronestorm
- `web/fallen-leaves/index.html`: Fallen Leaves (ten kinds of leaf in four colours, round tokens in a honeycomb)

The `gh-pages` branch holds the built pages and their art.

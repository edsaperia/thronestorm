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
./target/release/trainer train 10 200000 ../nets/h10.bin          # houses, games, output
./target/release/trainer train 10 200000 ../nets/h10.bin resume=../nets/h10-v2.bin
./target/release/trainer eval 10 2000 ../nets/h10.bin greedy       # or random, or another net; FILE@1 = one-move lookahead, FILE@2d = always digs
./target/release/trainer transcript 10 200 1 | PYTHONPATH=.. python ../tests/verify_rust.py
```

Options for `train`: `threads=`, `eps=` (exploration), `lambda=`, `lr=`, `seed=`.
Set `NOPASS=1` to play the variant where passing is only allowed with no other move.

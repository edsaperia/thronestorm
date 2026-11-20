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

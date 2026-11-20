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

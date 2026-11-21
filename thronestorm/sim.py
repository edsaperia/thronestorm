"""Self-play simulations.

    python -m thronestorm.sim --games 500 --houses 4 6 8 10
"""

from __future__ import annotations

import argparse
import random
from multiprocessing import Pool

from .agents import AGENTS
from .game import State


def play(n_houses, a0, a1, seed, partial=False, max_turns=300):
    rng = random.Random(seed)
    s = State.new(n_houses, rng, partial_diagonals=partial)
    agents = (AGENTS[a0], AGENTS[a1])
    turns = 0
    while s.winner is None and turns < max_turns:
        s.apply(agents[s.to_move](s, rng))
        turns += 1
    return s.winner, turns


def _job(args):
    return play(*args)


def run(n_houses, a0, a1, games, partial, pool):
    jobs = [(n_houses, a0, a1, seed, partial) for seed in range(games)]
    results = pool.map(_job, jobs, chunksize=8)
    w0 = sum(1 for w, _ in results if w == 0)
    w1 = sum(1 for w, _ in results if w == 1)
    draws = games - w0 - w1
    decided = [t for w, t in results if w is not None]
    mean = sum(decided) / len(decided) if decided else float("nan")
    return w0 / games, w1 / games, draws / games, mean


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--games", type=int, default=500)
    ap.add_argument("--houses", type=int, nargs="+", default=[4, 6, 8, 10])
    ap.add_argument("--matchups", nargs="+", default=["greedy:greedy", "greedy:random", "random:greedy"])
    args = ap.parse_args()
    print("houses  diagonals  first:second     P0 win  P1 win  unfinished  mean turns")
    with Pool() as pool:
        for n in args.houses:
            for partial in (False, True):
                for mu in args.matchups:
                    a0, a1 = mu.split(":")
                    w0, w1, d, mean = run(n, a0, a1, args.games, partial, pool)
                    print(f"{n:>6}  {'partial ' if partial else 'complete'}   {mu:<15}  "
                          f"{w0:>6.1%}  {w1:>6.1%}  {d:>10.1%}  {mean:>10.1f}", flush=True)


if __name__ == "__main__":
    main()

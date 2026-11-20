"""Baseline players. None of these look at face-down relics."""

from __future__ import annotations

import random

from .game import PASS, State, house


def random_agent(state: State, rng: random.Random):
    """Uniform over legal moves, but never passes when it can act."""
    moves = state.legal_moves()
    acting = [m for m in moves if m != PASS]
    return rng.choice(acting or moves)


def _best_set(inv) -> int:
    counts = {}
    for c in inv:
        if c is not None:
            counts[house(c)] = counts.get(house(c), 0) + 1
    return max(counts.values(), default=0)


def greedy_agent(state: State, rng: random.Random):
    """Win if possible; otherwise avoid moves that let the opponent win at
    once, and among those maximise the largest same-house set held."""
    me = state.to_move
    best, best_key = [], None
    for m in state.legal_moves():
        s = state.copy().apply(m, flip=False)
        if s.winner == me:
            return m
        key = (not s.can_win_now(1 - me), _best_set(s.inv[me]), m != PASS)
        if best_key is None or key > best_key:
            best, best_key = [m], key
        elif key == best_key:
            best.append(m)
    return rng.choice(best)


AGENTS = {"random": random_agent, "greedy": greedy_agent}

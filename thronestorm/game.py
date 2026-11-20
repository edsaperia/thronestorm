"""Thronestorm rules engine.

Cards are ints: card = house * 4 + type.
Types: 0 sword, 1 shield, 2 helmet, 3 chalice.

Realm coordinates are (row, x). Row 0 is the row nearest player 1, row 2 the
row nearest player 0, row 1 the middle. Rows are offset like brickwork, so x is
doubled: rows 0 and 2 use even x, row 1 uses odd x. The starting diamond is
(0, 0), (1, -1), (1, 1), (2, 0).

Column k (k = 1, 2, 3, ...) on a side sits at x = +/-(k + 1). Odd columns have
two slots (rows 0 and 2); even columns have one slot (row 1).
"""

from __future__ import annotations

import random
from dataclasses import dataclass, field
from itertools import permutations

SWORD, SHIELD, HELMET, CHALICE = range(4)
TYPE_NAMES = ("sword", "shield", "helmet", "chalice")
START_SLOTS = ((0, 0), (1, -1), (1, 1), (2, 0))

PASS = ("pass",)


def house(card: int) -> int:
    return card >> 2


def ctype(card: int) -> int:
    return card & 3


def bottom_row(player: int) -> int:
    return 2 if player == 0 else 0


def diagonals(player: int, x: int):
    """The two diagonals rising from this player's bottom-row slot at x."""
    b = bottom_row(player)
    t = 2 - b
    for d in (-1, 1):
        yield ((b, x), (1, x + d), (t, x + 2 * d))


@dataclass
class State:
    n_houses: int
    realm: dict = field(default_factory=dict)  # (row, x) -> card
    inv: list = field(default_factory=lambda: [[None] * 4, [None] * 4])
    decks: list = field(default_factory=lambda: [[], [], [], []])  # top = last
    revealed: list = field(default_factory=lambda: [True] * 4)
    to_move: int = 0
    winner: int | None = None
    partial_diagonals: bool = False  # variant: allow plots on incomplete diagonals

    @classmethod
    def new(cls, n_houses: int = 10, rng: random.Random | None = None,
            partial_diagonals: bool = False) -> "State":
        if n_houses < 4:
            raise ValueError("need at least 4 houses to fill the starting diamond")
        rng = rng or random.Random()
        s = cls(n_houses=n_houses, partial_diagonals=partial_diagonals)
        for t in range(4):
            deck = [h * 4 + t for h in range(n_houses)]
            rng.shuffle(deck)
            s.decks[t] = deck
        for slot in START_SLOTS:
            s.realm[slot] = s.decks[CHALICE].pop()
        return s

    def copy(self) -> "State":
        return State(
            n_houses=self.n_houses,
            realm=dict(self.realm),
            inv=[list(self.inv[0]), list(self.inv[1])],
            decks=[list(d) for d in self.decks],
            revealed=list(self.revealed),
            to_move=self.to_move,
            winner=self.winner,
            partial_diagonals=self.partial_diagonals,
        )

    # ----- queries -----

    def face_up(self, t: int):
        d = self.decks[t]
        return d[-1] if d and self.revealed[t] else None

    def open_slots(self):
        """Slots where a relic may be played from a claim."""
        out = []
        for sign in (-1, 1):
            k = 1
            while True:
                x = sign * (k + 1)
                slots = ((0, x), (2, x)) if k % 2 else ((1, x),)
                empty = [p for p in slots if p not in self.realm]
                if empty:
                    out.extend(empty)
                    break
                k += 1
        return out

    def plot_lines(self, player: int):
        """Diagonals this player may plot with: tuples of occupied positions."""
        houses = {house(c) for c in self.inv[player] if c is not None}
        if not houses:
            return []
        b = bottom_row(player)
        lines = []
        for (row, x), card in self.realm.items():
            if row != b or house(card) not in houses:
                continue
            for diag in diagonals(player, x):
                present = tuple(p for p in diag if p in self.realm)
                if len(present) == 3 or self.partial_diagonals:
                    lines.append(present)
        return lines

    def legal_moves(self):
        if self.winner is not None:
            return []
        p = self.to_move
        inv = self.inv[p]
        moves = [PASS]
        slots = None
        for t in range(4):
            c = self.face_up(t)
            if c is None:
                continue
            held = inv[t]
            if held is None:
                moves.append(("claim", t, None, None))
            else:
                if slots is None:
                    slots = self.open_slots()
                for played in (c, held):
                    for slot in slots:
                        moves.append(("claim", t, played, slot))
        held_cards = [c for c in inv if c is not None]
        for line in self.plot_lines(p):
            picked = [self.realm[pos] for pos in line]
            pool = picked + held_cards
            seen = set()
            for placed in permutations(pool, len(line)):
                if placed == tuple(picked):
                    continue  # null plot: same as passing
                if placed in seen:
                    continue
                seen.add(placed)
                rest = [c for c in pool if c not in placed]
                types = [ctype(c) for c in rest]
                if len(set(types)) == len(types):
                    moves.append(("plot", line, placed))
        return moves

    # ----- transitions -----

    def apply(self, move, flip: bool = True) -> "State":
        """Apply a move in place. With flip=False the next relic of a claimed
        deck stays face down (for lookahead that must not peek)."""
        p = self.to_move
        inv = self.inv[p]
        kind = move[0]
        if kind == "claim":
            _, t, played, slot = move
            c = self.decks[t].pop()
            if played is None:
                inv[t] = c
            else:
                keep = inv[t] if played == c else c
                inv[t] = keep
                self.realm[slot] = played
            self.revealed[t] = flip
        elif kind == "plot":
            _, line, placed = move
            pool = [self.realm[pos] for pos in line] + [c for c in inv if c is not None]
            for pos, c in zip(line, placed):
                self.realm[pos] = c
            new_inv = [None] * 4
            for c in pool:
                if c not in placed:
                    new_inv[ctype(c)] = c
            self.inv[p] = new_inv
        if has_won(self.inv[p]):
            self.winner = p
        self.to_move = 1 - p
        return self

    def can_win_now(self, player: int) -> bool:
        """Could `player` win with one move from here, using visible relics?"""
        inv = self.inv[player]
        for t in range(4):
            c = self.face_up(t)
            if c is None:
                continue
            h = house(c)
            if all(inv[u] is not None and house(inv[u]) == h for u in range(4) if u != t):
                return True
        if any(c is None for c in inv):
            return False  # a plot never changes inventory size
        for line in self.plot_lines(player):
            counts = {}
            for c in [self.realm[pos] for pos in line] + inv:
                counts[house(c)] = counts.get(house(c), 0) + 1
            if 4 in counts.values():
                return True
        return False

    # ----- display -----

    def render(self) -> str:
        def name(c):
            return "  .  " if c is None else f"{chr(65 + house(c))}{TYPE_NAMES[ctype(c)][:3]} "
        xs = [x for _, x in self.realm]
        lo, hi = min(xs) - 2, max(xs) + 2
        lines = ["P1 inv: " + " ".join(name(c) for c in self.inv[1])]
        for row in range(3):
            cells = []
            for x in range(lo, hi + 1):
                if (x - row) % 2:
                    cells.append("  ")
                else:
                    cells.append(name(self.realm.get((row, x))).strip().ljust(4)[:4]
                                 if (row, x) in self.realm else " .  ")
            lines.append("".join(cells))
        lines.append("P0 inv: " + " ".join(name(c) for c in self.inv[0]))
        lines.append("reliquary: " + " ".join(
            f"{name(self.face_up(t)).strip() or '-'}({len(self.decks[t])})" for t in range(4)))
        return "\n".join(lines)


def has_won(inv) -> bool:
    return all(c is not None for c in inv) and len({house(c) for c in inv}) == 1

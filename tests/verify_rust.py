"""Replay game transcripts from the Rust trainer through the Python engine.

    trainer/target/release/trainer transcript 10 200 1 | python tests/verify_rust.py

Checks that every Rust move is legal in Python, that both engines count the
same number of legal moves at every turn, and that they agree on the winner.
"""
import json
import sys

from thronestorm.game import START_SLOTS, State


def to_py(m):
    if m[0] == "pass":
        return ("pass",)
    if m[0] == "claim":
        return ("claim", m[1], m[2], tuple(m[3]) if m[3] else None)
    return ("plot", tuple(tuple(p) for p in m[1]), tuple(m[2]))


def main():
    games = turns = 0
    for line in sys.stdin:
        g = json.loads(line)
        s = State(n_houses=g["n"])
        s.decks = [list(d) for d in g["decks"]]
        s.realm = dict(zip(START_SLOTS, g["realm"]))
        for raw, count in zip(g["moves"], g["counts"]):
            legal = s.legal_moves()
            move = to_py(raw)
            assert len(legal) == count, (games, turns, len(legal), count)
            assert move in legal, (games, move)
            s.apply(move)
            turns += 1
        assert s.winner == g["winner"], (games, s.winner, g["winner"])
        games += 1
    print(f"ok: {games} games, {turns} turns agree")


if __name__ == "__main__":
    main()

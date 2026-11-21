import random

import pytest

from thronestorm.agents import greedy_agent, random_agent
from thronestorm.game import CHALICE, PASS, SHIELD, SWORD, State, ctype, has_won, house


def card(h, t):
    return h * 4 + t


def test_setup():
    s = State.new(10, random.Random(1))
    assert sorted(s.realm) == [(0, 0), (1, -1), (1, 1), (2, 0)]
    assert all(ctype(c) == CHALICE for c in s.realm.values())
    assert [len(d) for d in s.decks] == [10, 10, 10, 6]
    assert s.inv == [[None] * 4, [None] * 4]
    with pytest.raises(ValueError):
        State.new(3)


def test_first_turn_is_four_claims_or_pass():
    s = State.new(10, random.Random(1))
    moves = s.legal_moves()
    assert PASS in moves
    assert sorted(m[1] for m in moves if m[0] == "claim") == [0, 1, 2, 3]
    assert not [m for m in moves if m[0] == "plot"]


def test_column_fill_order():
    s = State.new(10, random.Random(1))
    assert sorted(s.open_slots()) == [(0, -2), (0, 2), (2, -2), (2, 2)]
    s.realm[(0, 2)] = card(0, SWORD)
    assert sorted(s.open_slots()) == [(0, -2), (2, -2), (2, 2)]
    s.realm[(2, 2)] = card(1, SWORD)
    assert sorted(s.open_slots()) == [(0, -2), (1, 3), (2, -2)]
    s.realm[(1, 3)] = card(2, SWORD)
    assert sorted(s.open_slots()) == [(0, -2), (0, 4), (2, -2), (2, 4)]


def test_claim_duplicate_must_play_one():
    s = State.new(10, random.Random(1))
    top = s.face_up(SWORD)
    s.apply(("claim", SWORD, None, None))
    assert s.inv[0][SWORD] == top and s.to_move == 1
    s.apply(PASS)
    new = s.face_up(SWORD)
    moves = [m for m in s.legal_moves() if m[0] == "claim" and m[1] == SWORD]
    assert {m[2] for m in moves} == {top, new}
    assert len(moves) == 8
    s.apply(("claim", SWORD, top, (2, 2)))
    assert s.inv[0][SWORD] == new and s.realm[(2, 2)] == top


def _plot_state(partial=False):
    s = State(n_houses=5, partial_diagonals=partial)
    s.realm = {
        (0, 0): card(4, CHALICE), (1, -1): card(3, CHALICE),
        (1, 1): card(0, SHIELD), (2, 0): card(0, CHALICE),
        (0, 2): card(0, 2),
    }
    s.inv[0] = [card(0, SWORD), None, None, None]
    s.decks = [[], [], [], []]
    return s


def test_plot_needs_complete_diagonal_and_house_match():
    s = _plot_state()
    assert s.plot_lines(0) == [((2, 0), (1, 1), (0, 2))]
    assert s.plot_lines(1) == []
    assert len(_plot_state(partial=True).plot_lines(0)) == 2


def test_plot_can_win_and_respects_type_limit():
    s = _plot_state()
    s.inv[0] = [card(0, SWORD), card(1, SHIELD), card(1, 2), card(1, CHALICE)]
    assert s.can_win_now(0)
    for m in s.legal_moves():
        t = s.copy().apply(m)
        held = [c for c in t.inv[0] if c is not None]
        assert len(held) == 4
        assert sorted(ctype(c) for c in held) == [0, 1, 2, 3]
        assert len(t.realm) == 5
    wins = [m for m in s.legal_moves() if s.copy().apply(m).winner == 0]
    assert len(wins) == 6  # the three leftovers in any order


def test_opponent_bottom_row_is_row_zero():
    s = _plot_state()
    s.to_move = 1
    s.inv[1] = [None, None, None, card(4, SWORD)]
    s.realm[(2, 2)] = card(2, SWORD)
    assert s.plot_lines(1) == [((0, 0), (1, 1), (2, 2))]


@pytest.mark.parametrize("agent", [random_agent, greedy_agent])
@pytest.mark.parametrize("partial", [False, True])
def test_playouts_conserve_cards(agent, partial):
    for seed in range(15):
        rng = random.Random(seed)
        s = State.new(6, rng, partial_diagonals=partial)
        for _ in range(200):
            if s.winner is not None:
                break
            assert s.can_win_now(s.to_move) == any(
                s.copy().apply(m, flip=False).winner == s.to_move for m in s.legal_moves())
            s.apply(agent(s, rng))
            cards = list(s.realm.values()) + [c for i in s.inv for c in i if c is not None]
            cards += [c for d in s.decks for c in d]
            assert sorted(cards) == list(range(24))
            for i in s.inv:
                assert all(c is None or ctype(c) == t for t, c in enumerate(i))
        if s.winner is not None:
            assert has_won(s.inv[s.winner])
            assert len({house(c) for c in s.inv[s.winner]}) == 1

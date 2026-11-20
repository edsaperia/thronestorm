//! Thronestorm rules (complete-diagonal ruleset), mirroring thronestorm/game.py.
//!
//! card = house * 4 + type. Rows: 0 is nearest player 1, 2 nearest player 0.
//! x is doubled so the brickwork offset is implicit (rows 0 and 2 use even x).

pub const XM: i32 = 40;
pub const W: usize = (2 * XM + 1) as usize;
pub const NC: usize = 3 * W;
pub const NONE: u8 = 255;
pub const MAXH: usize = 12;

pub fn cid(r: i32, x: i32) -> usize {
    r as usize * W + (x + XM) as usize
}
pub fn house(c: u8) -> usize {
    (c >> 2) as usize
}
pub fn ctype(c: u8) -> usize {
    (c & 3) as usize
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Move {
    Pass,
    Claim { t: usize, played: u8, slot: (i32, i32) },
    Plot { line: [(i32, i32); 3], placed: [u8; 3] },
}

#[derive(Clone, Copy)]
pub struct State {
    pub n: usize,
    pub cells: [u8; NC],
    pub lo: i32,
    pub hi: i32,
    pub hands: [[u8; 4]; 2],
    pub decks: [[u8; MAXH]; 4],
    pub dlen: [usize; 4],
    pub revealed: [bool; 4],
    pub to_move: usize,
    pub winner: Option<usize>,
}

pub struct Rng(pub u64);
impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
    pub fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    pub fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }
    pub fn normal(&mut self) -> f32 {
        let u = self.unit().max(1e-7);
        let v = self.unit();
        (-2.0 * u.ln()).sqrt() * (2.0 * std::f32::consts::PI * v).cos()
    }
}

pub const START: [(i32, i32); 4] = [(0, 0), (1, -1), (1, 1), (2, 0)];

pub fn hand_wins(h: &[u8; 4]) -> bool {
    h[0] != NONE && h.iter().all(|&c| c != NONE && house(c) == house(h[0]))
}

pub fn hand_mask(h: &[u8; 4]) -> u32 {
    h.iter().filter(|&&c| c != NONE).fold(0, |m, &c| m | (1 << house(c)))
}

impl State {
    pub fn new(n: usize, rng: &mut Rng) -> State {
        assert!((4..=MAXH).contains(&n));
        let mut s = State {
            n,
            cells: [NONE; NC],
            lo: -1,
            hi: 1,
            hands: [[NONE; 4]; 2],
            decks: [[NONE; MAXH]; 4],
            dlen: [n; 4],
            revealed: [true; 4],
            to_move: 0,
            winner: None,
        };
        for t in 0..4 {
            for h in 0..n {
                s.decks[t][h] = (h * 4 + t) as u8;
            }
            for i in (1..n).rev() {
                s.decks[t].swap(i, rng.below(i + 1));
            }
        }
        for (r, x) in START {
            s.dlen[3] -= 1;
            s.cells[cid(r, x)] = s.decks[3][s.dlen[3]];
        }
        s
    }

    pub fn face_up(&self, t: usize) -> u8 {
        if self.dlen[t] > 0 && self.revealed[t] {
            self.decks[t][self.dlen[t] - 1]
        } else {
            NONE
        }
    }

    pub fn open_slots(&self, out: &mut Vec<(i32, i32)>) {
        out.clear();
        for sign in [-1, 1] {
            let mut k = 1;
            loop {
                let x = sign * (k + 1);
                let before = out.len();
                if k % 2 == 1 {
                    for r in [0, 2] {
                        if self.cells[cid(r, x)] == NONE {
                            out.push((r, x));
                        }
                    }
                } else if self.cells[cid(1, x)] == NONE {
                    out.push((1, x));
                }
                if out.len() > before {
                    break;
                }
                k += 1;
            }
        }
    }

    /// Complete lines as [p's bottom cell, middle, p's top cell].
    pub fn lines(&self, p: usize, out: &mut Vec<[(i32, i32); 3]>) {
        out.clear();
        let b = if p == 0 { 2 } else { 0 };
        let t = 2 - b;
        let mut x = self.lo - (self.lo.rem_euclid(2));
        while x <= self.hi {
            if self.cells[cid(b, x)] != NONE {
                for d in [-1, 1] {
                    if self.cells[cid(1, x + d)] != NONE && self.cells[cid(t, x + 2 * d)] != NONE {
                        out.push([(b, x), (1, x + d), (t, x + 2 * d)]);
                    }
                }
            }
            x += 2;
        }
    }

    fn finish(mut self, p: usize) -> State {
        if hand_wins(&self.hands[p]) {
            self.winner = Some(p);
        }
        self.to_move = 1 - p;
        self
    }

    /// Every legal move with its resulting state. A claim leaves the next relic
    /// of that deck face down; call `flip` on the chosen state to reveal it.
    pub fn successors(&self, out: &mut Vec<(State, Move)>) {
        out.clear();
        if self.winner.is_some() {
            return;
        }
        let p = self.to_move;
        out.push((self.finish(p), Move::Pass));
        let mut slots = Vec::new();
        for t in 0..4 {
            let c = self.face_up(t);
            if c == NONE {
                continue;
            }
            let held = self.hands[p][t];
            let mut base = *self;
            base.dlen[t] -= 1;
            base.revealed[t] = false;
            if held == NONE {
                base.hands[p][t] = c;
                out.push((base.finish(p), Move::Claim { t, played: NONE, slot: (0, 0) }));
            } else {
                if slots.is_empty() {
                    self.open_slots(&mut slots);
                }
                for played in [c, held] {
                    for &slot in &slots {
                        let mut s = base;
                        s.hands[p][t] = if played == c { held } else { c };
                        s.cells[cid(slot.0, slot.1)] = played;
                        s.lo = s.lo.min(slot.1);
                        s.hi = s.hi.max(slot.1);
                        out.push((s.finish(p), Move::Claim { t, played, slot }));
                    }
                }
            }
        }
        let mask = hand_mask(&self.hands[p]);
        if mask == 0 {
            return;
        }
        let mut lines = Vec::new();
        self.lines(p, &mut lines);
        for line in &lines {
            let c = [
                self.cells[cid(line[0].0, line[0].1)],
                self.cells[cid(line[1].0, line[1].1)],
                self.cells[cid(line[2].0, line[2].1)],
            ];
            if mask & (1 << house(c[0])) == 0 {
                continue;
            }
            let mut pool = [NONE; 7];
            pool[..3].copy_from_slice(&c);
            let mut m = 3;
            for &h in &self.hands[p] {
                if h != NONE {
                    pool[m] = h;
                    m += 1;
                }
            }
            for i in 0..m {
                for j in 0..m {
                    if j == i {
                        continue;
                    }
                    for k in 0..m {
                        if k == i || k == j || (i, j, k) == (0, 1, 2) {
                            continue;
                        }
                        let mut nh = [NONE; 4];
                        let mut ok = true;
                        for (q, &card) in pool[..m].iter().enumerate() {
                            if q == i || q == j || q == k {
                                continue;
                            }
                            if nh[ctype(card)] != NONE {
                                ok = false;
                                break;
                            }
                            nh[ctype(card)] = card;
                        }
                        if !ok {
                            continue;
                        }
                        let placed = [pool[i], pool[j], pool[k]];
                        let mut s = *self;
                        s.hands[p] = nh;
                        for q in 0..3 {
                            s.cells[cid(line[q].0, line[q].1)] = placed[q];
                        }
                        out.push((s.finish(p), Move::Plot { line: *line, placed }));
                    }
                }
            }
        }
    }

    pub fn flip(&mut self) {
        self.revealed = [true; 4];
    }

    /// Could p win with one move from here, using only visible relics?
    pub fn can_win_now(&self, p: usize) -> bool {
        let hand = &self.hands[p];
        for t in 0..4 {
            let c = self.face_up(t);
            if c == NONE {
                continue;
            }
            if (0..4).all(|u| u == t || (hand[u] != NONE && house(hand[u]) == house(c))) {
                return true;
            }
        }
        if hand.iter().any(|&c| c == NONE) {
            return false;
        }
        let mask = hand_mask(hand);
        let mut lines = Vec::new();
        self.lines(p, &mut lines);
        let mut cnt = [0u8; MAXH];
        for &c in hand {
            cnt[house(c)] += 1;
        }
        for line in &lines {
            let cs = [
                self.cells[cid(line[0].0, line[0].1)],
                self.cells[cid(line[1].0, line[1].1)],
                self.cells[cid(line[2].0, line[2].1)],
            ];
            if mask & (1 << house(cs[0])) == 0 {
                continue;
            }
            let mut hit = false;
            for &c in &cs {
                cnt[house(c)] += 1;
                hit |= cnt[house(c)] == 4;
            }
            for &c in &cs {
                cnt[house(c)] -= 1;
            }
            if hit {
                return true;
            }
        }
        false
    }
}

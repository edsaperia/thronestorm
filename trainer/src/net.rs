//! Features and a small value network.
//!
//! The network estimates, for a position, each player's chance of winning. Houses are
//! interchangeable, so each house gets the same small sub-network and the
//! results are summed before the final layers.

use crate::game::*;

pub const F: usize = 27; // per-house features
pub const G: usize = 20; // global features
pub const D1: usize = 32;
pub const D2: usize = 64;

#[derive(Clone)]
pub struct Feats {
    pub houses: Vec<[f32; F]>,
    pub global: [f32; G],
}

/// Largest and second-largest value among the houses where `live` holds.
fn top2(v: &[f32; MAXH], live: &[bool; MAXH], n: usize) -> (f32, f32) {
    let (mut a, mut b) = (0f32, 0f32);
    for h in 0..n {
        if live[h] {
            if v[h] > a {
                b = a;
                a = v[h];
            } else if v[h] > b {
                b = v[h];
            }
        }
    }
    (a, b)
}

pub fn features(s: &State) -> Feats {
    let me = s.to_move;
    let n = s.n;
    let mut hand = [[0f32; MAXH]; 2];
    let mut face = [0f32; MAXH];
    let mut hidden = [0f32; MAXH];
    let mut realm = [0f32; MAXH];
    let mut keys = [[0f32; MAXH]; 2];
    let mut n_face = 0f32;
    let mut n_hidden = 0f32;
    let mut deck_hidden = [0f32; 4];
    let mut is_hidden = [false; MAXH * 4];
    for p in 0..2 {
        for &c in &s.hands[p] {
            if c != NONE {
                hand[p][house(c)] += 1.0;
            }
        }
    }
    for t in 0..4 {
        for i in 0..s.dlen[t] {
            let c = s.decks[t][i];
            if i + 1 == s.dlen[t] && s.revealed[t] {
                face[house(c)] += 1.0;
                n_face += 1.0;
            } else {
                hidden[house(c)] += 1.0;
                n_hidden += 1.0;
                deck_hidden[t] += 1.0;
                is_hidden[c as usize] = true;
            }
        }
    }
    let mut x = s.lo;
    while x <= s.hi {
        for r in 0..3 {
            let c = s.cells[cid(r, x)];
            if c != NONE {
                realm[house(c)] += 1.0;
                if r == 2 {
                    keys[0][house(c)] += 1.0;
                }
                if r == 0 {
                    keys[1][house(c)] += 1.0;
                }
            }
        }
        x += 1;
    }
    // Lines each player can trigger now, and the cells those lines cover.
    let mut lines: [Vec<[usize; 3]>; 2] = [Vec::new(), Vec::new()];
    let mut usable: [Vec<bool>; 2] = [Vec::new(), Vec::new()];
    let mut seen = [[false; NC]; 2];
    let mut reach = [[0f32; MAXH]; 2];
    let mut raw = Vec::new();
    for p in 0..2 {
        let mask = hand_mask(&s.hands[p]);
        s.lines(p, &mut raw);
        for line in &raw {
            let ids = [cid(line[0].0, line[0].1), cid(line[1].0, line[1].1), cid(line[2].0, line[2].1)];
            let ok = mask & (1 << house(s.cells[ids[0]])) != 0;
            if ok {
                for &id in &ids {
                    if !seen[p][id] {
                        seen[p][id] = true;
                        reach[p][house(s.cells[id])] += 1.0;
                    }
                }
            }
            lines[p].push(ids);
            usable[p].push(ok);
        }
    }
    // pool: best hand + line count on a line I can trigger.
    // safe: the same, but only on lines the opponent cannot touch this turn.
    // potential: the same over every complete line, triggerable or not.
    let mut pool = hand;
    let mut safe = hand;
    let mut potential = hand;
    for p in 0..2 {
        for (ids, &ok) in lines[p].iter().zip(&usable[p]) {
            let contested = ids.iter().any(|&id| seen[1 - p][id]);
            let mut cnt = [0f32; MAXH];
            for &id in ids {
                cnt[house(s.cells[id])] += 1.0;
            }
            for h in 0..n {
                if cnt[h] > 0.0 {
                    let v = hand[p][h] + cnt[h];
                    potential[p][h] = potential[p][h].max(v);
                    if ok {
                        pool[p][h] = pool[p][h].max(v);
                        if !contested {
                            safe[p][h] = safe[p][h].max(v);
                        }
                    }
                }
            }
        }
    }
    // Holding three of a house with the fourth still face down: the chance
    // that the next flip of that deck reveals it.
    let mut out = [[0f32; MAXH]; 2];
    for p in 0..2 {
        for t in 0..4 {
            let others: Vec<u8> = (0..4).filter(|&u| u != t).map(|u| s.hands[p][u]).collect();
            if others.iter().all(|&c| c != NONE && house(c) == house(others[0])) {
                let h = house(others[0]);
                if is_hidden[h * 4 + t] {
                    out[p][h] = 1.0 / deck_hidden[t].max(1.0);
                }
            }
        }
    }
    let o = 1 - me;
    let b = |v: bool| if v { 1.0 } else { 0.0 };
    let mut live = [[false; MAXH]; 2];
    for h in 0..n {
        live[me][h] = hand[o][h] == 0.0;
        live[o][h] = hand[me][h] == 0.0;
    }
    let mut houses = Vec::with_capacity(n);
    for h in 0..n {
        let my_live = b(live[me][h]);
        let op_live = b(live[o][h]);
        houses.push([
            hand[me][h] / 4.0,
            hand[o][h] / 4.0,
            face[h] / 4.0,
            hidden[h] / 4.0,
            realm[h] / 4.0,
            pool[me][h] / 4.0,
            pool[o][h] / 4.0,
            reach[me][h] / 4.0,
            reach[o][h] / 4.0,
            my_live,
            op_live,
            keys[me][h] / 4.0,
            keys[o][h] / 4.0,
            b(hand[me][h] >= 2.0),
            b(hand[me][h] >= 3.0),
            b(hand[o][h] >= 2.0),
            b(hand[o][h] >= 3.0),
            b(pool[me][h] >= 3.0),
            b(pool[o][h] >= 3.0),
            my_live * pool[me][h] / 4.0,
            op_live * pool[o][h] / 4.0,
            my_live * potential[me][h] / 4.0,
            op_live * potential[o][h] / 4.0,
            my_live * safe[me][h] / 4.0,
            op_live * safe[o][h] / 4.0,
            out[me][h],
            out[o][h],
        ]);
    }
    let size = |p: usize| s.hands[p].iter().filter(|&&c| c != NONE).count() as f32 / 4.0;
    let div = |p: usize| hand_mask(&s.hands[p]).count_ones() as f32 / 4.0;
    let count = |p: usize| usable[p].iter().filter(|&&u| u).count() as f32 / 20.0;
    let (mp1, mp2) = top2(&pool[me], &live[me], n);
    let (op1, op2) = top2(&pool[o], &live[o], n);
    let (ms1, ms2) = top2(&safe[me], &live[me], n);
    let (os1, os2) = top2(&safe[o], &live[o], n);
    let global = [
        size(me),
        size(o),
        count(me),
        count(o),
        lines[0].len() as f32 / 20.0,
        n_hidden / 36.0,
        n_face / 4.0,
        b(s.can_win_now(me)),
        b(s.can_win_now(o)),
        1.0,
        div(me),
        div(o),
        mp1 / 4.0,
        mp2 / 4.0,
        op1 / 4.0,
        op2 / 4.0,
        ms1 / 4.0,
        ms2 / 4.0,
        os1 / 4.0,
        os2 / 4.0,
    ];
    Feats { houses, global }
}

const O_W1: usize = 0;
const O_B1: usize = O_W1 + F * D1;
const O_W2: usize = O_B1 + D1;
const O_B2: usize = O_W2 + (D1 + G) * D2;
const O_W3: usize = O_B2 + D2;
const O_B3: usize = O_W3 + D2 * 2;
pub const NP: usize = O_B3 + 2;
const POOL_SCALE: f32 = 0.25;

/// Two outputs in [0, 1]: the (discounted) chance that the player to move
/// wins, and the chance that the other player wins. They need not sum to one;
/// the remainder is the chance that nobody does.
#[derive(Clone)]
pub struct Net {
    pub p: Vec<f32>,
}

impl Net {
    pub fn new(rng: &mut Rng) -> Net {
        let mut p = vec![0f32; NP];
        let mut init = |from: usize, to: usize, fan_in: usize| {
            let s = 1.0 / (fan_in as f32).sqrt();
            for v in &mut p[from..to] {
                *v = rng.normal() * s;
            }
        };
        init(O_W1, O_B1, F);
        init(O_W2, O_B2, D1 + G);
        init(O_W3, O_B3, D2);
        for v in &mut p[O_W3..O_B3] {
            *v *= 0.1; // start flat so early play is close to uniform
        }
        Net { p }
    }

    /// Forward pass. If `train` is given, accumulates the gradient of the
    /// squared error against the two targets.
    pub fn eval(&self, f: &Feats, train: Option<([f32; 2], &mut [f32])>) -> [f32; 2] {
        let p = &self.p;
        let nh = f.houses.len();
        let mut h1 = vec![0f32; nh * D1];
        let mut z = [0f32; D1 + G];
        for (hi, x) in f.houses.iter().enumerate() {
            for j in 0..D1 {
                let mut a = p[O_B1 + j];
                for i in 0..F {
                    a += x[i] * p[O_W1 + i * D1 + j];
                }
                let a = a.tanh();
                h1[hi * D1 + j] = a;
                z[j] += a * POOL_SCALE;
            }
        }
        z[D1..].copy_from_slice(&f.global);
        let mut h2 = [0f32; D2];
        for j in 0..D2 {
            let mut a = p[O_B2 + j];
            for i in 0..D1 + G {
                a += z[i] * p[O_W2 + i * D2 + j];
            }
            h2[j] = a.tanh();
        }
        let mut v = [0f32; 2];
        for k in 0..2 {
            let mut a = p[O_B3 + k];
            for j in 0..D2 {
                a += h2[j] * p[O_W3 + j * 2 + k];
            }
            v[k] = 1.0 / (1.0 + (-a).exp());
        }
        if let Some((target, g)) = train {
            let d3 = [(v[0] - target[0]) * v[0] * (1.0 - v[0]), (v[1] - target[1]) * v[1] * (1.0 - v[1])];
            let mut dz = [0f32; D1 + G];
            for k in 0..2 {
                g[O_B3 + k] += d3[k];
            }
            for j in 0..D2 {
                let mut dh = 0.0;
                for k in 0..2 {
                    g[O_W3 + j * 2 + k] += d3[k] * h2[j];
                    dh += d3[k] * p[O_W3 + j * 2 + k];
                }
                let d2 = dh * (1.0 - h2[j] * h2[j]);
                g[O_B2 + j] += d2;
                for i in 0..D1 + G {
                    g[O_W2 + i * D2 + j] += d2 * z[i];
                    dz[i] += d2 * p[O_W2 + i * D2 + j];
                }
            }
            for (hi, x) in f.houses.iter().enumerate() {
                for j in 0..D1 {
                    let a = h1[hi * D1 + j];
                    let d1 = dz[j] * POOL_SCALE * (1.0 - a * a);
                    g[O_B1 + j] += d1;
                    for i in 0..F {
                        g[O_W1 + i * D1 + j] += d1 * x[i];
                    }
                }
            }
        }
        v
    }

    pub fn save(&self, path: &str) {
        let mut bytes = Vec::with_capacity(NP * 4 + 16);
        for v in [F as u32, G as u32, D1 as u32, D2 as u32 + 1000] {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        for v in &self.p {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        std::fs::write(path, bytes).expect("could not write network file");
    }

    pub fn load(path: &str) -> Net {
        let bytes = std::fs::read(path).expect("could not read network file");
        let dims: Vec<u32> = (0..4).map(|i| u32::from_le_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap())).collect();
        assert_eq!(dims, [F as u32, G as u32, D1 as u32, D2 as u32 + 1000], "network file has a different shape");
        let p = bytes[16..].chunks(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect::<Vec<_>>();
        assert_eq!(p.len(), NP);
        Net { p }
    }
}

pub struct Adam {
    m: Vec<f32>,
    v: Vec<f32>,
    t: i32,
    pub lr: f32,
}

impl Adam {
    pub fn new(lr: f32) -> Adam {
        Adam { m: vec![0.0; NP], v: vec![0.0; NP], t: 0, lr }
    }
    pub fn step(&mut self, net: &mut Net, grad: &[f32], batch: f32) {
        self.t += 1;
        let (b1, b2) = (0.9f32, 0.999f32);
        let c1 = 1.0 - b1.powi(self.t);
        let c2 = 1.0 - b2.powi(self.t);
        for i in 0..NP {
            let g = grad[i] / batch;
            self.m[i] = b1 * self.m[i] + (1.0 - b1) * g;
            self.v[i] = b2 * self.v[i] + (1.0 - b2) * g * g;
            net.p[i] -= self.lr * (self.m[i] / c1) / ((self.v[i] / c2).sqrt() + 1e-8);
        }
    }
}

//! Features and a small value network.
//!
//! The network scores a position for the player to move, in [-1, 1]. Houses are
//! interchangeable, so each house gets the same small sub-network and the
//! results are summed before the final layers.

use crate::game::*;

pub const F: usize = 21; // per-house features
pub const G: usize = 12; // global features
pub const D1: usize = 32;
pub const D2: usize = 64;

#[derive(Clone)]
pub struct Feats {
    pub houses: Vec<[f32; F]>,
    pub global: [f32; G],
}

pub fn features(s: &State) -> Feats {
    let me = s.to_move;
    let n = s.n;
    let mut hand = [[0f32; MAXH]; 2];
    let mut face = [0f32; MAXH];
    let mut hidden = [0f32; MAXH];
    let mut realm = [0f32; MAXH];
    let mut keys = [[0f32; MAXH]; 2];
    let mut pool = [[0f32; MAXH]; 2];
    let mut reach = [[0f32; MAXH]; 2];
    let mut usable = [0f32; 2];
    let mut total_lines = 0f32;
    let mut n_face = 0f32;
    let mut n_hidden = 0f32;
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
    let mut lines = Vec::new();
    for p in 0..2 {
        pool[p] = hand[p];
        let mask = hand_mask(&s.hands[p]);
        s.lines(p, &mut lines);
        if p == 0 {
            total_lines = lines.len() as f32;
        }
        let mut seen = [false; NC];
        for line in &lines {
            let ids = [cid(line[0].0, line[0].1), cid(line[1].0, line[1].1), cid(line[2].0, line[2].1)];
            if mask & (1 << house(s.cells[ids[0]])) == 0 {
                continue;
            }
            usable[p] += 1.0;
            let mut cnt = [0f32; MAXH];
            for &id in &ids {
                let h = house(s.cells[id]);
                cnt[h] += 1.0;
                if !seen[id] {
                    seen[id] = true;
                    reach[p][h] += 1.0;
                }
            }
            for h in 0..n {
                if cnt[h] > 0.0 {
                    pool[p][h] = pool[p][h].max(hand[p][h] + cnt[h]);
                }
            }
        }
    }
    let o = 1 - me;
    let b = |v: bool| if v { 1.0 } else { 0.0 };
    let mut houses = Vec::with_capacity(n);
    for h in 0..n {
        let my_live = b(hand[o][h] == 0.0);
        let op_live = b(hand[me][h] == 0.0);
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
        ]);
    }
    let size = |p: usize| s.hands[p].iter().filter(|&&c| c != NONE).count() as f32 / 4.0;
    let div = |p: usize| hand_mask(&s.hands[p]).count_ones() as f32 / 4.0;
    let global = [
        size(me),
        size(o),
        usable[me] / 20.0,
        usable[o] / 20.0,
        total_lines / 20.0,
        n_hidden / 36.0,
        n_face / 4.0,
        b(s.can_win_now(me)),
        b(s.can_win_now(o)),
        1.0,
        div(me),
        div(o),
    ];
    Feats { houses, global }
}

const O_W1: usize = 0;
const O_B1: usize = O_W1 + F * D1;
const O_W2: usize = O_B1 + D1;
const O_B2: usize = O_W2 + (D1 + G) * D2;
const O_W3: usize = O_B2 + D2;
const O_B3: usize = O_W3 + D2;
pub const NP: usize = O_B3 + 1;
const POOL_SCALE: f32 = 0.25;

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
            *v *= 0.1; // start near zero so early play is close to uniform
        }
        Net { p }
    }

    /// Forward pass. If `grad` is given, accumulates d(0.5 * (v - target)^2).
    pub fn eval(&self, f: &Feats, train: Option<(f32, &mut [f32])>) -> f32 {
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
        let mut a = p[O_B3];
        for j in 0..D2 {
            a += h2[j] * p[O_W3 + j];
        }
        let v = a.tanh();
        if let Some((target, g)) = train {
            let d3 = (v - target) * (1.0 - v * v);
            g[O_B3] += d3;
            let mut dz = [0f32; D1 + G];
            for j in 0..D2 {
                g[O_W3 + j] += d3 * h2[j];
                let d2 = d3 * p[O_W3 + j] * (1.0 - h2[j] * h2[j]);
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
        for v in [F as u32, G as u32, D1 as u32, D2 as u32] {
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
        assert_eq!(dims, [F as u32, G as u32, D1 as u32, D2 as u32], "network file has a different shape");
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

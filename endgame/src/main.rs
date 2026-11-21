//! Forced-win search in the deck-empty Thronestorm endgame.
//!
//! Decks are empty, the realm is full and both hands hold four relics, so the
//! game is deterministic with nothing hidden. The attacker may make a limited
//! number of quiet moves; all his other moves must threaten to win at once.
//! The defender gets every legal reply. A win found this way is a proof. A
//! failure to find one is not a proof of a draw.
//!
//! usage: endgame <positions> <max_depth> <seed> <diverse 0|1> <quiet moves> <houses...>

use std::collections::HashMap;
use std::env;
use std::thread;

const MAXC: usize = 40;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Pos {
    cells: [u8; MAXC],
    hands: [[u8; 4]; 2],
}

/// Complete lines as [row-2 cell, row-1 cell, row-0 cell].
/// Player 0 triggers on index 0, player 1 on index 2.
struct Geo {
    n_cells: usize,
    lines: Vec<[usize; 3]>,
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn house(c: u8) -> usize {
    (c >> 2) as usize
}
fn ctype(c: u8) -> usize {
    (c & 3) as usize
}

fn build_geo(target: usize, rng: &mut Rng) -> Geo {
    let mut idx: HashMap<(i32, i32), usize> = HashMap::new();
    for p in [(0, 0), (1, -1), (1, 1), (2, 0)] {
        let n = idx.len();
        idx.insert(p, n);
    }
    let rest = target - 4;
    let extra = if rest % 2 == 1 { rng.below(2) } else { 2 };
    for (s, sign) in [-1i32, 1].iter().enumerate() {
        let mut m = rest / 2 + if s == extra { 1 } else { 0 };
        let mut k = 1;
        while m > 0 {
            let x = sign * (k + 1);
            let slots: Vec<(i32, i32)> = if k % 2 == 1 { vec![(0, x), (2, x)] } else { vec![(1, x)] };
            if m >= slots.len() {
                for p in &slots {
                    let n = idx.len();
                    idx.insert(*p, n);
                }
                m -= slots.len();
            } else {
                let n = idx.len();
                idx.insert(slots[rng.below(2)], n);
                m = 0;
            }
            k += 1;
        }
    }
    let mut lines = Vec::new();
    let mut keys: Vec<(i32, i32)> = idx.keys().cloned().collect();
    keys.sort();
    for (row, x) in keys {
        if row != 2 {
            continue;
        }
        for d in [-1, 1] {
            if let (Some(&m), Some(&t)) = (idx.get(&(1, x + d)), idx.get(&(0, x + 2 * d))) {
                lines.push([idx[&(2, x)], m, t]);
            }
        }
    }
    Geo { n_cells: idx.len(), lines }
}

fn hand_mask(h: &[u8; 4]) -> u16 {
    h.iter().fold(0, |m, &c| m | (1 << house(c)))
}

fn hand_wins(h: &[u8; 4]) -> bool {
    let a = house(h[0]);
    h.iter().all(|&c| house(c) == a)
}

const PERMS: [[usize; 3]; 6] = [[0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]];

/// All plot successors for player p, with a flag for an immediate win.
fn successors(pos: &Pos, geo: &Geo, p: usize, out: &mut Vec<(Pos, bool)>) {
    out.clear();
    let hand = pos.hands[p];
    let mask = hand_mask(&hand);
    let ti = if p == 0 { 0 } else { 2 };
    for line in &geo.lines {
        let c = [pos.cells[line[0]], pos.cells[line[1]], pos.cells[line[2]]];
        if mask & (1 << house(c[ti])) == 0 {
            continue;
        }
        let mut cand = [[0u8; 4]; 4];
        let mut cnt = [0usize; 4];
        for t in 0..4 {
            cand[t][0] = hand[t];
            cnt[t] = 1;
        }
        for &x in &c {
            let t = ctype(x);
            cand[t][cnt[t]] = x;
            cnt[t] += 1;
        }
        let pool = [c[0], c[1], c[2], hand[0], hand[1], hand[2], hand[3]];
        for i0 in 0..cnt[0] {
            for i1 in 0..cnt[1] {
                for i2 in 0..cnt[2] {
                    for i3 in 0..cnt[3] {
                        let nh = [cand[0][i0], cand[1][i1], cand[2][i2], cand[3][i3]];
                        let mut rest = [0u8; 3];
                        let mut r = 0;
                        for &x in &pool {
                            if !nh.contains(&x) {
                                rest[r] = x;
                                r += 1;
                            }
                        }
                        let win = hand_wins(&nh);
                        for perm in &PERMS {
                            let placed = [rest[perm[0]], rest[perm[1]], rest[perm[2]]];
                            if nh == hand && placed == c {
                                continue;
                            }
                            let mut q = *pos;
                            q.hands[p] = nh;
                            for j in 0..3 {
                                q.cells[line[j]] = placed[j];
                            }
                            out.push((q, win));
                            if win {
                                break; // order of the leftovers is irrelevant
                            }
                        }
                    }
                }
            }
        }
    }
}

fn can_win(pos: &Pos, geo: &Geo, p: usize) -> bool {
    let hand = pos.hands[p];
    let mask = hand_mask(&hand);
    let ti = if p == 0 { 0 } else { 2 };
    let mut cnt = [0u8; 16];
    for &c in &hand {
        cnt[house(c)] += 1;
    }
    for line in &geo.lines {
        if mask & (1 << house(pos.cells[line[ti]])) == 0 {
            continue;
        }
        let mut hit = false;
        for j in 0..3 {
            let h = house(pos.cells[line[j]]);
            cnt[h] += 1;
            if cnt[h] == 4 {
                hit = true;
            }
        }
        for j in 0..3 {
            cnt[house(pos.cells[line[j]])] -= 1;
        }
        if hit {
            return true;
        }
    }
    false
}

struct Ctx {
    memo: HashMap<(Pos, u8, u8, u8), bool>,
    nodes: u64,
    limit: u64,
    aborted: bool,
    killer: [[u8; 4]; 32], // defender hand that last refuted an attack, by depth
}

/// Attacker `a` to move: can he force a win within `depth` of his own moves?
/// He may make at most `quiet` moves that do not threaten an immediate win;
/// the defender always gets every legal reply, including a pass.
fn attack(pos: &Pos, geo: &Geo, a: usize, depth: u8, quiet: u8, cx: &mut Ctx) -> bool {
    if can_win(pos, geo, a) {
        return true;
    }
    if depth <= 1 {
        return false;
    }
    cx.nodes += 1;
    if cx.nodes > cx.limit {
        cx.aborted = true;
        return false;
    }
    let key = (*pos, a as u8, depth, quiet);
    if let Some(&v) = cx.memo.get(&key) {
        return v;
    }
    let d = 1 - a;
    let mut mine = Vec::new();
    let mut theirs = Vec::new();
    successors(pos, geo, a, &mut mine);
    let mut result = false;
    'outer: for pass in 0..2 {
        // forcing moves first, then quiet ones
        for (s, _) in &mine {
            let forcing = can_win(s, geo, a);
            if forcing != (pass == 0) || (!forcing && quiet == 0) || can_win(s, geo, d) {
                continue;
            }
            let q = if forcing { quiet } else { quiet - 1 };
            // a pass never parries a live threat, so it only needs testing after a quiet move
            if !forcing && !attack(s, geo, a, depth - 1, q, cx) {
                continue;
            }
            successors(s, geo, d, &mut theirs);
            let killer = cx.killer[depth as usize];
            let mut refuted = false;
            for first in [true, false] {
                for (r, _) in &theirs {
                    if (r.hands[d] == killer) != first {
                        continue;
                    }
                    if !attack(r, geo, a, depth - 1, q, cx) {
                        cx.killer[depth as usize] = r.hands[d];
                        refuted = true;
                        break;
                    }
                }
                if refuted {
                    break;
                }
            }
            if cx.aborted {
                return false;
            }
            if !refuted {
                result = true;
                break 'outer;
            }
        }
    }
    if cx.aborted {
        return false;
    }
    cx.memo.insert(key, result);
    result
}

/// Player 0 to move. Returns (depth of the mover's forced win, depth of the
/// opponent's forced win if the mover passes, plots available, budget hit).
fn classify(pos: &Pos, geo: &Geo, max_depth: u8, quiet: u8, limit: u64) -> (Option<u8>, Option<u8>, usize, bool) {
    let mut moves = Vec::new();
    successors(pos, geo, 0, &mut moves);
    let n_moves = moves.len();
    let mut hit = false;
    for who in 0..2 {
        let mut cx = Ctx { memo: HashMap::new(), nodes: 0, limit, aborted: false, killer: [[255; 4]; 32] };
        for k in 1..=max_depth {
            if attack(pos, geo, who, k, quiet.min(k.saturating_sub(2)), &mut cx) {
                if who == 0 {
                    return (Some(k), None, n_moves, false);
                }
                // The opponent wins if the mover passes. Does any plot by the
                // mover escape every attack within the same horizon?
                // Encoded as +100 on the depth.
                let q = quiet.min(max_depth.saturating_sub(2));
                let escape = moves.iter().any(|(s, _)| !attack(s, geo, 1, max_depth, q, &mut cx));
                return (None, Some(if escape { k + 100 } else { k }), n_moves, hit);
            }
            if cx.aborted {
                hit = true;
                break;
            }
        }
    }
    (None, None, n_moves, hit)
}

fn random_position(n: usize, diverse: bool, rng: &mut Rng) -> (Pos, Geo) {
    let geo = build_geo(4 * n - 8, rng);
    loop {
        let mut hands = [[0u8; 4]; 2];
        let mut used = vec![false; 4 * n];
        for t in 0..4 {
            let h0 = rng.below(n);
            let mut h1 = rng.below(n - 1);
            if h1 >= h0 {
                h1 += 1;
            }
            hands[0][t] = (h0 * 4 + t) as u8;
            hands[1][t] = (h1 * 4 + t) as u8;
            used[h0 * 4 + t] = true;
            used[h1 * 4 + t] = true;
        }
        if diverse && (hand_mask(&hands[0]).count_ones() < 4 || hand_mask(&hands[1]).count_ones() < 4) {
            continue;
        }
        if hand_wins(&hands[0]) || hand_wins(&hands[1]) {
            continue;
        }
        let mut rest: Vec<u8> = (0..4 * n).filter(|&c| !used[c]).map(|c| c as u8).collect();
        for i in (1..rest.len()).rev() {
            rest.swap(i, rng.below(i + 1));
        }
        let mut cells = [255u8; MAXC];
        cells[..geo.n_cells].copy_from_slice(&rest);
        return (Pos { cells, hands }, geo);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let positions: usize = args[1].parse().unwrap();
    let max_depth: u8 = args[2].parse().unwrap();
    let seed: u64 = args[3].parse().unwrap();
    let diverse = args[4] == "1";
    let quiet: u8 = args[5].parse().unwrap();
    let houses: Vec<usize> = args[6..].iter().map(|s| s.parse().unwrap()).collect();
    let limit: u64 = env::var("NODES").ok().and_then(|v| v.parse().ok()).unwrap_or(u64::MAX);
    let threads = thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
    println!("positions={positions} max_depth={max_depth} quiet_moves={quiet} diverse_hands={diverse}");
    println!("depth = number of the winner's own moves; columns are cumulative % of positions");
    for n in houses {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                thread::spawn(move || {
                    let mut out = Vec::new();
                    let mut i = t;
                    while i < positions {
                        let mut rng = Rng(seed.wrapping_mul(1_000_003).wrapping_add((n * 100_000 + i) as u64));
                        let (pos, geo) = random_position(n, diverse, &mut rng);
                        out.push((classify(&pos, &geo, max_depth, quiet, limit), geo.lines.len()));
                        i += threads;
                    }
                    out
                })
            })
            .collect();
        let res: Vec<_> = handles.into_iter().flat_map(|h| h.join().unwrap()).collect();
        let total = res.len() as f64;
        let mut line = format!("houses {n:>2} | mover wins by depth:");
        for k in 1..=max_depth {
            let c = res.iter().filter(|((w, _, _, _), _)| w.map_or(false, |d| d <= k)).count();
            line += &format!(" {k}:{:5.1}%", 100.0 * c as f64 / total);
        }
        line += " | else opponent wins if mover passes, by depth:";
        for k in 1..=max_depth {
            let c = res.iter().filter(|((_, l, _, _), _)| l.map_or(false, |d| d % 100 <= k)).count();
            line += &format!(" {k}:{:5.1}%", 100.0 * c as f64 / total);
        }
        let open = res.iter().filter(|((w, l, _, _), _)| w.is_none() && l.is_none()).count();
        let moves: f64 = res.iter().map(|((_, _, m, _), _)| *m as f64).sum::<f64>() / total;
        let lines: f64 = res.iter().map(|(_, l)| *l as f64).sum::<f64>() / total;
        let esc = res.iter().filter(|((_, l, _, _), _)| l.map_or(false, |d| d >= 100)).count();
        line += &format!(" (mover has an escape in {:5.1}% of all positions)", 100.0 * esc as f64 / total);
        let hit = res.iter().filter(|((w, l, _, h), _)| w.is_none() && l.is_none() && *h).count();
        line += &format!(" | budget hit {:5.1}%", 100.0 * hit as f64 / total);
        line += &format!(" | unresolved {:5.1}% | mean plots {moves:.0}, lines {lines:.0}", 100.0 * open as f64 / total);
        println!("{line}");
    }
}

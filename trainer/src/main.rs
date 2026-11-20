//! Self-play trainer for Thronestorm.
//!
//!   trainer train <houses> <games> <net.bin> [threads=N] [resume=FILE] [eps=0.1]
//!                 [lambda=0.7] [lr=0.001] [seed=1] [depth=2]
//!   trainer eval <houses> <games> <A> <B>      A, B: greedy | random | FILE | FILE@1 | FILE@2d
//!   trainer transcript <houses> <games> <seed>
//!
//! Every player first applies exact one-move tactics: take a win if there is
//! one, and avoid moves that hand the opponent an immediate win. With
//! lookahead 2 the network player also plays any move that leaves the opponent
//! no answer, and avoids moves that allow the opponent one. The network
//! chooses among what is left.

mod game;
mod net;

use game::*;
use net::*;
use std::sync::Arc;

const MAX_TURNS: usize = 200;

/// Rule variant, set with NOPASS=1: a player may pass only with no other legal move.
fn no_pass() -> bool {
    static FLAG: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *FLAG.get_or_init(|| std::env::var("NOPASS").map_or(false, |v| v == "1"))
}

#[derive(Clone)]
enum Player {
    Net(Arc<Net>, f32, u8, bool), // network, exploration rate, lookahead (1 or 2), always dig
    Greedy,
    Random,
}

fn best_set(hand: &[u8; 4]) -> i32 {
    let mut cnt = [0i32; MAXH];
    let mut best = 0;
    for &c in hand {
        if c != NONE {
            cnt[house(c)] += 1;
            best = best.max(cnt[house(c)]);
        }
    }
    best
}

/// Returns the index of the chosen successor.
fn choose(pl: &Player, s: &State, succ: &[(State, Move)], rng: &mut Rng) -> usize {
    let p = s.to_move;
    if let Some(i) = succ.iter().position(|(t, _)| t.winner == Some(p)) {
        return i;
    }
    let mut cands: Vec<usize> = (0..succ.len()).filter(|&i| !succ[i].0.can_win_now(1 - p)).collect();
    if cands.is_empty() {
        cands = (0..succ.len()).collect();
    }
    match pl {
        Player::Random => {
            if cands.len() > 1 {
                cands.retain(|&i| succ[i].1 != Move::Pass);
            }
            cands[rng.below(cands.len())]
        }
        Player::Greedy => {
            let key = |i: usize| (best_set(&succ[i].0.hands[p]), (succ[i].1 != Move::Pass) as i32);
            let top = cands.iter().map(|&i| key(i)).max().unwrap();
            let best: Vec<usize> = cands.into_iter().filter(|&i| key(i) == top).collect();
            best[rng.below(best.len())]
        }
        Player::Net(net, eps, depth, dig) => {
            if *eps > 0.0 && rng.unit() < *eps {
                return cands[rng.below(cands.len())];
            }
            let mut buf = Vec::new();
            if *depth >= 2 {
                // a move that leaves the opponent no answer to my threats
                if let Some(&i) = cands.iter().find(|&&i| forced_win(&succ[i].0, p, &mut buf)) {
                    return i;
                }
            }
            if *dig {
                // the digger claims whenever a claim is among its safe moves
                let claims: Vec<usize> =
                    cands.iter().cloned().filter(|&i| matches!(succ[i].1, Move::Claim { .. })).collect();
                if !claims.is_empty() {
                    cands = claims;
                }
            }
            // the successor is scored for the opponent, so lower is better
            let mut scored: Vec<(f32, usize)> =
                cands.iter().map(|&i| (net.eval(&features(&succ[i].0), None), i)).collect();
            scored.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            if *depth >= 2 {
                // skip moves that let the opponent set up such a position himself
                let mut theirs = Vec::new();
                for &(_, i) in scored.iter().take(CHECKED) {
                    succ[i].0.successors(&mut theirs);
                    if !theirs.iter().any(|(t, _)| !t.can_win_now(p) && forced_win(t, 1 - p, &mut buf)) {
                        return i;
                    }
                }
            }
            scored[0].1
        }
    }
}

/// How many of the network's preferred moves get the deeper safety check.
const CHECKED: usize = 6;

/// `after` is the position just after `attacker` moved. True if he threatens
/// to win and every reply still leaves him a winning move. A relic that a
/// claim has yet to turn up is ignored.
fn forced_win(after: &State, attacker: usize, buf: &mut Vec<(State, Move)>) -> bool {
    if after.winner.is_some() || !after.can_win_now(attacker) {
        return false;
    }
    after.successors(buf);
    buf.iter().all(|(r, _)| r.winner.is_none() && r.can_win_now(attacker))
}

struct Game {
    winner: Option<usize>,
    turns: usize,
    samples: Vec<(Feats, f32)>,
    moves: Vec<(Move, usize)>,
    deck_left: usize,
    passes: usize,
}

fn play(start: State, players: [&Player; 2], rng: &mut Rng, record: Option<(&Net, f32)>, log: bool) -> Game {
    let mut s = start;
    let mut succ = Vec::new();
    let mut after: Vec<Feats> = Vec::new();
    let mut moves = Vec::new();
    let mut turns = 0;
    let mut passes = 0;
    while s.winner.is_none() && turns < MAX_TURNS {
        s.successors(&mut succ);
        if no_pass() && succ.len() > 1 {
            succ.remove(0); // the pass is always listed first
        }
        let i = choose(players[s.to_move], &s, &succ, rng);
        if log {
            moves.push((succ[i].1, succ.len()));
        }
        passes += (succ[i].1 == Move::Pass) as usize;
        s = succ[i].0;
        if record.is_some() && s.winner.is_none() {
            after.push(features(&s));
        }
        s.flip();
        turns += 1;
    }
    // TD(lambda) targets over the positions each player left for the other.
    let mut samples = Vec::new();
    if let Some((net, lambda)) = record {
        let k = after.len();
        if k > 0 {
            let values: Vec<f32> = after.iter().map(|f| net.eval(f, None)).collect();
            let mut targets = vec![0f32; k];
            // the player to move in the last recorded position either won next move or the game was cut off
            targets[k - 1] = if s.winner.is_some() { 1.0 } else { 0.0 };
            for t in (0..k - 1).rev() {
                targets[t] = -((1.0 - lambda) * values[t + 1] + lambda * targets[t + 1]);
            }
            samples = after.into_iter().zip(targets).collect();
        }
    }
    Game { winner: s.winner, turns, samples, moves, deck_left: s.dlen.iter().sum(), passes }
}

fn load_player(name: &str) -> Player {
    match name {
        "greedy" => Player::Greedy,
        "random" => Player::Random,
        spec => {
            // FILE, FILE@1 or FILE@2 to set the lookahead; add d (FILE@2d) for a
            // player that claims whenever it safely can
            let (file, tag) = spec.rsplit_once('@').unwrap_or((spec, "2"));
            let dig = tag.ends_with('d');
            let depth: u8 = tag.trim_end_matches('d').parse().expect("lookahead must be 1 or 2");
            Player::Net(Arc::new(Net::load(file)), 0.0, depth, dig)
        }
    }
}

/// Returns (A wins, B wins, unfinished, mean turns) with seats alternating.
fn matchup(n: usize, games: usize, a: &Player, b: &Player, seed: u64, threads: usize) -> (f32, f32, f32, f32, f32, f32) {
    let results: Vec<(Option<usize>, usize, usize, usize, usize)> = std::thread::scope(|sc| {
        let hs: Vec<_> = (0..threads)
            .map(|t| {
                sc.spawn(move || {
                    let mut out = Vec::new();
                    let mut g = t;
                    while g < games {
                        let mut rng = Rng(seed.wrapping_mul(7919).wrapping_add(g as u64));
                        let seat = g % 2; // seat of A
                        let players = if seat == 0 { [a, b] } else { [b, a] };
                        let r = play(State::new(n, &mut rng), players, &mut rng, None, false);
                        out.push((r.winner, r.turns, seat, r.deck_left, r.passes));
                        g += threads;
                    }
                    out
                })
            })
            .collect();
        hs.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    let tot = results.len() as f32;
    let aw = results.iter().filter(|(w, _, seat, _, _)| *w == Some(*seat)).count() as f32;
    let bw = results.iter().filter(|(w, _, seat, _, _)| w.is_some() && *w != Some(*seat)).count() as f32;
    let turns: f32 = results.iter().map(|r| r.1 as f32).sum();
    let stalled: Vec<_> = results.iter().filter(|r| r.0.is_none()).collect();
    let sn = stalled.len().max(1) as f32;
    let left = stalled.iter().map(|r| r.3 as f32).sum::<f32>() / sn;
    let pass = stalled.iter().map(|r| r.4 as f32 / r.1 as f32).sum::<f32>() / sn;
    (aw / tot, bw / tot, 1.0 - (aw + bw) / tot, turns / tot, left, pass)
}

fn opt<T: std::str::FromStr>(args: &[String], key: &str, default: T) -> T {
    let prefix = format!("{key}=");
    args.iter().find_map(|a| a.strip_prefix(&prefix)).and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn train(args: &[String]) {
    let n: usize = args[0].parse().unwrap();
    let games: usize = args[1].parse().unwrap();
    let path = &args[2];
    let default_threads = std::thread::available_parallelism().map(|x| x.get()).unwrap_or(1);
    let threads: usize = opt(args, "threads", default_threads);
    let eps: f32 = opt(args, "eps", 0.1);
    let lambda: f32 = opt(args, "lambda", 0.7);
    let lr: f32 = opt(args, "lr", 0.001);
    let seed: u64 = opt(args, "seed", 1);
    let depth: u8 = opt(args, "depth", 2);
    let resume: String = opt(args, "resume", String::new());
    let mut rng = Rng(seed);
    let mut net = if resume.is_empty() { Net::new(&mut rng) } else { Net::load(&resume) };
    let mut adam = Adam::new(lr);
    let per_thread = 8;
    let round = threads * per_thread;
    let report_every = (games / 20).max(round);
    let (mut done, mut next_report) = (0usize, 0usize);
    let (mut decided, mut first, mut turns, mut loss, mut nsamp, mut count) = (0f32, 0f32, 0f32, 0f32, 0f32, 0f32);
    let started = std::time::Instant::now();
    println!("training {n} houses, {games} games, {threads} threads, eps {eps}, lambda {lambda}, lr {lr}");
    while done < games {
        let snap = Arc::new(net.clone());
        let batch: Vec<Game> = std::thread::scope(|sc| {
            let hs: Vec<_> = (0..threads)
                .map(|t| {
                    let snap = snap.clone();
                    let base = seed.wrapping_mul(1_000_003).wrapping_add((done + t * per_thread) as u64);
                    sc.spawn(move || {
                        let pl = Player::Net(snap.clone(), eps, depth, false);
                        (0..per_thread)
                            .map(|g| {
                                let mut rng = Rng(base.wrapping_add(g as u64).wrapping_mul(0x2545F4914F6CDD1D));
                                play(State::new(n, &mut rng), [&pl, &pl], &mut rng, Some((&snap, lambda)), false)
                            })
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            hs.into_iter().flat_map(|h| h.join().unwrap()).collect()
        });
        let mut samples: Vec<&(Feats, f32)> = Vec::new();
        for g in &batch {
            count += 1.0;
            turns += g.turns as f32;
            if let Some(w) = g.winner {
                decided += 1.0;
                first += (w == 0) as i32 as f32;
            }
            samples.extend(g.samples.iter());
        }
        for i in (1..samples.len()).rev() {
            samples.swap(i, rng.below(i + 1));
        }
        for chunk in samples.chunks(128) {
            let mut grad = vec![0f32; NP];
            for (f, target) in chunk {
                let v = net.eval(f, Some((*target, &mut grad)));
                loss += (v - target) * (v - target);
                nsamp += 1.0;
            }
            adam.step(&mut net, &grad, chunk.len() as f32);
        }
        done += round;
        if done >= next_report || done >= games {
            next_report += report_every;
            net.save(path);
            let me = Player::Net(Arc::new(net.clone()), 0.0, depth, false);
            let (w, l, d, _, _, _) = matchup(n, 200, &me, &Player::Greedy, 99, threads);
            println!(
                "games {done:>8} | {:>5.0}s | self-play: decided {:5.1}%, first player {:5.1}% of decided, {:5.1} turns, loss {:.4} | vs greedy: win {:5.1}% lose {:5.1}% unfinished {:5.1}%",
                started.elapsed().as_secs_f32(),
                100.0 * decided / count,
                100.0 * first / decided.max(1.0),
                turns / count,
                loss / nsamp.max(1.0),
                100.0 * w,
                100.0 * l,
                100.0 * d
            );
            (decided, first, turns, loss, nsamp, count) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        }
    }
    net.save(path);
    println!("saved {path}");
}

fn json_move(m: &Move) -> String {
    match m {
        Move::Pass => "[\"pass\"]".to_string(),
        Move::Claim { t, played, slot } => {
            if *played == NONE {
                format!("[\"claim\",{t},null,null]")
            } else {
                format!("[\"claim\",{t},{played},[{},{}]]", slot.0, slot.1)
            }
        }
        Move::Plot { line, placed } => format!(
            "[\"plot\",[[{},{}],[{},{}],[{},{}]],[{},{},{}]]",
            line[0].0, line[0].1, line[1].0, line[1].1, line[2].0, line[2].1, placed[0], placed[1], placed[2]
        ),
    }
}

fn transcript(args: &[String]) {
    let n: usize = args[0].parse().unwrap();
    let games: usize = args[1].parse().unwrap();
    let seed: u64 = args[2].parse().unwrap();
    for g in 0..games {
        let mut rng = Rng(seed.wrapping_mul(31).wrapping_add(g as u64));
        let s = State::new(n, &mut rng);
        let decks: Vec<String> = (0..4)
            .map(|t| format!("[{}]", s.decks[t][..s.dlen[t]].iter().map(|c| c.to_string()).collect::<Vec<_>>().join(",")))
            .collect();
        let realm: Vec<String> = START.iter().map(|&(r, x)| s.cells[cid(r, x)].to_string()).collect();
        let pl = if g % 2 == 0 { Player::Random } else { Player::Greedy };
        let r = play(s, [&pl, &pl], &mut rng, None, true);
        let moves: Vec<String> = r.moves.iter().map(|(m, _)| json_move(m)).collect();
        let counts: Vec<String> = r.moves.iter().map(|(_, c)| c.to_string()).collect();
        let winner = r.winner.map_or("null".to_string(), |w| w.to_string());
        println!(
            "{{\"n\":{n},\"decks\":[{}],\"realm\":[{}],\"moves\":[{}],\"counts\":[{}],\"winner\":{winner}}}",
            decks.join(","),
            realm.join(","),
            moves.join(","),
            counts.join(",")
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(|s| s.as_str()) {
        Some("train") => train(&args[1..]),
        Some("eval") => {
            let n: usize = args[1].parse().unwrap();
            let games: usize = args[2].parse().unwrap();
            let (a, b) = (load_player(&args[3]), load_player(&args[4]));
            let threads = std::thread::available_parallelism().map(|x| x.get()).unwrap_or(1);
            let (w, l, d, t, left, pass) = matchup(n, games, &a, &b, opt(&args, "seed", 7), threads);
            println!(
                "{} vs {} over {games} games ({n} houses, seats alternate): {:.1}% / {:.1}%, unfinished {:.1}%, mean {:.1} turns; unfinished games end with {:.1} relics left in the decks and {:.0}% of their moves were passes",
                args[3], args[4], 100.0 * w, 100.0 * l, 100.0 * d, t, left, 100.0 * pass
            );
        }
        Some("transcript") => transcript(&args[1..]),
        _ => eprintln!("usage: trainer train|eval|transcript ... (see src/main.rs)"),
    }
}

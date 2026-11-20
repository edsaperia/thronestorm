// Thronestorm rules, features, value network and move chooser.
// A port of trainer/src/{game,net,main}.rs; tests/web_check.js verifies it
// against the Rust trainer. Runs in a browser (window.TS) or under Node.
(function (root) {
  'use strict';
  const XM = 40, W = 2 * XM + 1, NC = 3 * W, NONE = 255, MAXH = 12;
  const F = 27, G = 20, D1 = 32, D2 = 64, POOL_SCALE = 0.25;
  const CHECKED = 6, DEEP = 30;
  const START = [[0, 0], [1, -1], [1, 1], [2, 0]];
  const cid = (r, x) => r * W + x + XM;
  const house = (c) => c >> 2;
  const ctype = (c) => c & 3;

  function newState(n, rand) {
    const s = {
      n, cells: new Uint8Array(NC).fill(NONE), lo: -1, hi: 1,
      hands: [new Uint8Array(4).fill(NONE), new Uint8Array(4).fill(NONE)],
      decks: [], dlen: [n, n, n, n], revealed: [true, true, true, true],
      toMove: 0, winner: -1,
    };
    for (let t = 0; t < 4; t++) {
      const d = [];
      for (let h = 0; h < n; h++) d.push(h * 4 + t);
      for (let i = n - 1; i > 0; i--) {
        const j = Math.floor(rand() * (i + 1));
        [d[i], d[j]] = [d[j], d[i]];
      }
      s.decks.push(d);
    }
    for (const [r, x] of START) {
      s.dlen[3] -= 1;
      s.cells[cid(r, x)] = s.decks[3][s.dlen[3]];
    }
    return s;
  }

  function clone(s) {
    return {
      n: s.n, cells: s.cells.slice(), lo: s.lo, hi: s.hi,
      hands: [s.hands[0].slice(), s.hands[1].slice()],
      decks: s.decks, dlen: s.dlen.slice(), revealed: s.revealed.slice(),
      toMove: s.toMove, winner: s.winner,
    };
  }

  function handWins(h) {
    if (h[0] === NONE) return false;
    const a = house(h[0]);
    for (let i = 1; i < 4; i++) if (h[i] === NONE || house(h[i]) !== a) return false;
    return true;
  }
  function handMask(h) {
    let m = 0;
    for (let i = 0; i < 4; i++) if (h[i] !== NONE) m |= 1 << house(h[i]);
    return m;
  }
  function faceUp(s, t) {
    return s.dlen[t] > 0 && s.revealed[t] ? s.decks[t][s.dlen[t] - 1] : NONE;
  }
  function openSlots(s) {
    const out = [];
    for (const sign of [-1, 1]) {
      for (let k = 1; ; k++) {
        const x = sign * (k + 1);
        const before = out.length;
        if (k % 2 === 1) {
          for (const r of [0, 2]) if (s.cells[cid(r, x)] === NONE) out.push([r, x]);
        } else if (s.cells[cid(1, x)] === NONE) out.push([1, x]);
        if (out.length > before) break;
      }
    }
    return out;
  }
  const evenStart = (lo) => lo - (((lo % 2) + 2) % 2);

  // Complete lines as [p's bottom cell, middle, p's top cell].
  function lines(s, p) {
    const out = [];
    const b = p === 0 ? 2 : 0, t = 2 - b;
    for (let x = evenStart(s.lo); x <= s.hi; x += 2) {
      if (s.cells[cid(b, x)] === NONE) continue;
      for (const d of [-1, 1]) {
        if (s.cells[cid(1, x + d)] !== NONE && s.cells[cid(t, x + 2 * d)] !== NONE) {
          out.push([[b, x], [1, x + d], [t, x + 2 * d]]);
        }
      }
    }
    return out;
  }

  function finish(s, p) {
    if (handWins(s.hands[p])) s.winner = p;
    s.toMove = 1 - p;
    return s;
  }

  // Every legal move with its resulting state. A claim leaves the next relic
  // of that deck face down; call flip() on the chosen state to reveal it.
  function successors(s) {
    const out = [];
    if (s.winner >= 0) return out;
    const p = s.toMove;
    out.push({ s: finish(clone(s), p), m: ['pass'] });
    let slots = null;
    for (let t = 0; t < 4; t++) {
      const c = faceUp(s, t);
      if (c === NONE) continue;
      const held = s.hands[p][t];
      if (held === NONE) {
        const n = clone(s);
        n.dlen[t] -= 1; n.revealed[t] = false; n.hands[p][t] = c;
        out.push({ s: finish(n, p), m: ['claim', t, null, null] });
      } else {
        if (!slots) slots = openSlots(s);
        for (const played of [c, held]) {
          for (const slot of slots) {
            const n = clone(s);
            n.dlen[t] -= 1; n.revealed[t] = false;
            n.hands[p][t] = played === c ? held : c;
            n.cells[cid(slot[0], slot[1])] = played;
            n.lo = Math.min(n.lo, slot[1]); n.hi = Math.max(n.hi, slot[1]);
            out.push({ s: finish(n, p), m: ['claim', t, played, slot] });
          }
        }
      }
    }
    const mask = handMask(s.hands[p]);
    if (mask === 0) return out;
    for (const line of lines(s, p)) {
      const ids = line.map(([r, x]) => cid(r, x));
      const c = ids.map((id) => s.cells[id]);
      if ((mask & (1 << house(c[0]))) === 0) continue;
      const pool = c.slice();
      for (let i = 0; i < 4; i++) if (s.hands[p][i] !== NONE) pool.push(s.hands[p][i]);
      const m = pool.length;
      for (let i = 0; i < m; i++) for (let j = 0; j < m; j++) {
        if (j === i) continue;
        for (let k = 0; k < m; k++) {
          if (k === i || k === j || (i === 0 && j === 1 && k === 2)) continue;
          const nh = [NONE, NONE, NONE, NONE];
          let ok = true;
          for (let q = 0; q < m; q++) {
            if (q === i || q === j || q === k) continue;
            const card = pool[q];
            if (nh[ctype(card)] !== NONE) { ok = false; break; }
            nh[ctype(card)] = card;
          }
          if (!ok) continue;
          const placed = [pool[i], pool[j], pool[k]];
          const n = clone(s);
          n.hands[p].set(nh);
          for (let q = 0; q < 3; q++) n.cells[ids[q]] = placed[q];
          out.push({ s: finish(n, p), m: ['plot', line, placed] });
        }
      }
    }
    return out;
  }

  function flip(s) { s.revealed = [true, true, true, true]; }

  // Could p win with one move from here, using only visible relics?
  function canWinNow(s, p) {
    const hand = s.hands[p];
    for (let t = 0; t < 4; t++) {
      const c = faceUp(s, t);
      if (c === NONE) continue;
      let all = true;
      for (let u = 0; u < 4; u++) {
        if (u !== t && (hand[u] === NONE || house(hand[u]) !== house(c))) { all = false; break; }
      }
      if (all) return true;
    }
    for (let i = 0; i < 4; i++) if (hand[i] === NONE) return false;
    const mask = handMask(hand);
    const cnt = new Uint8Array(MAXH);
    for (let i = 0; i < 4; i++) cnt[house(hand[i])]++;
    const b = p === 0 ? 2 : 0, t = 2 - b;
    for (let x = evenStart(s.lo); x <= s.hi; x += 2) {
      const c0 = s.cells[cid(b, x)];
      if (c0 === NONE || (mask & (1 << house(c0))) === 0) continue;
      for (const d of [-1, 1]) {
        const c1 = s.cells[cid(1, x + d)], c2 = s.cells[cid(t, x + 2 * d)];
        if (c1 === NONE || c2 === NONE) continue;
        let hit = false;
        for (const c of [c0, c1, c2]) { if (++cnt[house(c)] === 4) hit = true; }
        for (const c of [c0, c1, c2]) cnt[house(c)]--;
        if (hit) return true;
      }
    }
    return false;
  }

  function top2(v, live, n) {
    let a = 0, b = 0;
    for (let h = 0; h < n; h++) {
      if (!live[h]) continue;
      if (v[h] > a) { b = a; a = v[h]; } else if (v[h] > b) b = v[h];
    }
    return [a, b];
  }

  function features(s) {
    const me = s.toMove, o = 1 - me, n = s.n;
    const z = () => new Array(MAXH).fill(0);
    const hand = [z(), z()], face = z(), hidden = z(), realm = z(), keys = [z(), z()];
    let nFace = 0, nHidden = 0;
    const deckHidden = [0, 0, 0, 0];
    const isHidden = new Uint8Array(MAXH * 4);
    for (let p = 0; p < 2; p++) for (let i = 0; i < 4; i++) {
      const c = s.hands[p][i];
      if (c !== NONE) hand[p][house(c)]++;
    }
    for (let t = 0; t < 4; t++) for (let i = 0; i < s.dlen[t]; i++) {
      const c = s.decks[t][i];
      if (i + 1 === s.dlen[t] && s.revealed[t]) { face[house(c)]++; nFace++; }
      else { hidden[house(c)]++; nHidden++; deckHidden[t]++; isHidden[c] = 1; }
    }
    for (let x = s.lo; x <= s.hi; x++) for (let r = 0; r < 3; r++) {
      const c = s.cells[cid(r, x)];
      if (c === NONE) continue;
      realm[house(c)]++;
      if (r === 2) keys[0][house(c)]++;
      if (r === 0) keys[1][house(c)]++;
    }
    const lns = [[], []], usable = [[], []], reach = [z(), z()];
    const seen = [new Uint8Array(NC), new Uint8Array(NC)];
    for (let p = 0; p < 2; p++) {
      const mask = handMask(s.hands[p]);
      for (const line of lines(s, p)) {
        const ids = line.map(([r, x]) => cid(r, x));
        const ok = (mask & (1 << house(s.cells[ids[0]]))) !== 0;
        if (ok) for (const id of ids) {
          if (!seen[p][id]) { seen[p][id] = 1; reach[p][house(s.cells[id])]++; }
        }
        lns[p].push(ids); usable[p].push(ok);
      }
    }
    const pool = [hand[0].slice(), hand[1].slice()];
    const safe = [hand[0].slice(), hand[1].slice()];
    const potential = [hand[0].slice(), hand[1].slice()];
    for (let p = 0; p < 2; p++) {
      lns[p].forEach((ids, li) => {
        const ok = usable[p][li];
        const contested = ids.some((id) => seen[1 - p][id]);
        const cnt = z();
        for (const id of ids) cnt[house(s.cells[id])]++;
        for (let h = 0; h < n; h++) {
          if (cnt[h] <= 0) continue;
          const v = hand[p][h] + cnt[h];
          potential[p][h] = Math.max(potential[p][h], v);
          if (ok) {
            pool[p][h] = Math.max(pool[p][h], v);
            if (!contested) safe[p][h] = Math.max(safe[p][h], v);
          }
        }
      });
    }
    const out = [z(), z()];
    for (let p = 0; p < 2; p++) for (let t = 0; t < 4; t++) {
      const others = [];
      for (let u = 0; u < 4; u++) if (u !== t) others.push(s.hands[p][u]);
      if (others.every((c) => c !== NONE && house(c) === house(others[0]))) {
        const h = house(others[0]);
        if (isHidden[h * 4 + t]) out[p][h] = 1 / Math.max(deckHidden[t], 1);
      }
    }
    const b = (v) => (v ? 1 : 0);
    const live = [[], []];
    for (let h = 0; h < n; h++) { live[me][h] = hand[o][h] === 0; live[o][h] = hand[me][h] === 0; }
    const houses = new Float32Array(n * F);
    for (let h = 0; h < n; h++) {
      const ml = b(live[me][h]), ol = b(live[o][h]);
      houses.set([
        hand[me][h] / 4, hand[o][h] / 4, face[h] / 4, hidden[h] / 4, realm[h] / 4,
        pool[me][h] / 4, pool[o][h] / 4, reach[me][h] / 4, reach[o][h] / 4, ml, ol,
        keys[me][h] / 4, keys[o][h] / 4,
        b(hand[me][h] >= 2), b(hand[me][h] >= 3), b(hand[o][h] >= 2), b(hand[o][h] >= 3),
        b(pool[me][h] >= 3), b(pool[o][h] >= 3),
        ml * pool[me][h] / 4, ol * pool[o][h] / 4,
        ml * potential[me][h] / 4, ol * potential[o][h] / 4,
        ml * safe[me][h] / 4, ol * safe[o][h] / 4,
        out[me][h], out[o][h],
      ], h * F);
    }
    const size = (p) => { let k = 0; for (let i = 0; i < 4; i++) if (s.hands[p][i] !== NONE) k++; return k / 4; };
    const div = (p) => { let m = handMask(s.hands[p]), k = 0; while (m) { k += m & 1; m >>= 1; } return k / 4; };
    const count = (p) => usable[p].filter(Boolean).length / 20;
    const [mp1, mp2] = top2(pool[me], live[me], n), [op1, op2] = top2(pool[o], live[o], n);
    const [ms1, ms2] = top2(safe[me], live[me], n), [os1, os2] = top2(safe[o], live[o], n);
    const global = new Float32Array([
      size(me), size(o), count(me), count(o), lns[0].length / 20, nHidden / 36, nFace / 4,
      b(canWinNow(s, me)), b(canWinNow(s, o)), 1, div(me), div(o),
      mp1 / 4, mp2 / 4, op1 / 4, op2 / 4, ms1 / 4, ms2 / 4, os1 / 4, os2 / 4,
    ]);
    return { houses, global, n };
  }

  const O_W1 = 0, O_B1 = O_W1 + F * D1, O_W2 = O_B1 + D1, O_B2 = O_W2 + (D1 + G) * D2;
  const O_W3 = O_B2 + D2, O_B3 = O_W3 + D2 * 2, NP = O_B3 + 2;

  function loadNet(bytes) {
    const dv = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    const dims = [0, 4, 8, 12].map((o) => dv.getUint32(o, true));
    if (dims[0] !== F || dims[1] !== G || dims[2] !== D1 || dims[3] !== D2 + 1000) throw new Error('network shape');
    const p = new Float32Array(NP);
    for (let i = 0; i < NP; i++) p[i] = dv.getFloat32(16 + i * 4, true);
    return p;
  }

  // [chance the player to move wins, chance the other player wins]
  function evalNet(p, f) {
    const z = new Float32Array(D1 + G);
    for (let h = 0; h < f.n; h++) {
      for (let j = 0; j < D1; j++) {
        let a = p[O_B1 + j];
        for (let i = 0; i < F; i++) a += f.houses[h * F + i] * p[O_W1 + i * D1 + j];
        z[j] += Math.tanh(a) * POOL_SCALE;
      }
    }
    z.set(f.global, D1);
    const h2 = new Float32Array(D2);
    for (let j = 0; j < D2; j++) {
      let a = p[O_B2 + j];
      for (let i = 0; i < D1 + G; i++) a += z[i] * p[O_W2 + i * D2 + j];
      h2[j] = Math.tanh(a);
    }
    const v = [0, 0];
    for (let k = 0; k < 2; k++) {
      let a = p[O_B3 + k];
      for (let j = 0; j < D2; j++) a += h2[j] * p[O_W3 + j * 2 + k];
      v[k] = 1 / (1 + Math.exp(-a));
    }
    return v;
  }

  // `after` is the position just after `attacker` moved. True if he threatens
  // to win and every reply still leaves him a winning move.
  function forcedWin(after, attacker) {
    if (after.winner >= 0 || !canWinNow(after, attacker)) return false;
    return successors(after).every((r) => r.s.winner < 0 && canWinNow(r.s, attacker));
  }
  function hasForcingWin(s, me) {
    if (canWinNow(s, me)) return true;
    return successors(s).some((t) => forcedWin(t.s, me));
  }
  function forcedWin3(after, attacker) {
    if (after.winner >= 0 || canWinNow(after, 1 - attacker)) return false;
    return successors(after).every((r) => r.s.winner < 0 && hasForcingWin(r.s, attacker));
  }

  // Index of the move to play. opts: { net, depth (1-4), rivalry }
  function choose(s, succ, opts) {
    const p = s.toMove, { net, depth, rivalry } = opts;
    const win = succ.findIndex((t) => t.s.winner === p);
    if (win >= 0) return win;
    let cands = [];
    for (let i = 0; i < succ.length; i++) if (!canWinNow(succ[i].s, 1 - p)) cands.push(i);
    if (!cands.length) cands = succ.map((_, i) => i);
    if (depth >= 2) {
      const f = cands.find((i) => forcedWin(succ[i].s, p));
      if (f !== undefined) return f;
    }
    const score = (st) => { const v = evalNet(net, features(st)); return rivalry * v[0] - v[1]; };
    const scored = cands.map((i) => [score(succ[i].s), i]).sort((a, b) => a[0] - b[0]);
    if (depth >= 3) {
      const f = scored.slice(0, DEEP).find(([, i]) => forcedWin3(succ[i].s, p));
      if (f) return f[1];
    }
    if (depth >= 2) {
      for (const [, i] of scored.slice(0, CHECKED)) {
        const theirs = successors(succ[i].s);
        let bad = theirs.some((t) => !canWinNow(t.s, p) && forcedWin(t.s, 1 - p));
        if (!bad && depth >= 4) {
          const rated = [];
          theirs.forEach((t, j) => { if (t.s.winner < 0) rated.push([score(t.s), j]); });
          rated.sort((a, b) => a[0] - b[0]);
          bad = rated.slice(0, DEEP).some(([, j]) => forcedWin3(theirs[j].s, 1 - p));
        }
        if (!bad) return i;
      }
    }
    return scored[0][1];
  }

  const api = {
    NONE, START, cid, house, ctype, newState, clone, faceUp, openSlots, lines,
    successors, flip, canWinNow, handWins, handMask, features, loadNet, evalNet, choose,
  };
  if (typeof module !== 'undefined' && module.exports) module.exports = api;
  else root.TS = api;
})(typeof window !== 'undefined' ? window : globalThis);

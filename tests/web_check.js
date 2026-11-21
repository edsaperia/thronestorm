// Replays Rust transcripts through web/engine.js.
//   trainer transcript 10 200 1 | node tests/web_check.js
//   trainer transcript 10 20 1 nets/h10-win.bin@4 rivalry=0 | node tests/web_check.js nets/h10-win.bin 4 0
// With a network, also checks that the JavaScript player picks the same move.
const fs = require('fs');
const TS = require('../web/engine.js');
const [netFile, depth, rivalry] = process.argv.slice(2);
const net = netFile ? TS.loadNet(new Uint8Array(fs.readFileSync(netFile))) : null;
const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
let games = 0, turns = 0, agree = 0;
for (const line of fs.readFileSync(0, 'utf8').split('\n').filter(Boolean)) {
  const g = JSON.parse(line);
  const s0 = TS.newState(g.n, Math.random);
  s0.decks = g.decks.map((d) => d.slice());
  s0.dlen = g.decks.map((d) => d.length);
  TS.START.forEach(([r, x], i) => { s0.cells[TS.cid(r, x)] = g.realm[i]; });
  let s = s0;
  g.moves.forEach((mv, i) => {
    const succ = TS.successors(s);
    if (succ.length !== g.counts[i]) throw new Error(`game ${games} turn ${i}: ${succ.length} moves, Rust has ${g.counts[i]}`);
    const idx = succ.findIndex((t) => same(t.m, mv));
    if (idx < 0) throw new Error(`game ${games} turn ${i}: move not legal here: ${JSON.stringify(mv)}`);
    if (net && TS.choose(s, succ, { net, depth: +depth, rivalry: +rivalry }) === idx) agree++;
    s = succ[idx].s;
    TS.flip(s);
    turns++;
  });
  const w = s.winner < 0 ? null : s.winner;
  if (w !== g.winner) throw new Error(`game ${games}: winner ${w}, Rust has ${g.winner}`);
  games++;
}
console.log(`ok: ${games} games, ${turns} turns agree` + (net ? `; same move chosen on ${agree} of ${turns}` : ''));

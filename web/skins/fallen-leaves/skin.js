// Fallen Leaves: ten kinds of leaf, each in four colours. The same game as
// Thronestorm; a leaf's kind plays the part of a house and its colour the
// part of sword, shield, helmet or trinket. You pick leaves from the piles
// and make matches along diagonals in the middle.
window.SKIN = (function () {
  const KIND = ['green', 'yellow', 'orange', 'red'];
  const HOUSE = ['oak', 'chestnut', 'ginkgo', 'elm', 'willow', 'maple', 'aspen', 'hawthorn', 'ivy', 'ash'];
  const house = (c) => c >> 2, kind = (c) => c & 3;
  const name = (c) => `${KIND[kind(c)]} ${HOUSE[house(c)]} leaf`;
  const chip = (c) => `<span class="chip k${kind(c)}">${KIND[kind(c)]} ${HOUSE[house(c)]}</span>`;
  const chips = (cs) => cs.map(chip).join(' ');
  return {
    kinds: KIND, houses: HOUSE, name, chip,
    ratio: 1, maxWidth: 96, gap: 4, handGap: 8, rowPitch: 0.866, handPitch: 1.22, // circles pack like a honeycomb
    ripple: true,
    badge: () => '',
    slot: (t) => `<span class="dot k${t}"></span>`,
    // the leaves beneath a pile's top one show as rims running out to the left
    pileStyle: (n) => '--edges:' + (Array.from({ length: n }, (_, i) => `${-2 * (i + 1)}px 0 0 ${i % 2 ? 'rgba(10,18,22,.6)' : 'rgba(220,235,240,.45)'}`).join(',') || 'none'),
    words: {
      thinking: 'The opponent is thinking.',
      dragToPick: 'Drag it to your row to pick it.',
      yourTurn: 'Your turn. Pick a leaf from the piles by dragging it to your row, or tap a glowing leaf in the middle to match.',
      two: (t) => `You now hold two ${KIND[t]} leaves. Drag one of them to a glowing place in the middle.`,
      twoDiagonals: 'Two diagonals rise from that leaf. Tap another leaf on the one you want.',
      putBack: (n) => `Put ${['', 'one more leaf', 'two more leaves', 'three leaves'][n]} back on the diagonal, keeping one of each colour. Tap a leaf twice to send it to the next place.`,
      wouldHoldTwo: (t) => `You would hold two ${KIND[t]} leaves. Swap one of them onto the diagonal.`,
      backAsWas: 'You put the diagonal back as it was.',
      notAllowed: 'That move is not allowed.',
      noKindYet: (t) => `You hold no ${KIND[t]} leaf yet, so that one goes to your row.`,
      peekNeither: 'Neither diagonal from that leaf is complete yet.',
      peekIncomplete: 'That diagonal is not complete yet.',
      peekNoStake: (h) => `You hold no ${HOUSE[h]} leaf, so you cannot match from that one.`,
      won: (me, h) => `${me ? 'You hold' : 'The opponent holds'} the ${HOUSE[h]} in all four colours.`,
      wholeGame: ' That is the whole game.',
      noKind: (t) => `no ${KIND[t]} leaf`,
      emptyPlace: 'empty place in the middle',
      faceDown: 'face-down leaf',
      pile: (t, n) => `${KIND[t]} pile, ${n} beneath`,
      log: {
        pass: (Who) => `${Who} passed.`,
        claim: (Who, c) => `${Who} picked the ${chip(c)}.`,
        sent: (Who, c) => `${Who} put the ${chip(c)} straight into the middle.`,
        claimSent: (Who, c, p) => `${Who} picked the ${chip(c)} and put the ${chip(p)} in the middle.`,
        rearranged: (Who) => `${Who} rearranged a diagonal.`,
        plotted: (Who, took, left) => `${Who} made a match: took ${chips(took)}, left ${chips(left)}.`,
      },
      hint: {
        pass: 'Hint: pass.',
        claim: (c) => `Hint: pick the ${chip(c)}.`,
        send: (c, where) => `Hint: put the ${chip(c)} from the piles straight onto ${where}.`,
        claimSend: (c, p, where) => `Hint: pick the ${chip(c)} and put your ${chip(p)} on ${where}.`,
        plot: (trig, a, b, c) => `Hint: match from the ${chip(trig)}. Put back the ${chip(a)} nearest you, the ${chip(b)} in the centre and the ${chip(c)} farthest.`,
        rows: ['the far row', 'the centre row', 'the near row'],
        wins: ' It wins the game.',
        stops: ' It stops the opponent winning next turn.',
        noAnswer: ' It leaves the opponent no answer.',
      },
      tutorial: [
        { idle: `You win by holding one kind of leaf in all four colours. You already hold the ${chip(25)}. Pick the ${chip(26)}: drag it from the piles to your row.`,
          nope: `Pick the ${chip(26)} first.` },
        { idle: `You can keep only one leaf of each colour. Pick the ${chip(24)} anyway.`,
          claim: `Now you hold two green leaves. Drag the ${chip(28)} to the glowing place on the far row, on the right.`,
          nope: `Pick the ${chip(24)}, then put the ${chip(28)} at the right-hand end of the far row.` },
        { idle: `That filled a diagonal in the middle: one leaf on the near row, one in the centre, one on the far row. The ${chip(27)} glows because you hold other aspen leaves. Tap it to match.`,
          plot: 'The diagonal is in your hand. Put three leaves back, keeping the four aspen leaves.',
          nope: 'Keep all four aspen leaves, and put the other three back.' },
      ],
    },
  };
})();

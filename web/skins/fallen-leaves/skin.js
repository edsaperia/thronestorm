// Fallen Leaves: ten kinds of leaf, each in four colours. The same game as
// Thronestorm; a leaf's kind plays the part of a house and its colour the
// part of sword, shield, helmet or trinket.
window.SKIN = (function () {
  const KIND = ['green', 'yellow', 'orange', 'red'];
  const HOUSE = ['oak', 'chestnut', 'ginkgo', 'elm', 'willow', 'maple', 'aspen', 'hawthorn', 'ivy', 'ash'];
  const house = (c) => c >> 2, kind = (c) => c & 3;
  const name = (c) => `${KIND[kind(c)]} ${HOUSE[house(c)]} leaf`;
  const chip = (c) => `<span class="chip k${kind(c)}">${KIND[kind(c)]} ${HOUSE[house(c)]}</span>`;
  const chips = (cs) => cs.map(chip).join(' ');
  return {
    kinds: KIND, houses: HOUSE, name, chip,
    ratio: 1, maxWidth: 76, gap: 4, handGap: 12, rowPitch: 0.866, // circles pack like a honeycomb
    badge: () => '',
    slot: (t) => `<span class="dot k${t}"></span>`,
    // the leaves beneath a pile's top one show as rims running out to the left
    pileStyle: (n) => '--edges:' + (Array.from({ length: n }, (_, i) => `${-2 * (i + 1)}px 0 0 ${i % 2 ? 'rgba(20,70,78,.55)' : 'rgba(255,255,255,.5)'}`).join(',') || 'none'),
    words: {
      thinking: 'The opponent is thinking.',
      yourTurn: 'Your turn. Drag a leaf from the piles to your row, or tap a glowing leaf on the ground to sweep.',
      two: (t) => `You now hold two ${KIND[t]} leaves. Drag one of them to a glowing place on the ground.`,
      twoDiagonals: 'Two diagonals rise from that leaf. Tap another leaf on the one you want.',
      putBack: (n) => `Put ${['', 'one more leaf', 'two more leaves', 'three leaves'][n]} back on the diagonal, keeping one of each colour. Tap a leaf twice to send it to the next place.`,
      wouldHoldTwo: (t) => `You would hold two ${KIND[t]} leaves. Swap one of them onto the diagonal.`,
      backAsWas: 'You put the diagonal back as it was.',
      notAllowed: 'That move is not allowed.',
      noKindYet: (t) => `You hold no ${KIND[t]} leaf yet, so that one goes to your row.`,
      peekNeither: 'Neither diagonal from that leaf is complete yet.',
      peekIncomplete: 'That diagonal is not complete yet.',
      peekNoStake: (h) => `You hold no ${HOUSE[h]} leaf, so you cannot sweep from that one.`,
      won: (me, h) => `${me ? 'You have' : 'The opponent has'} gathered the ${HOUSE[h]} in all four colours.`,
      wholeGame: ' That is the whole game.',
      noKind: (t) => `no ${KIND[t]} leaf`,
      emptyPlace: 'empty place on the ground',
      faceDown: 'face-down leaf',
      pile: (t, n) => `${KIND[t]} pile, ${n} beneath`,
      log: {
        pass: (Who) => `${Who} passed.`,
        claim: (Who, c) => `${Who} gathered the ${chip(c)}.`,
        sent: (Who, c) => `${Who} dropped the ${chip(c)} straight on the ground.`,
        claimSent: (Who, c, p) => `${Who} gathered the ${chip(c)} and dropped the ${chip(p)} on the ground.`,
        rearranged: (Who) => `${Who} rearranged a diagonal.`,
        plotted: (Who, took, left) => `${Who} swept a diagonal: took ${chips(took)}, left ${chips(left)}.`,
      },
      hint: {
        pass: 'Hint: pass.',
        claim: (c) => `Hint: gather the ${chip(c)}.`,
        send: (c, where) => `Hint: drop the ${chip(c)} from the piles straight onto ${where}.`,
        claimSend: (c, p, where) => `Hint: gather the ${chip(c)} and drop your ${chip(p)} on ${where}.`,
        plot: (trig, a, b, c) => `Hint: sweep the diagonal from the ${chip(trig)}. Put back the ${chip(a)} nearest you, the ${chip(b)} in the middle and the ${chip(c)} farthest.`,
        rows: ['the far row', 'the middle row', 'your row of the ground'],
        wins: ' It wins the game.',
        stops: ' It stops the opponent winning next turn.',
        noAnswer: ' It leaves the opponent no answer.',
      },
      tutorial: [
        { idle: `You win by holding one kind of leaf in all four colours. You already hold the ${chip(25)}. Drag the ${chip(26)} from the piles to your row.`,
          nope: `Take the ${chip(26)} first.` },
        { idle: `You can keep only one leaf of each colour. Take the ${chip(24)} anyway.`,
          claim: `Now you hold two green leaves. Drag the ${chip(28)} to the glowing place on the far row, on the right.`,
          nope: `Take the ${chip(24)}, then drop the ${chip(28)} at the right-hand end of the far row.` },
        { idle: `That filled a diagonal: one leaf on your row of the ground, one in the middle, one on the far row. The ${chip(27)} glows because you hold other aspen leaves. Tap it to sweep up the diagonal.`,
          plot: 'The diagonal is in your hand. Put three leaves back, keeping the four aspen leaves.',
          nope: 'Keep all four aspen leaves, and put the other three back.' },
      ],
    },
  };
})();

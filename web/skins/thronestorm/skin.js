// Thronestorm: ten noble houses, each with a sword, shield, helmet and trinket.
window.SKIN = (function () {
  const KIND = ['sword', 'shield', 'helmet', 'trinket'];
  const HOUSE = ['Cleric', 'Necromancer', 'Warrior', 'Paladin', 'Monk', 'Assassin', 'Wizard', 'Bard', 'Swashbuckler', 'Barbarian'];
  const house = (c) => c >> 2, kind = (c) => c & 3;
  const glyph = (t) => `<svg aria-hidden="true"><use href="#g${t}"/></svg>`;
  const name = (c) => `${HOUSE[house(c)]}'s ${KIND[kind(c)]}`;
  const chip = (c) => `<span class="chip h${house(c)}">${name(c)}</span>`;
  const chips = (cs) => cs.map(chip).join(' ');
  return {
    kinds: KIND, houses: HOUSE, name, chip,
    ratio: 14 / 9, maxWidth: 72, gap: 5, handGap: 5, rowPitch: 1, handPitch: 1.5,
    badge: (c) => `<span class="kind">${glyph(kind(c))}</span>`,
    slot: glyph,
    pileStyle: () => '',
    words: {
      thinking: 'The opponent is thinking.',
      dragToPick: 'Drag it to your row to claim it.',
      yourTurn: 'Your turn. Drag a relic from the reliquary to your row, or tap a glowing relic in the realm to plot.',
      two: (t) => `You now hold two ${KIND[t]}s. Drag one of them to a glowing place in the realm.`,
      twoDiagonals: 'Two diagonals rise from that relic. Tap another relic on the one you want.',
      putBack: (n) => `Put ${['', 'one more relic', 'two more relics', 'three relics'][n]} back on the diagonal, keeping one of each kind. Tap a relic twice to send it to the next place.`,
      wouldHoldTwo: (t) => `You would hold two ${KIND[t]}s. Swap one of them onto the diagonal.`,
      backAsWas: 'You put the diagonal back as it was.',
      notAllowed: 'That move is not allowed.',
      noKindYet: (t) => `You hold no ${KIND[t]} yet, so that one goes to your row.`,
      peekNeither: 'Neither diagonal from that relic is complete yet.',
      peekIncomplete: 'That diagonal is not complete yet.',
      peekNoStake: (h) => `You hold nothing from the ${HOUSE[h]}'s house, so you cannot plot from that relic.`,
      won: (me, h) => `${me ? 'You have' : 'The opponent has'} united the house of the ${HOUSE[h]}.`,
      wholeGame: ' That is the whole game.',
      noKind: (t) => `no ${KIND[t]}`,
      emptyPlace: 'empty place in the realm',
      faceDown: 'face-down relic',
      pile: (t, n) => `${KIND[t]} pile, ${n} beneath`,
      log: {
        pass: (Who) => `${Who} passed.`,
        claim: (Who, c) => `${Who} claimed the ${chip(c)}.`,
        sent: (Who, c) => `${Who} sent the ${chip(c)} straight to the realm.`,
        claimSent: (Who, c, p) => `${Who} claimed the ${chip(c)} and sent the ${chip(p)} to the realm.`,
        rearranged: (Who) => `${Who} rearranged a diagonal.`,
        plotted: (Who, took, left) => `${Who} plotted: took ${chips(took)}, left ${chips(left)}.`,
      },
      hint: {
        pass: 'Hint: pass.',
        claim: (c) => `Hint: claim the ${chip(c)}.`,
        send: (c, where) => `Hint: send the ${chip(c)} from the reliquary straight to ${where}.`,
        claimSend: (c, p, where) => `Hint: claim the ${chip(c)} and send your ${chip(p)} to ${where}.`,
        plot: (trig, a, b, c) => `Hint: pick up the diagonal from the ${chip(trig)}. Put back the ${chip(a)} nearest you, the ${chip(b)} in the middle and the ${chip(c)} farthest.`,
        rows: ['the far row', 'the middle row', 'your row of the realm'],
        wins: ' It wins the game.',
        stops: ' It stops the opponent winning next turn.',
        noAnswer: ' It leaves the opponent no answer.',
      },
      tutorial: [
        { idle: `You win by holding all four relics of one house. You already hold the ${chip(25)}. Drag the ${chip(26)} from the reliquary to your row.`,
          nope: `Take the ${chip(26)} first.` },
        { idle: `You can keep only one of each kind. Take the ${chip(24)} anyway.`,
          claim: `Now you hold two swords. Drag the ${chip(28)} to the glowing place on the far row, on the right.`,
          nope: `Take the ${chip(24)}, then send the ${chip(28)} to the right-hand end of the far row.` },
        { idle: `That filled a diagonal: one relic on your row of the realm, one in the middle, one on the far row. The ${chip(27)} glows because you hold other Wizard relics. Tap it to pick up the diagonal.`,
          plot: 'The diagonal is in your hand. Put three relics back, keeping the four Wizard relics.',
          nope: 'Keep all four Wizard relics, and put the other three back.' },
      ],
    },
  };
})();

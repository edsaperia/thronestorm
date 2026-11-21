# Thronestorm rules, as implemented

**Relics.** n houses x 4 types (sword, shield, helmet, trinket). Standard game: n = 10.

**Goal.** Hold all four relics of one house in your inventory at the end of your own turn.

**Setup.** One shuffled face-down deck per type (the reliquary). Four random
trinkets start face up in the realm as a diamond: one on the top row, two on
the middle row, one on the bottom row. The top relic of each deck is turned
face up. Inventories start empty and hold at most one relic of each type.
Inventories are open information.

**Turn.** Claim a relic, hatch a plot, or pass.

**Claim.** Take a face-up relic from the reliquary. If you now hold two of that
type, play either one into the realm. Then turn up the next relic of that deck.
Claiming is legal whenever a face-up relic exists.

**The realm.** Three rows, offset like brickwork. Top and bottom rows use
columns 1, 3, 5...; the middle row uses columns 2, 4, 6... on each side of the
centre. Odd columns have two slots, even columns one. On each side, a column
must be full before the next is started.

**Plot.** Your bottom row is the row nearest you (your opponent's is the far
row). If a relic on your bottom row shares a house with any relic you hold, you
may plot with either diagonal rising from it (bottom, middle, top), provided
all three slots are occupied. Pick up the three relics and put three back, in
any order, drawn from those and your inventory. The rest go to your inventory,
which must end with at most one relic of each type. Only the starting position
is checked for the house match.

**Passing** is allowed. There is no draw rule yet.

**Variants**
- `partial_diagonals`: a plot may use a diagonal with empty slots; you pick up
  and replace only the relics present.
- Beginner: 8 houses. Advanced peek: not implemented.

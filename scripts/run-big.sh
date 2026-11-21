#!/bin/sh
# Overnight self-play run for a larger Thronestorm network (ten houses, win-seeker),
# trained from scratch. Fifteen segments of 100,000 games with a falling learning
# rate; on this laptop each takes about half an hour, so about eight hours in all.
# Each segment resumes from the last and h10-big.bin is saved about every 5,000 games.
# Progress goes to training-big.log. Safe to stop and restart: it picks up where it was.
# Every finished segment is kept as h10-big-after-segment-N.bin, so a stopped run is still useful.
cd "$(dirname "$0")" || exit 1
chmod +x ./trainer-linux-big 2>/dev/null
games=${GAMES:-100000}
segments=${SEGMENTS:-15}
i=$(cat big-segment.txt 2>/dev/null || echo 0)
while [ "$i" -lt "$segments" ]; do
  if [ "$i" -lt 5 ]; then lr=0.001; elif [ "$i" -lt 9 ]; then lr=0.0005; elif [ "$i" -lt 12 ]; then lr=0.0003; else lr=0.0002; fi
  from=""; [ -f h10-big.bin ] && from="resume=h10-big.bin"
  echo "=== segment $((i+1)) of $segments, lr $lr, started $(date -u +%FT%TZ)" >> training-big.log
  ./trainer-linux-big train 10 "$games" h10-big.bin $from rivalry=0 gamma=0.98 depth=3 lr=$lr seed=$((200+i)) >> training-big.log 2>&1 || { echo "trainer failed, see training-big.log"; exit 1; }
  cp h10-big.bin h10-big-after-segment-$((i+1)).bin
  i=$((i+1)); echo $i > big-segment.txt
done
echo "=== finished $(date -u +%FT%TZ)" >> training-big.log
echo "finished"

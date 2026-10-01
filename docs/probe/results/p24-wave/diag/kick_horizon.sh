#!/bin/bash
# run (2026-10-01), after the result, not part of any registered wave and not scored: CT2's two
# Crowded kick sets again on the wave's binary with the horizon doubled and halved, to tell a
# decaying mode (the tail falls with H) from a floor (it does not).
B=/root/scratch/p24-bcd/markets
O=/root/scratch/p24-diag
mkdir -p $O
sha256sum $B
run() { d=$O/$1; shift; mkdir -p $d; ( cd $d; "$B" "$@" > out.txt 2> err.txt; echo "rc $?" > exit ) & }
for t in b.food=1.2@dated exit.To=17.55@dated; do
  for h in 70500 282000; do
    run "$(echo $t | tr '=@' '__')-H$h" kick $t --inst ct2 --ticks 141000 --horizon $h --csv $O/$(echo $t | tr '=@' '__')-H$h
  done
done
wait
for d in $O/*; do echo "== $d"; cat $d/exit; head -2 $d/out.txt | cut -c1-200; cat $d/*.tsv; done

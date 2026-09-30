#!/usr/bin/env bash
# D2.4 (label run, 2026-09-30): write docs/demo/v2/results/ and figs/ from the scored wave.
# usage: tables.sh GATHERED SCORED   (gather.py's DEST and score.py's OUT)
set -euo pipefail
G=$1
S=$2
V=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
R=$V/results
T=$R/tools
W=$(cd "$V/../../../worlds/demo-gb" && pwd)
for f in lines.csv county-dates.csv long-run.csv aggregates.csv; do
    cp "$S/$f" "$R/$f"
done
python3 "$T/engine_runs.py" "$V" "$G" >"$R/battery-runs.csv"
awk -F'\t' 'BEGIN{OFS=","} NR==1{print "county,year,kick,g_tick,g_year,three_t6"; next} {print $1,$2,$3,$4,$5,$6}' \
    "$G/kicks.tsv" >"$R/kicks.csv"
awk -F'\t' 'BEGIN{OFS=","} NR==1{print "county,year,mode_a,largest_gap_in_log"; next} {printf "%s,%s,%s,%.3e\n", $1,$2,$3,$5*1e-3}' \
    "$G/modea.tsv" >"$R/modea.csv"
python3 "$T/report.py" "$W" "$R" >"$R/by-class.csv"
python3 "$T/lag.py" "$W" "$G/series.csv" /mnt/d/rustyecon-d2/design/out-mirror/long-c2g-final.json >"$R/capital-lag.csv"
cp "$G/long-run.csv" "$R/long-run-engine.csv"
python3 "$T/plots.py" "$R" "$G" /mnt/d/rustyecon-d2/design/out-mirror/long-c2g-final.json "$V" "$V/figs"
ls -la "$R" "$V/figs"

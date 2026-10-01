"""P2.4 (label run, 2026-09-30): reading 11 of the README, reported. The trap's E9 controls on C1
(Tier 3 at the 17 dial settings, on the B-D binary) are the same runs as wave A's C1 Tier 3 at the
same settings (on the P2.4.2 binary), with the same length and CSV rows. Built after P2.4.2, the
B-D binary must give the registered C1 bit for bit (R1): this compares each pair's summary.tsv,
stats.tsv and CSV byte for byte (a gzipped copy read decompressed).
Usage (WSL): python3 r1check.py WAVE_A_ROOT BCD_ROOT (the runs' directories or their archives).
Run on 2026-10-01 from both archives, D:/rustyecon-p24/runs/families and runs/bcd (r1check.out)."""
import gzip
import os
import sys

TR = {"*": "x", "/": "d", "(": "_", ")": "_", ",": "_", "=": "_", "@": "_", "+": "_", "[": "_",
      "]": "_"}
SETTINGS = ([f"rate.*={f}" for f in ("0.75", "0.9", "1.1", "1.25")]
            + [f"buffer.*={f}" for f in ("0.75", "0.9", "1.1", "1.25")]
            + [f"adjust.*={f}" for f in ("0.75", "0.9", "1.1", "1.25")]
            + [f"tilt.*={v}" for v in ("0.05", "0.1", "0.25", "0.5", "1")])


def fname(run):
    return "".join(TR.get(c, c) for c in run)


def read(path):
    if os.path.exists(path):
        return open(path, "rb").read()
    if os.path.exists(path + ".gz"):
        return gzip.open(path + ".gz").read()
    return None


def main(a_root, b_root):
    n, same, diff = 0, 0, []
    for s in SETTINGS:
        bdir = os.path.join(b_root, "trap", "ctl", "c1", fname(s))
        for run_dir in sorted(os.listdir(bdir)):
            n += 1
            adir = os.path.join(a_root, "dial", "c1", fname(s), run_dir)
            bad = []
            for f in ("summary.tsv", "stats.tsv", run_dir + ".csv"):
                x, y = read(os.path.join(adir, f)), read(os.path.join(bdir, run_dir, f))
                if x is None or y is None or x != y:
                    bad.append(f)
            if bad:
                diff.append((s, run_dir, bad))
            else:
                same += 1
    print(f"C1 Tier 3 at the 17 settings: {n} trap-control runs; {same} byte for byte wave A's "
          f"(summary.tsv, stats.tsv and the CSV); {len(diff)} differ")
    for d in diff[:20]:
        print("  ", d)
    return not diff and n > 0


if __name__ == "__main__":
    sys.exit(0 if main(*sys.argv[1:3]) else 1)

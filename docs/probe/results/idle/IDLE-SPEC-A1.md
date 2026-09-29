# IDLE-SPEC-A1: an amendment to IDLE-SPEC (O47), after the engine review

Dated 2026-09-29. Label `fix-report`, step L0.7 on branch `phase2-loops`. It amends
`D:/rustyecon-p2l/idle-scan/IDLE-SPEC.md` (sha256
`c5c6527a7226ed462b0fe321d92e86339ed286e02ef880fc0f11da2fd373634e`, registered at L0.3), which
stays as registered. This file has its own sha256 in `SHA256SUMS` beside it. It changes no
registered prediction (§8) and no engine result: the L0.6 runs stand, and the review's sample of
17 of them, rerun from L0.7's build, equals them in every CSV row, summary line and statistic.

## A1.1 Off is structural (§6 "Switched off", §7 step 1)

§6 says `markup < 0.0` is false for every markup the engine can form, since p_K > 0 and c > 0.
That is not enough. The markup is p_K·(1 − δ·a/κ)/c, and 1 − δ·a/κ is negative wherever
δ·a/κ > 1, which no load check rules out and a dated `SetParam` can reach. There a maker with ψ 0,
or with `reserve` absent, withheld its whole finished stock where P2.2a's maker offered it (H1
with 1,000 finished heads and a = 1,000: 0 heads against 999.96).

Read §6 and §7 step 1 as: the maker withholds only when ψ > 0 and μ < ψ. With ψ absent or 0 the
offer is P2.2a's at any markup, a negative one included. The engine does this from L0.7
(`withholds(psi, markup)`), with a test that fails without it. Steps 2–5 of §7 need 0 < ψ < 1
and are unchanged.

## A1.2 The bound on the idle price (§0, §9's 240, O53)

"It falls at most one step below ψ times replacement cost, whatever the bids" holds for the
replacement cost at the maker's last offer. The price never falls on a withheld tick, but the
wage and fodder keep moving while the maker withholds, and its markup at current costs moves
with them. Read §0 and §9's 240 as: the price falls at most one step below ψ times the
replacement cost at the maker's last offer; the markup at current costs can go lower. On the
engine at heads × 10 it reached 0.144–0.206 (L0.6's `heads_x10.csv`). The lowest horse price
there was 0.135–0.193 of target, as registered.

## A1.3 "Every kick set" (§0, §7 step 4)

§0's "mode A and every kick set" are P2.2a's is a claim about the mirror's rest point and its
base kick set. On the engine the base kick sets and the hold slowest modes are P2.2a's at all 14
instances. At the dated cost target b × 2@dated the rule acts on the way to the new target (46
withheld ticks at H2), so the kicks start from another end state: at the 10 instances with that
target the kick and slowest-mode files differ from P2.2a's (the slowest mode a year moves by at
most 0.0066, at F4), and every kick set passes. §7 step 4's local argument is unchanged: it is
about a neighbourhood of the rest point.

## A1.4 The harness (§6 "The harness")

§6 has the harness compute the markup "from posted prices and the maker's" coefficients. L0.4
did that with rule A's running recipe written into the harness's code. From L0.7 the harness
reads the tape's own recipes through the rule's own code (`maker_reservation`), so the readout
is the rule's on any instance. On every instance run so far the two read the same.

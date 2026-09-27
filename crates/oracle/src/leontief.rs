//! Gaussian elimination without pivoting, for unit 1c's Leontief systems
//! (docs/unit-1c.md §5.1).
//!
//! **Why this algorithm.** Every matrix unit 1c factors is a Z-matrix, I − A with A ≥ 0:
//! the categories' I − A_cc, the machine block's I − Â and I − A^q, their transposes, and
//! the two-recipe system of §5.1 step 2. Such a matrix is a nonsingular M-matrix, which is
//! the paper's "spectral radius of A below one" (SSRN p.8 and A.1), exactly when every
//! pivot of Gaussian elimination without pivoting, in index order, is positive. So:
//!
//! - the elimination is its own test of productivity and viability: the least pivot is
//!   unit 1a's D generalised, and D > 0 is the viability condition;
//! - on an M-matrix it is stable without pivoting: the multipliers are nonpositive, the
//!   off-diagonal entries stay nonpositive, and back substitution adds nonnegative terms,
//!   so the only cancellation is on the diagonal, where a pivot near 0 is the viability
//!   edge itself (1a's D → 0), not an artefact of the method;
//! - it is deterministic: no pivot is chosen from the data, every operation has a fixed
//!   place in a fixed order, and so the one-type case repeats unit 1a's and 1b's
//!   floating-point operations bit for bit (§2.6).
//!
//! Partial pivoting would reorder rows by the data and break that nesting, and on an
//! M-matrix it buys nothing. A Neumann series (I + A + A² + …) converges slowly as the
//! spectral radius nears 1 and is not exact even for A = 0 after truncation. The
//! Grassmann-Taksar-Heyman variant, which forms each pivot from the off-diagonal sums
//! without subtraction, needs the row sums of a stochastic matrix, which a Leontief matrix
//! does not have.

/// The factors of an n×n matrix G by Gaussian elimination without pivoting in index order:
/// for k, for i > k, m = G_ik/G_kk and G_ij ← G_ij − m·G_kj for j > k.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Factors {
    n: usize,
    /// Row-major n×n: the multipliers m_ik below the diagonal, U on and above it.
    lu: Vec<f64>,
}

impl Factors {
    /// Eliminates G, given row-major. Nothing is skipped: a zero multiplier is still
    /// applied, so the operations are the same whatever the entries.
    pub(crate) fn new(n: usize, mut g: Vec<f64>) -> Factors {
        assert_eq!(g.len(), n * n, "a {n}x{n} matrix");
        for k in 0..n {
            let pivot = g[k * n + k];
            for i in k + 1..n {
                let m = g[i * n + k] / pivot;
                g[i * n + k] = m;
                for j in k + 1..n {
                    g[i * n + j] -= m * g[k * n + j];
                }
            }
        }
        Factors { n, lu: g }
    }

    /// U_kk, the k-th pivot after elimination.
    pub(crate) fn pivot(&self, k: usize) -> f64 {
        self.lu[k * self.n + k]
    }

    /// Every pivot, in index order.
    pub(crate) fn pivots(&self) -> Vec<f64> {
        (0..self.n).map(|k| self.pivot(k)).collect()
    }

    /// The least pivot: the first that is not positive (NaN included), where the
    /// elimination stops meaning anything, or else the smallest. For unit 1a's machine row
    /// the pivots are 1.0 and D, and this is D.
    pub(crate) fn least_pivot(&self) -> f64 {
        let mut least = f64::INFINITY;
        for k in 0..self.n {
            let pivot = self.pivot(k);
            if pivot.is_nan() || pivot <= 0.0 {
                return pivot;
            }
            if pivot < least {
                least = pivot;
            }
        }
        least
    }

    /// Whether every pivot is positive: the matrix is a nonsingular M-matrix when it is a
    /// Z-matrix.
    pub(crate) fn is_m_matrix(&self) -> bool {
        (0..self.n).all(|k| self.pivot(k) > 0.0)
    }

    /// Forward substitution in the elimination's order: for k, for i > k,
    /// r_i ← r_i − m_ik·r_k.
    pub(crate) fn forward(&self, r: &mut [f64]) {
        let n = self.n;
        assert_eq!(r.len(), n);
        for k in 0..n {
            for i in k + 1..n {
                r[i] -= self.lu[i * n + k] * r[k];
            }
        }
    }

    /// Back substitution with the division deferred: from the last row up, the numerator
    /// n_i = r_i − Σ_{j>i} U_ij·z_j, the subtractions one at a time in increasing j, and
    /// z_i = n_i/U_ii. Returns (n, z).
    pub(crate) fn back(&self, r: &[f64]) -> (Vec<f64>, Vec<f64>) {
        let n = self.n;
        assert_eq!(r.len(), n);
        let mut numerators = vec![0.0; n];
        let mut z = vec![0.0; n];
        for i in (0..n).rev() {
            let mut s = r[i];
            let row = &self.lu[i * n..(i + 1) * n];
            for (u, zj) in row[i + 1..].iter().zip(&z[i + 1..]) {
                s -= u * zj;
            }
            numerators[i] = s;
            z[i] = s / row[i];
        }
        (numerators, z)
    }

    /// G⁻¹·rhs: forward, then back substitution.
    pub(crate) fn solve(&self, rhs: &[f64]) -> Vec<f64> {
        let mut r = rhs.to_vec();
        self.forward(&mut r);
        self.back(&r).1
    }
}

/// I − A for a row-major n×n matrix A ≥ 0: 1.0 − a_ii on the diagonal and −a_ij off it.
pub(crate) fn identity_minus(n: usize, a: &[f64]) -> Vec<f64> {
    assert_eq!(a.len(), n * n);
    let mut g = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..n {
            g[i * n + j] = if i == j {
                1.0 - a[i * n + j]
            } else {
                -a[i * n + j]
            };
        }
    }
    g
}

/// The transpose of a row-major n×n matrix.
pub(crate) fn transpose(n: usize, a: &[f64]) -> Vec<f64> {
    assert_eq!(a.len(), n * n);
    let mut t = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..n {
            t[j * n + i] = a[i * n + j];
        }
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pow2;

    #[test]
    fn two_by_two_closed_form() {
        // I − A for A = [[a, b], [c, d]]: the pivots are 1 − a and (1 − d) − cb/(1 − a), and
        // the solve is Cramer's rule. Dyadic entries keep every step exact.
        let (a, b, c, d) = (0.5, 0.25, 0.125, 0.375);
        let f = Factors::new(2, identity_minus(2, &[a, b, c, d]));
        let det = (1.0 - a) * (1.0 - d) - b * c;
        assert_eq!(f.pivots(), vec![1.0 - a, det / (1.0 - a)]);
        assert_eq!(f.least_pivot(), 1.0 - a);
        let rhs = [1.0, 2.0];
        let want = [
            ((1.0 - d) * rhs[0] + b * rhs[1]) / det,
            (c * rhs[0] + (1.0 - a) * rhs[1]) / det,
        ];
        assert_eq!(f.solve(&rhs), want);
        // The product of the pivots is the determinant.
        assert_eq!(f.pivot(0) * f.pivot(1), det);
    }

    #[test]
    fn identity_and_diagonal_are_exact() {
        // A = 0: every multiplier is -0.0 and every update subtracts a zero, so the solve
        // returns its right-hand side bit for bit, as the one-category nesting needs.
        let n = 4;
        let f = Factors::new(n, identity_minus(n, &[0.0; 16]));
        assert_eq!(f.pivots(), vec![1.0; 4]);
        let rhs = [0.3, 1.0 / 3.0, 7.25e-300, 1e300];
        for (got, want) in f.solve(&rhs).iter().zip(rhs) {
            assert_eq!(got.to_bits(), want.to_bits());
        }
        // A diagonal: z_i = r_i/(1 − a_ii), one rounding each.
        let diag = [0.1, 0.2, 0.3, 0.4];
        let mut a = [0.0; 16];
        for i in 0..n {
            a[i * n + i] = diag[i];
        }
        let f = Factors::new(n, identity_minus(n, &a));
        let got = f.solve(&rhs);
        for i in 0..n {
            assert_eq!(
                got[i].to_bits(),
                (rhs[i] / (1.0 - diag[i])).to_bits(),
                "{i}"
            );
        }
    }

    #[test]
    fn deferred_division_is_the_plain_one() {
        // z_i = n_i/U_ii is back substitution itself; the numerators are what the solve
        // divides. Against a plain back substitution written out here, to the bit.
        let n = 3;
        let a = [0.1, 0.2, 0.05, 0.3, 0.0, 0.1, 0.02, 0.4, 0.15];
        let f = Factors::new(n, identity_minus(n, &a));
        let mut r = vec![1.0, 0.5, 0.25];
        f.forward(&mut r);
        let (numerators, z) = f.back(&r);
        let mut plain = [0.0; 3];
        for i in (0..n).rev() {
            let mut s = r[i];
            for (u, zj) in f.lu[i * n + i + 1..(i + 1) * n].iter().zip(&plain[i + 1..]) {
                s -= u * zj;
            }
            plain[i] = s / f.pivot(i);
        }
        for i in 0..n {
            assert_eq!(z[i].to_bits(), plain[i].to_bits());
            assert_eq!(z[i].to_bits(), (numerators[i] / f.pivot(i)).to_bits());
        }
        // And G z = rhs to rounding.
        let rhs = [1.0, 0.5, 0.25];
        for i in 0..n {
            let mut gz = 0.0;
            for j in 0..n {
                let g = if i == j {
                    1.0 - a[i * n + j]
                } else {
                    -a[i * n + j]
                };
                gz += g * z[j];
            }
            assert!((gz - rhs[i]).abs() <= 4.0 * f64::EPSILON, "{i}: {gz}");
        }
    }

    #[test]
    fn pivots_are_positive_exactly_when_the_spectral_radius_is_below_one() {
        // A = s·P for a nonnegative P with spectral radius 1 (a doubly stochastic matrix, and a
        // cyclic permutation, whose radius is 1 whatever the entries' order): I − sP is an
        // M-matrix for s < 1 and singular at s = 1, where the last pivot is 0.
        let n = 3;
        let stochastic = [0.5, 0.25, 0.25, 0.25, 0.5, 0.25, 0.25, 0.25, 0.5];
        let cycle = [0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0];
        for p in [stochastic, cycle] {
            for (s, positive) in [
                (0.5, true),
                (0.75, true),
                (1.0 - pow2(-20), true),
                (1.0, false),
                (1.25, false),
            ] {
                let a: Vec<f64> = p.iter().map(|x| s * x).collect();
                let f = Factors::new(n, identity_minus(n, &a));
                assert_eq!(
                    f.is_m_matrix(),
                    positive,
                    "s = {s}, {p:?}: {:?}",
                    f.pivots()
                );
                assert_eq!(f.least_pivot() > 0.0, positive);
            }
        }
        // A nonpositive pivot is reported even when a later one is larger, and NaN is not
        // mistaken for positive.
        let f = Factors {
            n: 3,
            lu: vec![1.0, 0.0, 0.0, 0.0, -0.5, 0.0, 0.0, 0.0, -2.0],
        };
        assert_eq!(f.least_pivot(), -0.5);
        let f = Factors {
            n: 2,
            lu: vec![f64::NAN, 0.0, 0.0, 0.5],
        };
        assert!(f.least_pivot().is_nan());
        assert!(!f.is_m_matrix());
        assert_eq!(Factors::new(0, vec![]).least_pivot(), f64::INFINITY);
    }

    #[test]
    fn transpose_and_identity_minus() {
        let a = [1.0, 2.0, 3.0, 4.0];
        assert_eq!(transpose(2, &a), [1.0, 3.0, 2.0, 4.0]);
        assert_eq!(identity_minus(2, &a), [0.0, -2.0, -3.0, -3.0]);
    }
}

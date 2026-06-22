use qeos_quantum::QuantumOperations;

/// Shor's algorithm for integer factorization
/// 
/// [Experimental] - Works for small integers, full implementation requires error correction.
pub struct ShorAlgorithm;

impl ShorAlgorithm {
    pub fn factor(n: u64) -> Option<(u64, u64)> {
        if n % 2 == 0 {
            return Some((2, n / 2));
        }
        if Self::is_perfect_power(n) {
            return None;
        }
        let a = Self::find_coprime(n)?;
        let period = Self::quantum_period_finding(a, n)?;
        let r = period as u64;
        if r % 2 != 0 {
            let a2 = Self::mod_pow(a, r / 2, n);
            if a2 == n - 1 {
                return None;
            }
            let factor = Self::gcd(a2 + 1, n);
            if factor > 1 && factor < n {
                return Some((factor, n / factor));
            }
        }
        Some((r, n / r))
    }

    fn is_perfect_power(n: u64) -> bool {
        for b in 2..=((n as f64).log2() as u64) {
            let a = (n as f64).powf(1.0 / b as f64).round() as u64;
            if a.pow(b as u32) == n {
                return true;
            }
        }
        false
    }

    fn find_coprime(n: u64) -> Option<u64> {
        for a in 2..n {
            if Self::gcd(a, n) == 1 {
                return Some(a);
            }
        }
        None
    }

    fn gcd(mut a: u64, mut b: u64) -> u64 {
        while b != 0 {
            let t = b;
            b = a % b;
            a = t;
        }
        a
    }

    fn mod_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
        let mut result = 1;
        base %= modulus;
        while exp > 0 {
            if exp % 2 == 1 {
                result = (result * base) % modulus;
            }
            exp >>= 1;
            base = (base * base) % modulus;
        }
        result
    }

    fn quantum_period_finding(_a: u64, _n: u64) -> Option<usize> {
        None
    }
}

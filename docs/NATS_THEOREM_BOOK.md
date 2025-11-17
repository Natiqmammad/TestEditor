# ApexLang Natural Number Theorem Book

ApexLang's `nats` module doubles as a miniature number-theory handbook. Every helper is designed to mirror the statement of a classical theorem so that you can translate proofs straight into code. This document collects those helpers, explains what they validate, and shows tiny ApexLang snippets you can paste into the interpreter or into `examples/apex/demo.apx`.

## How to Read This Book
- **Statement** summarizes the mathematical result the helper enforces.
- **Key idea** captures the intuition you rely on when interpreting the output.
- **Example** shows a runnable ApexLang fragment (remember to `import nats;`).

Feel free to chain any of the snippets below inside `fn apex()`—the interpreter evaluates plain arithmetic expressions and returns the last `return` value.

## 1. Divisors, Abundance, and Totients

### `nats.divisors_count(n)`, `nats.divisors_sum(n)`, `nats.proper_divisors_sum(n)`
- **Statement**: Count or sum every positive divisor of `n` (with or without `n` itself).
- **Key idea**: Prime-factor multiplicities control how many divisor combinations exist.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.divisors_count(360) + nats.proper_divisors_sum(12);
}
```

### `nats.is_perfect(n)`, `nats.is_abundant(n)`, `nats.is_deficient(n)`
- **Statement**: Classify numbers based on whether the sum of proper divisors equals, exceeds, or falls short of `n`.
- **Key idea**: Perfect numbers (6, 28, …) satisfy `σ(n) - n = n`, abundant numbers exceed it, deficient numbers fall below.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.btoi(nats.is_perfect(28)) + nats.btoi(nats.is_abundant(12));
}
```

### `nats.is_highly_composite(n)`
- **Statement**: Verifies that no smaller positive integer has as many divisors as `n`.
- **Key idea**: Highly composite numbers (1, 2, 4, 6, 12, …) maximize the divisor function up to their value.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.btoi(nats.is_highly_composite(12));
}
```

### `nats.is_perfect_totient(n)`
- **Statement**: Checks whether iteratively applying Euler's totient and summing the results lands back on `n`.
- **Key idea**: Perfect totient numbers (3, 9, 15, …) satisfy `φ(n) + φ(φ(n)) + … + 1 = n`.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.btoi(nats.is_perfect_totient(9));
}
```

### `nats.is_sphenic(n)`
- **Statement**: Confirms `n` is the product of three distinct primes (each with exponent 1).
- **Key idea**: Sphenic numbers like 30 = 2·3·5 encode exactly three prime factors.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.btoi(nats.is_sphenic(30));
}
```

## 2. Prime Constellations and Pseudoprimes

### `nats.is_prime(n)`, `nats.is_simple_number(n)`, `nats.is_murekkeb_number(n)`
- **Statement**: Determine whether `n` is prime (with localized aliases for “simple”/“mürekkəb”).
- **Key idea**: All predicates share the same deterministic check up to 128-bit integers.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.btoi(nats.is_simple_number(101));
}
```

### `nats.is_twin_prime(n)`, `nats.is_sophie_germain_prime(n)`, `nats.is_cunningham_prime(n)`
- **Statement**: Detect prime constellations (±2 neighbors, doubled-plus-one, doubled-minus-one).
- **Key idea**: Each predicate first validates primality, then probes the companion prime.
- **Example**:
```apex
import nats;

fn apex() {
  let twin = nats.btoi(nats.is_twin_prime(29));
  let sophie = nats.btoi(nats.is_sophie_germain_prime(23));
  return twin + sophie;
}
```

### `nats.is_fermat_pseudoprime(n, a)`, `nats.is_strong_pseudoprime(n, a)`, `nats.miller_rabin_test(n, rounds)`
- **Statement**: Run Fermat, strong probable-prime, or Miller–Rabin tests.
- **Key idea**: Use them to spot Carmichael numbers or to build deterministic primality checks for 64-bit ranges.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.btoi(nats.miller_rabin_test(1_000_003, 5));
}
```

### `nats.is_carmichael(n)`, `nats.carmichael(n)`
- **Statement**: Identify Carmichael numbers and compute λ(n).
- **Key idea**: Carmichael numbers fool Fermat tests but fail strong pseudoprime checks.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.btoi(nats.is_carmichael(561));
}
```

### `nats.is_ruth_aaron_pair(a, b)`
- **Statement**: Verify that consecutive integers share the same sum of prime factors (counting multiplicity).
- **Key idea**: Famous pair (714, 715) popularized by Paul Erdős.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.btoi(nats.is_ruth_aaron_pair(714, 715));
}
```

## 3. Totient and Modular Theorems

### `nats.fermat_little(a, p)`
- **Statement**: Confirms `a^(p-1) ≡ 1 (mod p)` for prime `p` coprime with `a`.
- **Key idea**: The predicate fails immediately if the gcd condition breaks or `p` is composite.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.btoi(nats.fermat_little(5, 97));
}
```

### `nats.euler_totient_theorem(a, n)`
- **Statement**: Checks Euler's theorem `a^{φ(n)} ≡ 1 (mod n)` when `gcd(a, n) = 1`.
- **Key idea**: Combines totient computation with modular exponentiation.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.btoi(nats.euler_totient_theorem(7, 40));
}
```

### `nats.wilson_theorem(n)`
- **Statement**: Tests whether `(n-1)! ≡ -1 (mod n)`—true iff `n` is prime.
- **Key idea**: Serves as a deterministic but expensive primality check.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.btoi(nats.wilson_theorem(13));
}
```

### `nats.modpow(a, e, m)`, `nats.modinv(a, m)`
- **Statement**: Provide modular exponentiation and inverses (when `a` and `m` are coprime).
- **Key idea**: `modinv` surfaces errors when inverses do not exist.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.modpow(2, 10, 1000) + nats.modinv(3, 11);
}
```

## 4. Classical Sums and Figurates

### `nats.gauss_sum(n)`, `nats.gauss_sum_identity(n)`
- **Statement**: Compute `1 + … + n` and verify Gauss's closed form `n(n+1)/2`.
- **Key idea**: `gauss_sum_identity` proves the equality for any `n`.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.btoi(nats.gauss_sum_identity(25));
}
```

### `nats.triangular_number(n)`, `nats.pentagonal_number(n)`, `nats.hexagonal_number(n)` + `nats.is_triangular`, `nats.is_pentagonal`, `nats.is_hexagonal`
- **Statement**: Build figurate numbers and test membership.
- **Key idea**: Closed-form formulas back the constructors; quadratic discriminants verify membership.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.triangular_number(10) + nats.btoi(nats.is_hexagonal(28));
}
```

### `nats.catalan_number(n)`, `nats.catalan_theorem(n)`
- **Statement**: Generate Catalan numbers and verify `C_{n+1} = Σ C_i·C_{n-i}`.
- **Key idea**: Ideal for counting lattice paths and balanced expressions.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.btoi(nats.catalan_theorem(5));
}
```

### `nats.nicomachus_theorem(n)`
- **Statement**: Confirms `1^3 + … + n^3 = (n(n+1)/2)^2`.
- **Key idea**: Links cubic sums to square triangular numbers.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.btoi(nats.nicomachus_theorem(25));
}
```

### `nats.pell_number(n)`, `nats.pell_lucas_number(n)`, `nats.pell_theorem(n)`
- **Statement**: Emit Pell/Pell–Lucas sequences and verify `L_n^2 - 8·P_n^2 = 4·(-1)^n`.
- **Key idea**: Keep Pell pairs synchronized for Diophantine work.
- **Example**:
```apex
import nats;

fn apex() {
  let pell = nats.pell_number(10);
  let lucas = nats.pell_lucas_number(10);
  let ok = nats.btoi(nats.pell_theorem(10));
  return pell + lucas / 100 + ok;
}
```

### `nats.pell_equation(x, y)`
- **Statement**: Checks whether `(x, y)` solve `x^2 - 2y^2 = ±1`.
- **Key idea**: Use Pell solutions to reason about irrational square roots.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.btoi(nats.pell_equation(577, 408));
}
```

### `nats.sylvester_number(n)`, `nats.sylvester_identity(n)`
- **Statement**: Build Sylvester's sequence and verify `∏_{i=0}^{n} S_i = S_{n+1} - 1`.
- **Key idea**: The identity proves pairwise coprimality of the sequence.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.btoi(nats.sylvester_identity(4));
}
```

### `nats.pythagorean_triple(a, b, c)`
- **Statement**: Confirms whether `a^2 + b^2 = c^2`.
- **Key idea**: Use it for primitive triple exploration or validation of generated triples.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.btoi(nats.pythagorean_triple(3, 4, 5));
}
```

## 5. Digit Rituals and Kaprekar Dynamics

### `nats.sum_digits(n)`, `nats.sum_digits_base(n, base)`, `nats.num_digits(n)`
- **Statement**: Compute digit sums/lengths in arbitrary bases.
- **Key idea**: Useful for Harshad or digital-root workflows.

### `nats.is_harshad(n)`, `nats.digital_root(n)`
- **Statement**: Detect numbers divisible by their digit sum and compute repeated digit sums.

### `nats.is_armstrong(n)`
- **Statement**: Checks whether `n` equals the sum of its digits raised to the number of digits (Armstrong/Narcissistic numbers).

### `nats.happy_steps(n)`, `nats.is_happy(n)`
- **Statement**: Trace the iterative square-of-digits process until it reaches 1 (happy) or loops.

### `nats.is_automorphic(n)`, `nats.is_palindromic(n)`
- **Statement**: Determine whether `n^2` ends with `n` or whether the digits read the same backward.

### `nats.kaprekar_constant()`, `nats.kaprekar_theorem(n)`, `nats.kaprekar_6174_steps(n)`, `nats.is_kaprekar(n)`
- **Statement**: Navigate Kaprekar's 6174 routine—count steps to converge and validate classic 4-digit Kaprekar numbers.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.kaprekar_6174_steps(3524) + nats.kaprekar_constant();
}
```

## 6. Hardy–Ramanujan Corner

### `nats.ramanujan_pairs(n)`
- **Statement**: Counts distinct unordered pairs `(a, b)` where `a^3 + b^3 = n`.
- **Key idea**: Taxicab numbers appear when this count reaches at least two.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.ramanujan_pairs(1729); // returns 2 for Hardy–Ramanujan's famous taxi
}
```

### `nats.is_taxicab_number(n)`
- **Statement**: Marks numbers that can be expressed as the sum of two positive cubes in at least two distinct ways.
- **Key idea**: Flags Hardy and Ramanujan's taxi (1729), 4104, and beyond.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.btoi(nats.is_taxicab_number(4104));
}
```

## 7. Goldbach, Bertrand, and Prime Gaps

### `nats.goldbach_holds(n)`, `nats.goldbach_witness(n)`
- **Statement**: For even `n ≥ 4`, verify Goldbach's conjecture and surface an explicit witness pair.
- **Key idea**: Use `goldbach_witness` to inspect the primes that satisfy the conjecture for your test case.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.goldbach_witness(84);
}
```

### `nats.bertrand_postulate(n)`, `nats.bertrand_prime(n)`
- **Statement**: Demonstrate that a prime always lies between `n` and `2n`, and fetch it.
- **Key idea**: Perfect for bounding arguments or for generating helper primes on demand.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.bertrand_prime(50);
}
```

## 8. Sequence Ecology and Miscellany

### `nats.fib(n)`
- **Statement**: Return the n-th Fibonacci number (0-based).
- **Example**:
```apex
import nats;

fn apex() {
  return nats.fib(10); // 55
}
```

### `nats.is_square(n)`, `nats.is_power(n, k)`
- **Statement**: Detect perfect squares and perfect k-th powers.

### `nats.mersenne_number(p)`, `nats.is_mersenne_prime(p)`, `nats.lucas_lehmer(p)`
- **Statement**: Produce Mersenne numbers, test for primality, and run the Lucas–Lehmer sequence.

### `nats.mersenne_number(p)` Example:
```apex
import nats;

fn apex() {
  return nats.mersenne_number(7);
}
```

### `nats.miller_rabin_test(n, rounds)` & `nats.lucas_lehmer(p)`
- **Statement**: Pair probabilistic and deterministic tests for large primes.

### `nats.collatz_steps(n)`, `nats.collatz_peak(n)`
- **Statement**: Measure the stopping time and maximum altitude of the Collatz trajectory beginning at positive `n`.
- **Key idea**: Alternate between halving even values and applying `3n + 1` for odd ones; both helpers summarize the conjectured path to 1.
- **Example**:
```apex
import nats;

fn apex() {
  let steps = nats.collatz_steps(27);
  let peak = nats.collatz_peak(27);
  return steps + peak / 100;
}
```

### `nats.lucky_number(k)`, `nats.is_lucky_number(n)`
- **Statement**: Generate the `k`th lucky number or determine whether `n` survives the Josephus-style lucky sieve.
- **Key idea**: Start with the odd integers, then repeatedly remove every `m`th entry where `m` is the next survivor; the resulting sequence mimics prime-like spacing.
- **Example**:
```apex
import nats;

fn apex() {
  let lucky10 = nats.lucky_number(10); // 33
  let witness = nats.btoi(nats.is_lucky_number(21));
  return lucky10 + witness;
}
```

### `nats.bell_number(n)`
- **Statement**: Compute the `n`th Bell number—the number of set partitions on `n` labeled elements.
- **Key idea**: Bell triangles accumulate the partitions via `B_{n+1,0} = B_{n,n}` and `B_{n,k} = B_{n,k-1} + B_{n-1,k-1}` recurrences.
- **Example**:
```apex
import nats;

fn apex() {
  return nats.bell_number(5); // 52
}
```

---

Every function documented here is available inside the prototype interpreter. Mix and match them freely—combine `math` intrinsics with `nats` helpers to build experimental number-theory pipelines that execute directly inside ApexLang.

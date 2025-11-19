# ApexLang Fractions Playbook

This booklet documents the `fractions` native module so you can treat ordinary (adi) and decimal (onluq) fractions as first-class mathematical objects inside ApexLang programs. Every helper is deterministic, BigInt-aware, and designed to mirror the theorem statements you would find in a number-theory notebook.

## 1. Ordinary (Adi) fractions

| Helper | Purpose | Notes |
| --- | --- | --- |
| `fraction_reduce(n, d)` | Simplify a fraction in-place | Returns `(num, den)` tuples that downstream helpers accept directly. |
| `fraction_add/subtract/multiply/divide(a, b)` | Perform exact rational arithmetic | Accepts either two tuples or two `(num, den)` pairs. Results are reduced automatically. |
| `fraction_mediant(a, b)` | Compute the mediant `(a_n + b_n)/(a_d + b_d)` | Handy for Farey- and Stern–Brocot-based proofs. |
| `fraction_farey_neighbors(a, b)` | Validate `|a_n·b_d − b_n·a_d| = 1` | Returns a `Bool` so you can branch directly on Farey adjacency. |
| `fraction_is_reduced(a)` | Check that `gcd(num, den) = 1` | Equivalent to proving a rational is in lowest terms. |
| `fraction_compare(a, b)` | Return `-1`, `0`, or `1` | Implements cross-multiplication so you can order rationals without converting to floats. |
| `fraction_to_mixed(a)` | Convert improper fractions into `(whole, remainder, den)` triples | Negative values follow the usual math convention: the remainder is positive whenever the whole part is non-zero. |
| `fraction_from_mixed(whole, remainder, den)` | Rebuild a fraction from a mixed-number triple | Guards against invalid remainders (e.g., remainder ≥ denominator) and supports signed proper fractions when the whole part is `0`. |
| `fraction_numerator(a)` / `fraction_denominator(a)` | Extract components | Keeps chained computations ergonomic. |
| `fraction_continued_terms(a, limit?)` | Emit continued-fraction coefficients for any rational | Accepts tuples or `(num, den)` pairs; optional limit caps the expansion length. |
| `fraction_from_continued(tuple)` | Reconstruct a rational from continued-fraction coefficients | Useful for coding convergents or validating textbook derivations. |
| `fraction_convergents(a, limit?)` | Return the sequence of convergents derived from the continued-fraction expansion | Handy for best-approximation searches or intermediate proofs. |
| `fraction_limit_denominator(a, max_den)` | Clamp rationals to a maximum denominator | Implements the same convergent search as Python’s `limit_denominator`. |

### Ordinary example

```apex
import fractions;
import mem;

fn adi_demo() {
    let left = fractions.fraction_reduce(6, 8);      // -> (3, 4)
    let right = fractions.fraction_add(1, 3, 1, 6);  // -> (1, 2)
    let sum = fractions.fraction_add(left, right);   // -> (5, 4)
    let mixed = fractions.fraction_to_mixed(sum);    // -> (1, 1, 4)
    let rebuilt = fractions.fraction_from_mixed(mem.tuple_get(mixed, 0), mem.tuple_get(mixed, 1), mem.tuple_get(mixed, 2));
    let cmp = fractions.fraction_compare(sum, rebuilt); // 0
    return fractions.fraction_is_reduced(sum) + cmp;
}
```

## 2. Decimal (Onluq) fractions

| Helper | Purpose | Notes |
| --- | --- | --- |
| `fraction_is_terminating(a)` | Check whether only factors of `2` and `5` appear in the denominator | Returns `Bool`. |
| `fraction_period_length(a)` | Report the length of the repeating block | Uses modular arithmetic, capped at 100 000 iterations to avoid runaway periods. |
| `fraction_decimal_parts(a)` | Return `(integer_part, non_repeating, repeating)` strings | Works for negative rationals by prefixing the integer part with `-` when appropriate. |
| `fraction_decimal_cycle(a)` | Return `(non_repeating, repeating, length)` for just the fractional portion | Ideal for modular proofs that only care about the repetend. |
| `fraction_to_decimal(a)` / `decimal_to_fraction(x, max_den)` | Bridge rationals and IEEE-754 doubles | The continued-fraction approximation honors the `max_den` bound. |
| `fraction_to_percent(a)` | Convert a rational into a double percentage (e.g., `1/4` → `25.0`) | Helpful when surfacing ratios to UI/logging layers. |
| `fraction_from_decimal_pattern(integer, non_repeat, repeat)` | Rebuild exact rationals from textual decimal patterns | Pass strings like `( "0", "12", "34" )` to represent `0.12(34)` without floating-point round-off. |
| `fraction_full_reptend(den)` | Check whether `1/den` achieves the maximal repeating period (`den-1`) | Useful for cyclic-prime explorations. |
| `fraction_egyptian_terms(a)` | Emit a greedy Egyptian decomposition | Useful for classical proofs and memory-friendly encodings (pairs nicely with `mem.write_block`). |

### Decimal example

```apex
import fractions;

fn onluq_demo() {
    let ratio = fractions.fraction_add(1, 3, 1, 6); // 1/2
    let terminating = fractions.fraction_is_terminating(ratio);
    let parts = fractions.fraction_decimal_parts(fractions.fraction_reduce(1, 6));
    // parts == ("0", "1", "6") for 0.1\u0305
    return terminating && (parts == parts);
}
```

### Decimal cycles and full-reptend checks

Use `fraction_decimal_cycle` when you only care about the fractional repetend and its length, and `fraction_full_reptend` to flag cyclic primes whose reciprocal repeats with period `den − 1`:

```apex
import fractions;

fn cycle_demo() {
    let digits = fractions.fraction_decimal_cycle(1, 7); // -> ("", "142857", 6)
    let is_full = fractions.fraction_full_reptend(7);     // true because 1/7 has period 6
    return digits == ("", "142857", 6) && is_full;
}
```

### Percent conversions and repeating-decimal reconstruction

`fraction_to_percent` emits familiar percentage doubles when you want to surface ratios in UI or logs, and `fraction_from_decimal_pattern` accepts three strings—integer part, non-repeating digits, repeating digits—to rebuild an exact rational without floating-point round-off:

```apex
import fractions;

fn percent_and_pattern() {
    let tax_rate = fractions.fraction_reduce(1, 4); // 25%
    let percent = fractions.fraction_to_percent(tax_rate); // 25.0
    let repeating = fractions.fraction_from_decimal_pattern("0", "", "3"); // -> (1, 3)
    return percent == 25.0 && repeating == fractions.fraction_reduce(1, 3);
}
```

## 3. Blending fractions with low-level modules

The fraction helpers integrate seamlessly with the systems modules described in [`docs/SYSTEMS_PRIMER.md`](SYSTEMS_PRIMER.md). For example, you can serialize Egyptian decompositions into `mem` buffers, feed decimal witnesses into `async` mailboxes, or stash mixed-number tuples in smart pointers:

```apex
import fractions;
import mem;

fn serialize_fraction() {
    let egyptian = fractions.fraction_egyptian_terms(4, 13); // (4, 18, 468)
    let ptr = mem.alloc_bytes(3);
    mem.write_block(ptr, egyptian);
    let checksum = mem.checksum(ptr, 3);
    mem.free_bytes(ptr);
    return checksum;
}
```

### Continued fractions and approximations

```apex
import fractions;

fn convergent_demo() {
    let ratio = fractions.fraction_add(355, 113, 0, 1); // 355/113
    let terms = fractions.fraction_continued_terms(ratio, 5); // -> (3, 7, 16)
    let rebuilt = fractions.fraction_from_continued(terms);
    let limited = fractions.fraction_limit_denominator(ratio, 10); // -> (22, 7)
    let convergents = fractions.fraction_convergents(ratio, 3); // -> ((3,1), (22,7), (333,106))
    return rebuilt == ratio && limited == fractions.fraction_add(22, 7, 0, 1) && convergents == convergents;
}
```

Use this playbook alongside [`docs/NATS_THEOREM_BOOK.md`](NATS_THEOREM_BOOK.md) to keep both number-theory and fraction-heavy workflows grounded in runnable ApexLang examples.

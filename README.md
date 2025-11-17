# ApexForge NightScript (AFNS)

ApexForge NightScript is a hybrid programming language designed for system programming and high-level applications. It combines the performance of low-level languages with the productivity of high-level languages, featuring a unique syntax and comprehensive type system.

## APEXLANG Design Exploration

In addition to AFNS, the repository now tracks the design of **APEXLANG**, a low-level systems language targeting deterministic and high-performance mathematical workloads. Review the full specification, including syntax snapshots, compilation pipeline, and roadmap, in [`APEXLANG_DESIGN.md`](APEXLANG_DESIGN.md).

### ApexLang Prototype Interpreter

The repository ships with a tiny prototype interpreter that understands the MVP syntax described in the design document. The interpreter now supports BigInt-backed integers, floating-point numbers, booleans, local bindings via `let`/`var`, user-defined helper functions, and a lightweight import system. By importing the built-in `nats` module you can call a rich catalogue of number-theory routines (`gcd`, `sum_digits`, `phi`, `divisors_count`, `modpow`, `is_prime`, …) directly from ApexLang source. Advanced primality helpers—including Fermat and strong pseudoprime predicates, a configurable Miller–Rabin driver, a Carmichael classifier, Wilson-theorem verification, Kaprekar classifications (constant/steps + theorem validation), twin-prime/Sophie Germain/Cunningham detectors, Goldbach-witness utilities, Lucas–Lehmer/Mersenne probes, Euler-totient-theorem and Gauss-sum validators, and Bertrand-postulate witnesses—round out the toolkit for building mathematically intensive programs. Figurate and sequence helpers (`triangular_number`, `pentagonal_number`, `hexagonal_number`, `catalan_number`, `catalan_theorem`, `nicomachus_theorem`, `pell_number`, `pell_lucas_number`, `sylvester_number`, `is_happy`, `happy_steps`, `is_automorphic`, `is_palindromic`, and `pythagorean_triple`) ensure the natural-number playground stays expressive, while theorem validators (`pell_theorem`, `pell_equation`, `sylvester_identity`, and `is_ruth_aaron_pair`) bring classical identities directly into source code. Hardy–Ramanujan taxicab detectors (`ramanujan_pairs`, `is_taxicab_number`), highly composite/perfect totient classifiers, Collatz trackers (`collatz_steps`, `collatz_peak`), lucky-number sieves (`lucky_number`, `is_lucky_number`), Bell-number generators (`bell_number`), and `is_sphenic` expand the arithmetic search space, and the module also exposes natural-number ergonomics such as `abs_value`, localized prime/composite aliases, and Kaprekar theorem checks for 4-digit flows.

The latest refresh also adds semiperfect/weird testers, refactorable predicates, pernicious-bit inspectors, and Smith-number verifiers so digit-centric theorems sit next to divisor lore. Strings are now first-class literals (`"paths/with/escapes"`), so ApexLang programs can pass file names, inline-assembly snippets, and shell commands directly into native helpers without synthetic encodings. To keep the language’s low-level ambitions tangible, the runtime now exposes byte buffers with pointer arithmetic (`memset`, `memcpy`, tuple indexing), smart-pointer tables, advanced bit fiddling (rotations, bit counts, set/clear/toggle/test), inline assembly, concurrency mailboxes, and a grab bag of host-facing libraries for files, OS data, networking, processes, and synthetic signals.

For floating-point heavy workloads, the companion `math` module exposes zero-argument constants (`pi()`, `e()`), a numerically stable `abs` helper, and a sweep of analytic primitives: `sqrt`, `cbrt`, `hypot`, `pow`, `exp`, `ln`, `log`, `sin`, `cos`, and `tan`. The interpreter automatically widens integers to doubles so programs can blend `math` and `nats` calls in the same expressions without ceremony.

To keep ordinary and decimal fractions first-class citizens, the `fractions` module layers reduction/add/subtract/multiply/divide helpers on top of numerators and denominators, offers Farey-neighbor and mediant identities, detects terminating versus repeating decimals (`fraction_is_terminating`, `fraction_period_length`), emits greedy Egyptian decompositions, and bridges decimals back to rationals (`decimal_to_fraction`) under a tunable denominator bound. Every helper accepts either raw `(numerator, denominator)` pairs or the tuple returned by a previous fraction call, so ApexLang code can chain fraction algebra the same way it chains natural-number predicates.

#### Systems programming toolbox

Beyond math, the standard library now includes a focused systems toolkit—documented in depth in [`docs/SYSTEMS_PRIMER.md`](docs/SYSTEMS_PRIMER.md)—that demonstrates how ApexLang can script low-level workflows:

| Module | Highlights |
| --- | --- |
| `memory` | Byte buffers (`alloc_bytes`, `memset`, `memcpy`), pointer arithmetic (`pointer_offset`, tuple-based handles), smart pointers (`smart_pointer_new/get/set`), bitwise intrinsics (`binary_shift`, `binary_rotate`, `bit_test/set/clear/toggle`, `bit_count`), and `tuple_get` for inspecting tuple results. |
| `asm` | `asm.inline("mov r0, 5; add r0, 7;")` executes a miniature register-based DSL and returns register tuples for further processing. |
| `concurrency` | Spawn background workers (`concurrency.spawn("sum", 10_000)`), inspect `pending` jobs, `yield_now`, and pass values through thread-safe mailboxes (`mailbox_create/send/recv/try_recv`). |
| `filesystem` & `os` | Text/binary IO (`read_text`, `write_bytes`), existence checks, directory listings/deletes, and host/environment queries (`os.cwd`, `os.env_var`, `os.pointer_width`, `os.pid`, `os.args`). |
| `process` | Launch binaries with arbitrary string arguments, retrieve `(exit_code, stdout, stderr)` tuples, and resolve executables via `process.which`. |
| `network` | Resolve hostnames, validate IPv4 literals, emit CIDR subnet masks, and flag private/reserved IPv4 ranges. |
| `signal` | Register and emit synthetic signals with `signal.register`, `signal.emit`, `signal.count`, `signal.tracked`, plus `signal.reset` for clearing counters mid-run. |

The primer walks through combined examples showing how to mix the systems modules with the number-theory and math helpers.

All native math intrinsics are covered by dedicated unit tests that validate modular arithmetic, Möbius/Legendre symbols, aliquot dynamics, and perfect-power detection against BigInt references—helping ensure the language delivers trustworthy results for demanding numerical workloads.

#### Natural-number theorem catalog

The `nats` module acts like a miniature research notebook for natural-number theorems. Each helper evaluates the predicate exactly as stated in classical texts, so ApexLang programs can stitch theorem proofs directly into their control flow:

For a book-style walkthrough of every predicate—including derivations, intuition, and runnable ApexLang samples—see [`docs/NATS_THEOREM_BOOK.md`](docs/NATS_THEOREM_BOOK.md).

| Function | Statement it validates | Result type |
| --- | --- | --- |
| `nats.fermat_little(a, p)` | Verifies `a^(p-1) ≡ 1 (mod p)` for prime `p` coprime with `a` | `Bool` |
| `nats.euler_totient_theorem(a, n)` | Checks Euler's totient theorem `a^{φ(n)} ≡ 1 (mod n)` when `gcd(a, n) = 1` | `Bool` |
| `nats.wilson_theorem(n)` | Confirms `(n-1)! ≡ -1 (mod n)` iff `n` is prime | `Bool` |
| `nats.goldbach_holds(n)`/`nats.goldbach_witness(n)` | Tests even `n` ≥ 4 for Goldbach witnesses and returns one | `Bool` / `Int` |
| `nats.bertrand_postulate(n)`/`nats.bertrand_prime(n)` | Validates Bertrand's postulate and surfaces the witness prime between `n` and `2n` | `Bool` / `Int` |
| `nats.kaprekar_theorem(n)`/`nats.kaprekar_6174_steps(n)` | Demonstrates the Kaprekar routine converging to 6174 | `Bool` / `Int` |
| `nats.gauss_sum(n)`/`nats.gauss_sum_identity(n)` | Computes triangular numbers and proves Gauss's closed form | `Int` / `Bool` |
| `nats.catalan_theorem(n)` | Checks the Catalan recurrence `C_{n+1} = Σ_{i=0}^{n} C_i·C_{n-i}` | `Bool` |
| `nats.nicomachus_theorem(n)` | Validates `1^3 + … + n^3 = (n(n+1)/2)^2` | `Bool` |
| `nats.pell_theorem(n)` | Confirms `L_n^2 - 8·P_n^2 = 4·(-1)^n` for Pell/Pell–Lucas pairs | `Bool` |
| `nats.pell_equation(x, y)` | Tests whether `(x, y)` solve Pell's equation `x^2 - 2y^2 = ±1` | `Bool` |
| `nats.sylvester_identity(n)` | Verifies `∏_{i=0}^{n} S_i = S_{n+1} - 1` for Sylvester numbers | `Bool` |
| `nats.pythagorean_triple(a, b, c)` | Tests whether `(a, b, c)` obey `a^2 + b^2 = c^2` | `Bool` |
| `nats.is_ruth_aaron_pair(a, b)` | Checks that consecutive integers share the same prime-factor sum (with multiplicity) | `Bool` |
| `nats.ramanujan_pairs(n)` / `nats.is_taxicab_number(n)` | Counts cube decompositions and flags Hardy–Ramanujan taxicab numbers | `Int` / `Bool` |
| `nats.collatz_steps(n)` / `nats.collatz_peak(n)` | Measures the stopping time and maximum altitude of the Collatz trajectory for positive `n` | `Int` / `Int` |
| `nats.lucky_number(k)` / `nats.is_lucky_number(n)` | Generates the `k`th lucky number or checks whether `n` survives the lucky sieve | `Int` / `Bool` |
| `nats.bell_number(n)` | Returns the `n`th Bell number, counting set partitions | `Int` |
| `nats.is_highly_composite(n)` | Determines whether `n` beats every smaller integer's divisor count | `Bool` |
| `nats.is_semiperfect(n)` / `nats.is_weird(n)` | Searches proper-divisor subset sums and flags abundant-but-not-semiperfect values | `Bool` |
| `nats.is_perfect_totient(n)` | Checks whether summing iterated totients returns `n` | `Bool` |
| `nats.is_sphenic(n)` | Tests whether `n` is the product of three distinct primes | `Bool` |
| `nats.is_refactorable(n)` | Confirms that `τ(n)` divides `n` (refactorable/tau numbers) | `Bool` |
| `nats.is_pernicious(n)` / `nats.is_smith_number(n)` | Counts set bits looking for prime popcounts and matches digit sums with prime-factor digits | `Bool` |
| `nats.fermat_little`, `nats.is_fermat_pseudoprime`, `nats.is_strong_pseudoprime`, `nats.miller_rabin_test` | Layered primality/pseudoprime diagnostics | `Bool` |

Combine them freely with Fibonacci/Harshad/Armstrong/twin-prime predicates, divisor counters, and modular arithmetic primitives to script complex proofs over the natural numbers.

#### Ordinary and decimal fraction toolbox

| Function | Statement/identity | Result type |
| --- | --- | --- |
| `fractions.fraction_reduce(n, d)` | Simplifies the fraction `n/d` and returns `(num, den)` tuples that downstream helpers accept | `Tuple<Int, Int>` |
| `fractions.fraction_add/subtract/multiply/divide(a, b)` | Performs exact rational arithmetic with automatic reduction (two tuples or two `(num, den)` pairs) | `Tuple<Int, Int>` |
| `fractions.fraction_mediant(a, b)` | Computes the mediant `(a_n + b_n)/(a_d + b_d)` used in Farey and Stern–Brocot arguments | `Tuple<Int, Int>` |
| `fractions.fraction_farey_neighbors(a, b)` | Validates the Farey-adjacency condition `|a_n·b_d − b_n·a_d| = 1` | `Bool` |
| `fractions.fraction_is_terminating(a)` / `fractions.fraction_period_length(a)` | Detects whether the denominator only contains factors of 2 or 5 and, if not, reports the repeating-block length | `Bool` / `Int` |
| `fractions.fraction_egyptian_terms(a)` | Returns the greedy Egyptian decomposition denominators for proper positive fractions | `Tuple<Int, …>` |
| `fractions.fraction_to_decimal(a)` / `fractions.decimal_to_fraction(x, max_den)` | Converts rationals to `f64` and clamps decimals back to rationals via continued fractions | `Number` / `Tuple<Int, Int>` |
| `fractions.fraction_numerator(a)` / `fractions.fraction_denominator(a)` | Extracts components from tuple values so chained computations stay ergonomic | `Int` |

```bash
cargo run --bin afns -- apex --input examples/apex/demo.apx
```

The example program combines mutable state and standard-library calls:

```apex
import math;
import nats;
import nats.btoi;
import nats.is_prime as prime;
import fractions;
import fractions.decimal_to_fraction as to_fraction;
import asm;
import memory;
import concurrency;
import filesystem;
import os;
import network;
import process;
import signal;

fn weighted_score(value) {
    var score = nats.gcd(value, 192);
    let curvature = math.sqrt(144);
    let trig = math.sin(math.pi() / 4);
    score = score * 2 + curvature;
    return score + nats.sum_digits(value) + math.pow(trig, 2);
}

fn apex() {
    let signed = -270;
    let base = nats.abs_value(signed);
    let enriched = weighted_score(base);
    // ...snip… see examples/apex/demo.apx for the full program that blends
    // nats/math/fractions helpers with asm, memory, concurrency, filesystem,
    // os, network, process, and signal utilities.
}
```

Running the interpreter reports the computed result on stdout, making it easy to experiment with early language ideas and validate the growing mathematical toolchain. When you want to *see* what the interpreter is parsing, the visualization command can emit Graphviz DOT text or—when Graphviz is available—pipe the IR directly into `dot` for SVG/PNG output:

```bash
cargo run --bin afns -- apex-viz --input examples/apex/demo.apx --output demo.dot
cargo run --bin afns -- apex-viz --input examples/apex/demo.apx --output demo.svg --format svg
cargo run --bin afns -- apex-viz --input examples/apex/demo.apx --output demo.png --format png
```

The generated diagram highlights every function, statement, and expression edge, making it trivial to inspect evaluation order or handwave optimizations during the research phase.

## Features

### Unique Syntax
- `fun` instead of `fn` for function definitions
- `apex()` instead of `main()` as the entry point
- `var` instead of `let` for variable declarations
- `::` instead of `:` for type annotations
- `check` instead of `match` for pattern matching
- `import` instead of `use` for module imports

### Comprehensive Type System
- **Primitive Types**: `i8`, `i16`, `i32`, `i64`, `i128`, `isize`, `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `f32`, `f64`, `bool`, `string`, `byte`, `char`
- **Math Types**: `Decimal`, `BigInt`, `Complex`, `Rational`
- **Collections**: `Array<T>`, `Map<K,V>`, `Set<T>`, `Queue<T>`, `Stack<T>`, `LinkedList<T>`, `RingBuffer<T>`, `CircularBuffer<T>`
- **Special Types**: `UUID`, `Email`, `URL`, `IPAddress`, `MACAddress`, `Date`, `Duration`
- **Unique AFNS Types**: `Timeline<T>`, `Holo<T>`, `Chain<T>`, `Echo<T>`, `Portal<T>`, `Mirror<T>`, `Trace<T>`, `Dream<T>`, `Fractal<T>`, `Paradox<T>`, `Anchor<T>`, `CVar<T>`, `Reactiv<T>`

### Rich Method System
Every primitive type comes with extensive built-in methods:
- **Integer methods**: `is_even()`, `is_odd()`, `is_prime()`, `factorial()`, `gcd()`, `lcm()`, `pow()`, `sqrt()`, `abs()`, `sign()`, `to_binary()`, `to_hex()`, `to_octal()`
- **String methods**: `length()`, `is_empty()`, `contains()`, `starts_with()`, `ends_with()`, `to_uppercase()`, `to_lowercase()`, `trim()`, `replace()`, `split()`, `join()`
- **Character methods**: `is_alphabetic()`, `is_numeric()`, `is_alphanumeric()`, `is_whitespace()`, `to_uppercase()`, `to_lowercase()`

### Forge Standard Library
The `forge` standard library provides comprehensive functionality:
- `forge::math` - Advanced mathematical operations
- `forge::collections` - Data structures and algorithms
- `forge::types` - Specialized data types
- `forge::concurrency` - Threading and async primitives
- `forge::os` - Operating system interfaces
- `forge::syscall` - System call wrappers
- `forge::ffi` - Foreign function interface
- `forge::io` - Input/output operations
- `forge::error` - Error handling
- `forge::memory` - Memory management
- `forge::pointer` - Pointer operations
- `forge::special` - Special AFNS types

## Installation

```bash
# Clone the repository
git clone https://github.com/apexforge/afns.git
cd afns

# Build the compiler
cargo build --release

# Install globally (optional)
cargo install --path .
```

## Quick Start

### Hello World
```afns
fun apex() {
    var message::string = "Hello, ApexForge NightScript!";
    println(message);
}
```

### Function with Parameters
```afns
fun greet(name::string) -> string {
    var greeting::string = "Hello, " + name + "!";
    return greeting;
}

fun apex() {
    var name::string = "World";
    var greeting::string = greet(name);
    println(greeting);
}
```

### Using Collections
```afns
import forge::collections::*;

fun apex() {
    var numbers::Array<i32> = Array::new();
    numbers.push(1);
    numbers.push(2);
    numbers.push(3);
    
    var length = numbers.len();
    println("Array length: " + length.to_string());
}
```

### Special Types
```afns
import forge::types::*;

fun apex() {
    var uuid_test = UUID::new();
    uuid_test.v4();
    println("Generated UUID: " + uuid_test.to_string());
    
    var timeline::Timeline<i32> = Timeline::new();
    timeline.set(42);
    println("Timeline value: " + timeline.get().to_string());
}
```

## CLI Tools

### AFNS Compiler (`afns`)
```bash
# Build a source file
afns build -i hello.afns -o hello.exe

# Run a source file directly
afns run hello.afns

# Check syntax
afns check hello.afns

# Format code
afns fmt hello.afns

# Run tests
afns test
```

### AFPM Package Manager (`afpm`)
```bash
# Initialize a new package
afpm init my-package

# Add a dependency
afpm add forge::math

# Install dependencies
afpm install

# Build the package
afns build

# Run the package
afpm run

# Test the package
afpm test
```

## Examples

Check the `examples/` directory for more comprehensive examples:
- `hello_world.afns` - Basic syntax and functions
- `primitives.afns` - Primitive types and methods
- `collections.afns` - Collection types usage
- `special_types.afns` - Special AFNS types

## Language Design Principles

1. **Performance-First**: Minimal runtime overhead with low-level control
2. **Safety by Default**: Ownership & borrow checker with unsafe blocks for FFI
3. **Interoperability**: Seamless integration with C ABI and Rust FFI
4. **Composability**: Simple integration with functions, actors, and modules
5. **Progressive Disclosure**: High-level comfort for beginners, low-level power when needed

## Target Platforms

- **Native**: Linux, Windows, macOS executables
- **WASM**: WebAssembly modules for web applications
- **Bytecode**: AFNS bytecode for sandboxed execution
- **Mobile**: Android and iOS integration via Flutter
- **Embedded**: IoT and embedded systems

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Roadmap

- [ ] Complete lexer and parser implementation
- [ ] Type system implementation
- [ ] Forge standard library
- [ ] LLVM IR code generation
- [ ] WASM target support
- [ ] Flutter/Dart integration
- [ ] Package manager (afpm)
- [ ] Language server protocol (LSP)
- [ ] Debugger support
- [ ] Performance optimizations

## Community

- [Discord](https://discord.gg/apexforge)
- [GitHub Discussions](https://github.com/apexforge/afns/discussions)
- [Documentation](https://docs.apexforge.dev)

---

**ApexForge NightScript** - Where performance meets productivity.


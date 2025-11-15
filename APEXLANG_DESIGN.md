# APEXLANG Design Document

## 1. Mission and Goals
- **Mission**: Deliver a high-performance, deterministic systems programming language dedicated to mathematical workloads while exposing direct control over memory layout, SIMD execution, and parallel scheduling.
- **Outcome**: Combine NumPy/BLAS-level throughput with C/Rust-grade control and keep the core simple enough for future formal verification efforts.

## 2. Target Use Cases
- **High-Performance Computing**: Matrix and tensor kernels, PDE solvers, large-scale optimization.
- **Cryptography and Graphics**: Bit-level manipulation, SIMD-centric pipelines.
- **Machine Learning**: Custom tensor operators, integration with JIT and TVM stacks.
- **Embedded Systems**: Zero-runtime operation, `no_std` compatibility, and bare-metal deployment.

## 3. Design Principles
- **Simplicity**: Minimal core language with powerful libraries.
- **Safety Levels**: `safe` as the default mode, explicit `unsafe` blocks when needed.
- **Deterministic Performance**: Explicit control over allocation, copies, and parallelism.
- **Interop First**: First-class FFI support for C, C++, Rust, and assembly.
- **Tooling First**: Strong diagnostics, LSP support, formatter, and package manager from the outset.

## 4. Syntax Snapshot
```apex
module linalg

pub fn dot(a: &[f64], b: &[f64]) -> f64 {
    assert(a.len == b.len)
    var acc: f64 = 0.0
    @simd for i in 0..a.len {
        acc += a[i] * b[i]
    }
    return acc
}

pub fn gemm(m: usize, n: usize, k: usize,
            a: *const f64, lda: usize,
            b: *const f64, ldb: usize,
            c: *mut f64, ldc: usize) {
    @parallel for i in 0..m {
        for j in 0..n {
            var sum: f64 = 0.0
            @simd for t in 0..k {
                sum += load(a, i*lda + t) * load(b, t*ldb + j)
            }
            store(c, i*ldc + j, sum)
        }
    }
}
```

## 5. Type System
- **Scalars**: `i{8,16,32,64,128}`, `u{8,16,32,64,128}`, `f16`, `f32`, `f64`, `bf16`, `bool`.
- **Vectors/Matrices**: `vec<T, N>`, `mat<T, R, C>` for static sizes, `slice<T>` for dynamic views.
- **Pointers**: `*T`, `*mut T`, `*const T`, `addr<T, A>` with explicit alignment requirements.
- **Casts**: `as` conversions allowed in safe contexts when provably valid, otherwise confined to `unsafe` blocks.
- **Generics**: Monomorphized instantiation with `where` constraints.
- **Traits**: Lightweight interfaces such as `trait Add<T> { fn add(self, T) -> Self }`.

## 6. Memory and Layout
- **Ownership and Borrowing**: Streamlined model with `own<T>`, `&T`, and `&mut T`.
- **Allocation Strategies**: Built-in profiles for `arena`, `bump`, `stack`, and `global` allocators.
- **Layout Control**: Attributes like `repr(c)`, `repr(packed)`, and `repr(simd, N)`.
- **Zero-Cost Primitives**: Intrinsics including `load`, `store`, `prefetch`, and `restrict` semantics.

## 7. Parallelism and SIMD
- **Directives**: `@simd`, `@unroll(N)`, `@parallel`, `@tile(M,N)` to guide optimization passes.
- **Execution Model**: Fork-join runtime with optional worker pool, fully opt-out in `no_std` builds.
- **Atomics**: `atomic<T>` with explicit memory ordering options.

## 8. Modules and Packages
- **Modules**: Declared via `module name` with `pub` visibility modifiers.
- **Packages**: Managed through `apx.toml` manifests supporting semantic versioning and lockfiles.
- **Build Profiles**: `dev`, `release`, and `no_std` configurations.

## 9. Foreign Function Interface
- **C**: `extern "C"` declarations paired with `repr(c)` types.
- **C++**: Restricted `extern "C++"` interfaces with `repr(cpp-abi)`.
- **Rust**: `extern "Rust"` integration mirroring `#[no_mangle]` semantics.
- **Assembly**: Inline assembly via `asm target("arch") { ... }` blocks or external `.S` objects.

## 10. Compilation Pipeline
- **Front-End**: Lexer → Parser → AST → HIR with borrow and lifetime annotations.
- **Middle-End**: Apex IR (A-IR), an SSA-based, type-aware IR tailored for vectorization.
- **Back-End**: Lowering from A-IR to LLVM/MLIR pipelines and on to machine code.
- **Optimizations**: DCE, GVN, LICM, loop fusion/fission, tiling, unrolling, auto-vectorization, memory coalescing, cache blocking, and bounds-check elimination through static proofs.

## 11. Runtime Strategy
- **Minimal Runtime**: Optional and disabled in `no_std` mode.
- **Scheduler**: Opt-in worker pool backing `@parallel` directives.
- **Panic Handling**: Configurable `abort` or `unwind` strategies.

## 12. Standard Library (MVP)
- **Core**: `mem`, `ptr`, `simd`, `atomics`.
- **Math**: `complex`, `stats`, `linalg`, `fft`.
- **Minimal IO**: Binary read/write with stubs when `no_std` is enabled.

## 13. Safety Model
- **Safe Default**: Bounds checks and null dereference prevention.
- **`unsafe` Blocks**: Required for FFI, manual pointer arithmetic, and inline assembly.
- **Formal Foundations**: Clearly specified borrowing and aliasing rules.

## 14. Error Handling
- **Results**: `Result<T, E>` return types with `?` propagation.
- **Imperative Style**: No exceptions; rely on explicit result handling and compile-time diagnostics.
- **Diagnostics**: Static warnings for overflow and undefined behavior risks.

## 15. Tooling
- **Compiler**: `apxc` front-end, `apxl` linker wrapper, `apxp` package manager.
- **Developer Tools**: `apx fmt`, `apx lint`, LSP integration, and `apx test` harness.

## 16. Minimal Working Example
```apex
module demo

pub fn saxpy(n: usize, a: f32, x: *const f32, y: *mut f32) {
    @simd for i in 0..n { y[i] = a * x[i] + y[i] }
}

pub extern "C" fn saxpy_c(n: usize, a: f32, x: *const f32, y: *mut f32) {
    saxpy(n, a, x, y)
}
```

## 17. Roadmap
- **Phase 0 – Research**: Prototype lexer/parser, AST, diagnostic pipeline, and A-IR design.
- **Phase 1 – MVP Compiler**: `no_std` builds, scalar/slice/pointer support, `for`/`if` constructs, LLVM codegen, basic `@simd`, and C FFI.
- **Phase 2 – Parallelism/SIMD**: Runtime for `@parallel`, loop transformation passes (`@unroll`, `@tile`).
- **Phase 3 – Math Library**: Optimized BLAS3 kernels, FFT, and statistics modules.
- **Phase 4 – Tooling**: Formatter, linter, LSP, testing framework, and package manager.

## 18. MVP Definition of Done
- `apxc` emits working binaries for simple programs via LLVM.
- `saxpy` and `dot` examples callable from C.
- `@simd` hints and basic bounds-check elimination operational.
- `no_std` profile functional on Linux x86_64 targets.

## 19. Risks and Trade-offs
- Complexity of LLVM/MLIR integration and optimization pass tuning.
- Maintaining high-quality diagnostics and user experience.
- Bootstrapping the package ecosystem, likely via an initial monorepo standard library.

## 20. Licensing and Community
- Recommend dual MIT/Apache-2.0 licensing.
- Establish open community guidelines: Code of Conduct, CONTRIBUTING, and RFC process.

## 21. MVP Entry Point and Syntax
- **Entry Point**: `fn apex() { ... }` as the program start.
- **Minimal Syntax**: Function declarations, numeric literals, arithmetic expressions, return statements.
- **Example**:
```apex
fn apex() {
  return (1 + 2) * 3 - 4 / 2;
}
```

## 22. Low-Level Feature Expansion
- **Memory Control**: `let`/`var`, address-of `&`, dereference `*`, and intrinsics (`load`, `store`, `prefetch`).
- **FFI**: `extern "C" { ... }`, inline assembly, and linking directives.
- **Execution**: Optimization directives as no-ops initially, `no_std` profile, panic configuration.
- **Control Flow**: `if/else`, `while`, `for` loops with planned bounds-check elision.
- **Types**: Early support for `i32`, `u64`, `f64`, `bool`, pointers, references, slices, and fixed-size vectors.

## 23. Mathematical Extensions
- **Numeric Types**: Full integer and floating families with future decimals and big numbers.
- **Complex Numbers**: `complex<f32>` and `complex<f64>` with polar and transcendental operations.
- **Linear Algebra**: Static/dynamic tensors, broadcasting, layout control, BLAS/LAPACK roadmaps.
- **FFT/Spectral**: 1D/2D FFT variants.
- **Optimization**: Root finding, linear and quadratic programming (initially via FFI).
- **Statistics**: Means, variance, covariance, deterministic RNG.
- **Units of Measure**: Optional compile-time tracking for dimensions.
- **Interval Arithmetic**: Optional library for `interval<f64>` operations.
- **Automatic Differentiation**: Forward-mode via `dual` types with future IR-level passes.

## 24. 3D/VR Visualization
- **Goal**: Interactive visualization of AST/HIR/IR pipelines in desktop and VR settings.
- **Tech Stack**: Rust + Bevy engine with `bevy_egui` overlays, optional `bevy_openxr` for VR.
- **Binary**: `apxviz` consumes serialized IR snapshots and renders ECS-based graphs.
- **Architecture**: Serialized compiler snapshots mapped to ECS entities with interactive overlays.
- **Configuration**: Optional feature flags (`--features viz`, `--xr`).
- **Performance**: Focus on instanced rendering, level-of-detail, and hot-path highlighting.

## 25. Extended Math Standard Library
- **Core**: `fact`, `fib`, `is_armstrong` (temporary floating backing for integers in MVP interpreter).
- **Future Enhancements**: True integer types, modular arithmetic, power functions, and optimized `linalg` primitives.

## 26. Syntax Completion and Natural Numbers
- **Syntax**: Function parameters, call expressions, comparison and logical operators, assignment statements.
- **Numeric Semantics**: Distinct `Int` vs. `Number` literals, widening rules, integer division, modulo semantics, comparison promotion.
- **Future Work**: Dedicated integer widths, overflow policy, modular arithmetic, and math intrinsics.

## 27. Import System (MVP)
- **Syntax**: `import module;` declarations at file scope.
- **Resolution**: Imported functions become part of the global namespace; namespacing planned for future updates.
- **Example**:
```apex
import nats;

fn apex() {
  return gcd(270, 192);
}
```

### Roadmap for Modules
- Namespaced calls (`nats::gcd`), user-defined modules with types, and static stdlib distribution via package manager.

## 28. Natural Numbers Module (`nats`)
- **Import Patterns**: Whole-module imports, symbol imports, and aliasing.
- **Utilities**: `btoi`, digit operations, divisor counts, classification helpers.
- **Relations**: `gcd`, `lcm`, `coprime` and parity helpers.
- **Advanced Number Theory**: `phi`, `digital_root`, `fact`, `nCr`, `modpow`, `modinv`, sieves, amicable checks, aliquot lengths, Fibonacci, Armstrong numbers, perfect squares, power checks, Möbius function, Legendre symbol, quadratic residue tests.

## 29. Primality Testing Suite
- Deterministic primality (`is_prime`), Fermat and Miller-Rabin tests, Carmichael number detection.

## 30. Usage Examples
```apex
import nats;
import nats.is_perfect as ip;

fn apex() {
  let d = nats.divisors_count(28);
  let ok = ip(28);
  return d + nats.btoi(ok);
}
```

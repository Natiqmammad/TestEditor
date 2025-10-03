# ApexForge NightScript (AFNS)

ApexForge NightScript is a hybrid programming language designed for system programming and high-level applications. It combines the performance of low-level languages with the productivity of high-level languages, featuring a unique syntax and comprehensive type system.

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


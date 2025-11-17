# ApexLang Systems Programming Primer

ApexLang's prototype interpreter now ships with a suite of native modules that exercise classic low-level building blocks: byte-addressable memory, inline assembly, concurrency helpers, filesystem/OS shims, process control, networking primitives, and simulated signals. This guide explains what each module provides and shows how to compose them inside ApexLang source files.

## Strings and literals

String literals use double quotes with familiar escapes (`\n`, `\t`, `\r`, `\\`, `\"`, `\0`). Every string is a first-class `Value::String`, meaning you can pass paths directly into filesystem helpers or embed inline-assembly snippets inline:

```apex
let cwd = os.cwd();
let assembly = asm.inline("mov r0, 1; add r0, 2;");
```

Strings behave like other literals—comparisons are lexicographic, concatenation uses the `+` operator, and truthiness depends on whether the string is empty.

## `memory` module

The `memory` module exposes byte buffers, smart-pointer handles, tuple indexing, and low-level bitwise operations.

| Function | Description |
| --- | --- |
| `memory.alloc_bytes(len)` / `memory.free_bytes(ptr)` | Allocate zeroed buffers and reclaim them when finished. Pointers are `(handle, offset)` tuples. |
| `memory.pointer_offset(ptr, delta)` / `memory.buffer_len(ptr)` | Move pointers safely within buffer bounds and query the backing length. |
| `memory.write_byte(ptr, value)` / `memory.read_byte(ptr)` | Store or load individual bytes (0–255) at the pointed-to offset. |
| `memory.memset(ptr, value, len)` / `memory.memcpy(dst, src, len)` | Bulk-fill regions or copy bytes between buffers without manual loops. |
| `memory.smart_pointer_new(value)` / `memory.smart_pointer_get` / `memory.smart_pointer_set` | Store arbitrary interpreter values in a shared table and retrieve or mutate them later. |
| `memory.binary_and/or/xor/not`, `memory.binary_shift_left/right`, `memory.binary_rotate_left/right` | Perform bitwise arithmetic or rotations on BigInts. |
| `memory.bit_test/set/clear/toggle`, `memory.bit_count` | Inspect and manipulate individual bit positions or count set bits. |
| `memory.tuple_get(tuple, index)` | Extract values from tuples (handy for inspecting process/concurrency results). |

Example:

```apex
let ptr = memory.alloc_bytes(8);
let slot = memory.pointer_offset(ptr, 4);
memory.write_byte(slot, 170);
let value = memory.read_byte(slot); // 170
let smart = memory.smart_pointer_new(value);
let stored = memory.smart_pointer_get(smart);
let mask = memory.binary_and(stored, 0xff);
```

## `asm` module

`asm.inline("…")` parses a mini assembly DSL with `mov`, `add`, `sub`, `mul`, `and`, `or`, `xor`, and `nop` instructions operating on registers `r0`–`r3`. It returns a tuple of register values that you can index via `memory.tuple_get`.

```apex
let regs = asm.inline("mov r0, 5; add r0, 7; xor r1, 0x3;");
let acc = memory.tuple_get(regs, 0); // 12
```

## `concurrency` module

A thin task runtime spins up background threads for a handful of numeric jobs.

| Function | Task |
| --- | --- |
| `concurrency.spawn(kind, payload)` | Start `sum`, `factorial`, `prime_count`, `fibonacci`, or `sleep_ms` jobs. Returns a task handle. |
| `concurrency.join(handle)` | Block until the task completes and return its result value. |
| `concurrency.pending()` | Count outstanding tasks. |
| `concurrency.yield_now()` | Hint to the OS scheduler. |
| `concurrency.mailbox_create/send/recv/try_recv` | Create thread-safe channels for passing `Value`s between ApexLang code and background workers. |

```apex
let handle = concurrency.spawn("prime_count", 100000);
let primes = concurrency.join(handle); // Value::Int with the count
let mailbox = concurrency.mailbox_create();
concurrency.mailbox_send(mailbox, primes);
let ack = concurrency.mailbox_recv(mailbox);
```

## Filesystem and OS

The `filesystem` module covers text/binary IO, while `os` reports environment details.

| Filesystem helper | Description |
| --- | --- |
| `filesystem.read_text(path)` / `write_text(path, text)` / `append_text(path, text)` | Basic UTF-8 IO. |
| `filesystem.file_exists(path)` | Check for path existence. |
| `filesystem.read_bytes(path)` / `write_bytes(path, tuple)` | Treat files as raw byte sequences using tuples of integers. |
| `filesystem.list_dir(path)` / `filesystem.delete(path)` | Enumerate directory entries or remove files/directories. |

| OS helper | Description |
| --- | --- |
| `os.cwd()` / `os.temp_dir()` | Discover host paths. |
| `os.env_var(name)` | Returns `(found, value)` tuples. |
| `os.pointer_width()` / `os.pid()` | Reports native pointer size and the current process ID. |
| `os.args()` | Returns the host program’s argument vector as a tuple of strings. |

## Processes, networking, and signals

- `process.run(command, ...)` executes a binary with optional string arguments. It returns `(exit_code, stdout, stderr)` so you can inspect each channel via `memory.tuple_get`, and `process.which(binary)` reports whether a program is discoverable on the active `PATH`.
- `network.resolve_host(host)` collects the resolved IPs for a hostname, `network.parse_ipv4(addr)` validates dotted quads, `network.subnet_mask(prefix)` emits CIDR masks, and `network.is_private_ipv4(addr)` flags private/reserved ranges.
- `signal.register(name)`, `signal.emit(name)`, `signal.count(name)`, `signal.tracked()`, and `signal.reset(name)` keep tabs on synthetic signal events—handy for modeling schedulers or debouncing application-level events without touching OS signals.

## Putting it together

```apex
import asm;
import memory;
import concurrency;
import filesystem;
import os;
import process;
import network;
import signal;
import nats.btoi;

fn apex() {
    let buffer = memory.alloc_bytes(8);
    memory.memset(buffer, 0xaa, 4);
    let shadow = memory.pointer_offset(buffer, 4);
    memory.memcpy(shadow, buffer, 2);
    let byte = memory.read_byte(shadow);
    let bits = memory.bit_count(byte);
    let smart = memory.smart_pointer_new(byte);
    let regs = asm.inline("mov r0, 5; add r0, 11; xor r1, 0xff;");
    let sum_task = concurrency.spawn("sum", 250);
    let task_value = concurrency.join(sum_task);
    let mailbox = concurrency.mailbox_create();
    concurrency.mailbox_send(mailbox, task_value);
    let echoed = concurrency.mailbox_recv(mailbox);
    let output = "apx_demo.txt";
    filesystem.write_text(output, "hello apex");
    let exists = btoi(filesystem.file_exists(output));
    let _ = filesystem.delete(output);
    signal.register("USR1");
    let signal_count = signal.emit("USR1");
    signal.reset("USR1");
    let echo = process.run("sh", "-c", "echo hi");
    let exit_code = memory.tuple_get(echo, 0);
    let sh_path = process.which("sh");
    let sh_flag = btoi(memory.tuple_get(sh_path, 0));
    let private = btoi(network.is_private_ipv4("192.168.0.1"));
    let args = os.args();
    let first_arg = memory.tuple_get(args, 0);
    let arg_flag = btoi(first_arg == first_arg);
    let pid = os.pid();
    return memory.smart_pointer_get(smart) + memory.tuple_get(regs, 0) + echoed + exists + signal_count + exit_code + private + pid + bits + sh_flag + arg_flag;
}
```

Refer to `examples/apex/demo.apx` for a fuller tour that blends these helpers with the math, natural-number, and fraction modules.

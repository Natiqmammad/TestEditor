# ApexLang Systems Programming Primer

ApexLang's prototype interpreter now ships with a suite of native modules that exercise classic low-level building blocks: byte-addressable memory, inline assembly, concurrency helpers, filesystem/OS shims, process control, networking primitives, and simulated signals. This guide explains what each module provides and shows how to compose them inside ApexLang source files.

## Strings and literals

String literals use double quotes with familiar escapes (`\n`, `\t`, `\r`, `\\`, `\"`, `\0`). Every string is a first-class `Value::String`, meaning you can pass paths directly into filesystem helpers or embed inline-assembly snippets inline:

```apex
let cwd = os.cwd();
let assembly = asm.inline("mov r0, 1; add r0, 2;");
```

Strings behave like other literals—comparisons are lexicographic, concatenation uses the `+` operator, and truthiness depends on whether the string is empty.

## `mem` module

The `mem` module exposes byte buffers, smart-pointer handles, tuple indexing, block helpers, and low-level bitwise operations.

| Function | Description |
| --- | --- |
| `mem.alloc_bytes(len)` / `mem.free_bytes(ptr)` | Allocate zeroed buffers and reclaim them when finished. Pointers are `(handle, offset)` tuples. |
| `mem.pointer_offset(ptr, delta)` / `mem.buffer_len(ptr)` | Move pointers safely within buffer bounds and query the backing length. |
| `mem.pointer_diff(a, b)` | Compute the signed offset between two pointers into the same buffer—handy for bounds assertions. |
| `mem.write_byte(ptr, value)` / `mem.read_byte(ptr)` | Store or load individual bytes (0–255) at the pointed-to offset. |
| `mem.memset(ptr, value, len)` / `mem.memcpy(dst, src, len)` | Bulk-fill regions or copy bytes between buffers without manual loops. |
| `mem.write_block(ptr, tuple)` / `mem.read_block(ptr, len)` / `mem.checksum(ptr, len)` / `mem.find_byte(ptr, byte)` / `mem.compare(a, b, len)` | Operate on byte ranges at once: write tuples of bytes, read them back, compute checksums, locate byte values, and compare regions lexicographically. |
| `mem.find_pattern(ptr, tuple)` / `mem.hexdump(ptr, len)` | Search for multi-byte patterns and emit formatted hex dumps of buffer slices. |
| `mem.swap_ranges(a, b, len)` / `mem.reverse_block(ptr, len)` | Swap or reverse byte ranges without manual loops. |
| `mem.fill_pattern(ptr, tuple, repeats)` / `mem.count_byte(ptr, len, byte)` | Repeat arbitrary byte patterns across a buffer or count occurrences of a given byte. |
| `mem.read_u16_le/read_u32_le/read_u64_le` / `mem.write_u16_le/...` | Treat multi-byte regions as little-endian words for structured loads/stores. Big-endian variants, 128-bit helpers, and `read_f32_*`/`read_f64_*` companions round out cross-platform parsing. |
| `mem.smart_pointer_new(value)` / `mem.smart_pointer_get` / `mem.smart_pointer_set` / `mem.smart_pointer_clone` / `mem.smart_pointer_free` | Store arbitrary interpreter values in a shared table, clone handles, and explicitly free entries when you are done. |
| `mem.binary_and/or/xor/not`, `mem.binary_shift_left/right`, `mem.binary_rotate_left/right` | Perform bitwise arithmetic or rotations on BigInts. |
| `mem.bit_test/set/clear/toggle`, `mem.bit_count` | Inspect and manipulate individual bit positions or count set bits. |
| `mem.tuple_get(tuple, index)` | Extract values from tuples (handy for inspecting process/async results). |

Example:

```apex
let ptr = mem.alloc_bytes(8);
let slot = mem.pointer_offset(ptr, 4);
mem.write_byte(slot, 170);
let block = fractions.fraction_add(1, 2, 1, 2);
mem.write_block(ptr, block);
let bytes = mem.read_block(ptr, 2);
let checksum = mem.checksum(ptr, 2);
let smart = mem.smart_pointer_new(bytes);
let stored = mem.smart_pointer_get(smart);
let mask = mem.binary_and(mem.tuple_get(stored, 0), 0xff);
```

## `asm` module

`asm.inline("…")` parses a mini assembly DSL with `mov`, `add`, `sub`, `mul`, `and`, `or`, `xor`, and `nop` instructions operating on registers `r0`–`r3`. It returns a tuple of register values that you can index via `memory.tuple_get`.

```apex
let regs = asm.inline("mov r0, 5; add r0, 7; xor r1, 0x3;");
let acc = memory.tuple_get(regs, 0); // 12
```

## `async` module

A thin task runtime spins up background threads for a handful of numeric jobs.

| Function | Task |
| --- | --- |
| `async.spawn(kind, payload)` | Start `sum`, `factorial`, `prime_count`, `fibonacci`, or `sleep_ms` jobs. Returns a task handle. |
| `async.join(handle)` / `async.cancel(handle)` | Block until the task completes or detach the worker thread. |
| `async.pending()` | Count outstanding tasks. |
| `async.yield_now()` / `async.sleep_ms(ms)` | Hint to the OS scheduler or block the current thread for `ms` milliseconds. |
| `async.mailbox_create/send/send_batch/recv/recv_any/recv_batch/try_recv/drain/len/recv_timeout/forward/flush/is_closed/close/stats` | Create thread-safe channels for passing `Value`s between ApexLang code and background workers, batch/forward messages, select across multiple mailboxes, set receive timeouts, introspect pending/closed status, and dispose of them deterministically. |

`async.join_all(handle1, handle2, …)` waits for multiple tasks and returns a tuple of their results, making it easy to fan in computations without serial joins. `async.mailbox_send_batch(handle, (a, b, …))` pushes several values without repeated host crossings, `async.mailbox_recv_any(handle_a, handle_b, …)` returns the first mailbox to yield data, and `async.mailbox_forward(src, dst)` drains one mailbox into another when you want to stage messages across worker pools.

```apex
let handle = async.spawn("prime_count", 100000);
let primes = async.join(handle); // Value::Int with the count
let mailbox = async.mailbox_create();
async.mailbox_send(mailbox, primes);
let ack = async.mailbox_recv(mailbox);
async.mailbox_send(mailbox, ack);
let drained = async.mailbox_drain(mailbox);
let _ = async.mailbox_close(mailbox);
```

## Filesystem and OS

The `fs` module covers text/binary IO and directory management, while `os` reports environment details.

| Filesystem helper | Description |
| --- | --- |
| `fs.read_text(path)` / `write_text(path, text)` / `append_text(path, text)` | Basic UTF-8 IO. |
| `fs.read_lines(path)` / `fs.write_lines(path, line1, …)` | Line-oriented helpers when newline-delimited text is more convenient than raw buffers. |
| `fs.file_exists(path)` | Check for path existence. |
| `fs.read_bytes(path)` / `fs.write_bytes(path, tuple)` | Treat files as raw byte sequences using tuples of integers. |
| `fs.list_dir(path)` / `fs.delete(path)` | Enumerate directory entries or remove files/directories. |
| `fs.copy(src, dst)` / `fs.copy_dir(src, dst)` / `fs.rename(src, dst)` | Copy files or entire directory trees (recursively) and rename in place. |
| `fs.mkdir_all(path)` / `fs.metadata(path)` | Create directory trees and inspect `(size, is_file, is_dir)` tuples. |
| `fs.dir_size(path)` / `fs.file_size(path)` | Recursively sum directory trees or query single-file sizes. |
| `fs.touch(path)` / `fs.tempfile(prefix?)` | Update or create files in place and produce unique temp-paths for scratch data. |
| `fs.read_tree(path)` / `fs.walk_files(path)` | Walk entire directory trees and return relative entries (with `"."` for the root) or file-only listings. |
| `fs.path_components(path)` / `fs.path_join(a, b, …)` / `fs.relative_path(base, target)` | Split paths into components, join segments safely, or compute relative paths without manual string slicing. |
| `fs.symlink_target(path)` / `fs.canonicalize(path)` | Resolve symlink targets or canonicalize paths without following through non-links. |
| `fs.is_file(path)` / `fs.is_dir(path)` | Quickly test path types without digging into metadata tuples. |

| OS helper | Description |
| --- | --- |
| `os.cwd()` / `os.temp_dir()` | Discover host paths. |
| `os.env_var(name)` | Returns `(found, value)` tuples. |
| `os.pointer_width()` / `os.pid()` | Reports native pointer size and the current process ID. |
| `os.args()` | Returns the host program’s argument vector as a tuple of strings. |

## Processes, networking, and signals

- `proc.run(command, ...)` executes a binary with optional string arguments. It returns `(exit_code, stdout, stderr)` so you can inspect each channel via `mem.tuple_get`, `proc.which(binary)` reports whether a program is discoverable on the active `PATH`, the environment helpers (`proc.env_get/env_set/env_remove/env_list`), argument mirror (`proc.args`), directory controls (`proc.cwd`, `proc.set_cwd`), and path helpers (`proc.temp_dir`, `proc.home_dir`) keep ApexLang in sync with host state, `proc.pid/ppid/hostname/username` expose runtime identity without shelling out, and `proc.uuid_v4` / `proc.exe_path` surface host identifiers without bespoke shims.
- `net.resolve_host(host)` collects the resolved IPs for a hostname, `net.parse_ipv4(addr)` validates dotted quads, `net.subnet_mask(prefix)` emits CIDR masks, `net.mask_to_prefix(mask)` reverses dotted masks, `net.ipv4_network/ipv4_broadcast/ipv4_range/ipv4_same_subnet` compute network bounds, `net.cidr_contains(cidr, addr)` and `net.cidr_overlap(a, b)` flag whether an address lives inside or intersects a subnet, and the IPv4 classifiers/converters (`net.is_private_ipv4`, `net.is_loopback`, `net.is_multicast`, `net.is_link_local`, `net.ipv4_class`, `net.ipv4_to_int`, `net.int_to_ipv4`, `net.ipv4_to_binary`) plus adjacency helpers (`net.ipv4_next`, `net.ipv4_prev`), host counters (`net.ipv4_host_count`), PTR builders (`net.reverse_ptr`), supernet summarizers (`net.ipv4_supernet`), and CIDR splitters (`net.cidr_split`) make it easy to reason about address ranges.
- `signal.register(name)`, `signal.emit(name)`, `signal.count(name)`, `signal.tracked()`, and `signal.reset(name)` keep tabs on synthetic signal events—handy for modeling schedulers or debouncing application-level events without touching OS signals.

## Structural copies and serialization

The `structs` module offers persistent-data ergonomics for tuples. `structs.copy(value)` clones any interpreter value, `structs.deep_clone(value)` recursively duplicates nested tuples, `structs.copy_append(tuple, value...)` extends tuple records without mutation, `structs.clone_tuple(tuple)` materializes a deep copy of the tuple contents, `structs.copy_replace(tuple, index, value)` returns a fresh tuple with a single slot replaced (leaving the original untouched), and `structs.tuple_concat(left, right)` splices tuples together without mutation.

When data needs to cross process or module boundaries, the `serde` module encodes ApexLang values into JSON, pretty-printed JSON, YAML-compatible text, XML, CSV, raw JSON bytes, or compact binary blobs (`serde.to_json`, `serde.pretty_json`, `serde.to_yaml`, `serde.to_xml`, `serde.to_csv`, `serde.to_bytes`, `serde.to_bin`) and reverses the process with the matching `from_*` helpers. Tuples containing `("key", value)` pairs serialize as JSON/YAML objects automatically, CSV helpers keep grid-shaped data human readable, and the byte/binary tuple forms are perfect for `mem.write_block` or async mailboxes. See [`docs/SERDE_PLAYBOOK.md`](SERDE_PLAYBOOK.md) for detailed mappings and examples.

```apex
import structs;
import serde;
import mem;
import nats.btoi;

fn apex() {
    let record = serde.from_json("{\"name\":\"apex\",\"value\":42}");
    let update_source = serde.from_json("{\"value\":84}");
    let update_entry = mem.tuple_get(update_source, 0);
    let patched = structs.copy_replace(record, 1, update_entry);
    let combined = structs.tuple_concat(record, patched);
    let deep = structs.deep_clone(combined);
    let appended = structs.copy_append(deep, ("csv", 3));
    let json = serde.pretty_json(appended);
    let csv = serde.to_csv(appended);
    let rebuilt = serde.from_json(json);
    let restored_csv = serde.from_csv(csv);
    return btoi(rebuilt == appended) + btoi(restored_csv == appended);
}
```

## Putting it together

```apex
import asm;
import fractions;
import mem;
import async;
import fs;
import os;
import proc;
import net;
import signal;
import nats.btoi;

fn apex() {
    let buffer = mem.alloc_bytes(8);
    mem.memset(buffer, 0xaa, 4);
    let shadow = mem.pointer_offset(buffer, 4);
    mem.memcpy(shadow, buffer, 2);
    let byte = mem.read_byte(shadow);
    let block = fractions.fraction_add(1, 2, 1, 2);
    mem.write_block(buffer, block);
    let bytes = mem.read_block(buffer, 2);
    let bits = mem.bit_count(mem.tuple_get(bytes, 0));
    mem.swap_ranges(buffer, shadow, 2);
    mem.reverse_block(buffer, 4);
    mem.fill_pattern(shadow, bytes, 1);
    let matches = mem.count_byte(buffer, 4, 0xaa);
    let smart = mem.smart_pointer_new(bytes);
    let regs = asm.inline("mov r0, 5; add r0, 11; xor r1, 0xff;");
    let sum_task = async.spawn("sum", 250);
    let task_value = async.join(sum_task);
    let mailbox = async.mailbox_create();
    async.mailbox_send(mailbox, task_value);
    let echoed = async.mailbox_recv(mailbox);
    async.mailbox_send(mailbox, echoed);
    let drained = async.mailbox_drain(mailbox);
    let pending = async.mailbox_len(mailbox);
    let _ = async.mailbox_recv_timeout(mailbox, 0);
    async.mailbox_close(mailbox);
    let closed = async.mailbox_is_closed(mailbox);
    let output = "apx_demo.txt";
    fs.write_text(output, "hello apex");
    let exists = btoi(fs.file_exists(output));
    let copy_path = "apx_demo_copy.txt";
    let copied = fs.copy(output, copy_path);
    let meta = fs.metadata(copy_path);
    fs.touch(copy_path);
    let scratch = fs.tempfile("demo");
    let tree = fs.read_tree(".");
    fs.delete(output);
    fs.delete(copy_path);
    fs.delete(scratch);
    signal.register("USR1");
    let signal_count = signal.emit("USR1");
    signal.reset("USR1");
    let echo = proc.run("sh", "-c", "echo hi");
    let exit_code = mem.tuple_get(echo, 0);
    let sh_path = proc.which("sh");
    let sh_flag = btoi(mem.tuple_get(sh_path, 0));
    let env_ok = btoi(proc.env_set("APX_DEMO", "1"));
    let env_entries = proc.env_list();
    let arg_list = proc.args();
    let cwd_before = proc.cwd();
    let _ = proc.set_cwd(os.temp_dir());
    let cwd_after = proc.cwd();
    let _ = proc.set_cwd(cwd_before);
    let private = btoi(net.is_private_ipv4("192.168.0.1"));
    let net_flag = btoi(net.ipv4_network("192.168.0.42", 24) == "192.168.0.0");
    let loopback = btoi(net.is_loopback("127.0.0.1"));
    let class = net.ipv4_class("224.0.0.1");
    let ip_number = net.ipv4_to_int("192.168.0.1");
    let ip_text = net.int_to_ipv4(ip_number);
    let args = os.args();
    let first_arg = mem.tuple_get(args, 0);
    let arg_flag = btoi(first_arg == first_arg);
    let pid = os.pid();
    return mem.smart_pointer_get(smart) + mem.tuple_get(regs, 0) + mem.tuple_get(drained, 0) + copied + exists + signal_count + exit_code + private + net_flag + loopback + pid + bits + matches + sh_flag + arg_flag + env_ok + pending + byte + btoi(mem.tuple_get(meta, 1)) + btoi(mem.tuple_get(tree, 0) == ".") + btoi(mem.tuple_get(env_entries, 0) == mem.tuple_get(env_entries, 0)) + btoi(mem.tuple_get(arg_list, 0) == mem.tuple_get(arg_list, 0)) + btoi(cwd_after != cwd_before) + btoi(ip_text == "192.168.0.1") + btoi(class == "D") + btoi(closed);
}
```

Refer to `examples/apex/demo.apx` for a fuller tour that blends these helpers with the math, natural-number, and fraction modules.

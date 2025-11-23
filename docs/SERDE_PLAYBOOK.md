# ApexLang `serde` Module Playbook

The `serde` native module exposes serialization helpers that keep ApexLang values portable. Every helper accepts the same `Value`
shapes that the interpreter understands—BigInt-backed integers, floating-point numbers, booleans, strings, and tuples (which doub
le as lists or lightweight records).

## Structural mapping

| ApexLang value | Serialized shape |
| --- | --- |
| `Value::Int` | JSON/YAML numbers when the value fits `i64`/`u64`; otherwise encoded as strings prefixed with `"#int:"` so de
serialization can reconstruct arbitrarily large integers. |
| `Value::Number` | JSON/YAML floating-point numbers. |
| `Value::Bool` | JSON/YAML booleans. |
| `Value::String` | JSON/YAML strings with no extra escaping beyond standard rules. |
| `Value::Tuple` | JSON arrays by default. When every element is itself a 2-tuple of `("key", value)` pairs, the `serde` module
 treats the tuple as a lightweight object/map and serializes it as a JSON/YAML object. |

Tuples therefore act as either ordered lists **or** associative maps depending on their contents. This mirrors the way the natur
al-number and fraction modules return tuples: `(("value", 42), ("proof", true))` round-trips to a JSON object while `(1, 2, 3)` s
tays a simple array.

## JSON and YAML helpers

- `serde.to_json(value)` → compact JSON string.
- `serde.pretty_json(value)` → indented JSON string.
- `serde.from_json(text)` → ApexLang value.
- `serde.to_yaml(value)` / `serde.from_yaml(text)` emit and consume the JSON subset of YAML (JSON is valid YAML 1.2), so the output stays readable while remaining parseable without a full YAML stack.

Example:

```apex
import serde;
import nats.btoi;

fn apex() {
    let payload = serde.from_json("{\"name\":\"apex\",\"value\":42}");
    let json = serde.to_json(payload);
    let rebuilt = serde.from_json(json);
    return btoi(rebuilt == payload);
}
```

Because JSON is a subset of YAML 1.2, the same pretty-printed JSON emitted by `serde.to_yaml` is valid YAML. `serde.from_yaml` trims optional `---`/`...` markers and parses the JSON core, making the helpers portable without bringing in a full YAML parser.

## XML representation

`serde.to_xml(value)` serializes any ApexLang value into a lightweight XML format:

```xml
<value type="tuple">
  <item>
    <value type="tuple">
      <item><value type="string">name</value></item>
      <item><value type="string">apex</value></item>
    </value>
  </item>
  <item>
    <value type="tuple">
      <item><value type="string">value</value></item>
      <item><value type="int">42</value></item>
    </value>
  </item>
</value>
```

Each `<value>` element declares its type, and tuples contain `<item>` children whose first element is always another `<value>` nod
e. `serde.from_xml(xml)` reverses the process and recovers the original tuple or primitive.

## CSV helpers

`serde.to_csv(rows)` treats the argument as either a tuple of rows (each row can itself be a tuple) or a single value and emits a comma-separated string. Fields containing commas, quotes, or newlines are quoted automatically, and nested tuples are stringified with `|` separators so the original structure is preserved. `serde.from_csv(text)` walks each line, parses integers, floats, and booleans when possible, and falls back to strings—returning a tuple of row tuples so ApexLang code can inspect or reshape the grid.

```apex
import serde;
import structs;

fn apex() {
  let rows = ((1, "alpha"), (true, 3.5));
  let csv = serde.to_csv(rows);
  let restored = serde.from_csv(csv);
  // append an extra cell without touching the original rows
  return structs.copy_append(restored, "extra");
}
```

## Byte-oriented formats

`serde.to_bytes(value)` encodes the JSON representation as a tuple of byte integers (0–255) so ApexLang code can ship complex str
uctures through the `mem` module, async mailboxes, or files without reaching for host-side libraries. `serde.from_bytes(tuple)` e
xpects that tuple, rebuilds the JSON string, and returns the original value.

```apex
import serde;
import mem;

fn apex() {
  let tuple = serde.from_json("{\"name\":\"apex\",\"value\":42}");
  let bytes = serde.to_bytes(tuple);
  mem.write_block(mem.alloc_bytes(16), bytes);
  let restored = serde.from_bytes(bytes);
  return restored;
}
```

## Working with `structs`

The companion `structs` module adds value-level copy helpers:

- `structs.copy(value)` → returns a clone of any value.
- `structs.deep_clone(value)` → recursively duplicates nested tuples.
- `structs.copy_append(tuple, value...)` → produce a new tuple with extra elements appended.
- `structs.clone_tuple(tuple)` → explicitly clone tuple elements.
- `structs.copy_replace(tuple, index, value)` → produce a new tuple with a single slot updated (original untouched).
- `structs.tuple_concat(left, right)` → concatenate tuples without mutating inputs.

Combined with `serde`, it is easy to build persistent record-like structures, serialize them to multiple formats, and verify roun
d-trips entirely inside ApexLang source files.

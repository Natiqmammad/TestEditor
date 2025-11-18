use std::collections::HashMap;
use std::sync::Mutex;

use num_bigint::BigInt;
use num_traits::{Signed, ToPrimitive, Zero};
use once_cell::sync::Lazy;

use crate::apexlang::ast::Value;
use crate::apexlang::error::ApexError;

use super::support::{
    expect_int_arg, expect_tuple_arg, expect_u32_arg, expect_u64_arg, expect_usize_arg,
};
use super::{NativeCallable, NativeRegistry};

pub(super) fn register(registry: &mut NativeRegistry) {
    let mut functions: HashMap<String, NativeCallable> = HashMap::new();

    macro_rules! add {
        ($map:expr, $name:literal, $func:ident) => {
            $map.insert(
                $name.to_string(),
                NativeCallable::new(concat!("mem::", $name), $func),
            );
        };
    }

    add!(functions, "alloc_bytes", alloc_bytes);
    add!(functions, "free_bytes", free_bytes);
    add!(functions, "buffer_len", buffer_len);
    add!(functions, "pointer_offset", pointer_offset);
    add!(functions, "write_byte", write_byte);
    add!(functions, "read_byte", read_byte);
    add!(functions, "memset", memset);
    add!(functions, "memcpy", memcpy);
    add!(functions, "read_block", read_block);
    add!(functions, "write_block", write_block);
    add!(functions, "compare", compare_regions);
    add!(functions, "find_byte", find_byte);
    add!(functions, "checksum", checksum_region);
    add!(functions, "swap_ranges", swap_ranges);
    add!(functions, "reverse_block", reverse_block);
    add!(functions, "count_byte", count_byte);
    add!(functions, "fill_pattern", fill_pattern);
    add!(functions, "binary_and", binary_and);
    add!(functions, "binary_or", binary_or);
    add!(functions, "binary_xor", binary_xor);
    add!(functions, "binary_not", binary_not);
    add!(functions, "binary_shift_left", binary_shift_left);
    add!(functions, "binary_shift_right", binary_shift_right);
    add!(functions, "binary_rotate_left", binary_rotate_left);
    add!(functions, "binary_rotate_right", binary_rotate_right);
    add!(functions, "bit_test", bit_test);
    add!(functions, "bit_set", bit_set);
    add!(functions, "bit_clear", bit_clear);
    add!(functions, "bit_toggle", bit_toggle);
    add!(functions, "bit_count", bit_count);
    add!(functions, "smart_pointer_new", smart_pointer_new);
    add!(functions, "smart_pointer_get", smart_pointer_get);
    add!(functions, "smart_pointer_set", smart_pointer_set);
    add!(functions, "tuple_get", tuple_get);

    registry.register_module("mem", functions);
}

static BUFFER_HEAP: Lazy<Mutex<BufferHeap>> = Lazy::new(|| Mutex::new(BufferHeap::default()));
static SMART_POINTERS: Lazy<Mutex<SmartPointers>> =
    Lazy::new(|| Mutex::new(SmartPointers::default()));

#[derive(Default)]
struct BufferHeap {
    next_handle: u64,
    buffers: HashMap<u64, Vec<u8>>,
}

#[derive(Default)]
struct SmartPointers {
    next_handle: u64,
    values: HashMap<u64, Value>,
}

fn alloc_bytes(args: &[Value]) -> Result<Value, ApexError> {
    let size = expect_usize_arg(args, 0, "alloc_bytes")?;
    let mut heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let handle = heap.next_handle;
    heap.next_handle += 1;
    heap.buffers.insert(handle, vec![0; size]);
    Ok(pointer_value(handle, 0))
}

fn free_bytes(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, _) = expect_pointer(args, 0, "free_bytes")?;
    let mut heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    Ok(Value::Bool(heap.buffers.remove(&handle).is_some()))
}

fn buffer_len(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, _) = expect_pointer(args, 0, "buffer_len")?;
    let heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let len = heap
        .buffers
        .get(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?
        .len();
    Ok(Value::Int(BigInt::from(len)))
}

fn pointer_offset(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "pointer_offset")?;
    let delta = expect_usize_arg(args, 1, "pointer_offset")?;
    let heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let len = heap
        .buffers
        .get(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?
        .len();
    let new_offset = offset + delta;
    if new_offset > len {
        return Err(ApexError::new("Pointer offset exceeds buffer length"));
    }
    Ok(pointer_value(handle, new_offset))
}

fn write_byte(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "write_byte")?;
    let value = expect_u32_arg(args, 1, "write_byte")?;
    if value > 255 {
        return Err(ApexError::new(
            "write_byte expects a value between 0 and 255",
        ));
    }
    let mut heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get_mut(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset >= buffer.len() {
        return Err(ApexError::new("Pointer offset exceeds buffer length"));
    }
    buffer[offset] = value as u8;
    Ok(Value::Bool(true))
}

fn read_byte(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "read_byte")?;
    let heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset >= buffer.len() {
        return Err(ApexError::new("Pointer offset exceeds buffer length"));
    }
    Ok(Value::Int(BigInt::from(buffer[offset])))
}

fn memset(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "memset")?;
    let value = expect_u32_arg(args, 1, "memset")?;
    if value > 255 {
        return Err(ApexError::new("memset expects a value between 0 and 255"));
    }
    let len = expect_usize_arg(args, 2, "memset")?;
    let mut heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get_mut(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + len > buffer.len() {
        return Err(ApexError::new("memset exceeds buffer length"));
    }
    for byte in &mut buffer[offset..offset + len] {
        *byte = value as u8;
    }
    Ok(Value::Bool(true))
}

fn memcpy(args: &[Value]) -> Result<Value, ApexError> {
    let (dst_handle, dst_offset) = expect_pointer(args, 0, "memcpy")?;
    let (src_handle, src_offset) = expect_pointer(args, 1, "memcpy")?;
    let len = expect_usize_arg(args, 2, "memcpy")?;
    let mut heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let temp = {
        let src = heap
            .buffers
            .get(&src_handle)
            .ok_or_else(|| ApexError::new("Unknown source buffer handle"))?;
        if src_offset + len > src.len() {
            return Err(ApexError::new("memcpy exceeds source buffer length"));
        }
        src[src_offset..src_offset + len].to_vec()
    };
    let dst = heap
        .buffers
        .get_mut(&dst_handle)
        .ok_or_else(|| ApexError::new("Unknown destination buffer handle"))?;
    if dst_offset + len > dst.len() {
        return Err(ApexError::new("memcpy exceeds destination buffer length"));
    }
    dst[dst_offset..dst_offset + len].copy_from_slice(&temp);
    Ok(Value::Bool(true))
}

fn read_block(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.read_block")?;
    let len = expect_usize_arg(args, 1, "mem.read_block")?;
    let heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + len > buffer.len() {
        return Err(ApexError::new("mem.read_block exceeds buffer length"));
    }
    let values = buffer[offset..offset + len]
        .iter()
        .map(|byte| Value::Int(BigInt::from(*byte)))
        .collect();
    Ok(Value::Tuple(values))
}

fn write_block(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.write_block")?;
    let tuple = expect_tuple_arg(args, 1, "mem.write_block")?;
    let mut heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get_mut(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + tuple.len() > buffer.len() {
        return Err(ApexError::new("mem.write_block exceeds buffer length"));
    }
    for (index, value) in tuple.iter().enumerate() {
        let byte = match value {
            Value::Int(v) => v
                .to_u32()
                .ok_or_else(|| ApexError::new("mem.write_block expects byte values"))?,
            _ => {
                return Err(ApexError::new(
                    "mem.write_block expects tuple of integer byte values",
                ))
            }
        };
        if byte > 255 {
            return Err(ApexError::new(
                "mem.write_block expects byte values in the 0-255 range",
            ));
        }
        buffer[offset + index] = byte as u8;
    }
    Ok(Value::Bool(true))
}

fn compare_regions(args: &[Value]) -> Result<Value, ApexError> {
    let (left_handle, left_offset) = expect_pointer(args, 0, "mem.compare")?;
    let (right_handle, right_offset) = expect_pointer(args, 1, "mem.compare")?;
    let len = expect_usize_arg(args, 2, "mem.compare")?;
    let heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let left = heap
        .buffers
        .get(&left_handle)
        .ok_or_else(|| ApexError::new("Unknown left buffer handle"))?;
    let right = heap
        .buffers
        .get(&right_handle)
        .ok_or_else(|| ApexError::new("Unknown right buffer handle"))?;
    if left_offset + len > left.len() || right_offset + len > right.len() {
        return Err(ApexError::new("mem.compare exceeds buffer bounds"));
    }
    for idx in 0..len {
        let l = left[left_offset + idx];
        let r = right[right_offset + idx];
        if l != r {
            let ordering = if l < r { -1 } else { 1 };
            return Ok(Value::Int(BigInt::from(ordering)));
        }
    }
    Ok(Value::Int(BigInt::from(0)))
}

fn find_byte(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.find_byte")?;
    let target = expect_u32_arg(args, 1, "mem.find_byte")?;
    if target > 255 {
        return Err(ApexError::new("mem.find_byte expects a byte value"));
    }
    let heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset >= buffer.len() {
        return Err(ApexError::new("mem.find_byte offset exceeds buffer length"));
    }
    for (index, byte) in buffer[offset..].iter().enumerate() {
        if *byte == target as u8 {
            return Ok(Value::Int(BigInt::from(offset + index)));
        }
    }
    Ok(Value::Int(BigInt::from(-1)))
}

fn checksum_region(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.checksum")?;
    let len = expect_usize_arg(args, 1, "mem.checksum")?;
    let heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + len > buffer.len() {
        return Err(ApexError::new("mem.checksum exceeds buffer length"));
    }
    let sum: u64 = buffer[offset..offset + len]
        .iter()
        .map(|byte| *byte as u64)
        .sum();
    Ok(Value::Int(BigInt::from(sum)))
}

fn swap_ranges(args: &[Value]) -> Result<Value, ApexError> {
    let (left_handle, left_offset) = expect_pointer(args, 0, "mem.swap_ranges")?;
    let (right_handle, right_offset) = expect_pointer(args, 1, "mem.swap_ranges")?;
    let len = expect_usize_arg(args, 2, "mem.swap_ranges")?;
    if len == 0 {
        return Ok(Value::Bool(true));
    }
    let mut heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    if left_handle == right_handle {
        let buffer = heap
            .buffers
            .get_mut(&left_handle)
            .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
        if left_offset + len > buffer.len() || right_offset + len > buffer.len() {
            return Err(ApexError::new("swap_ranges exceeds buffer length"));
        }
        for idx in 0..len {
            buffer.swap(left_offset + idx, right_offset + idx);
        }
        return Ok(Value::Bool(true));
    }
    let left_bytes = {
        let buffer = heap
            .buffers
            .get(&left_handle)
            .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
        if left_offset + len > buffer.len() {
            return Err(ApexError::new("swap_ranges exceeds left buffer length"));
        }
        buffer[left_offset..left_offset + len].to_vec()
    };
    let right_bytes = {
        let buffer = heap
            .buffers
            .get(&right_handle)
            .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
        if right_offset + len > buffer.len() {
            return Err(ApexError::new("swap_ranges exceeds right buffer length"));
        }
        buffer[right_offset..right_offset + len].to_vec()
    };
    {
        let buffer = heap
            .buffers
            .get_mut(&left_handle)
            .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
        buffer[left_offset..left_offset + len].copy_from_slice(&right_bytes);
    }
    {
        let buffer = heap
            .buffers
            .get_mut(&right_handle)
            .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
        buffer[right_offset..right_offset + len].copy_from_slice(&left_bytes);
    }
    Ok(Value::Bool(true))
}

fn reverse_block(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.reverse_block")?;
    let len = expect_usize_arg(args, 1, "mem.reverse_block")?;
    let mut heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get_mut(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + len > buffer.len() {
        return Err(ApexError::new("reverse_block exceeds buffer length"));
    }
    buffer[offset..offset + len].reverse();
    Ok(Value::Bool(true))
}

fn count_byte(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.count_byte")?;
    let len = expect_usize_arg(args, 1, "mem.count_byte")?;
    let byte = expect_u32_arg(args, 2, "mem.count_byte")?;
    if byte > 255 {
        return Err(ApexError::new("mem.count_byte expects byte in [0, 255]"));
    }
    let heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + len > buffer.len() {
        return Err(ApexError::new("mem.count_byte exceeds buffer length"));
    }
    let count = buffer[offset..offset + len]
        .iter()
        .filter(|entry| **entry == byte as u8)
        .count();
    Ok(Value::Int(BigInt::from(count)))
}

fn fill_pattern(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.fill_pattern")?;
    let tuple = expect_tuple_arg(args, 1, "mem.fill_pattern")?;
    if tuple.is_empty() {
        return Err(ApexError::new(
            "mem.fill_pattern expects non-empty pattern tuple",
        ));
    }
    let repeat = expect_usize_arg(args, 2, "mem.fill_pattern")?;
    let mut pattern = Vec::with_capacity(tuple.len());
    for value in tuple {
        match value {
            Value::Int(num) => {
                let byte = num.to_u32().ok_or_else(|| {
                    ApexError::new("mem.fill_pattern expects byte values in [0, 255]")
                })?;
                if byte > 255 {
                    return Err(ApexError::new(
                        "mem.fill_pattern expects byte values in [0, 255]",
                    ));
                }
                pattern.push(byte as u8);
            }
            _ => {
                return Err(ApexError::new(
                    "mem.fill_pattern expects tuple of integer byte values",
                ))
            }
        }
    }
    let mut heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get_mut(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    let total = pattern.len() * repeat;
    if offset + total > buffer.len() {
        return Err(ApexError::new("mem.fill_pattern exceeds buffer length"));
    }
    for idx in 0..total {
        buffer[offset + idx] = pattern[idx % pattern.len()];
    }
    Ok(Value::Bool(true))
}

fn binary_and(args: &[Value]) -> Result<Value, ApexError> {
    let left = expect_int_arg(args, 0, "binary_and")?;
    let right = expect_int_arg(args, 1, "binary_and")?;
    Ok(Value::Int(left & right))
}

fn binary_or(args: &[Value]) -> Result<Value, ApexError> {
    let left = expect_int_arg(args, 0, "binary_or")?;
    let right = expect_int_arg(args, 1, "binary_or")?;
    Ok(Value::Int(left | right))
}

fn binary_xor(args: &[Value]) -> Result<Value, ApexError> {
    let left = expect_int_arg(args, 0, "binary_xor")?;
    let right = expect_int_arg(args, 1, "binary_xor")?;
    Ok(Value::Int(left ^ right))
}

fn binary_not(args: &[Value]) -> Result<Value, ApexError> {
    let value = expect_int_arg(args, 0, "binary_not")?;
    Ok(Value::Int(!value))
}

fn binary_shift_left(args: &[Value]) -> Result<Value, ApexError> {
    let value = expect_int_arg(args, 0, "binary_shift_left")?;
    let amount = expect_u64_arg(args, 1, "binary_shift_left")?;
    Ok(Value::Int(value << amount))
}

fn binary_shift_right(args: &[Value]) -> Result<Value, ApexError> {
    let value = expect_int_arg(args, 0, "binary_shift_right")?;
    let amount = expect_u64_arg(args, 1, "binary_shift_right")?;
    Ok(Value::Int(value >> amount))
}

fn binary_rotate_left(args: &[Value]) -> Result<Value, ApexError> {
    let value = expect_int_arg(args, 0, "binary_rotate_left")?;
    let bits = expect_u64_arg(args, 1, "binary_rotate_left")?;
    rotate_bits(value, bits as usize, true)
}

fn binary_rotate_right(args: &[Value]) -> Result<Value, ApexError> {
    let value = expect_int_arg(args, 0, "binary_rotate_right")?;
    let bits = expect_u64_arg(args, 1, "binary_rotate_right")?;
    rotate_bits(value, bits as usize, false)
}

fn bit_test(args: &[Value]) -> Result<Value, ApexError> {
    let value = expect_int_arg(args, 0, "bit_test")?;
    let position = expect_u64_arg(args, 1, "bit_test")?;
    let mask = BigInt::from(1u8) << position;
    Ok(Value::Bool((value & mask) != BigInt::zero()))
}

fn bit_set(args: &[Value]) -> Result<Value, ApexError> {
    let value = expect_int_arg(args, 0, "bit_set")?;
    let position = expect_u64_arg(args, 1, "bit_set")?;
    let mask = BigInt::from(1u8) << position;
    Ok(Value::Int(value | mask))
}

fn bit_clear(args: &[Value]) -> Result<Value, ApexError> {
    let value = expect_int_arg(args, 0, "bit_clear")?;
    let position = expect_u64_arg(args, 1, "bit_clear")?;
    let mask = BigInt::from(1u8) << position;
    Ok(Value::Int(value & !mask))
}

fn bit_toggle(args: &[Value]) -> Result<Value, ApexError> {
    let value = expect_int_arg(args, 0, "bit_toggle")?;
    let position = expect_u64_arg(args, 1, "bit_toggle")?;
    let mask = BigInt::from(1u8) << position;
    Ok(Value::Int(value ^ mask))
}

fn bit_count(args: &[Value]) -> Result<Value, ApexError> {
    let value = expect_int_arg(args, 0, "bit_count")?;
    let mut n = value.abs();
    let mut count = BigInt::zero();
    let one = BigInt::from(1u8);
    while n > BigInt::zero() {
        if (&n & &one) == one {
            count += &one;
        }
        n >>= 1;
    }
    Ok(Value::Int(count))
}

fn smart_pointer_new(args: &[Value]) -> Result<Value, ApexError> {
    let value = args
        .get(0)
        .cloned()
        .ok_or_else(|| ApexError::new("smart_pointer_new expects a value"))?;
    let mut table = SMART_POINTERS
        .lock()
        .map_err(|_| ApexError::new("Smart pointer table lock poisoned"))?;
    let handle = table.next_handle;
    table.next_handle += 1;
    table.values.insert(handle, value);
    Ok(Value::Tuple(vec![Value::Int(BigInt::from(handle))]))
}

fn smart_pointer_get(args: &[Value]) -> Result<Value, ApexError> {
    let handle = expect_smart_handle(args, 0, "smart_pointer_get")?;
    let table = SMART_POINTERS
        .lock()
        .map_err(|_| ApexError::new("Smart pointer table lock poisoned"))?;
    table
        .values
        .get(&handle)
        .cloned()
        .ok_or_else(|| ApexError::new("Unknown smart pointer handle"))
}

fn smart_pointer_set(args: &[Value]) -> Result<Value, ApexError> {
    let handle = expect_smart_handle(args, 0, "smart_pointer_set")?;
    let value = args
        .get(1)
        .cloned()
        .ok_or_else(|| ApexError::new("smart_pointer_set expects a value"))?;
    let mut table = SMART_POINTERS
        .lock()
        .map_err(|_| ApexError::new("Smart pointer table lock poisoned"))?;
    if let Some(entry) = table.values.get_mut(&handle) {
        *entry = value;
        Ok(Value::Bool(true))
    } else {
        Err(ApexError::new("Unknown smart pointer handle"))
    }
}

fn tuple_get(args: &[Value]) -> Result<Value, ApexError> {
    let tuple = expect_tuple_arg(args, 0, "mem.tuple_get")?;
    let index = expect_usize_arg(args, 1, "mem.tuple_get")?;
    tuple
        .get(index)
        .cloned()
        .ok_or_else(|| ApexError::new("tuple index out of bounds"))
}

fn pointer_value(handle: u64, offset: usize) -> Value {
    Value::Tuple(vec![
        Value::Int(BigInt::from(handle)),
        Value::Int(BigInt::from(offset)),
    ])
}

fn expect_pointer(args: &[Value], index: usize, name: &str) -> Result<(u64, usize), ApexError> {
    let tuple = expect_tuple_arg(args, index, name)?;
    if tuple.len() != 2 {
        return Err(ApexError::new(format!(
            "{} expects a pointer tuple (handle, offset)",
            name
        )));
    }
    let handle = match &tuple[0] {
        Value::Int(value) => value
            .to_u64()
            .ok_or_else(|| ApexError::new(format!("{} pointer handle is too large", name)))?,
        _ => {
            return Err(ApexError::new(format!(
                "{} expects pointer handle as integer",
                name
            )))
        }
    };
    let offset = match &tuple[1] {
        Value::Int(value) => value
            .to_usize()
            .ok_or_else(|| ApexError::new(format!("{} pointer offset is too large", name)))?,
        _ => {
            return Err(ApexError::new(format!(
                "{} expects pointer offset as integer",
                name
            )))
        }
    };
    Ok((handle, offset))
}

fn expect_smart_handle(args: &[Value], index: usize, name: &str) -> Result<u64, ApexError> {
    let tuple = expect_tuple_arg(args, index, name)?;
    if tuple.len() != 1 {
        return Err(ApexError::new(format!(
            "{} expects a smart pointer tuple (handle)",
            name
        )));
    }
    match &tuple[0] {
        Value::Int(value) => value
            .to_u64()
            .ok_or_else(|| ApexError::new(format!("{} handle is too large", name))),
        _ => Err(ApexError::new(format!(
            "{} expects handle encoded as integer",
            name
        ))),
    }
}

fn rotate_bits(value: BigInt, bits: usize, left: bool) -> Result<Value, ApexError> {
    if bits == 0 {
        return Ok(Value::Int(value));
    }
    if value.is_zero() {
        return Ok(Value::Int(value));
    }
    let mut binary = value.clone();
    let mut is_negative = false;
    if binary.is_negative() {
        // For simplicity treat rotation on the absolute value and reapply the sign.
        binary = binary.abs();
        is_negative = true;
    }
    let mut bit_len = 0usize;
    let mut cursor = binary.clone();
    while cursor > BigInt::zero() {
        cursor >>= 1;
        bit_len += 1;
    }
    if bit_len == 0 {
        return Ok(Value::Int(value));
    }
    let normalized = bits % bit_len;
    let rotated = if normalized == 0 {
        binary
    } else if left {
        let high = (&binary >> (bit_len - normalized)) & ((BigInt::from(1u8) << normalized) - 1);
        (binary << normalized) | high
    } else {
        let low = &binary & ((BigInt::from(1u8) << normalized) - 1);
        (binary >> normalized) | (low << (bit_len - normalized))
    };
    let result = if is_negative { -rotated } else { rotated };
    Ok(Value::Int(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_round_trip() {
        let ptr = alloc_bytes(&[Value::Int(4.into())]).expect("alloc succeeds");
        assert!(write_byte(&[ptr.clone(), Value::Int(300.into())]).is_err());
        let ptr2 = pointer_offset(&[ptr.clone(), Value::Int(1.into())]).expect("offset");
        write_byte(&[ptr2.clone(), Value::Int(42.into())]).expect("write");
        let value = read_byte(&[ptr2.clone()]).expect("read");
        assert_eq!(value, Value::Int(42.into()));
        let len = buffer_len(&[ptr.clone()]).expect("len");
        assert_eq!(len, Value::Int(4.into()));
        assert!(free_bytes(&[ptr]).unwrap() == Value::Bool(true));
    }

    #[test]
    fn smart_pointers_store_values() {
        let handle = smart_pointer_new(&[Value::Int(5.into())]).expect("create");
        let value = smart_pointer_get(&[handle.clone()]).expect("get");
        assert_eq!(value, Value::Int(5.into()));
        smart_pointer_set(&[handle.clone(), Value::Int(9.into())]).expect("set");
        let updated = smart_pointer_get(&[handle]).expect("updated");
        assert_eq!(updated, Value::Int(9.into()));
    }

    #[test]
    fn binary_ops_work() {
        let and = binary_and(&[Value::Int(0b1010.into()), Value::Int(0b1100.into())]).unwrap();
        assert_eq!(and, Value::Int(0b1000.into()));
        let xor = binary_xor(&[Value::Int(0b1111.into()), Value::Int(0b0101.into())]).unwrap();
        assert_eq!(xor, Value::Int(0b1010.into()));
        let shl = binary_shift_left(&[Value::Int(1.into()), Value::Int(3.into())]).unwrap();
        assert_eq!(shl, Value::Int(8.into()));
        let shr = binary_shift_right(&[Value::Int(8.into()), Value::Int(2.into())]).unwrap();
        assert_eq!(shr, Value::Int(2.into()));
        let tuple = Value::Tuple(vec![Value::Int(7.into()), Value::Bool(true)]);
        let head = tuple_get(&[tuple, Value::Int(0.into())]).unwrap();
        assert_eq!(head, Value::Int(7.into()));
    }

    #[test]
    fn range_mutators_cover_pattern_math() {
        let ptr = alloc_bytes(&[Value::Int(16.into())]).expect("alloc");
        fill_pattern(&[
            ptr.clone(),
            Value::Tuple(vec![
                Value::Int(0.into()),
                Value::Int(1.into()),
                Value::Int(2.into()),
            ]),
            Value::Int(4.into()),
        ])
        .expect("fill");
        let ones =
            count_byte(&[ptr.clone(), Value::Int(12.into()), Value::Int(1.into())]).expect("count");
        assert_eq!(ones, Value::Int(4.into()));
        reverse_block(&[ptr.clone(), Value::Int(0.into()), Value::Int(12.into())])
            .expect("reverse");
        let ptr2 = alloc_bytes(&[Value::Int(16.into())]).expect("alloc2");
        memset(&[ptr2.clone(), Value::Int(0.into()), Value::Int(16.into())]).expect("memset");
        swap_ranges(&[ptr.clone(), ptr2.clone(), Value::Int(6.into())]).expect("swap");
        let drained = read_block(&[ptr2.clone(), Value::Int(6.into())]).unwrap();
        if let Value::Tuple(bytes) = drained {
            assert_eq!(bytes.len(), 6);
        } else {
            panic!("expected tuple");
        }
        free_bytes(&[ptr]).unwrap();
        free_bytes(&[ptr2]).unwrap();
    }

    #[test]
    fn block_helpers_cover_ranges() {
        let ptr = alloc_bytes(&[Value::Int(8.into())]).expect("alloc");
        write_block(&[
            ptr.clone(),
            Value::Tuple(vec![
                Value::Int(1.into()),
                Value::Int(2.into()),
                Value::Int(3.into()),
                Value::Int(4.into()),
            ]),
        ])
        .expect("write block");
        let block = read_block(&[ptr.clone(), Value::Int(4.into())]).expect("read");
        if let Value::Tuple(bytes) = block {
            assert_eq!(bytes.len(), 4);
            assert_eq!(bytes[0], Value::Int(1.into()));
            assert_eq!(bytes[3], Value::Int(4.into()));
        } else {
            panic!("expected tuple");
        }
        let other = alloc_bytes(&[Value::Int(8.into())]).expect("alloc 2");
        memcpy(&[other.clone(), ptr.clone(), Value::Int(4.into())]).expect("copy");
        let cmp = compare_regions(&[ptr.clone(), other.clone(), Value::Int(4.into())]).unwrap();
        assert_eq!(cmp, Value::Int(0.into()));
        let idx = find_byte(&[ptr.clone(), Value::Int(3.into())]).unwrap();
        assert_eq!(idx, Value::Int(2.into()));
        let checksum = checksum_region(&[ptr.clone(), Value::Int(4.into())]).unwrap();
        assert_eq!(checksum, Value::Int(10.into()));
        free_bytes(&[ptr]).unwrap();
        free_bytes(&[other]).unwrap();
    }
}

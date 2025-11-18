use std::collections::HashMap;
use std::sync::Mutex;

use num_bigint::BigInt;
use num_traits::{Signed, ToPrimitive, Zero};
use once_cell::sync::Lazy;

use crate::apexlang::ast::Value;
use crate::apexlang::error::ApexError;

use super::support::{
    expect_int_arg, expect_number_arg, expect_tuple_arg, expect_u128_arg, expect_u32_arg,
    expect_u64_arg, expect_usize_arg,
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
    add!(functions, "pointer_diff", pointer_diff);
    add!(functions, "write_byte", write_byte);
    add!(functions, "read_byte", read_byte);
    add!(functions, "memset", memset);
    add!(functions, "memcpy", memcpy);
    add!(functions, "read_block", read_block);
    add!(functions, "write_block", write_block);
    add!(functions, "compare", compare_regions);
    add!(functions, "find_byte", find_byte);
    add!(functions, "find_pattern", find_pattern);
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
    add!(functions, "read_u16_le", read_u16_le);
    add!(functions, "write_u16_le", write_u16_le);
    add!(functions, "read_u16_be", read_u16_be);
    add!(functions, "write_u16_be", write_u16_be);
    add!(functions, "read_u32_le", read_u32_le);
    add!(functions, "write_u32_le", write_u32_le);
    add!(functions, "read_u32_be", read_u32_be);
    add!(functions, "write_u32_be", write_u32_be);
    add!(functions, "read_u64_le", read_u64_le);
    add!(functions, "write_u64_le", write_u64_le);
    add!(functions, "read_u64_be", read_u64_be);
    add!(functions, "write_u64_be", write_u64_be);
    add!(functions, "read_u128_le", read_u128_le);
    add!(functions, "write_u128_le", write_u128_le);
    add!(functions, "read_u128_be", read_u128_be);
    add!(functions, "write_u128_be", write_u128_be);
    add!(functions, "read_f32_le", read_f32_le);
    add!(functions, "write_f32_le", write_f32_le);
    add!(functions, "read_f32_be", read_f32_be);
    add!(functions, "write_f32_be", write_f32_be);
    add!(functions, "read_f64_le", read_f64_le);
    add!(functions, "write_f64_le", write_f64_le);
    add!(functions, "read_f64_be", read_f64_be);
    add!(functions, "write_f64_be", write_f64_be);
    add!(functions, "hexdump", hexdump_region);
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

fn pointer_diff(args: &[Value]) -> Result<Value, ApexError> {
    let (left_handle, left_offset) = expect_pointer(args, 0, "pointer_diff")?;
    let (right_handle, right_offset) = expect_pointer(args, 1, "pointer_diff")?;
    if left_handle != right_handle {
        return Err(ApexError::new(
            "pointer_diff expects pointers derived from the same buffer",
        ));
    }
    let diff = BigInt::from(left_offset as u64) - BigInt::from(right_offset as u64);
    Ok(Value::Int(diff))
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

fn read_u16_le(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.read_u16_le")?;
    let heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + 2 > buffer.len() {
        return Err(ApexError::new("mem.read_u16_le exceeds buffer length"));
    }
    let mut bytes = [0u8; 2];
    bytes.copy_from_slice(&buffer[offset..offset + 2]);
    Ok(Value::Int(BigInt::from(u16::from_le_bytes(bytes))))
}

fn write_u16_le(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.write_u16_le")?;
    let raw = expect_u32_arg(args, 1, "mem.write_u16_le")?;
    if raw > u16::MAX as u32 {
        return Err(ApexError::new(
            "mem.write_u16_le expects a value that fits in 16 bits",
        ));
    }
    let mut heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get_mut(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + 2 > buffer.len() {
        return Err(ApexError::new("mem.write_u16_le exceeds buffer length"));
    }
    buffer[offset..offset + 2].copy_from_slice(&(raw as u16).to_le_bytes());
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

fn find_pattern(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.find_pattern")?;
    let pattern_tuple = expect_tuple_arg(args, 1, "mem.find_pattern")?;
    if pattern_tuple.is_empty() {
        return Err(ApexError::new(
            "mem.find_pattern expects a non-empty pattern tuple",
        ));
    }
    let mut pattern = Vec::with_capacity(pattern_tuple.len());
    for value in pattern_tuple {
        match value {
            Value::Int(v) => {
                let byte = v.to_u32().ok_or_else(|| {
                    ApexError::new("mem.find_pattern expects byte values in [0, 255]")
                })?;
                if byte > 255 {
                    return Err(ApexError::new(
                        "mem.find_pattern expects byte values in [0, 255]",
                    ));
                }
                pattern.push(byte as u8);
            }
            _ => {
                return Err(ApexError::new(
                    "mem.find_pattern expects a tuple of byte integers",
                ))
            }
        }
    }
    let heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + pattern.len() > buffer.len() {
        return Err(ApexError::new(
            "mem.find_pattern search exceeds buffer length",
        ));
    }
    for cursor in offset..=buffer.len() - pattern.len() {
        if buffer[cursor..cursor + pattern.len()] == pattern[..] {
            return Ok(Value::Int(BigInt::from(cursor)));
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

fn read_u16_be(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.read_u16_be")?;
    let heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + 2 > buffer.len() {
        return Err(ApexError::new("mem.read_u16_be exceeds buffer length"));
    }
    let mut bytes = [0u8; 2];
    bytes.copy_from_slice(&buffer[offset..offset + 2]);
    Ok(Value::Int(BigInt::from(u16::from_be_bytes(bytes))))
}

fn write_u16_be(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.write_u16_be")?;
    let raw = expect_u32_arg(args, 1, "mem.write_u16_be")?;
    if raw > u16::MAX as u32 {
        return Err(ApexError::new(
            "mem.write_u16_be expects a value that fits in 16 bits",
        ));
    }
    let mut heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get_mut(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + 2 > buffer.len() {
        return Err(ApexError::new("mem.write_u16_be exceeds buffer length"));
    }
    buffer[offset..offset + 2].copy_from_slice(&u16::to_be_bytes(raw as u16));
    Ok(Value::Bool(true))
}

fn read_u32_le(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.read_u32_le")?;
    let heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + 4 > buffer.len() {
        return Err(ApexError::new("mem.read_u32_le exceeds buffer length"));
    }
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(&buffer[offset..offset + 4]);
    Ok(Value::Int(BigInt::from(u32::from_le_bytes(bytes))))
}

fn write_u32_le(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.write_u32_le")?;
    let raw = expect_u64_arg(args, 1, "mem.write_u32_le")?;
    if raw > u32::MAX as u64 {
        return Err(ApexError::new(
            "mem.write_u32_le expects a value that fits in 32 bits",
        ));
    }
    let mut heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get_mut(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + 4 > buffer.len() {
        return Err(ApexError::new("mem.write_u32_le exceeds buffer length"));
    }
    let bytes = (raw as u32).to_le_bytes();
    buffer[offset..offset + 4].copy_from_slice(&bytes);
    Ok(Value::Bool(true))
}

fn read_u32_be(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.read_u32_be")?;
    let heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + 4 > buffer.len() {
        return Err(ApexError::new("mem.read_u32_be exceeds buffer length"));
    }
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(&buffer[offset..offset + 4]);
    Ok(Value::Int(BigInt::from(u32::from_be_bytes(bytes))))
}

fn write_u32_be(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.write_u32_be")?;
    let raw = expect_u32_arg(args, 1, "mem.write_u32_be")?;
    let mut heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get_mut(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + 4 > buffer.len() {
        return Err(ApexError::new("mem.write_u32_be exceeds buffer length"));
    }
    buffer[offset..offset + 4].copy_from_slice(&u32::to_be_bytes(raw));
    Ok(Value::Bool(true))
}

fn read_u64_le(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.read_u64_le")?;
    let heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + 8 > buffer.len() {
        return Err(ApexError::new("mem.read_u64_le exceeds buffer length"));
    }
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&buffer[offset..offset + 8]);
    Ok(Value::Int(BigInt::from(u64::from_le_bytes(bytes))))
}

fn read_u64_be(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.read_u64_be")?;
    let heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + 8 > buffer.len() {
        return Err(ApexError::new("mem.read_u64_be exceeds buffer length"));
    }
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&buffer[offset..offset + 8]);
    Ok(Value::Int(BigInt::from(u64::from_be_bytes(bytes))))
}

fn write_u64_be(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.write_u64_be")?;
    let raw = expect_u64_arg(args, 1, "mem.write_u64_be")?;
    let mut heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get_mut(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + 8 > buffer.len() {
        return Err(ApexError::new("mem.write_u64_be exceeds buffer length"));
    }
    buffer[offset..offset + 8].copy_from_slice(&u64::to_be_bytes(raw));
    Ok(Value::Bool(true))
}

fn read_u128_le(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.read_u128_le")?;
    let heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + 16 > buffer.len() {
        return Err(ApexError::new("mem.read_u128_le exceeds buffer length"));
    }
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&buffer[offset..offset + 16]);
    Ok(Value::Int(BigInt::from(u128::from_le_bytes(bytes))))
}

fn write_u128_le(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.write_u128_le")?;
    let raw = expect_u128_arg(args, 1, "mem.write_u128_le")?;
    let mut heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get_mut(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + 16 > buffer.len() {
        return Err(ApexError::new("mem.write_u128_le exceeds buffer length"));
    }
    buffer[offset..offset + 16].copy_from_slice(&u128::to_le_bytes(raw));
    Ok(Value::Bool(true))
}

fn read_u128_be(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.read_u128_be")?;
    let heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + 16 > buffer.len() {
        return Err(ApexError::new("mem.read_u128_be exceeds buffer length"));
    }
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&buffer[offset..offset + 16]);
    Ok(Value::Int(BigInt::from(u128::from_be_bytes(bytes))))
}

fn write_u128_be(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.write_u128_be")?;
    let raw = expect_u128_arg(args, 1, "mem.write_u128_be")?;
    let mut heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get_mut(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + 16 > buffer.len() {
        return Err(ApexError::new("mem.write_u128_be exceeds buffer length"));
    }
    buffer[offset..offset + 16].copy_from_slice(&u128::to_be_bytes(raw));
    Ok(Value::Bool(true))
}

fn read_f32_le(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.read_f32_le")?;
    let bytes = read_buffer_window(handle, offset, 4, "mem.read_f32_le")?;
    let mut block = [0u8; 4];
    block.copy_from_slice(&bytes);
    Ok(Value::Number(f32::from_le_bytes(block) as f64))
}

fn read_f32_be(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.read_f32_be")?;
    let bytes = read_buffer_window(handle, offset, 4, "mem.read_f32_be")?;
    let mut block = [0u8; 4];
    block.copy_from_slice(&bytes);
    Ok(Value::Number(f32::from_be_bytes(block) as f64))
}

fn write_f32_le(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.write_f32_le")?;
    let value = expect_number_arg(args, 1, "mem.write_f32_le")? as f32;
    write_buffer_window(handle, offset, &value.to_le_bytes(), "mem.write_f32_le")?;
    Ok(Value::Bool(true))
}

fn write_f32_be(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.write_f32_be")?;
    let value = expect_number_arg(args, 1, "mem.write_f32_be")? as f32;
    write_buffer_window(handle, offset, &value.to_be_bytes(), "mem.write_f32_be")?;
    Ok(Value::Bool(true))
}

fn read_f64_le(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.read_f64_le")?;
    let bytes = read_buffer_window(handle, offset, 8, "mem.read_f64_le")?;
    let mut block = [0u8; 8];
    block.copy_from_slice(&bytes);
    Ok(Value::Number(f64::from_le_bytes(block)))
}

fn read_f64_be(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.read_f64_be")?;
    let bytes = read_buffer_window(handle, offset, 8, "mem.read_f64_be")?;
    let mut block = [0u8; 8];
    block.copy_from_slice(&bytes);
    Ok(Value::Number(f64::from_be_bytes(block)))
}

fn write_f64_le(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.write_f64_le")?;
    let value = expect_number_arg(args, 1, "mem.write_f64_le")?;
    write_buffer_window(handle, offset, &value.to_le_bytes(), "mem.write_f64_le")?;
    Ok(Value::Bool(true))
}

fn write_f64_be(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.write_f64_be")?;
    let value = expect_number_arg(args, 1, "mem.write_f64_be")?;
    write_buffer_window(handle, offset, &value.to_be_bytes(), "mem.write_f64_be")?;
    Ok(Value::Bool(true))
}

fn write_u64_le(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.write_u64_le")?;
    let raw = expect_u64_arg(args, 1, "mem.write_u64_le")?;
    let mut heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get_mut(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + 8 > buffer.len() {
        return Err(ApexError::new("mem.write_u64_le exceeds buffer length"));
    }
    buffer[offset..offset + 8].copy_from_slice(&raw.to_le_bytes());
    Ok(Value::Bool(true))
}

fn hexdump_region(args: &[Value]) -> Result<Value, ApexError> {
    let (handle, offset) = expect_pointer(args, 0, "mem.hexdump")?;
    let len = expect_usize_arg(args, 1, "mem.hexdump")?;
    let heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + len > buffer.len() {
        return Err(ApexError::new("mem.hexdump exceeds buffer length"));
    }
    let mut chunks = Vec::with_capacity(len);
    for byte in &buffer[offset..offset + len] {
        chunks.push(format!("{:02x}", byte));
    }
    Ok(Value::String(chunks.join(" ")))
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

fn read_buffer_window(
    handle: u64,
    offset: usize,
    width: usize,
    name: &str,
) -> Result<Vec<u8>, ApexError> {
    let heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + width > buffer.len() {
        return Err(ApexError::new(format!("{} exceeds buffer length", name)));
    }
    Ok(buffer[offset..offset + width].to_vec())
}

fn write_buffer_window(
    handle: u64,
    offset: usize,
    bytes: &[u8],
    name: &str,
) -> Result<(), ApexError> {
    let mut heap = BUFFER_HEAP
        .lock()
        .map_err(|_| ApexError::new("Memory heap lock poisoned"))?;
    let buffer = heap
        .buffers
        .get_mut(&handle)
        .ok_or_else(|| ApexError::new("Unknown buffer handle"))?;
    if offset + bytes.len() > buffer.len() {
        return Err(ApexError::new(format!("{} exceeds buffer length", name)));
    }
    buffer[offset..offset + bytes.len()].copy_from_slice(bytes);
    Ok(())
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

    #[test]
    fn u32_helpers_and_hexdump_cover_bytes() {
        let ptr = alloc_bytes(&[Value::Int(32.into())]).expect("alloc");
        write_u32_le(&[ptr.clone(), Value::Int(0x11223344u64.into())]).expect("write");
        let read_back = read_u32_le(&[ptr.clone()]).expect("read");
        assert_eq!(read_back, Value::Int(0x11223344u32.into()));
        write_u16_le(&[ptr.clone(), Value::Int(0xbeefu32.into())]).expect("write16");
        let read16 = read_u16_le(&[ptr.clone()]).expect("read16");
        assert_eq!(read16, Value::Int(0xbeefu32.into()));
        write_u16_be(&[ptr.clone(), Value::Int(0x1234u32.into())]).expect("write16be");
        let read16_be = read_u16_be(&[ptr.clone()]).expect("read16be");
        assert_eq!(read16_be, Value::Int(0x1234u32.into()));
        write_u64_le(&[ptr.clone(), Value::Int(0x0102030405060708u64.into())]).expect("write64");
        let read64 = read_u64_le(&[ptr.clone()]).expect("read64");
        assert_eq!(read64, Value::Int(0x0102030405060708u64.into()));
        write_u64_be(&[ptr.clone(), Value::Int(0x0a0b0c0d0e0f1011u64.into())]).expect("write64be");
        let read64_be = read_u64_be(&[ptr.clone()]).expect("read64be");
        assert_eq!(read64_be, Value::Int(0x0a0b0c0d0e0f1011u64.into()));
        write_u32_be(&[ptr.clone(), Value::Int(0xcafebabeu64.into())]).expect("write32be");
        let read32_be = read_u32_be(&[ptr.clone()]).expect("read32be");
        assert_eq!(read32_be, Value::Int(0xcafebabeu64.into()));
        write_u128_le(&[
            ptr.clone(),
            Value::Int(0x0102030405060708090a0b0c0d0e0f10u128.into()),
        ])
        .expect("write128le");
        let read128_le = read_u128_le(&[ptr.clone()]).expect("read128le");
        assert_eq!(
            read128_le,
            Value::Int(0x0102030405060708090a0b0c0d0e0f10u128.into())
        );
        write_u128_be(&[
            ptr.clone(),
            Value::Int(0x1112131415161718191a1b1c1d1e1f20u128.into()),
        ])
        .expect("write128be");
        let read128_be = read_u128_be(&[ptr.clone()]).expect("read128be");
        assert_eq!(
            read128_be,
            Value::Int(0x1112131415161718191a1b1c1d1e1f20u128.into())
        );
        write_block(&[
            pointer_offset(&[ptr.clone(), Value::Int(4.into())]).unwrap(),
            Value::Tuple(vec![
                Value::Int(0.into()),
                Value::Int(1.into()),
                Value::Int(2.into()),
                Value::Int(3.into()),
            ]),
        ])
        .expect("write block");
        let pattern_idx = find_pattern(&[
            ptr.clone(),
            Value::Tuple(vec![
                Value::Int(1.into()),
                Value::Int(2.into()),
                Value::Int(3.into()),
            ]),
        ])
        .expect("pattern");
        assert_eq!(pattern_idx, Value::Int(5.into()));
        let hexdump = hexdump_region(&[ptr.clone(), Value::Int(8.into())]).unwrap();
        if let Value::String(text) = hexdump {
            assert!(text.contains("00 01 02"));
        } else {
            panic!("expected hexdump string");
        }
        free_bytes(&[ptr]).unwrap();
    }

    #[test]
    fn pointer_diff_tracks_offsets() {
        let ptr = alloc_bytes(&[Value::Int(8.into())]).expect("alloc");
        let ptr_offset = pointer_offset(&[ptr.clone(), Value::Int(3.into())]).unwrap();
        let diff = pointer_diff(&[ptr_offset.clone(), ptr.clone()]).unwrap();
        assert_eq!(diff, Value::Int(3.into()));
        let reverse = pointer_diff(&[ptr.clone(), ptr_offset.clone()]).unwrap();
        assert_eq!(reverse, Value::Int((-3).into()));
        free_bytes(&[ptr]).unwrap();
    }

    #[test]
    fn float_helpers_cover_endianness() {
        let ptr = alloc_bytes(&[Value::Int(32.into())]).expect("alloc");
        write_f32_le(&[ptr.clone(), Value::Number(3.25)]).expect("write f32 le");
        let read_back = read_f32_le(&[ptr.clone()]).expect("read f32 le");
        assert!(matches!(read_back, Value::Number(value) if (value - 3.25).abs() < 1e-6));
        write_f32_be(&[ptr.clone(), Value::Number(-1.5)]).expect("write f32 be");
        let read_be = read_f32_be(&[ptr.clone()]).expect("read f32 be");
        assert!(matches!(read_be, Value::Number(value) if (value + 1.5).abs() < 1e-6));
        write_f64_le(&[ptr.clone(), Value::Number(42.125)]).expect("write f64 le");
        let read64 = read_f64_le(&[ptr.clone()]).expect("read f64 le");
        assert!(matches!(read64, Value::Number(value) if (value - 42.125).abs() < 1e-10));
        write_f64_be(&[ptr.clone(), Value::Number(-9.75)]).expect("write f64 be");
        let read64_be = read_f64_be(&[ptr.clone()]).expect("read f64 be");
        assert!(matches!(read64_be, Value::Number(value) if (value + 9.75).abs() < 1e-10));
        free_bytes(&[ptr]).unwrap();
    }
}

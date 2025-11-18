use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use num_bigint::BigInt;
use num_traits::ToPrimitive;
use uuid::Uuid;

use crate::apexlang::ast::Value;
use crate::apexlang::error::ApexError;

use super::support::{expect_string_arg, expect_tuple_arg};
use super::{NativeCallable, NativeRegistry};

pub(super) fn register(registry: &mut NativeRegistry) {
    let mut functions: HashMap<String, NativeCallable> = HashMap::new();

    macro_rules! add {
        ($map:expr, $name:literal, $func:ident) => {
            $map.insert(
                $name.to_string(),
                NativeCallable::new(concat!("fs::", $name), $func),
            );
        };
    }

    add!(&mut functions, "read_text", read_text);
    add!(&mut functions, "write_text", write_text);
    add!(&mut functions, "append_text", append_text);
    add!(&mut functions, "file_exists", file_exists);
    add!(&mut functions, "read_bytes", read_bytes);
    add!(&mut functions, "write_bytes", write_bytes);
    add!(&mut functions, "delete", delete_path);
    add!(&mut functions, "list_dir", list_dir);
    add!(&mut functions, "copy", copy_path);
    add!(&mut functions, "rename", rename_path);
    add!(&mut functions, "mkdir_all", mkdir_all);
    add!(&mut functions, "metadata", metadata);
    add!(&mut functions, "touch", touch_path);
    add!(&mut functions, "tempfile", create_tempfile);
    add!(&mut functions, "read_tree", read_tree);
    registry.register_module("fs", functions);
}

fn read_text(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "fs.read_text")?;
    let contents = fs::read_to_string(&path)
        .map_err(|err| ApexError::new(format!("Failed to read '{}': {}", path, err)))?;
    Ok(Value::String(contents))
}

fn write_text(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "fs.write_text")?;
    let contents = expect_string_arg(args, 1, "fs.write_text")?;
    fs::write(&path, contents)
        .map_err(|err| ApexError::new(format!("Failed to write '{}': {}", path, err)))?;
    Ok(Value::Bool(true))
}

fn append_text(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "fs.append_text")?;
    let contents = expect_string_arg(args, 1, "fs.append_text")?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|err| ApexError::new(format!("Failed to open '{}': {}", path, err)))?;
    file.write_all(contents.as_bytes())
        .map_err(|err| ApexError::new(format!("Failed to append to '{}': {}", path, err)))?;
    Ok(Value::Bool(true))
}

fn file_exists(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "fs.file_exists")?;
    Ok(Value::Bool(Path::new(&path).exists()))
}

fn read_bytes(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "fs.read_bytes")?;
    let bytes = fs::read(&path)
        .map_err(|err| ApexError::new(format!("Failed to read '{}': {}", path, err)))?;
    Ok(Value::Tuple(
        bytes
            .into_iter()
            .map(|byte| Value::Int(BigInt::from(byte)))
            .collect(),
    ))
}

fn write_bytes(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "fs.write_bytes")?;
    let tuple = expect_tuple_arg(args, 1, "fs.write_bytes")?;
    let mut bytes = Vec::with_capacity(tuple.len());
    for value in tuple {
        match value {
            Value::Int(v) => {
                let byte = v.to_u32().ok_or_else(|| {
                    ApexError::new("fs.write_bytes expects byte values in [0, 255]")
                })?;
                if byte > 255 {
                    return Err(ApexError::new(
                        "fs.write_bytes expects byte values in [0, 255]",
                    ));
                }
                bytes.push(byte as u8);
            }
            _ => return Err(ApexError::new("fs.write_bytes expects tuple of integers")),
        }
    }
    fs::write(&path, bytes)
        .map_err(|err| ApexError::new(format!("Failed to write '{}': {}", path, err)))?;
    Ok(Value::Bool(true))
}

fn delete_path(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "fs.delete")?;
    if Path::new(&path).is_dir() {
        fs::remove_dir_all(&path).map_err(|err| {
            ApexError::new(format!("Failed to delete directory '{}': {}", path, err))
        })?;
    } else if Path::new(&path).exists() {
        fs::remove_file(&path)
            .map_err(|err| ApexError::new(format!("Failed to delete file '{}': {}", path, err)))?;
    } else {
        return Ok(Value::Bool(false));
    }
    Ok(Value::Bool(true))
}

fn list_dir(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "fs.list_dir")?;
    let entries = fs::read_dir(&path)
        .map_err(|err| ApexError::new(format!("Failed to list '{}': {}", path, err)))?;
    let mut names = Vec::new();
    for entry in entries {
        let entry =
            entry.map_err(|err| ApexError::new(format!("Failed to read entry: {}", err)))?;
        if let Some(name) = entry.file_name().to_str() {
            names.push(Value::String(name.to_string()));
        }
    }
    Ok(Value::Tuple(names))
}

fn copy_path(args: &[Value]) -> Result<Value, ApexError> {
    let source = expect_string_arg(args, 0, "fs.copy")?;
    let dest = expect_string_arg(args, 1, "fs.copy")?;
    let copied = fs::copy(&source, &dest)
        .map_err(|err| ApexError::new(format!("Failed to copy '{}': {}", source, err)))?;
    Ok(Value::Int(BigInt::from(copied)))
}

fn rename_path(args: &[Value]) -> Result<Value, ApexError> {
    let source = expect_string_arg(args, 0, "fs.rename")?;
    let dest = expect_string_arg(args, 1, "fs.rename")?;
    fs::rename(&source, &dest)
        .map_err(|err| ApexError::new(format!("Failed to rename '{}': {}", source, err)))?;
    Ok(Value::Bool(true))
}

fn mkdir_all(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "fs.mkdir_all")?;
    fs::create_dir_all(&path)
        .map_err(|err| ApexError::new(format!("Failed to create '{}': {}", path, err)))?;
    Ok(Value::Bool(true))
}

fn metadata(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "fs.metadata")?;
    let meta = fs::metadata(&path)
        .map_err(|err| ApexError::new(format!("Failed to stat '{}': {}", path, err)))?;
    let size = Value::Int(BigInt::from(meta.len()));
    let is_file = Value::Bool(meta.is_file());
    let is_dir = Value::Bool(meta.is_dir());
    Ok(Value::Tuple(vec![size, is_file, is_dir]))
}

fn touch_path(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "fs.touch")?;
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&path)
        .map_err(|err| ApexError::new(format!("Failed to touch '{}': {}", path, err)))?;
    file.write_all(&[])
        .map_err(|err| ApexError::new(format!("Failed to touch '{}': {}", path, err)))?;
    Ok(Value::Bool(true))
}

fn create_tempfile(args: &[Value]) -> Result<Value, ApexError> {
    let prefix = if args.is_empty() {
        "apex_temp".to_string()
    } else {
        expect_string_arg(args, 0, "fs.tempfile")?
    };
    let mut path = std::env::temp_dir();
    path.push(format!("{}_{}", prefix, Uuid::new_v4()));
    let string_path = path.to_string_lossy().to_string();
    OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&string_path)
        .map_err(|err| ApexError::new(format!("Failed to create '{}': {}", string_path, err)))?;
    Ok(Value::String(string_path))
}

fn read_tree(args: &[Value]) -> Result<Value, ApexError> {
    let root = expect_string_arg(args, 0, "fs.read_tree")?;
    let root_path = Path::new(&root);
    if !root_path.exists() {
        return Err(ApexError::new("fs.read_tree expects an existing path"));
    }
    let mut entries = Vec::new();
    collect_tree(root_path, root_path, &mut entries)?;
    Ok(Value::Tuple(
        entries.into_iter().map(Value::String).collect::<Vec<_>>(),
    ))
}

fn collect_tree(root: &Path, current: &Path, acc: &mut Vec<String>) -> Result<(), ApexError> {
    let rel = current
        .strip_prefix(root)
        .unwrap_or(current)
        .to_string_lossy()
        .to_string();
    let label = if rel.is_empty() {
        if current.is_dir() {
            ".".to_string()
        } else {
            current
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| current.to_string_lossy().to_string())
        }
    } else {
        rel
    };
    acc.push(label);
    if current.is_dir() {
        for entry in fs::read_dir(current).map_err(|err| {
            ApexError::new(format!("Failed to list '{}': {}", current.display(), err))
        })? {
            let entry =
                entry.map_err(|err| ApexError::new(format!("Failed to read entry: {}", err)))?;
            collect_tree(root, &entry.path(), acc)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn writes_and_reads_text() {
        let path = tempfile_path("text");
        write_text(&[Value::String(path.clone()), Value::String("hello".into())]).unwrap();
        append_text(&[Value::String(path.clone()), Value::String(" world".into())]).unwrap();
        let contents = read_text(&[Value::String(path.clone())]).unwrap();
        assert_eq!(contents, Value::String("hello world".into()));
        assert_eq!(
            file_exists(&[Value::String(path.clone())]).unwrap(),
            Value::Bool(true)
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn writes_and_reads_bytes() {
        let path = tempfile_path("bytes");
        let tuple = Value::Tuple(vec![Value::Int(1.into()), Value::Int(2.into())]);
        write_bytes(&[Value::String(path.clone()), tuple]).unwrap();
        let bytes = read_bytes(&[Value::String(path.clone())]).unwrap();
        if let Value::Tuple(values) = bytes {
            assert_eq!(values.len(), 2);
            assert_eq!(values[0], Value::Int(1.into()));
            assert_eq!(values[1], Value::Int(2.into()));
        } else {
            panic!("expected tuple");
        }
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn lists_and_deletes_paths() {
        let dir = tempfile_path("dir");
        fs::create_dir_all(&dir).unwrap();
        let file_path = format!("{}/entry.txt", dir);
        write_text(&[
            Value::String(file_path.clone()),
            Value::String("data".into()),
        ])
        .unwrap();
        let listing = list_dir(&[Value::String(dir.clone())]).unwrap();
        if let Value::Tuple(values) = listing {
            assert_eq!(values.len(), 1);
        } else {
            panic!("expected tuple");
        }
        assert_eq!(
            delete_path(&[Value::String(file_path.clone())]).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            delete_path(&[Value::String(file_path.clone())]).unwrap(),
            Value::Bool(false)
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn copies_moves_and_stats() {
        let src = tempfile_path("copy");
        write_text(&[Value::String(src.clone()), Value::String("abc".into())]).unwrap();
        let dest = format!("{}-dest", src);
        let copied = copy_path(&[Value::String(src.clone()), Value::String(dest.clone())]).unwrap();
        assert_eq!(copied, Value::Int(3.into()));
        let meta = metadata(&[Value::String(dest.clone())]).unwrap();
        if let Value::Tuple(values) = meta {
            assert_eq!(values[1], Value::Bool(true));
        } else {
            panic!("expected tuple");
        }
        let renamed = format!("{}-renamed", dest);
        rename_path(&[Value::String(dest.clone()), Value::String(renamed.clone())]).unwrap();
        let dir = tempfile_path("mkdir");
        mkdir_all(&[Value::String(dir.clone())]).unwrap();
        let dir_meta = metadata(&[Value::String(dir.clone())]).unwrap();
        if let Value::Tuple(values) = dir_meta {
            assert_eq!(values[2], Value::Bool(true));
        } else {
            panic!("expected tuple");
        }
        delete_path(&[Value::String(src)]).unwrap();
        delete_path(&[Value::String(renamed)]).unwrap();
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn touches_and_walks_tree() {
        let dir = tempfile_path("tree");
        fs::create_dir_all(&dir).unwrap();
        let file = format!("{}/note.txt", dir);
        touch_path(&[Value::String(file.clone())]).unwrap();
        let nested_dir = format!("{}/nested", dir);
        fs::create_dir_all(&nested_dir).unwrap();
        let nested_file = format!("{}/more.txt", nested_dir);
        write_text(&[
            Value::String(nested_file.clone()),
            Value::String("1".into()),
        ])
        .unwrap();
        let tree = read_tree(&[Value::String(dir.clone())]).unwrap();
        if let Value::Tuple(entries) = tree {
            assert!(entries
                .iter()
                .any(|entry| entry == &Value::String(".".into())));
            assert!(entries
                .iter()
                .any(|entry| entry == &Value::String("note.txt".into())));
            assert!(entries
                .iter()
                .any(|entry| entry == &Value::String("nested".into())));
        } else {
            panic!("expected tuple");
        }
        let temp = create_tempfile(&[Value::String("tree".into())]).unwrap();
        if let Value::String(path) = temp {
            assert!(Path::new(&path).exists());
            fs::remove_file(path).unwrap();
        } else {
            panic!("expected temp path");
        }
        fs::remove_dir_all(dir).unwrap();
    }

    fn tempfile_path(label: &str) -> String {
        let mut path = std::env::temp_dir();
        path.push(format!("apex_fs_{}_{}.txt", label, Uuid::new_v4()));
        path.to_string_lossy().to_string()
    }
}

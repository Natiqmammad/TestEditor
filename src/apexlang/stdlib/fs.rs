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
    add!(&mut functions, "copy_dir", copy_dir);
    add!(&mut functions, "rename", rename_path);
    add!(&mut functions, "mkdir_all", mkdir_all);
    add!(&mut functions, "metadata", metadata);
    add!(&mut functions, "touch", touch_path);
    add!(&mut functions, "tempfile", create_tempfile);
    add!(&mut functions, "read_tree", read_tree);
    add!(&mut functions, "walk_files", walk_files);
    add!(&mut functions, "path_components", path_components);
    add!(&mut functions, "path_join", path_join);
    add!(&mut functions, "relative_path", relative_path);
    add!(&mut functions, "read_lines", read_lines);
    add!(&mut functions, "write_lines", write_lines);
    add!(&mut functions, "dir_size", dir_size);
    add!(&mut functions, "file_size", file_size);
    add!(&mut functions, "symlink_target", symlink_target);
    add!(&mut functions, "canonicalize", canonicalize_path);
    add!(&mut functions, "is_file", is_file_path);
    add!(&mut functions, "is_dir", is_dir_path);
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

fn read_lines(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "fs.read_lines")?;
    let contents = fs::read_to_string(&path)
        .map_err(|err| ApexError::new(format!("Failed to read '{}': {}", path, err)))?;
    Ok(Value::Tuple(
        contents
            .lines()
            .map(|line| Value::String(line.to_string()))
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

fn write_lines(args: &[Value]) -> Result<Value, ApexError> {
    if args.len() < 2 {
        return Err(ApexError::new(
            "fs.write_lines expects at least one line argument",
        ));
    }
    let path = expect_string_arg(args, 0, "fs.write_lines")?;
    let mut buffer = String::new();
    for (index, value) in args.iter().enumerate().skip(1) {
        match value {
            Value::String(text) => {
                buffer.push_str(text);
                if index + 1 < args.len() {
                    buffer.push('\n');
                }
            }
            _ => {
                return Err(ApexError::new(
                    "fs.write_lines expects string arguments after the path",
                ))
            }
        }
    }
    fs::write(&path, buffer)
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

fn copy_dir(args: &[Value]) -> Result<Value, ApexError> {
    let source = expect_string_arg(args, 0, "fs.copy_dir")?;
    let dest = expect_string_arg(args, 1, "fs.copy_dir")?;
    let total = copy_dir_recursive(Path::new(&source), Path::new(&dest))?;
    Ok(Value::Int(BigInt::from(total)))
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

fn walk_files(args: &[Value]) -> Result<Value, ApexError> {
    let root = expect_string_arg(args, 0, "fs.walk_files")?;
    let root_path = Path::new(&root);
    if !root_path.exists() {
        return Err(ApexError::new("fs.walk_files expects an existing path"));
    }
    let mut entries = Vec::new();
    collect_files_only(root_path, root_path, &mut entries)?;
    Ok(Value::Tuple(
        entries.into_iter().map(Value::String).collect::<Vec<_>>(),
    ))
}

fn path_components(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "fs.path_components")?;
    let components = Path::new(&path)
        .components()
        .map(|component| Value::String(component.as_os_str().to_string_lossy().to_string()))
        .collect();
    Ok(Value::Tuple(components))
}

fn path_join(args: &[Value]) -> Result<Value, ApexError> {
    if args.len() < 2 {
        return Err(ApexError::new(
            "fs.path_join expects at least two string arguments",
        ));
    }
    let mut joined = std::path::PathBuf::from(expect_string_arg(args, 0, "fs.path_join")?);
    for (index, value) in args.iter().enumerate().skip(1) {
        match value {
            Value::String(component) => joined.push(component),
            _ => {
                return Err(ApexError::new(format!(
                    "fs.path_join expects string component at position {}",
                    index + 1
                )))
            }
        }
    }
    Ok(Value::String(joined.to_string_lossy().to_string()))
}

fn relative_path(args: &[Value]) -> Result<Value, ApexError> {
    if args.len() != 2 {
        return Err(ApexError::new(
            "fs.relative_path expects (base, target) string arguments",
        ));
    }
    let base = expect_string_arg(args, 0, "fs.relative_path")?;
    let target = expect_string_arg(args, 1, "fs.relative_path")?;
    let relative = Path::new(&target)
        .strip_prefix(Path::new(&base))
        .map_err(|_| ApexError::new("fs.relative_path expects target within the base"))?;
    Ok(Value::String(relative.to_string_lossy().to_string()))
}

fn dir_size(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "fs.dir_size")?;
    let total = compute_dir_size(Path::new(&path))?;
    Ok(Value::Int(BigInt::from(total)))
}

fn file_size(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "fs.file_size")?;
    let metadata = fs::metadata(&path)
        .map_err(|err| ApexError::new(format!("Failed to inspect '{}': {}", path, err)))?;
    if !metadata.is_file() {
        return Err(ApexError::new("fs.file_size expects a file path"));
    }
    Ok(Value::Int(BigInt::from(metadata.len())))
}

fn symlink_target(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "fs.symlink_target")?;
    match fs::read_link(&path) {
        Ok(target) => Ok(Value::Tuple(vec![
            Value::Bool(true),
            Value::String(target.to_string_lossy().to_string()),
        ])),
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound | std::io::ErrorKind::InvalidInput => {
                Ok(Value::Tuple(vec![
                    Value::Bool(false),
                    Value::String(String::new()),
                ]))
            }
            _ => Err(ApexError::new(format!(
                "Failed to read symlink '{}': {}",
                path, err
            ))),
        },
    }
}

fn canonicalize_path(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "fs.canonicalize")?;
    let absolute = fs::canonicalize(&path)
        .map_err(|err| ApexError::new(format!("Failed to canonicalize '{}': {}", path, err)))?;
    Ok(Value::String(absolute.to_string_lossy().to_string()))
}

fn is_file_path(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "fs.is_file")?;
    Ok(Value::Bool(Path::new(&path).is_file()))
}

fn is_dir_path(args: &[Value]) -> Result<Value, ApexError> {
    let path = expect_string_arg(args, 0, "fs.is_dir")?;
    Ok(Value::Bool(Path::new(&path).is_dir()))
}

fn compute_dir_size(path: &Path) -> Result<u64, ApexError> {
    if !path.exists() {
        return Ok(0);
    }
    if path.is_file() {
        let meta = fs::metadata(path).map_err(|err| {
            ApexError::new(format!("Failed to inspect '{}': {}", path.display(), err))
        })?;
        return Ok(meta.len());
    }
    let mut total = 0u64;
    for entry in fs::read_dir(path)
        .map_err(|err| ApexError::new(format!("Failed to read '{}': {}", path.display(), err)))?
    {
        let entry = entry.map_err(|err| {
            ApexError::new(format!("Failed to traverse '{}': {}", path.display(), err))
        })?;
        total += compute_dir_size(&entry.path())?;
    }
    Ok(total)
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

fn collect_files_only(root: &Path, current: &Path, acc: &mut Vec<String>) -> Result<(), ApexError> {
    if current.is_file() {
        let rel = current
            .strip_prefix(root)
            .unwrap_or(current)
            .to_string_lossy()
            .to_string();
        acc.push(rel);
        return Ok(());
    }
    if current.is_dir() {
        for entry in fs::read_dir(current).map_err(|err| {
            ApexError::new(format!("Failed to list '{}': {}", current.display(), err))
        })? {
            let entry =
                entry.map_err(|err| ApexError::new(format!("Failed to read entry: {}", err)))?;
            collect_files_only(root, &entry.path(), acc)?;
        }
    }
    Ok(())
}

fn copy_dir_recursive(source: &Path, dest: &Path) -> Result<u64, ApexError> {
    if !source.exists() {
        return Err(ApexError::new("fs.copy_dir expects an existing source"));
    }
    if source.is_file() {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                ApexError::new(format!("Failed to create '{}': {}", parent.display(), err))
            })?;
        }
        fs::copy(source, dest).map_err(|err| {
            ApexError::new(format!("Failed to copy '{}': {}", source.display(), err))
        })?;
        return Ok(1);
    }
    fs::create_dir_all(dest)
        .map_err(|err| ApexError::new(format!("Failed to create '{}': {}", dest.display(), err)))?;
    let mut total = 0u64;
    for entry in fs::read_dir(source)
        .map_err(|err| ApexError::new(format!("Failed to read '{}': {}", source.display(), err)))?
    {
        let entry = entry.map_err(|err| {
            ApexError::new(format!(
                "Failed to traverse '{}': {}",
                source.display(),
                err
            ))
        })?;
        let mut target_path = dest.to_path_buf();
        target_path.push(entry.file_name());
        total += copy_dir_recursive(&entry.path(), &target_path)?;
    }
    Ok(total)
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
    fn line_helpers_round_trip() {
        let path = tempfile_path("lines");
        write_lines(&[
            Value::String(path.clone()),
            Value::String("alpha".into()),
            Value::String("beta".into()),
        ])
        .unwrap();
        let contents = read_lines(&[Value::String(path.clone())]).unwrap();
        if let Value::Tuple(values) = contents {
            assert_eq!(values.len(), 2);
            assert_eq!(values[0], Value::String("alpha".into()));
            assert_eq!(values[1], Value::String("beta".into()));
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

    #[test]
    fn dir_size_counts_recursive_bytes() {
        let dir = tempfile_path("size");
        fs::create_dir_all(&dir).unwrap();
        let first = format!("{}/a.txt", dir);
        write_text(&[Value::String(first.clone()), Value::String("abcd".into())]).unwrap();
        let nested_dir = format!("{}/nest", dir);
        fs::create_dir_all(&nested_dir).unwrap();
        let nested = format!("{}/b.bin", nested_dir);
        write_bytes(&[
            Value::String(nested.clone()),
            Value::Tuple(vec![Value::Int(1.into()), Value::Int(2.into())]),
        ])
        .unwrap();
        let size = dir_size(&[Value::String(dir.clone())]).unwrap();
        if let Value::Int(total) = size {
            assert!(total >= BigInt::from(6));
        } else {
            panic!("expected size int");
        }
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn copies_directories_and_components() {
        let src = tempfile_path("copydir-src");
        let nested = format!("{}/nested", src);
        fs::create_dir_all(&nested).unwrap();
        write_text(&[
            Value::String(format!("{}/file.txt", nested)),
            Value::String("payload".into()),
        ])
        .unwrap();
        let dest = format!("{}-dest", src);
        let copied = copy_dir(&[Value::String(src.clone()), Value::String(dest.clone())]).unwrap();
        assert!(matches!(copied, Value::Int(value) if value > BigInt::from(0)));
        let tree = read_tree(&[Value::String(dest.clone())]).unwrap();
        if let Value::Tuple(entries) = tree {
            assert!(entries
                .iter()
                .any(|entry| entry == &Value::String("nested".into())));
        } else {
            panic!("expected tuple");
        }
        let components =
            path_components(&[Value::String(format!("{}/nested/file.txt", dest))]).unwrap();
        if let Value::Tuple(values) = components {
            assert!(values.len() >= 2);
        } else {
            panic!("expected components tuple");
        }
        fs::remove_dir_all(dest).unwrap();
    }

    #[test]
    fn walk_files_and_relative_paths() {
        let dir = tempfile_path("walk");
        fs::create_dir_all(&dir).unwrap();
        let nested = format!("{}/nested", dir);
        fs::create_dir_all(&nested).unwrap();
        let file_a = format!("{}/a.txt", dir);
        let file_b = format!("{}/b.txt", nested);
        write_text(&[Value::String(file_a.clone()), Value::String("A".into())]).unwrap();
        write_text(&[Value::String(file_b.clone()), Value::String("B".into())]).unwrap();
        let walk = walk_files(&[Value::String(dir.clone())]).unwrap();
        if let Value::Tuple(entries) = walk {
            assert_eq!(entries.len(), 2);
        } else {
            panic!("expected walk tuple");
        }
        let rel =
            relative_path(&[Value::String(dir.clone()), Value::String(file_b.clone())]).unwrap();
        if let Value::String(text) = rel {
            assert!(text.contains("nested"));
        } else {
            panic!("expected relative string");
        }
        let size = file_size(&[Value::String(file_a.clone())]).unwrap();
        assert_eq!(size, Value::Int(1.into()));
        let joined = path_join(&[
            Value::String(dir.clone()),
            Value::String("nested".into()),
            Value::String("b.txt".into()),
        ])
        .unwrap();
        if let Value::String(text) = joined {
            assert!(text.ends_with("b.txt"));
        }
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn canonicalize_and_type_checks() {
        let dir = tempfile_path("canon");
        fs::create_dir_all(&dir).unwrap();
        let file = format!("{}/note.txt", dir);
        write_text(&[Value::String(file.clone()), Value::String("x".into())]).unwrap();
        let canonical = canonicalize_path(&[Value::String(file.clone())]).unwrap();
        if let Value::String(path) = canonical {
            assert!(path.ends_with("note.txt"));
        } else {
            panic!("expected canonical string");
        }
        assert_eq!(
            is_file_path(&[Value::String(file.clone())]).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            is_dir_path(&[Value::String(dir.clone())]).unwrap(),
            Value::Bool(true)
        );
        fs::remove_file(file).unwrap();
        fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn symlink_targets_round_trip() {
        use std::os::unix::fs as unix_fs;

        let target = tempfile_path("symlink-target");
        write_text(&[Value::String(target.clone()), Value::String("link".into())]).unwrap();
        let link = tempfile_path("symlink-link");
        unix_fs::symlink(&target, &link).unwrap();
        let result = symlink_target(&[Value::String(link.clone())]).unwrap();
        if let Value::Tuple(values) = result {
            assert_eq!(values[0], Value::Bool(true));
        } else {
            panic!("expected tuple");
        }
        fs::remove_file(target).unwrap();
        fs::remove_file(link).unwrap();
    }

    fn tempfile_path(label: &str) -> String {
        let mut path = std::env::temp_dir();
        path.push(format!("apex_fs_{}_{}.txt", label, Uuid::new_v4()));
        path.to_string_lossy().to_string()
    }
}

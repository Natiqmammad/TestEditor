use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::TryRecvError;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use num_bigint::BigInt;
use num_traits::{One, ToPrimitive, Zero};
use once_cell::sync::Lazy;

use crate::apexlang::ast::Value;
use crate::apexlang::error::ApexError;

use super::support::{expect_int_arg, expect_string_arg, expect_tuple_arg};
use super::{NativeCallable, NativeRegistry};

pub(super) fn register(registry: &mut NativeRegistry) {
    let mut functions: HashMap<String, NativeCallable> = HashMap::new();

    macro_rules! add {
        ($map:expr, $name:literal, $func:ident) => {
            $map.insert(
                $name.to_string(),
                NativeCallable::new(concat!("async::", $name), $func),
            );
        };
    }

    add!(&mut functions, "spawn", spawn_task);
    add!(&mut functions, "join", join_task);
    add!(&mut functions, "cancel", cancel_task);
    add!(&mut functions, "join_all", join_all);
    add!(&mut functions, "pending", pending_tasks);
    add!(&mut functions, "yield_now", yield_now);
    add!(&mut functions, "sleep_ms", sleep_ms);
    add!(&mut functions, "mailbox_create", mailbox_create);
    add!(&mut functions, "mailbox_send", mailbox_send);
    add!(&mut functions, "mailbox_send_batch", mailbox_send_batch);
    add!(&mut functions, "mailbox_recv", mailbox_recv);
    add!(&mut functions, "mailbox_try_recv", mailbox_try_recv);
    add!(&mut functions, "mailbox_drain", mailbox_drain);
    add!(&mut functions, "mailbox_recv_batch", mailbox_recv_batch);
    add!(&mut functions, "mailbox_recv_any", mailbox_recv_any);
    add!(&mut functions, "mailbox_forward", mailbox_forward);
    add!(&mut functions, "mailbox_close", mailbox_close);
    add!(&mut functions, "mailbox_flush", mailbox_flush);
    add!(&mut functions, "mailbox_len", mailbox_len);
    add!(&mut functions, "mailbox_recv_timeout", mailbox_recv_timeout);
    add!(&mut functions, "mailbox_is_closed", mailbox_is_closed);
    add!(&mut functions, "mailbox_stats", mailbox_stats);
    registry.register_module("async", functions);
}

struct TaskRegistry {
    next_id: u64,
    tasks: HashMap<u64, thread::JoinHandle<Value>>,
}

impl Default for TaskRegistry {
    fn default() -> Self {
        Self {
            next_id: 0,
            tasks: HashMap::new(),
        }
    }
}

static TASKS: Lazy<Mutex<TaskRegistry>> = Lazy::new(|| Mutex::new(TaskRegistry::default()));

struct Mailbox {
    sender: mpsc::Sender<Value>,
    receiver: Arc<Mutex<mpsc::Receiver<Value>>>,
    pending: Arc<AtomicUsize>,
    closed: Arc<AtomicBool>,
}

#[derive(Default)]
struct MailboxRegistry {
    next_id: u64,
    boxes: HashMap<u64, Mailbox>,
}

static MAILBOXES: Lazy<Mutex<MailboxRegistry>> =
    Lazy::new(|| Mutex::new(MailboxRegistry::default()));

fn spawn_task(args: &[Value]) -> Result<Value, ApexError> {
    let name = expect_string_arg(args, 0, "async.spawn")?;
    let payload = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| Value::Int(BigInt::from(0)));
    let handle = match name.as_str() {
        "sum" => spawn_sum(payload)?,
        "factorial" => spawn_factorial(payload)?,
        "prime_count" => spawn_prime_count(payload)?,
        "sleep_ms" => spawn_sleep(payload)?,
        "fibonacci" => spawn_fibonacci(payload)?,
        other => return Err(ApexError::new(format!("Unknown async task '{}'", other))),
    };
    let mut registry = TASKS
        .lock()
        .map_err(|_| ApexError::new("Task registry lock poisoned"))?;
    let id = registry.next_id;
    registry.next_id += 1;
    registry.tasks.insert(id, handle);
    Ok(Value::Int(BigInt::from(id)))
}

fn join_task(args: &[Value]) -> Result<Value, ApexError> {
    let id = expect_int_arg(args, 0, "async.join")?;
    let id = id
        .to_u64()
        .ok_or_else(|| ApexError::new("Task id is too large"))?;
    let handle = {
        let mut registry = TASKS
            .lock()
            .map_err(|_| ApexError::new("Task registry lock poisoned"))?;
        registry
            .tasks
            .remove(&id)
            .ok_or_else(|| ApexError::new("Unknown task handle"))?
    };
    handle.join().map_err(|_| ApexError::new("Task panicked"))
}

fn cancel_task(args: &[Value]) -> Result<Value, ApexError> {
    let id = expect_int_arg(args, 0, "async.cancel")?;
    let id = id
        .to_u64()
        .ok_or_else(|| ApexError::new("Task id is too large"))?;
    let handle = {
        let mut registry = TASKS
            .lock()
            .map_err(|_| ApexError::new("Task registry lock poisoned"))?;
        registry.tasks.remove(&id)
    };
    if let Some(_handle) = handle {
        // Dropping a JoinHandle detaches the worker thread.
        Ok(Value::Bool(true))
    } else {
        Ok(Value::Bool(false))
    }
}

fn join_all(args: &[Value]) -> Result<Value, ApexError> {
    if args.is_empty() {
        return Ok(Value::Tuple(Vec::new()));
    }
    let mut join_handles = Vec::with_capacity(args.len());
    {
        let mut registry = TASKS
            .lock()
            .map_err(|_| ApexError::new("Task registry lock poisoned"))?;
        for (index, handle_value) in args.iter().enumerate() {
            let id = match handle_value {
                Value::Int(id) => id.to_u64().ok_or_else(|| {
                    ApexError::new(format!("async.join_all handle #{} is too large", index + 1))
                })?,
                _ => {
                    return Err(ApexError::new(format!(
                        "async.join_all expects integer handles (position {})",
                        index + 1
                    )))
                }
            };
            let handle = registry
                .tasks
                .remove(&id)
                .ok_or_else(|| ApexError::new("Unknown task handle"))?;
            join_handles.push(handle);
        }
    }
    let mut results = Vec::with_capacity(join_handles.len());
    for handle in join_handles {
        results.push(handle.join().map_err(|_| ApexError::new("Task panicked"))?);
    }
    Ok(Value::Tuple(results))
}

fn pending_tasks(_args: &[Value]) -> Result<Value, ApexError> {
    let registry = TASKS
        .lock()
        .map_err(|_| ApexError::new("Task registry lock poisoned"))?;
    Ok(Value::Int(BigInt::from(registry.tasks.len())))
}

fn yield_now(_args: &[Value]) -> Result<Value, ApexError> {
    thread::yield_now();
    Ok(Value::Bool(true))
}

fn sleep_ms(args: &[Value]) -> Result<Value, ApexError> {
    let millis = expect_int_arg(args, 0, "async.sleep_ms")?;
    let millis = millis
        .to_u64()
        .ok_or_else(|| ApexError::new("async.sleep_ms expects a non-negative integer"))?;
    thread::sleep(Duration::from_millis(millis));
    Ok(Value::Bool(true))
}

fn mailbox_create(_args: &[Value]) -> Result<Value, ApexError> {
    let handle = {
        let mut registry = MAILBOXES
            .lock()
            .map_err(|_| ApexError::new("Mailbox registry lock poisoned"))?;
        let id = registry.next_id;
        registry.next_id += 1;
        let (sender, receiver) = mpsc::channel();
        let pending = Arc::new(AtomicUsize::new(0));
        let closed = Arc::new(AtomicBool::new(false));
        registry.boxes.insert(
            id,
            Mailbox {
                sender,
                receiver: Arc::new(Mutex::new(receiver)),
                pending,
                closed,
            },
        );
        Value::Tuple(vec![Value::Int(BigInt::from(id))])
    };
    Ok(handle)
}

fn mailbox_send(args: &[Value]) -> Result<Value, ApexError> {
    let handle = expect_mailbox_handle(args, 0, "async.mailbox_send")?;
    let payload = args
        .get(1)
        .cloned()
        .ok_or_else(|| ApexError::new("async.mailbox_send expects a payload"))?;
    let view = get_mailbox_view(handle)?;
    if view.closed.load(Ordering::SeqCst) {
        return Err(ApexError::new("Mailbox is closed"));
    }
    view.sender
        .send(payload)
        .map_err(|_| ApexError::new("Mailbox receiver disconnected"))?;
    view.pending.fetch_add(1, Ordering::SeqCst);
    Ok(Value::Bool(true))
}

fn mailbox_send_batch(args: &[Value]) -> Result<Value, ApexError> {
    if args.len() < 2 {
        return Err(ApexError::new(
            "async.mailbox_send_batch expects a handle and tuple payload",
        ));
    }
    let handle = expect_mailbox_handle(args, 0, "async.mailbox_send_batch")?;
    let payloads = expect_tuple_arg(args, 1, "async.mailbox_send_batch")?;
    let view = get_mailbox_view(handle)?;
    if view.closed.load(Ordering::SeqCst) {
        return Err(ApexError::new("Mailbox is closed"));
    }
    let mut sent = 0usize;
    for payload in payloads {
        view.sender
            .send(payload)
            .map_err(|_| ApexError::new("Mailbox receiver disconnected"))?;
        view.pending.fetch_add(1, Ordering::SeqCst);
        sent += 1;
    }
    Ok(Value::Int(BigInt::from(sent)))
}

fn mailbox_recv(args: &[Value]) -> Result<Value, ApexError> {
    let handle = expect_mailbox_handle(args, 0, "async.mailbox_recv")?;
    let view = get_mailbox_view(handle)?;
    let value = view
        .receiver
        .lock()
        .map_err(|_| ApexError::new("Mailbox receiver lock poisoned"))?
        .recv()
        .map_err(|_| ApexError::new("Mailbox receiver disconnected"))?;
    view.pending.fetch_sub(1, Ordering::SeqCst);
    Ok(value)
}

fn mailbox_try_recv(args: &[Value]) -> Result<Value, ApexError> {
    let handle = expect_mailbox_handle(args, 0, "async.mailbox_try_recv")?;
    let view = get_mailbox_view(handle)?;
    let guard = view
        .receiver
        .lock()
        .map_err(|_| ApexError::new("Mailbox receiver lock poisoned"))?;
    match guard.try_recv() {
        Ok(value) => {
            view.pending.fetch_sub(1, Ordering::SeqCst);
            Ok(Value::Tuple(vec![Value::Bool(true), value]))
        }
        Err(mpsc::TryRecvError::Empty) => {
            Ok(Value::Tuple(vec![Value::Bool(false), Value::Bool(false)]))
        }
        Err(mpsc::TryRecvError::Disconnected) => {
            Err(ApexError::new("Mailbox receiver disconnected"))
        }
    }
}

fn mailbox_drain(args: &[Value]) -> Result<Value, ApexError> {
    let handle = expect_mailbox_handle(args, 0, "async.mailbox_drain")?;
    let view = get_mailbox_view(handle)?;
    Ok(Value::Tuple(drain_mailbox_values(&view)?))
}

fn mailbox_recv_batch(args: &[Value]) -> Result<Value, ApexError> {
    let handle = expect_mailbox_handle(args, 0, "async.mailbox_recv_batch")?;
    let limit = expect_int_arg(args, 1, "async.mailbox_recv_batch")?;
    let limit = limit
        .to_usize()
        .ok_or_else(|| ApexError::new("async.mailbox_recv_batch expects usize-sized limits"))?;
    if limit == 0 {
        return Err(ApexError::new(
            "async.mailbox_recv_batch expects a positive batch length",
        ));
    }
    let view = get_mailbox_view(handle)?;
    let guard = view
        .receiver
        .lock()
        .map_err(|_| ApexError::new("Mailbox receiver lock poisoned"))?;
    let mut results = Vec::new();
    let first = guard
        .recv()
        .map_err(|_| ApexError::new("Mailbox receiver disconnected"))?;
    view.pending.fetch_sub(1, Ordering::SeqCst);
    results.push(first);
    while results.len() < limit {
        match guard.try_recv() {
            Ok(value) => {
                view.pending.fetch_sub(1, Ordering::SeqCst);
                results.push(value);
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => break,
        }
    }
    Ok(Value::Tuple(results))
}

fn mailbox_recv_any(args: &[Value]) -> Result<Value, ApexError> {
    if args.is_empty() {
        return Err(ApexError::new(
            "async.mailbox_recv_any expects at least one mailbox handle",
        ));
    }
    let mut views = Vec::with_capacity(args.len());
    for (index, handle_value) in args.iter().enumerate() {
        let handle = mailbox_handle_from_value(
            handle_value,
            &format!("async.mailbox_recv_any handle #{}", index + 1),
        )?;
        views.push((handle, get_mailbox_view(handle)?));
    }
    loop {
        let mut live_mailboxes = false;
        for (handle, view) in &views {
            let maybe_value = {
                let guard = view
                    .receiver
                    .lock()
                    .map_err(|_| ApexError::new("Mailbox receiver lock poisoned"))?;
                guard.try_recv()
            };
            match maybe_value {
                Ok(value) => {
                    view.pending.fetch_sub(1, Ordering::SeqCst);
                    return Ok(Value::Tuple(vec![
                        Value::Tuple(vec![Value::Int(BigInt::from(*handle))]),
                        value,
                    ]));
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => continue,
            }
            if !view.closed.load(Ordering::SeqCst) || view.pending.load(Ordering::SeqCst) > 0 {
                live_mailboxes = true;
            }
        }
        if !live_mailboxes {
            return Err(ApexError::new("All mailboxes are closed"));
        }
        thread::sleep(Duration::from_millis(1));
    }
}

fn mailbox_forward(args: &[Value]) -> Result<Value, ApexError> {
    let source = expect_mailbox_handle(args, 0, "async.mailbox_forward")?;
    let target = expect_mailbox_handle(args, 1, "async.mailbox_forward")?;
    if source == target {
        return Ok(Value::Int(BigInt::from(0)));
    }
    let source_view = get_mailbox_view(source)?;
    let target_view = get_mailbox_view(target)?;
    if target_view.closed.load(Ordering::SeqCst) {
        return Err(ApexError::new("Target mailbox is closed"));
    }
    let mut moved = 0usize;
    let guard = source_view
        .receiver
        .lock()
        .map_err(|_| ApexError::new("Mailbox receiver lock poisoned"))?;
    loop {
        match guard.try_recv() {
            Ok(value) => {
                target_view
                    .sender
                    .send(value)
                    .map_err(|_| ApexError::new("Target mailbox receiver disconnected"))?;
                target_view.pending.fetch_add(1, Ordering::SeqCst);
                source_view.pending.fetch_sub(1, Ordering::SeqCst);
                moved += 1;
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => break,
        }
    }
    Ok(Value::Int(BigInt::from(moved)))
}

fn mailbox_close(args: &[Value]) -> Result<Value, ApexError> {
    let handle = expect_mailbox_handle(args, 0, "async.mailbox_close")?;
    Ok(Value::Bool(close_mailbox_entry(handle)?))
}

fn mailbox_flush(args: &[Value]) -> Result<Value, ApexError> {
    let handle = expect_mailbox_handle(args, 0, "async.mailbox_flush")?;
    let view = get_mailbox_view(handle)?;
    let drained = drain_mailbox_values(&view)?;
    let closed = close_mailbox_entry(handle)?;
    Ok(Value::Tuple(vec![
        Value::Tuple(drained),
        Value::Bool(closed),
    ]))
}

fn mailbox_len(args: &[Value]) -> Result<Value, ApexError> {
    let handle = expect_mailbox_handle(args, 0, "async.mailbox_len")?;
    let view = get_mailbox_view(handle)?;
    let pending = view.pending.load(Ordering::SeqCst);
    Ok(Value::Int(BigInt::from(pending)))
}

fn mailbox_recv_timeout(args: &[Value]) -> Result<Value, ApexError> {
    let handle = expect_mailbox_handle(args, 0, "async.mailbox_recv_timeout")?;
    let timeout = expect_int_arg(args, 1, "async.mailbox_recv_timeout")?;
    let millis = timeout.to_u64().ok_or_else(|| {
        ApexError::new("async.mailbox_recv_timeout expects a non-negative timeout")
    })?;
    let view = get_mailbox_view(handle)?;
    let value = view
        .receiver
        .lock()
        .map_err(|_| ApexError::new("Mailbox receiver lock poisoned"))?
        .recv_timeout(Duration::from_millis(millis))
        .map_err(|err| match err {
            mpsc::RecvTimeoutError::Timeout => ApexError::new("Mailbox receive timed out"),
            mpsc::RecvTimeoutError::Disconnected => ApexError::new("Mailbox receiver disconnected"),
        })?;
    view.pending.fetch_sub(1, Ordering::SeqCst);
    Ok(value)
}

fn mailbox_is_closed(args: &[Value]) -> Result<Value, ApexError> {
    let handle = expect_mailbox_handle(args, 0, "async.mailbox_is_closed")?;
    let view = get_mailbox_view(handle)?;
    Ok(Value::Bool(view.closed.load(Ordering::SeqCst)))
}

fn mailbox_stats(args: &[Value]) -> Result<Value, ApexError> {
    let handle = expect_mailbox_handle(args, 0, "async.mailbox_stats")?;
    let registry = MAILBOXES
        .lock()
        .map_err(|_| ApexError::new("Mailbox registry lock poisoned"))?;
    if let Some(mailbox) = registry.boxes.get(&handle) {
        let pending = mailbox.pending.load(Ordering::SeqCst);
        let closed = mailbox.closed.load(Ordering::SeqCst);
        Ok(Value::Tuple(vec![
            Value::Int(BigInt::from(pending)),
            Value::Bool(closed),
        ]))
    } else {
        Ok(Value::Tuple(vec![
            Value::Int(BigInt::from(0)),
            Value::Bool(true),
        ]))
    }
}

fn spawn_sum(value: Value) -> Result<thread::JoinHandle<Value>, ApexError> {
    let limit = expect_int_value(value, "sum")?;
    let n = limit
        .to_u64()
        .ok_or_else(|| ApexError::new("sum expects non-negative integer"))?;
    Ok(thread::spawn(move || Value::Int(sum_up_to(n))))
}

fn spawn_factorial(value: Value) -> Result<thread::JoinHandle<Value>, ApexError> {
    let limit = expect_int_value(value, "factorial")?;
    let n = limit
        .to_u64()
        .ok_or_else(|| ApexError::new("factorial expects non-negative integer"))?;
    Ok(thread::spawn(move || Value::Int(factorial(n))))
}

fn spawn_prime_count(value: Value) -> Result<thread::JoinHandle<Value>, ApexError> {
    let limit = expect_int_value(value, "prime_count")?;
    let n = limit
        .to_u64()
        .ok_or_else(|| ApexError::new("prime_count expects non-negative integer"))?;
    Ok(thread::spawn(move || {
        Value::Int(BigInt::from(count_primes(n)))
    }))
}

fn spawn_sleep(value: Value) -> Result<thread::JoinHandle<Value>, ApexError> {
    let duration = expect_int_value(value, "sleep_ms")?;
    let millis = duration
        .to_u64()
        .ok_or_else(|| ApexError::new("sleep_ms expects milliseconds that fit in u64"))?;
    Ok(thread::spawn(move || {
        thread::sleep(Duration::from_millis(millis));
        Value::Int(BigInt::from(millis))
    }))
}

fn spawn_fibonacci(value: Value) -> Result<thread::JoinHandle<Value>, ApexError> {
    let limit = expect_int_value(value, "fibonacci")?;
    let n = limit
        .to_u64()
        .ok_or_else(|| ApexError::new("fibonacci expects non-negative integer"))?;
    Ok(thread::spawn(move || Value::Int(fibonacci(n))))
}

fn expect_int_value(value: Value, name: &str) -> Result<BigInt, ApexError> {
    match value {
        Value::Int(v) => Ok(v),
        _ => Err(ApexError::new(format!(
            "{} task expects an integer payload",
            name
        ))),
    }
}

fn sum_up_to(n: u64) -> BigInt {
    BigInt::from(n) * BigInt::from(n + 1) / BigInt::from(2u8)
}

fn factorial(n: u64) -> BigInt {
    let mut acc = BigInt::one();
    for i in 1..=n {
        acc *= BigInt::from(i);
    }
    acc
}

fn count_primes(limit: u64) -> u64 {
    if limit < 2 {
        return 0;
    }
    let mut sieve = vec![true; (limit + 1) as usize];
    sieve[0] = false;
    sieve[1] = false;
    let mut p = 2;
    while p * p <= limit {
        if sieve[p as usize] {
            let mut multiple = p * p;
            while multiple <= limit {
                sieve[multiple as usize] = false;
                multiple += p;
            }
        }
        p += 1;
    }
    sieve.into_iter().filter(|v| *v).count() as u64
}

fn fibonacci(n: u64) -> BigInt {
    if n == 0 {
        return BigInt::zero();
    }
    let mut a = BigInt::zero();
    let mut b = BigInt::one();
    for _ in 1..n {
        let next = &a + &b;
        a = b;
        b = next;
    }
    b
}

fn expect_mailbox_handle(args: &[Value], index: usize, name: &str) -> Result<u64, ApexError> {
    let tuple = expect_tuple_arg(args, index, name)?;
    if tuple.len() != 1 {
        return Err(ApexError::new(format!(
            "{} expects a mailbox tuple handle",
            name
        )));
    }
    mailbox_handle_from_value(&tuple[0], name)
}

fn mailbox_handle_from_value(value: &Value, name: &str) -> Result<u64, ApexError> {
    match value {
        Value::Int(raw) => raw
            .to_u64()
            .ok_or_else(|| ApexError::new(format!("{} handle is too large", name))),
        Value::Tuple(values) if values.len() == 1 => mailbox_handle_from_value(&values[0], name),
        Value::Tuple(_) => Err(ApexError::new(format!(
            "{} expects a tuple containing a numeric mailbox handle",
            name
        ))),
        _ => Err(ApexError::new(format!(
            "{} expects a numeric mailbox handle",
            name
        ))),
    }
}

#[derive(Clone)]
struct MailboxView {
    sender: mpsc::Sender<Value>,
    receiver: Arc<Mutex<mpsc::Receiver<Value>>>,
    pending: Arc<AtomicUsize>,
    closed: Arc<AtomicBool>,
}

fn get_mailbox_view(handle: u64) -> Result<MailboxView, ApexError> {
    let registry = MAILBOXES
        .lock()
        .map_err(|_| ApexError::new("Mailbox registry lock poisoned"))?;
    registry
        .boxes
        .get(&handle)
        .map(|entry| MailboxView {
            sender: entry.sender.clone(),
            receiver: entry.receiver.clone(),
            pending: entry.pending.clone(),
            closed: entry.closed.clone(),
        })
        .ok_or_else(|| ApexError::new("Unknown mailbox handle"))
}

fn drain_mailbox_values(view: &MailboxView) -> Result<Vec<Value>, ApexError> {
    let guard = view
        .receiver
        .lock()
        .map_err(|_| ApexError::new("Mailbox receiver lock poisoned"))?;
    let mut drained = Vec::new();
    loop {
        match guard.try_recv() {
            Ok(value) => {
                view.pending.fetch_sub(1, Ordering::SeqCst);
                drained.push(value);
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => break,
        }
    }
    Ok(drained)
}

fn close_mailbox_entry(handle: u64) -> Result<bool, ApexError> {
    let mut registry = MAILBOXES
        .lock()
        .map_err(|_| ApexError::new("Mailbox registry lock poisoned"))?;
    if let Some(entry) = registry.boxes.remove(&handle) {
        entry.closed.store(true, Ordering::SeqCst);
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawns_and_joins_tasks() {
        let handle = spawn_task(&[Value::String("sum".into()), Value::Int(10.into())]).unwrap();
        let result = join_task(&[handle]).unwrap();
        assert_eq!(result, Value::Int(55.into()));
        let pending = pending_tasks(&[]).unwrap();
        assert_eq!(pending, Value::Int(0.into()));
        yield_now(&[]).unwrap();
    }

    #[test]
    fn join_all_collects_results() {
        let first = spawn_task(&[Value::String("sum".into()), Value::Int(4.into())]).unwrap();
        let second =
            spawn_task(&[Value::String("factorial".into()), Value::Int(4.into())]).unwrap();
        let combined = join_all(&[first, second]).expect("join_all");
        if let Value::Tuple(values) = combined {
            assert_eq!(values.len(), 2);
            assert_eq!(values[0], Value::Int(10.into()));
            assert_eq!(values[1], Value::Int(24.into()));
        } else {
            panic!("expected tuple");
        }
    }

    #[test]
    fn cancel_detaches_background_tasks() {
        let handle = spawn_task(&[Value::String("sleep_ms".into()), Value::Int(5.into())]).unwrap();
        let cancelled = cancel_task(&[handle.clone()]).expect("cancel");
        assert_eq!(cancelled, Value::Bool(true));
        // Second cancel reports false because the handle is gone.
        let again = cancel_task(&[handle]).expect("cancel twice");
        assert_eq!(again, Value::Bool(false));
    }

    #[test]
    fn mailbox_round_trip() {
        let handle = mailbox_create(&[]).expect("create mailbox");
        let task_handle = spawn_task(&[Value::String("sum".into()), Value::Int(5.into())]).unwrap();
        let sum_value = join_task(&[task_handle]).unwrap();
        mailbox_send(&[handle.clone(), sum_value.clone()]).expect("send");
        let received = mailbox_recv(&[handle.clone()]).expect("recv");
        assert_eq!(received, Value::Int(15.into()));
        mailbox_send(&[handle.clone(), Value::Bool(true)]).expect("send");
        let maybe = mailbox_try_recv(&[handle]).expect("try recv");
        if let Value::Tuple(values) = maybe {
            assert_eq!(values.len(), 2);
            assert_eq!(values[0], Value::Bool(true));
            assert_eq!(values[1], Value::Bool(true));
        } else {
            panic!("expected tuple");
        }
    }

    #[test]
    fn mailbox_drain_and_sleep() {
        sleep_ms(&[Value::Int(0.into())]).expect("sleep");
        let handle = mailbox_create(&[]).expect("create mailbox");
        mailbox_send(&[handle.clone(), Value::Int(7.into())]).expect("send");
        mailbox_send(&[handle.clone(), Value::Bool(false)]).expect("send");
        let drained = mailbox_drain(&[handle.clone()]).expect("drain");
        if let Value::Tuple(values) = drained {
            assert_eq!(values.len(), 2);
        } else {
            panic!("expected tuple");
        }
        let closed = mailbox_close(&[handle.clone()]).expect("close");
        assert_eq!(closed, Value::Bool(true));
        let closed_again = mailbox_close(&[handle]).expect("close again");
        assert_eq!(closed_again, Value::Bool(false));
    }

    #[test]
    fn mailbox_len_and_timeout_helpers() {
        let handle = mailbox_create(&[]).expect("create");
        assert_eq!(
            mailbox_len(&[handle.clone()]).unwrap(),
            Value::Int(0.into())
        );
        mailbox_send(&[handle.clone(), Value::Int(9.into())]).expect("send");
        assert_eq!(
            mailbox_len(&[handle.clone()]).unwrap(),
            Value::Int(1.into())
        );
        let value = mailbox_recv_timeout(&[handle.clone(), Value::Int(0.into())]).expect("recv");
        assert_eq!(value, Value::Int(9.into()));
        assert_eq!(
            mailbox_len(&[handle.clone()]).unwrap(),
            Value::Int(0.into())
        );
        let open = mailbox_is_closed(&[handle.clone()]).expect("open");
        assert_eq!(open, Value::Bool(false));
        assert_eq!(mailbox_close(&[handle]).unwrap(), Value::Bool(true));
    }

    #[test]
    fn mailbox_forward_moves_messages() {
        let source = mailbox_create(&[]).expect("source");
        let dest = mailbox_create(&[]).expect("dest");
        mailbox_send(&[source.clone(), Value::Int(1.into())]).unwrap();
        mailbox_send(&[source.clone(), Value::Int(2.into())]).unwrap();
        let moved = mailbox_forward(&[source.clone(), dest.clone()]).expect("forward");
        assert_eq!(moved, Value::Int(2.into()));
        assert_eq!(
            mailbox_len(&[source.clone()]).unwrap(),
            Value::Int(0.into())
        );
        assert_eq!(mailbox_len(&[dest.clone()]).unwrap(), Value::Int(2.into()));
        let drained = mailbox_drain(&[dest.clone()]).expect("drain dest");
        if let Value::Tuple(values) = drained {
            assert_eq!(values.len(), 2);
        } else {
            panic!("expected tuple");
        }
        mailbox_close(&[source]).unwrap();
        mailbox_close(&[dest]).unwrap();
    }

    #[test]
    fn mailbox_batch_and_flush() {
        let handle = mailbox_create(&[]).expect("mailbox");
        mailbox_send(&[handle.clone(), Value::Int(1.into())]).unwrap();
        mailbox_send(&[handle.clone(), Value::Int(2.into())]).unwrap();
        mailbox_send(&[handle.clone(), Value::Int(3.into())]).unwrap();
        let batch = mailbox_recv_batch(&[handle.clone(), Value::Int(2.into())]).unwrap();
        if let Value::Tuple(values) = &batch {
            assert_eq!(values.len(), 2);
        } else {
            panic!("expected batch tuple");
        }
        mailbox_send(&[handle.clone(), Value::Int(4.into())]).unwrap();
        let flushed = mailbox_flush(&[handle.clone()]).unwrap();
        if let Value::Tuple(parts) = flushed {
            assert_eq!(parts.len(), 2);
            assert_eq!(parts[1], Value::Bool(true));
        } else {
            panic!("expected flush tuple");
        }
    }

    #[test]
    fn mailbox_stats_report_pending_and_closed() {
        let handle = mailbox_create(&[]).expect("mailbox");
        mailbox_send(&[handle.clone(), Value::Int(7.into())]).unwrap();
        let stats = mailbox_stats(&[handle.clone()]).expect("stats");
        if let Value::Tuple(values) = stats {
            assert_eq!(values.len(), 2);
            assert_eq!(values[0], Value::Int(1.into()));
            assert_eq!(values[1], Value::Bool(false));
        } else {
            panic!("expected stats tuple");
        }
        mailbox_close(&[handle.clone()]).unwrap();
        let stats_after = mailbox_stats(&[handle.clone()]).expect("stats closed");
        if let Value::Tuple(values) = stats_after {
            assert_eq!(values[1], Value::Bool(true));
        } else {
            panic!("expected stats tuple");
        }
    }

    #[test]
    fn mailbox_send_batch_and_recv_any() {
        let left = mailbox_create(&[]).expect("left");
        let right = mailbox_create(&[]).expect("right");
        let payloads = Value::Tuple(vec![Value::Int(5.into()), Value::Int(6.into())]);
        let sent = mailbox_send_batch(&[right.clone(), payloads]).expect("batch");
        assert_eq!(sent, Value::Int(2.into()));
        mailbox_send(&[left.clone(), Value::Int(99.into())]).expect("send single");
        let winner = mailbox_recv_any(&[left.clone(), right.clone()]).expect("recv any");
        if let Value::Tuple(values) = winner {
            assert_eq!(values.len(), 2);
            // The first entry contains the handle tuple.
            assert!(matches!(&values[0], Value::Tuple(items) if !items.is_empty()));
        } else {
            panic!("expected select tuple");
        }
        mailbox_close(&[left]).unwrap();
        mailbox_close(&[right]).unwrap();
    }
}

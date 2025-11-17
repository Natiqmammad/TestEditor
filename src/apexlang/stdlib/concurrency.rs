use std::collections::HashMap;
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
                NativeCallable::new(concat!("concurrency::", $name), $func),
            );
        };
    }

    add!(&mut functions, "spawn", spawn_task);
    add!(&mut functions, "join", join_task);
    add!(&mut functions, "pending", pending_tasks);
    add!(&mut functions, "yield_now", yield_now);
    add!(&mut functions, "mailbox_create", mailbox_create);
    add!(&mut functions, "mailbox_send", mailbox_send);
    add!(&mut functions, "mailbox_recv", mailbox_recv);
    add!(&mut functions, "mailbox_try_recv", mailbox_try_recv);
    registry.register_module("concurrency", functions);
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
}

#[derive(Default)]
struct MailboxRegistry {
    next_id: u64,
    boxes: HashMap<u64, Mailbox>,
}

static MAILBOXES: Lazy<Mutex<MailboxRegistry>> =
    Lazy::new(|| Mutex::new(MailboxRegistry::default()));

fn spawn_task(args: &[Value]) -> Result<Value, ApexError> {
    let name = expect_string_arg(args, 0, "concurrency.spawn")?;
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
        other => {
            return Err(ApexError::new(format!(
                "Unknown concurrency task '{}'",
                other
            )))
        }
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
    let id = expect_int_arg(args, 0, "concurrency.join")?;
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

fn mailbox_create(_args: &[Value]) -> Result<Value, ApexError> {
    let handle = {
        let mut registry = MAILBOXES
            .lock()
            .map_err(|_| ApexError::new("Mailbox registry lock poisoned"))?;
        let id = registry.next_id;
        registry.next_id += 1;
        let (sender, receiver) = mpsc::channel();
        registry.boxes.insert(
            id,
            Mailbox {
                sender,
                receiver: Arc::new(Mutex::new(receiver)),
            },
        );
        Value::Tuple(vec![Value::Int(BigInt::from(id))])
    };
    Ok(handle)
}

fn mailbox_send(args: &[Value]) -> Result<Value, ApexError> {
    let handle = expect_mailbox_handle(args, 0, "concurrency.mailbox_send")?;
    let payload = args
        .get(1)
        .cloned()
        .ok_or_else(|| ApexError::new("concurrency.mailbox_send expects a payload"))?;
    let sender = {
        let registry = MAILBOXES
            .lock()
            .map_err(|_| ApexError::new("Mailbox registry lock poisoned"))?;
        registry
            .boxes
            .get(&handle)
            .map(|entry| entry.sender.clone())
            .ok_or_else(|| ApexError::new("Unknown mailbox handle"))?
    };
    sender
        .send(payload)
        .map_err(|_| ApexError::new("Mailbox receiver disconnected"))?;
    Ok(Value::Bool(true))
}

fn mailbox_recv(args: &[Value]) -> Result<Value, ApexError> {
    let handle = expect_mailbox_handle(args, 0, "concurrency.mailbox_recv")?;
    let receiver = get_mailbox_receiver(handle)?;
    let value = receiver
        .lock()
        .map_err(|_| ApexError::new("Mailbox receiver lock poisoned"))?
        .recv()
        .map_err(|_| ApexError::new("Mailbox receiver disconnected"))?;
    Ok(value)
}

fn mailbox_try_recv(args: &[Value]) -> Result<Value, ApexError> {
    let handle = expect_mailbox_handle(args, 0, "concurrency.mailbox_try_recv")?;
    let receiver = get_mailbox_receiver(handle)?;
    let guard = receiver
        .lock()
        .map_err(|_| ApexError::new("Mailbox receiver lock poisoned"))?;
    match guard.try_recv() {
        Ok(value) => Ok(Value::Tuple(vec![Value::Bool(true), value])),
        Err(mpsc::TryRecvError::Empty) => {
            Ok(Value::Tuple(vec![Value::Bool(false), Value::Bool(false)]))
        }
        Err(mpsc::TryRecvError::Disconnected) => {
            Err(ApexError::new("Mailbox receiver disconnected"))
        }
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

fn get_mailbox_receiver(handle: u64) -> Result<Arc<Mutex<mpsc::Receiver<Value>>>, ApexError> {
    let registry = MAILBOXES
        .lock()
        .map_err(|_| ApexError::new("Mailbox registry lock poisoned"))?;
    registry
        .boxes
        .get(&handle)
        .map(|entry| entry.receiver.clone())
        .ok_or_else(|| ApexError::new("Unknown mailbox handle"))
}

fn expect_mailbox_handle(args: &[Value], index: usize, name: &str) -> Result<u64, ApexError> {
    let tuple = expect_tuple_arg(args, index, name)?;
    if tuple.len() != 1 {
        return Err(ApexError::new(format!(
            "{} expects a mailbox tuple handle",
            name
        )));
    }
    match &tuple[0] {
        Value::Int(value) => value
            .to_u64()
            .ok_or_else(|| ApexError::new(format!("{} handle is too large", name))),
        _ => Err(ApexError::new(format!(
            "{} expects a numeric mailbox handle",
            name
        ))),
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
}

//! Points where a test runs its own code
//! in the middle of an operation, to replay a race between two threads.

use std::cell::RefCell;
use std::collections::HashMap;

type Hook = Box<dyn FnOnce()>;

thread_local! {
    static HOOKS: RefCell<HashMap<&'static str, Hook>> = RefCell::new(HashMap::new());
}

/// Runs `hook` the first time this thread reaches the point `name`.
pub(crate) fn once(name: &'static str, hook: impl FnOnce() + 'static) {
    HOOKS.with(|hooks| hooks.borrow_mut().insert(name, Box::new(hook)));
}

/// Runs and removes the hook this thread set for `name`, if any.
pub(crate) fn run(name: &'static str) {
    let hook = HOOKS.with(|hooks| hooks.borrow_mut().remove(name));
    if let Some(hook) = hook {
        hook();
    }
}

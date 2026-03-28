//! Reactive state management: observer pattern, no-alloc listener storage.

/// Observer trait for state changes.
pub trait StateListener<T> {
    fn on_change(&mut self, new_value: &T);
}

/// State holds a value and notifies registered listeners on every mutation.
///
/// Uses `heapless::Vec` so no heap allocation is required. The const generic
/// `N` controls the maximum number of listeners (default: 4).
pub struct State<T: 'static, const N: usize = 4> {
    value: T,
    listeners: heapless::Vec<&'static mut dyn StateListener<T>, N>,
}

impl<T: 'static, const N: usize> State<T, N> {
    /// Create a new `State` with the given initial value and no listeners.
    pub fn new(value: T) -> Self {
        Self {
            value,
            listeners: heapless::Vec::new(),
        }
    }

    /// Read the current value.
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Update the value and notify all registered listeners.
    pub fn set(&mut self, value: T) {
        self.value = value;
        // Split the borrow: iterate listeners separately from value.
        let value_ref = &self.value;
        for listener in self.listeners.iter_mut() {
            let l: &mut dyn StateListener<T> = *listener;
            l.on_change(value_ref);
        }
    }

    /// Register a static listener for state changes.
    /// Returns `Err(())` if the listener storage is full.
    pub fn add_listener(
        &mut self,
        listener: &'static mut dyn StateListener<T>,
    ) -> Result<(), ()> {
        self.listeners.push(listener).map_err(|_| ())
    }
}

/// A no-op listener used in examples and tests.
pub struct DummyListener;

impl StateListener<u32> for DummyListener {
    fn on_change(&mut self, new_value: &u32) {
        let _ = new_value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CountingListener {
        count: u32,
        last: u32,
    }

    impl StateListener<u32> for CountingListener {
        fn on_change(&mut self, new_value: &u32) {
            self.count += 1;
            self.last = *new_value;
        }
    }

    #[test]
    fn test_state_get_set() {
        let mut s: State<u32> = State::new(0);
        assert_eq!(*s.get(), 0);
        s.set(42);
        assert_eq!(*s.get(), 42);
    }

    #[test]
    fn test_state_listener_notified() {
        static mut LISTENER: CountingListener = CountingListener { count: 0, last: 0 };
        let mut s: State<u32, 2> = State::new(0);
        // SAFETY: test is single-threaded; static mut is only accessed here.
        s.add_listener(unsafe { &mut *core::ptr::addr_of_mut!(LISTENER) }).unwrap();
        s.set(7);
        unsafe {
            let count = core::ptr::addr_of!((*core::ptr::addr_of!(LISTENER)).count).read();
            let last = core::ptr::addr_of!((*core::ptr::addr_of!(LISTENER)).last).read();
            assert_eq!(count, 1);
            assert_eq!(last, 7);
        }
    }
}

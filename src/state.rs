//! Reactive state management

/// A simple observer trait for state changes.
pub trait StateListener<T> {
    fn on_change(&mut self, new_value: &T);
}

/// State holds a value and notifies listeners on change.
/// Uses heapless::Vec for no-alloc listener storage.
pub struct State<T, const N: usize = 4> {
    value: T,
    listeners: heapless::Vec<&'static mut dyn StateListener<T>, N>,
}

impl<T, const N: usize> State<T, N> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            listeners: heapless::Vec::new(),
        }
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn set(&mut self, value: T) {
        self.value = value;
        for listener in self.listeners.iter_mut() {
            listener.on_change(&self.value);
        }
    }

    /// Register a listener for state changes.
    pub fn add_listener(&mut self, listener: &'static mut dyn StateListener<T>) -> Result<(), ()> {
        self.listeners.push(listener).map_err(|_| ())
    }
}

// Example listener implementation
pub struct DummyListener;
impl StateListener<u32> for DummyListener {
    fn on_change(&mut self, new_value: &u32) {
        // Handle the state change (e.g., trigger redraw)
        let _ = new_value;
    }
}

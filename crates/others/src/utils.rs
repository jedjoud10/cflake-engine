use std::cell::Cell;

thread_local! {
    static MAIN_THREAD: Cell<bool> = Cell::new(false);
}

// We are on the main thread
pub fn set_main_thread() {
    MAIN_THREAD.with(|x| x.set(true));
}

// Check if we are on the main thread or not
pub fn on_main_thread() -> bool {
    MAIN_THREAD.with(|x| x.get())
}

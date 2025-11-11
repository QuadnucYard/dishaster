use std::sync::atomic::AtomicBool;

static PANIC_CAUGHT: AtomicBool = AtomicBool::new(false);

pub fn has_panic_occurred() -> bool {
    PANIC_CAUGHT.load(std::sync::atomic::Ordering::SeqCst)
}

pub fn init_backtrace_handle() {
    // godot_print!("Install backtrace handle");
    #[cfg(feature = "debug")]
    install_backtrace();
    init_panic_hook();
}

#[cfg(feature = "debug")]
fn install_backtrace() {
    use color_backtrace::{BacktracePrinter, default_output_stream};

    BacktracePrinter::new()
        .add_frame_filter(Box::new(|frames| {
            frames.retain(|frame| !frame.is_dependency_code() && frame.filename.is_some());
        }))
        .install(default_output_stream());
}

fn init_panic_hook() {
    let old_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        PANIC_CAUGHT.store(true, std::sync::atomic::Ordering::SeqCst);

        (*(old_hook.as_ref()))(panic_info);
    }));
}

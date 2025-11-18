use std::sync::{Mutex, atomic::AtomicBool};

static PANIC_CAUGHT: AtomicBool = AtomicBool::new(false);
static PANIC_MESSAGE: Mutex<Option<String>> = Mutex::new(None);

pub fn has_panic_occurred() -> bool {
    PANIC_CAUGHT.load(std::sync::atomic::Ordering::SeqCst)
}

/// Get the panic message if a panic occurred
pub fn get_panic_message() -> Option<String> {
    PANIC_MESSAGE.lock().ok()?.clone()
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

        // Capture panic message
        let message = format_backtrace_string().unwrap_or_default();
        if let Ok(mut msg) = PANIC_MESSAGE.lock() {
            *msg = Some(message);
        }

        (*(old_hook.as_ref()))(panic_info);
    }));
}

fn format_backtrace_string() -> Result<String, std::io::Error> {
    use backtrace::Backtrace;
    use color_backtrace::{BacktracePrinter, termcolor::NoColor};

    let mut out = NoColor::new(vec![]);
    BacktracePrinter::new().print_trace(&Backtrace::new(), &mut out)?;
    Ok(String::from_utf8(out.into_inner()).unwrap())
}

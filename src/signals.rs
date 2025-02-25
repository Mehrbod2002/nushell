use crossterm::{
    event::{self, KeyCode, KeyEvent},
    terminal, ExecutableCommand as _,
};
use nu_protocol::{engine::EngineState, Handlers, SignalAction, Signals};
use reedline::KeyModifiers;
use signal_hook::consts::{SIGINT, SIGTERM, SIGTSTP};
use signal_hook::iterator::Signals as SignalHook;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::{io, thread, time::Duration};

static INTERRUPT_FLAG: Mutex<Option<Arc<AtomicBool>>> = Mutex::new(None);

pub(crate) fn ctrl_protection(engine_state: &mut EngineState) {
    let pause_flag = Arc::new(AtomicBool::new(false));
    let interrupt = Arc::new(AtomicBool::new(false));
    engine_state.set_signals(Signals::new(interrupt.clone(), pause_flag.clone()));

    let signal_handlers = Handlers::new();
    engine_state.signal_handlers = Some(signal_handlers.clone());

    {
        let mut interrupt_guard = INTERRUPT_FLAG.lock().unwrap();
        *interrupt_guard = Some(interrupt.clone());
    }

    let interrupt_clone = interrupt.clone();
    let signal_handlers_clone = signal_handlers.clone();

    // Start a thread to listen for signals like SIGINT, SIGTERM, SIGTSTP
    thread::spawn(move || {
        if let Ok(mut signals) = SignalHook::new([SIGINT, SIGTERM, SIGTSTP]) {
            for signal in signals.forever() {
                match signal {
                    SIGTERM => {
                        pause_flag.store(true, Ordering::Relaxed);
                        signal_handlers.run(SignalAction::Pause);
                    }
                    SIGINT => {
                        interrupt_clone.store(true, Ordering::Relaxed);
                        signal_handlers.run(SignalAction::Interrupt);
                    }
                    SIGTSTP => {
                        interrupt_clone.store(true, Ordering::Relaxed);
                        signal_handlers.run(SignalAction::Pause);
                    }
                    _ => unreachable!(),
                }
            }
        }
    });

    terminal::enable_raw_mode().unwrap();

    let mut stdout = io::stdout();
    stdout
        .execute(crossterm::terminal::Clear(
            crossterm::terminal::ClearType::All,
        ))
        .unwrap();

    thread::spawn(move || loop {
        if event::poll(Duration::from_millis(100)).unwrap() {
            if let event::Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event::read().unwrap()
            {
                if code == KeyCode::Char('z') && modifiers == KeyModifiers::CONTROL {
                    interrupt.store(true, Ordering::Relaxed);
                    signal_handlers_clone.run(SignalAction::Pause);
                }
            }
        }
    });
}

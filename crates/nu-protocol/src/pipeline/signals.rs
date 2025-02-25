use crate::{ShellError, Span};
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

/// Used to check for signals to suspend or terminate the execution of Nushell code.
///
/// Supports interruption (Ctrl+C or SIGINT) and pausing (Ctrl+Z or SIGTSTP).
#[derive(Debug, Clone)]
pub struct Signals {
    pub interrupt: Option<Arc<AtomicBool>>, // Tracks Ctrl+C (SIGINT)
    pub pause: Option<Arc<AtomicBool>>,     // Tracks Ctrl+Z (SIGTSTP)
}

impl Signals {
    /// A [`Signals`] that is not hooked up to any event/signals source.
    ///
    /// This [`Signals`] will never be interrupted or paused.
    pub const EMPTY: Self = Signals {
        interrupt: None,
        pause: None,
    };

    /// Create a new [`Signals`] with `ctrlc` and `ctrlz` as signal sources.
    ///
    /// Once `ctrlc` is set to `true`, [`check`](Self::check) will error.
    /// If `ctrlz` is set to `true`, [`paused`](Self::paused) will return `true`.
    pub fn new(ctrlc: Arc<AtomicBool>, ctrlz: Arc<AtomicBool>) -> Self {
        Self {
            interrupt: Some(ctrlc),
            pause: Some(ctrlz),
        }
    }

    /// Create a [`Signals`] that is not hooked up to any event/signals source.
    ///
    /// So, the returned [`Signals`] will never be interrupted or paused.
    pub const fn empty() -> Self {
        Self::EMPTY
    }

    /// Returns an `Err` if an interrupt has been triggered.
    ///
    /// Otherwise, returns `Ok`.
    #[inline]
    pub fn check(&self, span: Span) -> Result<(), ShellError> {
        #[inline]
        #[cold]
        fn interrupt_error(span: Span) -> Result<(), ShellError> {
            Err(ShellError::Interrupted { span })
        }

        #[inline]
        #[cold]
        fn pause_error(span: Span) -> Result<(), ShellError> {
            Err(ShellError::SuspendedByUser { span: Some(span) })
        }

        if self.interrupted() {
            interrupt_error(span)
        } else if self.paused() {
            println!("paused");
            pause_error(span)
        } else {
            Ok(())
        }
    }

    /// Triggers an interrupt (e.g., when Ctrl+C is pressed).
    pub fn trigger(&self) {
        if let Some(interrupt) = &self.interrupt {
            interrupt.store(true, Ordering::SeqCst);
        }
    }

    /// Returns whether an interrupt has been triggered.
    #[inline]
    pub fn interrupted(&self) -> bool {
        self.interrupt
            .as_deref()
            .is_some_and(|b| b.load(Ordering::Acquire))
    }

    /// Triggers a pause (e.g., when Ctrl+Z is pressed).
    pub fn trigger_pause(&self) {
        if let Some(pause) = &self.pause {
            pause.store(true, Ordering::SeqCst);
        }
    }

    /// Returns whether a pause has been triggered.
    #[inline]
    pub fn paused(&self) -> bool {
        self.pause
            .as_deref()
            .is_some_and(|b| b.load(Ordering::Acquire))
    }

    /// Resets both interrupt and pause signals.
    pub fn reset(&self) {
        if let Some(interrupt) = &self.interrupt {
            interrupt.store(false, Ordering::Relaxed);
        }
        if let Some(pause) = &self.pause {
            pause.store(false, Ordering::Relaxed);
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.interrupt.is_none() && self.pause.is_none()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalAction {
    Interrupt,
    Reset,
    Pause,
}

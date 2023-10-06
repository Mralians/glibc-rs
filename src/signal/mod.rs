extern crate nix;

use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, Signal};
use nix::sys::signalfd::SigSet;
use nix::unistd::close;
use std::io::{self, Write};
use std::process;

pub unsafe fn abort() {
    // Unblock SIGABRT
    let mut sig_mask = nix::sys::signal::SigSet::empty();
    sig_mask.add(Signal::SIGABRT);

    match nix::sys::signal::sigprocmask(
        nix::sys::signal::SigmaskHow::SIG_UNBLOCK,
        Some(&sig_mask),
        None,
    ) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("sigprocmask: {e}");
            process::exit(libc::EXIT_FAILURE);
        }
    }

    // Register custom handler for SIGABRT
    let sa = SigAction::new(SigHandler::SigDfl, SaFlags::empty(), SigSet::empty());
    match sigaction(Signal::SIGABRT, &sa) {
        Ok(v) => {
            if v.handler() != SigHandler::SigIgn {
                match sigaction(Signal::SIGABRT, &v) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("sigaction: {e}");
                        process::exit(libc::EXIT_FAILURE);
                    }
                }
            }
            if v.handler() != SigHandler::SigDfl {
                nix::sys::signal::raise(Signal::SIGABRT).expect("raise");
                let sa = SigAction::new(SigHandler::SigDfl, SaFlags::empty(), SigSet::empty());
                match sigaction(Signal::SIGABRT, &sa) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("sigaction: {e}");
                        process::exit(libc::EXIT_FAILURE);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("sigaction: {e}");
            process::exit(libc::EXIT_FAILURE);
        }
    }
    cleanup_and_raise_sigabort();
}

fn cleanup_and_raise_sigabort() {
    // Flush stdout and stderr
    if io::stdout().flush().is_err() || io::stderr().flush().is_err() {
        eprintln!("fflush");
        process::exit(libc::EXIT_FAILURE);
    }

    // Close stdout and stderr
    match close(libc::STDOUT_FILENO) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("close: {e}");
            process::exit(libc::EXIT_FAILURE);
        }
    }
    match close(libc::STDERR_FILENO) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("close: {e}");
            process::exit(libc::EXIT_FAILURE);
        }
    }
    // Raise SIGABRT
    nix::sys::signal::raise(Signal::SIGABRT).expect("raise");
}


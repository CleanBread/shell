use anyhow::Result;
use libc::{POLLIN, SIGCHLD, WNOHANG, c_int, fork, pollfd};
use os_pipe::pipe;
use std::io::{Write, stderr, stdin, stdout};
use std::os::unix::io::AsRawFd;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::mpsc;
use std::thread;
use termion::{cursor, event::Key, input::TermRead, raw::IntoRawMode};

use crate::{
    command::Command,
    jobs::{JOBS, Job},
    utils::{get_input, get_paths},
};

mod command;
mod execute_output;
mod jobs;
mod redirection;
mod utils;

static SIGCHLD_PIPE_WRITE: AtomicI32 = AtomicI32::new(-1);

extern "C" fn sigchld_handler(_: c_int) {
    let fd = SIGCHLD_PIPE_WRITE.load(Ordering::Relaxed);

    if fd >= 0 {
        unsafe { libc::write(fd, b"\x00".as_ptr() as *const libc::c_void, 1) };
    }
}

pub fn run() -> Result<()> {
    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut stderr = stderr().into_raw_mode().unwrap();

    write!(
        stdout,
        "{}{}",
        termion::clear::All,
        cursor::BlinkingUnderline,
    )
    .unwrap();
    stdout.flush().unwrap();

    let (sigchld_reader, sigchld_writer) = pipe()?;
    let sigchld_rfd = sigchld_reader.as_raw_fd();
    let sigchld_wfd = sigchld_writer.as_raw_fd();

    SIGCHLD_PIPE_WRITE.store(sigchld_wfd, Ordering::Relaxed);
    unsafe { libc::signal(SIGCHLD, sigchld_handler as *const () as libc::sighandler_t) };

    let (key_tx, key_rx) = mpsc::channel::<Key>();
    thread::spawn(move || {
        for key in stdin().keys().flatten() {
            if key_tx.send(key).is_err() {
                break;
            }
        }
    });

    let (job_tx, job_rx) = mpsc::channel::<i32>();

    thread::spawn(move || {
        loop {
            let mut fds = [pollfd {
                fd: sigchld_rfd,
                events: POLLIN,
                revents: 0,
            }];
            unsafe { libc::poll(fds.as_mut_ptr(), 1, -1) };

            let mut buf = [0u8; 64];
            unsafe { libc::read(sigchld_rfd, buf.as_mut_ptr() as *mut libc::c_void, 64) };

            loop {
                let pid = unsafe { libc::waitpid(-1, std::ptr::null_mut(), WNOHANG) };
                if pid <= 0 {
                    break;
                }
                let mut jobs = JOBS.lock().unwrap();

                job_tx.send(pid).ok();

                let job_num = jobs
                    .items
                    .iter()
                    .find(|(_, job)| job.pid == pid)
                    .map(|(num, _)| *num);

                if let Some(num) = job_num {
                    let job = jobs.items.shift_remove(&num).unwrap();

                    if jobs.next_counter > num {
                        jobs.next_counter = num;
                    }

                    write!(
                        std::io::stdout(),
                        "\r[{}]   done       {}\r\n",
                        num,
                        job.command
                    )
                    .unwrap();
                    std::io::stdout().flush().unwrap();
                }
            }
        }
    });

    let paths = get_paths()?;

    loop {
        let input = get_input(&key_rx, &job_rx)?;

        if input.is_empty() {
            continue;
        }

        let (and_chain, is_background_job) = Command::parse_input(&input);

        if is_background_job {
            let pid = unsafe { fork() };

            if pid < 0 {
                write!(stdout, "\r\nfork failed\r\n")?;
            } else if pid == 0 {
                unsafe {
                    libc::close(sigchld_rfd);
                    libc::close(sigchld_wfd);
                }
                let _ = Command::execute(&paths, and_chain, true);
                std::process::exit(0);
            } else {
                let mut jobs = JOBS.lock().unwrap();
                let job_num = jobs.next_counter;
                jobs.items.insert(
                    job_num,
                    Job {
                        pid,
                        command: input.clone(),
                    },
                );

                let mut next_counter: u32 = 1;
                while jobs.items.contains_key(&next_counter) {
                    next_counter += 1;
                }
                jobs.next_counter = next_counter;

                write!(stdout, "[{}] {}\r\n", job_num, pid)?;
            }
        } else {
            if !Command::execute(&paths, and_chain, false)? {
                stdout.flush()?;
                stderr.flush()?;

                break;
            };
        }
    }

    Ok(())
}

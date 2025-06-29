use std::ffi::CString;
use std::os::fd::OwnedFd;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use crate::get_global_config_ref;
use crate::record_collection::collection::CompiledRecord;
use crate::record_collection::collection::TestRecord;

use nix::libc::WEXITSTATUS;
use nix::libc::WIFEXITED;
use nix::libc::WIFSIGNALED;
use nix::libc::WIFSTOPPED;
use nix::libc::WNOHANG;
use nix::libc::WSTOPSIG;
use nix::libc::WTERMSIG;
use nix::libc::posix_spawn;
use nix::libc::posix_spawn_file_actions_destroy;
use nix::libc::waitpid;
use nix::libc::{
    self, posix_spawn_file_actions_adddup2, posix_spawn_file_actions_init,
    posix_spawn_file_actions_t,
};
use termion::color;

use crate::collect::FileCollection;



#[derive(Debug)]
enum SpawnErr {
    FailedToConvertCChar,
    SpawnChildFailed,
}

#[derive(Debug)]
enum ProcessErr {
    FailedExit,
    Crashed,
    Stopped,
    UndefinedTermination,
}

type PidsTrack = (Box<[libc::pid_t]>, Box<[usize]>);

fn init_pipe_with_file_action() -> (Vec<OwnedFd>, Vec<OwnedFd>, Vec<posix_spawn_file_actions_t>) {
    let default_pool_count = get_global_config_ref().process.max_child_spawn;

    let mut readfd_list = Vec::<OwnedFd>::with_capacity(default_pool_count);
    let mut writefd_list = Vec::<OwnedFd>::with_capacity(default_pool_count);
    let mut action_files: Vec<posix_spawn_file_actions_t> = Vec::with_capacity(default_pool_count);

    for _ in 0..default_pool_count {
        let (readfd, writefd) = unistd::pipe().expect("Failed to create pipelines");

        action_files.push(file_action_t_init(&readfd, &writefd));

        readfd_list.push(readfd);
        writefd_list.push(writefd);
    }

    (readfd_list, writefd_list, action_files)
}

fn file_action_t_init(readfd: &OwnedFd, writefd: &OwnedFd) -> posix_spawn_file_actions_t {
    let mut file_action: posix_spawn_file_actions_t = unsafe { std::mem::zeroed() };

    use std::os::fd::AsRawFd;

    use nix::libc::posix_spawn_file_actions_addclose;

    let raw_file_action: *mut _ = &mut file_action;
    unsafe {
        posix_spawn_file_actions_init(raw_file_action);

        posix_spawn_file_actions_adddup2(raw_file_action, writefd.as_raw_fd(), libc::STDOUT_FILENO);

        posix_spawn_file_actions_addclose(raw_file_action, writefd.as_raw_fd());
        posix_spawn_file_actions_addclose(raw_file_action, readfd.as_raw_fd());
    }

    file_action
}

use nix::unistd;
use std::sync::Arc;

fn spawn_process(
    pid: &mut libc::pid_t,
    exe_str: &str,
    file_action: &posix_spawn_file_actions_t,
) -> Result<(), SpawnErr> {
    let pid_ref = pid as *mut _;

    let exe = CString::new(exe_str).map_err(|_| SpawnErr::FailedToConvertCChar)?;

    let file_action = file_action as *const _;

    let argv = [exe.as_ptr(), std::ptr::null()];

    let ret;
    unsafe {
        ret = posix_spawn(
            pid_ref,
            exe.as_ptr(),
            file_action,
            std::ptr::null(),
            argv.as_ptr() as *mut _,
            std::ptr::null(),
        );
    }

    if ret != 0 {
        return Err(SpawnErr::SpawnChildFailed);
    }

    Ok(())
}

fn read_pid_status(pid: &libc::pid_t, origin: &str) -> Result<bool, ProcessErr> {
    let mut status: libc::c_int = 0;

    let pid_r = unsafe { waitpid(*pid, &mut status as *mut _, WNOHANG) };

    if pid_r == 0 {
        //process still runing

        println!(
            "{}[ Running... ] {}{}",
            termion::color::Fg(color::Rgb(255, 195, 51)),
            origin,
            termion::color::Fg(color::Reset)
        );
        return Ok(false);
    }

    if WIFEXITED(status) {
        // Process returned from main

        let exit_stat = WEXITSTATUS(status);
        if exit_stat == 0 {
            println!(
                "{}[ Finished Executing - {} ]{}",
                termion::color::Fg(color::Green),
                origin,
                termion::color::Fg(color::Reset)
            );

            return Ok(true);
        } else {
            println!(
                "{}[ Failed - {} ]{}",
                termion::color::Fg(color::Red),
                origin,
                termion::color::Fg(color::Reset)
            );

            return Err(ProcessErr::FailedExit);
        }
    } else if WIFSIGNALED(status) {
        // Process crashed from segfault

        let signal = WTERMSIG(status);
        println!(
            "{}[ Process Crashed: Origin:{}, Signal:{} ]{}",
            termion::color::Fg(color::Red),
            origin,
            signal,
            termion::color::Fg(color::Reset)
        );
        return Err(ProcessErr::Crashed);
    } else if WIFSTOPPED(status) {
        let signal = WSTOPSIG(status);
        println!(
            "{}[ Process Stopped: Origin:{}, Signal:{} ]{}",
            termion::color::Fg(color::Red),
            origin,
            signal,
            termion::color::Fg(color::Reset)
        );
        return Err(ProcessErr::Stopped);
    }

    Err(ProcessErr::UndefinedTermination)
}

fn fill_spawn_pool(
    pid: &mut libc::pid_t,
    pid_index_ref: &mut usize,
    file_details: (usize, &(String, String)),
    file_action: &posix_spawn_file_actions_t,
    shared_collection: &mut TestRecord,
) {
    //spawn new process
    let stat = spawn_process(pid, file_details.1.1.as_str(), &file_action);

    match stat {
        Ok(_) => {
            println!(
                "{}[ Executing: {} ]{}",
                color::Fg(color::Rgb(255, 195, 51)),
                file_details.1.0,
                color::Fg(color::Reset)
            );
            *pid_index_ref = file_details.0;

            let _ = shared_collection.register_process(file_details.1.1.trim().to_owned());
        }
        Err(e) => {
            println!(
                "{}[ Set-up Failed: {} [{:?}]]{}",
                color::Fg(color::Magenta),
                file_details.0,
                e,
                color::Fg(color::Reset)
            );
        }
    }
}

pub fn spawn_executable(fc: FileCollection) -> Option<CompiledRecord> {
    let pool_limit = get_global_config_ref().process.max_child_spawn;

    let mut test_collection = TestRecord::new();

    let (readfd_list, writefd_list, mut file_actions) = init_pipe_with_file_action();

    let mut pids: PidsTrack = (
        vec![-1; pool_limit].into_boxed_slice(),
        vec![0; pool_limit].into_boxed_slice(),
    );

    let mut file_iter = fc.exe_info.iter().enumerate();

    let flag = Arc::new(AtomicBool::new(true));
    // let reports = Arc::new(TRecord::new());

    //start thread to read from pipline
    let flag_clone = flag.clone();
    let clone_collection = test_collection.clone();
    let pipeline_worker = thread::spawn(move || {
        pipe_handler::read_pipeline(clone_collection, readfd_list, flag_clone)
    });

    let mut executable_left = fc.len();
    while executable_left > 0 {
        for i in 0..pool_limit {
            if pids.0[i] != -1 {
                continue;
            }
            let Some(file_detials) = file_iter.next() else {
                break;
            };

            fill_spawn_pool(
                &mut pids.0[i],
                &mut pids.1[i],
                file_detials,
                &file_actions[i],
                &mut test_collection,
            );
        }

        for i in 0..pool_limit {
            if pids.0[i] == -1 {
                continue;
            }

            //Get proccesses progress
            sleep(Duration::from_millis(500));
            let stat = { read_pid_status(&pids.0[i], fc.str_file_name_from(pids.1[i])) };

            match stat {
                Ok(res) => {
                    if res {
                        //process finished
                        //store data that program finished
                        pids.0[i] = -1;
                    } else {
                        //process still running
                        continue;
                    }
                }
                Err(_) => {
                    // store in data that program failed
                    pids.0[i] = -1;
                }
            }

            executable_left = executable_left.saturating_sub(1);
        }
    }

    unsafe {
        while let Some(mut fa) = file_actions.pop() {
            posix_spawn_file_actions_destroy(&mut fa as *mut _);
        }
    }

    drop(writefd_list);
    flag.store(false, Ordering::Relaxed);

    if let Err(e) = pipeline_worker.join() {
        panic!("Pipeline worker panicked: {:?}", e);
    }

    test_collection.compile().ok()
}

mod pipe_handler {
    use std::{
        fmt::Display,
        fs::File,
        io::Read,
        marker::PhantomData,
        os::fd::{FromRawFd, IntoRawFd, OwnedFd},
        sync::{
            Arc, Mutex,
            atomic::{AtomicBool, Ordering},
            mpsc::{self, Receiver, Sender},
        },
    };

    use termion::color;
    use threadpool::ThreadPool;

    use crate::{
        get_global_config_ref,
        record_collection::{
            self, ProcessInfo,
            collection::{StoreData, TestRecord},
        },
    };


    pub fn read_pipeline(
        shared_collection: TestRecord,
        readfds: Vec<OwnedFd>,
        flag: Arc<AtomicBool>,
    ){
        println!("[ ThreadRunner is Listening ]");
        println!("[ Pipeline Reader Active ]");

        // Set all file descrptors as File Objects
        let readfds = set_fd_to_file(readfds);
        let mut bin_buff = Box::new([0u8; std::mem::size_of::<ProcessInfo>()]);

        // create thread pool
        let (tx, threadpool) = ThreadPoolGen::init_threadpool(shared_collection);

        while flag.load(Ordering::Relaxed) {
            for mut fd in readfds.iter() {
                if fd.read_exact(bin_buff.as_mut()).is_err() {
                    continue;
                };

                let payload = record_collection::bin_convert(&bin_buff);
                // send job to threadpool
                if tx.send(payload).is_err() {
                    println!("Unable to send Thread Jobs!");
                    break;
                }
            }
        }

        println!("[ Draining remaining pipe content ]");

        let mut clean = true;
        while clean {
            clean = false;

            for mut fd in readfds.iter() {
                if fd.read_exact(bin_buff.as_mut()).is_err() {
                    continue;
                };

                clean = true;

                let payload = record_collection::bin_convert(&bin_buff);

                // send job to threadpool
                if tx.send(payload).is_err() {
                    println!("Unable to send Thread Jobs!");
                    break;
                }
            }
        }

        println!("[ Closing Pipeline Reader ]");

        drop(tx);
        threadpool.join();

        ()
    }

    fn set_fd_to_file(readfds: Vec<OwnedFd>) -> Vec<File> {
        readfds
            .into_iter()
            .map(|i| unsafe { File::from_raw_fd(i.into_raw_fd()) })
            .collect()
    }

    struct ThreadPoolGen<T, S> {
        _phantom: PhantomData<(T, S)>,
    }

    impl<T, S> ThreadPoolGen<T, S>
    where
        T: Send + Sync + 'static,
        S: StoreData<T = T> + Clone + Send + Sync + 'static,
    {
        pub fn init_threadpool(op: S) -> (Sender<T>, ThreadPool) {
            let config_worker_count = get_global_config_ref().process.worker_count;

            let mut tp = ThreadPool::new(config_worker_count);
            let (tx, rx) = mpsc::channel::<T>();

            Self::init_jobs(rx, &mut tp, config_worker_count, op);

            (tx, tp)
        }

        fn init_jobs(
            rx: Receiver<T>,
            thread_pool: &mut ThreadPool,
            config_worker_count: usize,
            shared_collection: S,
        ) {
            let arc_rx = Arc::new(Mutex::new(rx));

            for _ in 0..config_worker_count {
                let clone_rx = arc_rx.clone();
                let clone_collection = shared_collection.clone();

                thread_pool.execute(move || {
                    loop {
                        let Ok(lock) = clone_rx.lock() else {
                            eprint!("Failed to get lock!");
                            break;
                        };

                        let Ok(item) = lock.recv() else {
                            // eprint!("Failed to recieve from sender!\n");
                            break;
                        };

                        let _u = clone_collection.store(item);
                    }
                });
            }
        }
    }

    trait ReportDisplay {}

    impl ReportDisplay for () {}
}

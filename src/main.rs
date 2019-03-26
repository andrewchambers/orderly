mod specs;
use std::time::Duration;

fn die(s: &str) -> ! {
    eprintln!("{}", s);
    std::process::exit(1);
}

struct Supervisor {
    spec: specs::SupervisorSpec,
    procs: Vec<Option<std::process::Child>>,
}

enum StopMethod {
    Gentle,
    Brutal,
}

#[derive(Debug)]
enum SupervisorError {
    IOError(std::io::Error),
    Interrupted,
    RestartLimitReached,
    ProcFailed,
}

#[derive(Debug)]
enum ProcStatus {
    Ok,
    Starting,
    Failed,
}

impl From<std::io::Error> for SupervisorError {
    fn from(e: std::io::Error) -> Self {
        SupervisorError::IOError(e)
    }
}

impl Supervisor {
    fn new(spec: specs::SupervisorSpec) -> Self {
        let mut procs = vec![];
        for _i in spec.procs.iter() {
            procs.push(None);
        }

        Supervisor {
            spec: spec,
            procs: procs,
        }
    }

    fn sleep(&mut self, d: Duration) -> Result<(), SupervisorError> {
        std::thread::sleep(d);
        Ok(())
    }

    fn kill_proc(&mut self, idx: usize) -> Result<(), SupervisorError> {
        let p = &mut self.procs[idx];

        match p {
            Some(c) => {
                match c.kill() {
                    Ok(()) => (),
                    Err(e) => match e.kind() {
                        std::io::ErrorKind::InvalidInput => (),
                        _ => Err(e)?,
                    },
                }
                c.wait()?;
                *p = None;
                Ok(())
            }
            None => Ok(()),
        }
    }

    fn check_proc(&mut self, idx: usize) -> Result<ProcStatus, SupervisorError> {
        let p = &mut self.procs[idx];

        dbg!(idx);

        match p {
            Some(c) => match c.try_wait()? {
                None => {
                    let s = self.spec.procs.get(idx).unwrap();
                    let mut command = std::process::Command::new(s.check.clone());
                    let mut c = command.spawn()?;
                    let rc = c.wait()?.code().unwrap_or(1);

                    match rc {
                        0 => Ok(ProcStatus::Ok),
                        2 => Ok(ProcStatus::Starting),
                        _ => Err(SupervisorError::ProcFailed),
                    }
                }
                Some(_) => {
                    *p = None;
                    Ok(ProcStatus::Failed)
                }
            },
            None => Ok(ProcStatus::Failed),
        }
    }

    fn clean_proc(&mut self, idx: usize) -> Result<(), SupervisorError> {
        {
            let p = &self.procs[idx];

            match p {
                Some(_) => panic!("bug, clean without kill."),
                None => (),
            }
        }

        let s = self.spec.procs.get(idx).unwrap();
        let mut command = std::process::Command::new(s.cleanup.clone());
        let mut c = command.spawn()?;
        let rc = c.wait()?;
        if rc.success() {
            Ok(())
        } else {
            Err(SupervisorError::ProcFailed)
        }
    }

    fn start_proc(&mut self, idx: usize) -> Result<(), SupervisorError> {
        {
            let p = &self.procs[idx];

            match p {
                Some(_) => self.kill_proc(idx)?,
                None => (),
            }
        }

        let s = self.spec.procs.get(idx).unwrap();

        let mut command = std::process::Command::new(s.run.clone());
        let c = command.spawn()?;

        {
            self.procs[idx] = Some(c);
        }

        loop {
            match self.check_proc(idx)? {
                ProcStatus::Ok => {
                    let s = self.spec.procs.get(idx).unwrap();
                    let mut command = std::process::Command::new(s.post.clone());
                    let mut c = command.spawn()?;
                    let rc = c.wait()?.code().unwrap_or(1);
                    if rc != 0 {
                        return Err(SupervisorError::ProcFailed);
                    }
                    return Ok(());
                }
                ProcStatus::Starting => self.sleep(Duration::from_millis(100))?,
                ProcStatus::Failed => return Err(SupervisorError::ProcFailed),
            };
        }
    }

    fn kill_all_procs(&mut self) -> Result<(), SupervisorError> {
        for i in (0..self.procs.len()).rev() {
            self.kill_proc(i)?;
        }
        Ok(())
    }

    fn restart_all_procs(&mut self) -> Result<(), SupervisorError> {
        self.kill_all_procs()?;

        for i in (0..self.procs.len()).rev() {
            self.clean_proc(i)?;
        }

        for i in (0..self.procs.len()).rev() {
            self.start_proc(i)?;
        }

        Ok(())
    }

    fn check_all_procs(&mut self) -> Result<(), SupervisorError> {
        for i in 0..self.procs.len() {
            self.check_proc(i)?;
        }

        Ok(())
    }

    fn one_for_all_run_procs(&mut self) -> SupervisorError {
        match self.restart_all_procs() {
            Ok(()) => (),
            Err(e) => return e,
        }

        loop {
            match self.check_all_procs() {
                Ok(()) => match self.sleep(Duration::from_secs(10)) {
                    Ok(()) => continue,
                    Err(e) => return e,
                },
                Err(e) => return e,
            }
        }
    }

    fn one_for_all_supervise_forever(&mut self) {
        loop {
            match self.one_for_all_run_procs() {
                e @ SupervisorError::IOError(_) | e @ SupervisorError::ProcFailed => {
                    eprintln!("supervisor encountered an error: {:?}", e);
                }
                SupervisorError::Interrupted | SupervisorError::RestartLimitReached => {
                    if let Err(e) = self.kill_all_procs() {
                        eprintln!("Unable kill child procs: {:?}", e)
                    };
                    std::process::exit(1)
                }
            }
        }
    }

    fn supervise_forever(&mut self) {
        match self.spec.strategy {
            specs::SupervisorStrategy::OneForAll => self.one_for_all_supervise_forever(),
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut arg_idx = 1;

    let mut supervisor_spec_builder = specs::SupervisorSpecBuilder::new();
    let mut proc_spec_builder = specs::ProcSpecBuilder::new();

    while arg_idx < args.len() {
        match args[arg_idx].as_ref() {
            "-strategy" => {
                match args
                    .get(arg_idx + 1)
                    .unwrap_or_else(|| die("-strategy expected an argument."))
                    .as_ref()
                {
                    "one-for-all" => {
                        supervisor_spec_builder.set_strategy(specs::SupervisorStrategy::OneForAll)
                    }
                    unknown => die(format!("{} is not a valid -strategy.", unknown).as_ref()),
                }
                arg_idx += 2;
            }
            "-restart-duration" => {
                let seconds = args.get(arg_idx + 1).unwrap_or_else(|| {
                    die("-restart-duration expected an unsigned number of seconds.")
                });

                let seconds = seconds.parse::<u64>().unwrap_or_else(|_e| {
                    die(format!("{} is not a valid unsigned number.", seconds).as_ref())
                });

                supervisor_spec_builder.set_restart_duration(Duration::from_secs(seconds));

                arg_idx += 2;
            }
            "-max-restarts" => {
                let max_restarts = args
                    .get(arg_idx + 1)
                    .unwrap_or_else(|| die("-max-restarts expected an unsigned number."));
                let max_restarts = max_restarts.parse::<usize>().unwrap_or_else(|_e| {
                    die(format!("{} is not a valid unsigned number.", max_restarts).as_ref())
                });
                supervisor_spec_builder.set_max_restarts(max_restarts);
                arg_idx += 2;
            }
            "-status-file" => {
                let status_file = args
                    .get(arg_idx + 1)
                    .unwrap_or_else(|| die("-status_file expected an argument."));
                supervisor_spec_builder.set_status_file(status_file.clone());
                arg_idx += 2;
            }
            "--" => {
                arg_idx += 1;
                break;
            }
            unknown => die(format!("unknown argument: {}.", unknown).as_ref()),
        }
    }

    while arg_idx < args.len() {
        match args.get(arg_idx).unwrap().as_ref() {
            "-name" => {
                let name = args
                    .get(arg_idx + 1)
                    .unwrap_or_else(|| die("-name expected an argument."));

                proc_spec_builder.set_name(name.clone());

                arg_idx += 2;
            }
            "-run" => {
                let check = args
                    .get(arg_idx + 1)
                    .unwrap_or_else(|| die("-run expected an argument."));
                proc_spec_builder.set_run(check.clone());
                arg_idx += 2;
            }
            "-check" => {
                let check = args
                    .get(arg_idx + 1)
                    .unwrap_or_else(|| die("-check expected an argument."));
                proc_spec_builder.set_check(check.clone());
                arg_idx += 2;
            }
            "-post" => {
                let post = args
                    .get(arg_idx + 1)
                    .unwrap_or_else(|| die("-post expected an argument."));
                proc_spec_builder.set_post(post.clone());
                arg_idx += 2;
            }
            "-cleanup" => {
                let cleanup = args
                    .get(arg_idx + 1)
                    .unwrap_or_else(|| die("-cleanup expected an argument."));
                proc_spec_builder.set_cleanup(cleanup.clone());
                arg_idx += 2;
            }
            "--" => {
                match proc_spec_builder.build() {
                    Ok(spec) => {
                        supervisor_spec_builder.add_proc_spec(spec);
                        proc_spec_builder = specs::ProcSpecBuilder::new();
                    }
                    Err(specs::SpecError::MissingField(f)) => {
                        die(format!("proc spec missing field '{}'", f).as_ref())
                    }
                }
                arg_idx += 1;
            }

            unknown => die(format!("unknown process spec argument: {}.", unknown).as_ref()),
        }
    }

    match proc_spec_builder.build() {
        Ok(spec) => supervisor_spec_builder.add_proc_spec(spec),
        Err(specs::SpecError::MissingField(f)) => {
            die(format!("proc spec missing field '{}'", f).as_ref())
        }
    };

    let spec = match supervisor_spec_builder.build() {
        Ok(spec) => spec,
        Err(specs::SpecError::MissingField(f)) => {
            die(format!("supervisor spec missing field '{}'", f).as_ref())
        }
    };

    let mut supervisor = Supervisor::new(spec);
    supervisor.supervise_forever();
}

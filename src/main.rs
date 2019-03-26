mod specs;
use std::io::BufRead;
use std::io::BufReader;
use std::os::unix::process::CommandExt;
use std::time::Duration;

// TODO:
// Embed ronn manual.
// Check wait period option.
// Startup timeout option.
// Graceful shutdown timeout.
// Startup initial check wait.
// Startup check rate option.
// Process groups?
// Write status file.

fn die(s: &str) -> ! {
  log::error!("{}", s);
  std::process::exit(1);
}

struct RateLimiter {
  capacity: f64,
  tokens: f64,
  tokens_per_sec: f64,
  last_add: std::time::Instant,
}

struct Supervisor {
  spec: specs::SupervisorSpec,
  procs: Vec<Option<std::process::Child>>,
  rate_limiter: RateLimiter,
  sigrx: crossbeam_channel::Receiver<Signal>,
}

enum Signal {
  Shutdown,
  Terminate,
}

#[derive(Debug)]
enum SupervisorError {
  IOError(std::io::Error),
  Shutdown,
  Terminated,
  RestartLimitReached,
  ProcFailed,
}

#[derive(Debug)]
enum ProcStatus {
  Starting,
  Running,
}

impl From<std::io::Error> for SupervisorError {
  fn from(e: std::io::Error) -> Self {
    SupervisorError::IOError(e)
  }
}

impl RateLimiter {
  pub fn new(mut capacity: f64, tokens_per_sec: f64) -> Self {
    if capacity < 1.0 {
      capacity = 1.0;
    };
    if tokens_per_sec < 0.0 {
      capacity = 0.0;
    };
    RateLimiter {
      capacity: capacity,
      tokens: capacity,
      tokens_per_sec: tokens_per_sec,
      last_add: std::time::Instant::now(),
    }
  }

  pub fn take(&mut self) -> bool {
    self.add_tokens();

    if self.tokens < 1.0 {
      false
    } else {
      self.tokens -= 1.0;
      true
    }
  }

  fn add_tokens(&mut self) {
    let now = std::time::Instant::now();
    let duration = now.duration_since(self.last_add);
    let millis = duration.as_millis();
    let new_tokens = ((millis as f64) * self.tokens_per_sec) / 1000.0;
    self.tokens += new_tokens;
    if self.tokens > self.capacity {
      self.tokens = self.capacity;
    }
    self.last_add = now;
  }
}

fn run_command_take_last_line(
  command: &str,
  env: &Vec<(String, String)>,
) -> Result<String, SupervisorError> {
  let mut cmd = std::process::Command::new(command);

  cmd.stdin(std::process::Stdio::null());
  cmd.stdout(std::process::Stdio::piped());
  for v in env {
    cmd.env(&v.0, &v.1);
  }
  let mut cmd = cmd.spawn()?;

  let stdout = cmd.stdout.as_mut().unwrap();
  let rdr = BufReader::new(stdout);
  let mut first = true;
  let mut last_line = String::new();
  for line in rdr.lines() {
    let line = line?;
    if !first {
      println!("{}", last_line);
    }
    last_line = line;
    first = false;
  }

  let wait_result = cmd.wait()?;

  if !wait_result.success() {
    return Err(SupervisorError::ProcFailed);
  }

  Ok(last_line)
}

fn run_command(command: &str, env: &Vec<(String, String)>) -> Result<(), SupervisorError> {
  let mut cmd = std::process::Command::new(command);
  cmd.stdin(std::process::Stdio::null());
  for v in env {
    cmd.env(&v.0, &v.1);
  }
  let mut c = cmd.spawn()?;
  let rc = c.wait()?;
  if rc.success() {
    Ok(())
  } else {
    Err(SupervisorError::ProcFailed)
  }
}

impl Supervisor {
  fn new(spec: specs::SupervisorSpec, sigrx: crossbeam_channel::Receiver<Signal>) -> Self {
    let mut procs = vec![];
    for _i in spec.procs.iter() {
      procs.push(None);
    }

    let limiter = RateLimiter::new(spec.max_restarts, spec.restarts_per_second);

    Supervisor {
      spec: spec,
      procs: procs,
      sigrx: sigrx,
      rate_limiter: limiter,
    }
  }

  fn check_signals(&mut self) -> Result<(), SupervisorError> {
    match self.sigrx.try_recv() {
      Ok(Signal::Shutdown) => return Err(SupervisorError::Shutdown),
      Ok(Signal::Terminate) => return Err(SupervisorError::Terminated),
      _ => Ok(()),
    }
  }

  fn sleep(&mut self, d: Duration) -> Result<(), SupervisorError> {
    crossbeam_channel::select! {
      recv(self.sigrx) -> sig => if let Ok(sig) = sig {
        match sig {
          Signal::Shutdown => return Err(SupervisorError::Shutdown),
          Signal::Terminate => return Err(SupervisorError::Terminated),
        }
      } else {
        return Err(SupervisorError::Terminated)
      },
      default(d) => (),
    }
    Ok(())
  }

  fn get_proc_script_env(&mut self, idx: usize) -> Vec<(String, String)> {
    match &self.procs[idx] {
      Some(c) => vec![(String::from("SUPERVISED_PID"), format!("{}", c.id()))],
      None => vec![],
    }
  }

  fn kill_proc(&mut self, idx: usize) -> Result<(), SupervisorError> {
    // Kill is not affected by signals...
    log::info!("killing {}", self.spec.procs[idx].name.as_str());
    let p = &mut self.procs[idx];

    match p {
      Some(c) => {
        let rc = unsafe { libc::killpg(c.id() as i32, libc::SIGKILL) };
        if rc != 0 {
          log::warn!("WARNING - killing process group failed");
        }

        match c.kill() {
          Ok(()) => (),
          Err(e) => match e.kind() {
            std::io::ErrorKind::InvalidInput => (),
            _ => return Err(e)?,
          },
        }
        c.wait()?;
        *p = None;
      }
      None => (),
    };

    if let Err(e) = self.clean_proc(idx) {
      log::warn!("ignoring error '{:?}' while cleaning killed process", e);
    }

    Ok(())
  }

  fn shutdown_proc(&mut self, idx: usize) -> Result<(), SupervisorError> {
    self.check_signals()?;

    log::info!("shutting down {}", self.spec.procs[idx].name.as_str());

    let env = self.get_proc_script_env(idx);
    let p = &mut self.procs[idx];

    match p {
      Some(c) => {
        match self.spec.procs[idx].shutdown {
          Some(ref shutdown) => match run_command(shutdown, &env) {
            Ok(()) => (),
            Err(_) => return self.kill_proc(idx),
          },
          None => return self.kill_proc(idx),
        }

        let _ = c.wait()?;
        *p = None;
      }
      None => (),
    };

    self.clean_proc(idx)?;

    Ok(())
  }

  fn check_proc(&mut self, idx: usize) -> Result<ProcStatus, SupervisorError> {
    self.check_signals()?;

    log::info!("checking {}", self.spec.procs[idx].name);

    let env = self.get_proc_script_env(idx);
    let p = &mut self.procs[idx];

    match p {
      Some(c) => match c.try_wait()? {
        None => {
          let s = self.spec.procs.get(idx).unwrap();
          match s.check {
            Some(ref check) => match run_command_take_last_line(check, &env)?.as_str() {
              "STARTING" => Ok(ProcStatus::Starting),
              "RUNNING" => Ok(ProcStatus::Running),
              _ => Err(SupervisorError::ProcFailed),
            },
            None => Ok(ProcStatus::Running),
          }
        }
        Some(_) => {
          *p = None;
          Err(SupervisorError::ProcFailed)
        }
      },
      None => Err(SupervisorError::ProcFailed),
    }
  }

  fn clean_proc(&mut self, idx: usize) -> Result<(), SupervisorError> {
    self.check_signals()?;

    {
      log::info!("running {} cleanup", self.spec.procs[idx].name);
      let p = &self.procs[idx];

      match p {
        Some(_) => panic!("bug, clean without kill."),
        None => (),
      }
    }

    let s = self.spec.procs.get(idx).unwrap();
    match s.cleanup {
      Some(ref cleanup) => run_command(cleanup, &vec![]),
      None => Ok(()),
    }
  }

  fn post_proc(&mut self, idx: usize) -> Result<(), SupervisorError> {
    self.check_signals()?;

    log::info!("running {} post", self.spec.procs[idx].name);

    let env = self.get_proc_script_env(idx);
    let s = &self.spec.procs[idx];
    match s.post {
      Some(ref post) => run_command(post, &env),
      None => Ok(()),
    }
  }

  fn start_proc(&mut self, idx: usize) -> Result<(), SupervisorError> {
    {
      self.check_signals()?;

      let p = &self.procs[idx];

      match p {
        Some(_) => self.kill_proc(idx)?,
        None => (),
      }
    }

    log::info!("starting {}", self.spec.procs[idx].name);

    let s = self.spec.procs.get(idx).unwrap();

    let mut cmd = std::process::Command::new(s.run.clone());
    cmd.before_exec(|| {
      match nix::unistd::setpgid(nix::unistd::Pid::from_raw(0), nix::unistd::Pid::from_raw(0)) {
        Ok(_pid) => Ok(()),
        Err(_err) => Err(std::io::Error::from(std::io::ErrorKind::Other)),
      }
    });
    let c = cmd.spawn()?;

    {
      self.procs[idx] = Some(c);
    }

    loop {
      self.sleep(Duration::from_millis(100))?;

      match self.check_proc(idx)? {
        ProcStatus::Starting => (),
        ProcStatus::Running => {
          log::info!(
            "proc is up, running post start script: {}",
            self.spec.procs[idx].name
          );
          return self.post_proc(idx);
        }
      }
    }
  }

  fn kill_all_procs(&mut self) -> Result<(), SupervisorError> {
    for i in (0..self.procs.len()).rev() {
      self.kill_proc(i)?;
    }
    Ok(())
  }

  fn shutdown_all_procs(&mut self) -> Result<(), SupervisorError> {
    for i in (0..self.procs.len()).rev() {
      self.shutdown_proc(i)?;
    }
    Ok(())
  }

  fn restart_all_procs(&mut self) -> Result<(), SupervisorError> {
    log::info!("(re)starting all procs.");

    if !self.rate_limiter.take() {
      return Err(SupervisorError::RestartLimitReached);
    }

    self.kill_all_procs()?;

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

  fn supervise_forever(&mut self) {
    loop {
      match self.one_for_all_run_procs() {
        e @ SupervisorError::IOError(_) | e @ SupervisorError::ProcFailed => {
          log::warn!("supervisor encountered an error: {:?}", e);
        }
        SupervisorError::Shutdown => {
          log::info!("supervisor shutting down gracefully");
          match self.shutdown_all_procs() {
            Ok(()) => (),
            Err(e) => {
              log::error!("unable shutdown child procs, killing instead: {:?}", e);
              match self.kill_all_procs() {
                Ok(()) => (),
                Err(e) => log::error!("unable kill child procs: {:?}", e),
              }
            }
          }
          std::process::exit(0)
        }
        SupervisorError::Terminated | SupervisorError::RestartLimitReached => {
          log::info!("supervisor shutting down brutally.");
          match self.kill_all_procs() {
            Ok(()) => (),
            Err(e) => log::error!("unable kill child procs: {:?}", e),
          }
          std::process::exit(1)
        }
      }
    }
  }
}

fn main() {
  simple_logger::init().unwrap();
  let args: Vec<String> = std::env::args().collect();
  let mut arg_idx = 1;

  let mut supervisor_spec_builder = specs::SupervisorSpecBuilder::new();
  let mut proc_spec_builder = specs::ProcSpecBuilder::new();

  while arg_idx < args.len() {
    match args[arg_idx].as_ref() {
      "-restarts-per-second" => {
        let rps = args
          .get(arg_idx + 1)
          .unwrap_or_else(|| die("-restarts-per-second expects a number."));

        let rps = rps
          .parse::<f64>()
          .unwrap_or_else(|_e| die(format!("{} is not a valid f64.", rps).as_ref()));

        supervisor_spec_builder.set_restarts_per_second(rps);

        arg_idx += 2;
      }
      "-max-restarts" => {
        let max_restarts = args
          .get(arg_idx + 1)
          .unwrap_or_else(|| die("-max-restarts expected a number number."));
        let max_restarts = max_restarts.parse::<f64>().unwrap_or_else(|_e| {
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
      "-post-start" => {
        let post = args
          .get(arg_idx + 1)
          .unwrap_or_else(|| die("-post-start expected an argument."));
        proc_spec_builder.set_post_start(post.clone());
        arg_idx += 2;
      }
      "-cleanup" => {
        let cleanup = args
          .get(arg_idx + 1)
          .unwrap_or_else(|| die("-cleanup expected an argument."));
        proc_spec_builder.set_cleanup(cleanup.clone());
        arg_idx += 2;
      }
      "-shutdown" => {
        let shutdown = args
          .get(arg_idx + 1)
          .unwrap_or_else(|| die("-shutdown expected an argument."));
        proc_spec_builder.set_shutdown(shutdown.clone());
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

  let (sigtx, sigrx) = crossbeam_channel::bounded::<Signal>(64);

  let _ = std::thread::spawn(move || {
    if let Ok(signals) =
      signal_hook::iterator::Signals::new(&[signal_hook::SIGINT, signal_hook::SIGTERM])
    {
      for signal in signals.forever() {
        match signal {
          signal_hook::SIGINT => {
            let _ = sigtx.send(Signal::Shutdown);
          }
          signal_hook::SIGTERM => {
            let _ = sigtx.send(Signal::Terminate);
          }
          _ => (),
        }
      }
    }
  });

  let mut supervisor = Supervisor::new(spec, sigrx);
  supervisor.supervise_forever();
}

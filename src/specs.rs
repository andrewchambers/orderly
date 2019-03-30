#[derive(Debug)]
pub enum SpecError {
  MissingField(&'static str),
}

#[derive(Debug)]
pub struct ProcSpecBuilder {
  name: Option<String>,
  run: Option<String>,
  wait_started: Option<String>,
  wait_started_timeout_seconds: Option<f64>,
  check: Option<String>,
  check_timeout_seconds: Option<f64>,
  shutdown: Option<String>,
  shutdown_timeout_seconds: Option<f64>,
  terminate_timeout_seconds: Option<f64>,
  cleanup: Option<String>,
  cleanup_timeout_seconds: Option<f64>,
}

fn set_optional_timeout(v: &mut Option<f64>, timeout_seconds: f64) {
  *v = if timeout_seconds > 0.0 {
    Some(timeout_seconds)
  } else {
    None
  }
}

impl ProcSpecBuilder {
  pub fn new() -> Self {
    ProcSpecBuilder {
      name: None,
      run: None,
      check: None,
      check_timeout_seconds: Some(120.0),
      wait_started: None,
      wait_started_timeout_seconds: Some(120.0),
      shutdown: None,
      shutdown_timeout_seconds: Some(120.0),
      cleanup: None,
      cleanup_timeout_seconds: Some(120.0),
      terminate_timeout_seconds: Some(10.0),
    }
  }

  pub fn set_name(&mut self, name: String) {
    self.name = Some(name)
  }

  pub fn set_run(&mut self, run: String) {
    self.run = Some(run)
  }

  pub fn set_check(&mut self, check: String) {
    self.check = Some(check)
  }

  pub fn set_wait_started(&mut self, wait_started: String) {
    self.wait_started = Some(wait_started)
  }

  pub fn set_cleanup(&mut self, cleanup: String) {
    self.cleanup = Some(cleanup)
  }

  pub fn set_shutdown(&mut self, shutdown: String) {
    self.shutdown = Some(shutdown)
  }

  pub fn set_wait_started_timeout_seconds(&mut self, timeout_seconds: f64) {
    set_optional_timeout(&mut self.wait_started_timeout_seconds, timeout_seconds)
  }

  pub fn set_check_timeout_seconds(&mut self, timeout_seconds: f64) {
    set_optional_timeout(&mut self.check_timeout_seconds, timeout_seconds)
  }

  pub fn set_shutdown_timeout_seconds(&mut self, timeout_seconds: f64) {
    set_optional_timeout(&mut self.shutdown_timeout_seconds, timeout_seconds)
  }

  pub fn set_terminate_timeout_seconds(&mut self, timeout_seconds: f64) {
    set_optional_timeout(&mut self.terminate_timeout_seconds, timeout_seconds)
  }

  pub fn set_cleanup_timeout_seconds(&mut self, timeout_seconds: f64) {
    set_optional_timeout(&mut self.cleanup_timeout_seconds, timeout_seconds)
  }

  pub fn build(self) -> Result<ProcSpec, SpecError> {
    let mut spec = ProcSpec {
      name: "".to_string(),
      run: "".to_string(),
      check: self.check,
      check_timeout_seconds: self.check_timeout_seconds,
      shutdown: self.shutdown,
      shutdown_timeout_seconds: self.shutdown_timeout_seconds,
      terminate_timeout_seconds: self.terminate_timeout_seconds,
      cleanup: self.cleanup,
      cleanup_timeout_seconds: self.cleanup_timeout_seconds,
      wait_started: self.wait_started,
      wait_started_timeout_seconds: self.wait_started_timeout_seconds,
    };
    match &self.name {
      Some(name) => spec.name = name.clone(),
      None => return Err(SpecError::MissingField("name")),
    }

    match &self.run {
      Some(run) => spec.run = run.clone(),
      None => return Err(SpecError::MissingField("run")),
    }

    Ok(spec)
  }
}

#[derive(Debug)]
pub struct ProcSpec {
  pub name: String,
  pub run: String,
  pub wait_started: Option<String>,
  pub wait_started_timeout_seconds: Option<f64>,
  pub check: Option<String>,
  pub check_timeout_seconds: Option<f64>,
  pub shutdown: Option<String>,
  pub shutdown_timeout_seconds: Option<f64>,
  pub terminate_timeout_seconds: Option<f64>,
  pub cleanup: Option<String>,
  pub cleanup_timeout_seconds: Option<f64>,
}

#[derive(Debug)]
pub struct SupervisorSpecBuilder {
  status_file: Option<String>,
  pub restart_tokens_per_second: f64,
  pub max_restart_tokens: f64,
  pub check_delay_seconds: f64,
  pub start_complete: Option<String>,
  pub start_complete_timeout: Option<f64>,
  pub restart: Option<String>,
  pub restart_timeout: Option<f64>,
  pub failure: Option<String>,
  pub failure_timeout: Option<f64>,
  procs: Vec<ProcSpec>,
}

#[derive(Debug)]
pub struct SupervisorSpec {
  pub status_file: Option<String>,
  pub restart_tokens_per_second: f64,
  pub check_delay_seconds: f64,
  pub max_restart_tokens: f64,
  pub start_complete: Option<String>,
  pub start_complete_timeout: Option<f64>,
  pub restart: Option<String>,
  pub restart_timeout: Option<f64>,
  pub failure: Option<String>,
  pub failure_timeout: Option<f64>,
  pub procs: Vec<ProcSpec>,
}

impl SupervisorSpecBuilder {
  pub fn new() -> Self {
    SupervisorSpecBuilder {
      restart_tokens_per_second: 0.1,
      max_restart_tokens: 5.0,
      check_delay_seconds: 5.0,
      start_complete: None,
      start_complete_timeout: Some(120.0),
      restart: None,
      restart_timeout: Some(120.0),
      failure: None,
      failure_timeout: Some(120.0),
      status_file: None,
      procs: vec![],
    }
  }

  pub fn set_restart_tokens_per_second(&mut self, rps: f64) {
    self.restart_tokens_per_second = rps;
  }

  pub fn set_max_restart_tokens(&mut self, max_restart_tokens: f64) {
    self.max_restart_tokens = max_restart_tokens;
  }

  pub fn set_check_delay_seconds(&mut self, check_delay_seconds: f64) {
    self.check_delay_seconds = check_delay_seconds;
  }

  pub fn set_status_file(&mut self, status_file: String) {
    self.status_file = Some(status_file);
  }

  pub fn set_start_complete(&mut self, command: String) {
    self.start_complete = Some(command);
  }

  pub fn set_start_complete_timeout(&mut self, timeout_seconds: f64) {
    set_optional_timeout(&mut self.start_complete_timeout, timeout_seconds)
  }

  pub fn set_failure(&mut self, command: String) {
    self.failure = Some(command);
  }

  pub fn set_failure_timeout(&mut self, timeout_seconds: f64) {
    set_optional_timeout(&mut self.failure_timeout, timeout_seconds)
  }

  pub fn set_restart(&mut self, command: String) {
    self.restart = Some(command);
  }

  pub fn set_restart_timeout(&mut self, timeout_seconds: f64) {
    set_optional_timeout(&mut self.restart_timeout, timeout_seconds)
  }

  pub fn add_proc_spec(&mut self, spec: ProcSpec) {
    self.procs.push(spec);
  }

  pub fn build(self) -> Result<SupervisorSpec, SpecError> {
    let mut spec = SupervisorSpec {
      restart_tokens_per_second: self.restart_tokens_per_second,
      check_delay_seconds: self.check_delay_seconds,
      max_restart_tokens: self.max_restart_tokens,
      status_file: self.status_file,
      start_complete: self.start_complete,
      start_complete_timeout: self.start_complete_timeout,
      restart: self.restart,
      restart_timeout: self.restart_timeout,
      failure: self.failure,
      failure_timeout: self.failure_timeout,
      procs: vec![],
    };

    spec.procs = self.procs;

    Ok(spec)
  }
}

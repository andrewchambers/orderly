#[derive(Debug)]
pub enum SpecError {
  MissingField(&'static str),
}

#[derive(Debug)]
pub struct ProcSpecBuilder {
  name: Option<String>,
  run: Option<String>,
  check: Option<String>,
  post: Option<String>,
  shutdown: Option<String>,
  cleanup: Option<String>,
}

impl ProcSpecBuilder {
  pub fn new() -> Self {
    ProcSpecBuilder {
      name: None,
      run: None,
      check: None,
      post: None,
      shutdown: None,
      cleanup: None,
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

  pub fn set_post_start(&mut self, post: String) {
    self.post = Some(post)
  }

  pub fn set_cleanup(&mut self, cleanup: String) {
    self.cleanup = Some(cleanup)
  }

  pub fn set_shutdown(&mut self, shutdown: String) {
    self.shutdown = Some(shutdown)
  }

  pub fn build(self) -> Result<ProcSpec, SpecError> {
    let mut spec = ProcSpec {
      name: "".to_string(),
      run: "".to_string(),
      check: self.check,
      post: self.post,
      shutdown: self.shutdown,
      cleanup: self.cleanup,
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
  pub check: Option<String>,
  pub post: Option<String>,
  pub shutdown: Option<String>,
  pub cleanup: Option<String>,
}

#[derive(Debug)]
pub struct SupervisorSpecBuilder {
  status_file: Option<String>,
  pub restarts_per_second: f64,
  pub max_restarts: f64,
  procs: Vec<ProcSpec>,
}

#[derive(Debug)]
pub struct SupervisorSpec {
  pub status_file: String,
  pub restarts_per_second: f64,
  pub max_restarts: f64,
  pub procs: Vec<ProcSpec>,
}

impl SupervisorSpecBuilder {
  pub fn new() -> Self {
    SupervisorSpecBuilder {
      restarts_per_second: 0.1,
      max_restarts: 5.0,
      status_file: None,
      procs: vec![],
    }
  }

  pub fn set_restarts_per_second(&mut self, rps: f64) {
    self.restarts_per_second = rps;
  }

  pub fn set_max_restarts(&mut self, max_restarts: f64) {
    self.max_restarts = max_restarts;
  }

  pub fn set_status_file(&mut self, status_file: String) {
    self.status_file = Some(status_file);
  }

  pub fn add_proc_spec(&mut self, spec: ProcSpec) {
    self.procs.push(spec);
  }

  pub fn build(self) -> Result<SupervisorSpec, SpecError> {
    let mut spec = SupervisorSpec {
      restarts_per_second: self.restarts_per_second,
      max_restarts: self.max_restarts,
      status_file: "".to_string(),
      procs: vec![],
    };

    match &self.status_file {
      Some(status_file) => spec.status_file = status_file.clone(),
      None => return Err(SpecError::MissingField("status-file")),
    }

    spec.procs = self.procs;

    Ok(spec)
  }
}

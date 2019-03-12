use std::time::Duration;

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
    cleanup: Option<String>,
}

impl ProcSpecBuilder {
    pub fn new() -> Self {
        ProcSpecBuilder {
            name: None,
            run: None,
            check: None,
            post: None,
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

    pub fn set_post(&mut self, post: String) {
        self.post = Some(post)
    }

    pub fn set_cleanup(&mut self, cleanup: String) {
        self.cleanup = Some(cleanup)
    }

    pub fn build(self) -> Result<ProcSpec, SpecError> {
        let mut spec = ProcSpec {
            name: "".to_string(),
            run: "".to_string(),
            check: "".to_string(),
            post: "".to_string(),
            cleanup: "".to_string(),
        };
        match &self.name {
            Some(name) => spec.name = name.clone(),
            None => return Err(SpecError::MissingField("name")),
        }

        match &self.run {
            Some(run) => spec.run = run.clone(),
            None => return Err(SpecError::MissingField("run")),
        }

        match &self.check {
            Some(check) => spec.check = check.clone(),
            None => return Err(SpecError::MissingField("check")),
        }

        match &self.post {
            Some(post) => spec.post = post.clone(),
            None => return Err(SpecError::MissingField("post")),
        }

        match &self.cleanup {
            Some(cleanup) => spec.cleanup = cleanup.clone(),
            None => return Err(SpecError::MissingField("cleanup")),
        }

        Ok(spec)
    }
}

#[derive(Debug)]
pub struct ProcSpec {
    pub name: String,
    pub run: String,
    pub check: String,
    pub post: String,
    pub cleanup: String,
}

#[derive(Debug)]
pub enum SupervisorStrategy {
    OneForAll,
}

#[derive(Debug)]
pub struct SupervisorSpecBuilder {
    strategy: Option<SupervisorStrategy>,
    status_file: Option<String>,
    restart_duration: Duration,
    max_restarts: usize,
    procs: Vec<ProcSpec>,
}

#[derive(Debug)]
pub struct SupervisorSpec {
    pub strategy: SupervisorStrategy,
    pub status_file: String,
    pub restart_duration: Duration,
    pub max_restarts: usize,
    pub procs: Vec<ProcSpec>,
}

impl SupervisorSpecBuilder {
    pub fn new() -> Self {
        SupervisorSpecBuilder {
            strategy: None,
            restart_duration: Duration::from_secs(60),
            max_restarts: 10,
            status_file: None,
            procs: vec![],
        }
    }

    pub fn set_strategy(&mut self, strat: SupervisorStrategy) {
        self.strategy = Some(strat);
    }

    pub fn set_restart_duration(&mut self, restart_duration: Duration) {
        self.restart_duration = restart_duration;
    }

    pub fn set_max_restarts(&mut self, max_restarts: usize) {
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
            strategy: SupervisorStrategy::OneForAll,
            restart_duration: Duration::from_secs(0),
            max_restarts: 0,
            status_file: "".to_string(),
            procs: vec![],
        };

        match self.strategy {
            Some(strategy) => spec.strategy = strategy,
            None => return Err(SpecError::MissingField("strategy")),
        }

        match &self.status_file {
            Some(status_file) => spec.status_file = status_file.clone(),
            None => return Err(SpecError::MissingField("status-file")),
        }

        spec.restart_duration = self.restart_duration;
        spec.max_restarts = self.max_restarts;
        spec.procs = self.procs;

        Ok(spec)
    }
}

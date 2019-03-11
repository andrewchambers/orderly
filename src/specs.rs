use std::time::Duration;

pub enum SpecError {
    MissingField(&'static str),
}

pub struct ProcSpecBuilder {
    name: Option<String>,
    run: Option<String>,
    check: Option<String>,
    log: Option<String>,
    post: Option<String>,
    cleanup: Option<String>,
}

impl ProcSpecBuilder {
    pub fn new() -> Self {
        ProcSpecBuilder {
            name: None,
            run: None,
            check: None,
            log: None,
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

    pub fn set_log(&mut self, log: String) {
        self.log = Some(log)
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
            log: "".to_string(),
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

        match &self.log {
            Some(log) => spec.log = log.clone(),
            None => return Err(SpecError::MissingField("log")),
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

pub struct ProcSpec {
    name: String,
    run: String,
    check: String,
    log: String,
    post: String,
    cleanup: String,
}

enum SupervisorStrategy {
    OneForAll,
}

pub struct SupervisorSpecBuilder {
    strategy: Option<SupervisorStrategy>,
    status_file: Option<String>,
    restart_duration: Duration,
    max_restarts: usize,
    procs: Vec<ProcSpec>,
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
}

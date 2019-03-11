mod specs;
use std::time::Duration;

enum SupervisorStrategy {
    OneForAll,
}

/*
struct Builder {
    strategy: Option<SupervisorStrategy>,
    status_file: Option<String>,
    restart_duration: Duration,
    max_restarts: usize,
    procs: Vec<ProcSpecBuilder>,
}


impl Builder {
    fn new() -> Self {
        Builder {
            strategy: None,
            restart_duration: Duration::from_secs(60),
            max_restarts: 10,
            status_file: None,
            procs: vec![],
        }
    }

    fn set_strategy(&mut self, strat: SupervisorStrategy) {
        self.strategy = Some(strat);
    }

    /*
    fn set_proc_check(&mut self, check: String) {
        self.check = Some(check)
    }
    */
}

*/

fn die(s: &str) -> ! {
    eprintln!("{}", s);
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut arg_idx = 1;

    let supervisor_spec_builder = specs::SupervisorSpecBuilder::new();
    let proc_spec_builder = specs::ProcSpecBuilder::new();

    while arg_idx < args.len() {
        match args.get(arg_idx).unwrap().as_ref() {
            "-strategy" => {
                match args
                    .get(arg_idx + 1)
                    .unwrap_or_else(|| die("-strategy expected an argument."))
                    .as_ref()
                {
                    "one-for-all" => {}
                    unknown => die(format!("{} is not a valid -strategy.", unknown).as_ref()),
                }
                arg_idx += 2;
            }
            "-restart-duration" => {
                let duration = args.get(arg_idx + 1).unwrap_or_else(|| {
                    die("-restart-duration expected an unsigned number of seconds.")
                });

                let duration = duration.parse::<usize>().unwrap_or_else(|_e| {
                    die(format!("{} is not a valid unsigned number.", duration).as_ref())
                });

                arg_idx += 2;
            }
            "-max-restarts" => {
                let max_restarts = args
                    .get(arg_idx + 1)
                    .unwrap_or_else(|| die("-max-restarts expected an unsigned number."));
                let max_restarts = max_restarts.parse::<usize>().unwrap_or_else(|_e| {
                    die(format!("{} is not a valid unsigned number.", max_restarts).as_ref())
                });
                arg_idx += 2;
            }
            "-status_file" => {
                let status_file = args
                    .get(arg_idx + 1)
                    .unwrap_or_else(|| die("-status_file expected an argument."));
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
                arg_idx += 2;
            }
            "-check" => {
                let check = args
                    .get(arg_idx + 1)
                    .unwrap_or_else(|| die("-check expected an argument."));
                arg_idx += 2;
            }
            "-log" => {
                let log = args
                    .get(arg_idx + 1)
                    .unwrap_or_else(|| die("-log expected an argument."));
                arg_idx += 2;
            }
            "-post" => {
                let post = args
                    .get(arg_idx + 1)
                    .unwrap_or_else(|| die("-post expected an argument."));
                arg_idx += 2;
            }
            "--" => {
                arg_idx += 1;
                break;
            }

            unknown => die(format!("unknown process spec argument: {}.", unknown).as_ref()),
        }
    }
}

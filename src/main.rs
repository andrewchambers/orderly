mod specs;
use std::time::Duration;

fn die(s: &str) -> ! {
    eprintln!("{}", s);
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut arg_idx = 1;

    let mut supervisor_spec_builder = specs::SupervisorSpecBuilder::new();
    let mut proc_spec_builder = specs::ProcSpecBuilder::new();

    while arg_idx < args.len() {
        match args.get(arg_idx).unwrap().as_ref() {
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
            "-log" => {
                let log = args
                    .get(arg_idx + 1)
                    .unwrap_or_else(|| die("-log expected an argument."));
                proc_spec_builder.set_log(log.clone());
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

    dbg!(spec);
}

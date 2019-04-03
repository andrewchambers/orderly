# ORDERLY -- run and supervise processes

``` 
  orderly [<SUPERVISOR-FLAGS>] [ -- <PROCESS-FLAGS> ]+
```

## DESCRIPTION

**orderly** Provides ordered starting, supervision and stopping of a
collection of processes. **orderly** starts a list of processes in
order, then monitors them with provided health check scripts.

On failure, processes are stopped and restarted in a well specified
order. Scripts can also be provided to cleanup after a failed process
(unmounting filesystems, deleting files, etc.).

Arrangement of **orderly** invocations in a tree hierarchy allows the
creation of fault tolerant process supervision trees.

## SUPERVISOR SPEC FLAGS

### \-max-start-tokens NUM (default=5)

The size of the start pool, when this pool less than one and a (re)start is
required **orderly** aborts. Each (re)start decreases the pool size by one. 
Note that the initial start is counted towards the quota,
so the pool size must be at least 1 for a successful startup.

### \-start-tokens-per-second NUM (default=0.1)

The rate at which restarts are added into the (re)start pool.

### \-start-tokens-per-minute NUM (default=6)

An alias for the rate at which restarts are added into the (re)start pool.

### \-start-tokens-per-hour NUM (default=360)

An alias for the rate at which restarts are added into the (re)start pool.

### \-status-file PATH

If specified, a file to be written containing the current status of
**orderly**. The file will contain either "STARTING", "RUNNING".
**orderly** transitions from starting, to running after all procesess it
is controlling have started successfully at least one time. The main use
for this file is for creating nested **orderly** supervision trees that
start in order.

### \-on-start-complete BIN

An optional hook to run when the first startup completes successfully,

### \-on-restart BIN

An optional hook to run before each restart that is triggered by a command failure.

### \-on-failure BIN

An optional hook to run when orderly encounters an unrecoverable error, and
must abort operation.

### \-on-shutdown BIN

An optional hook to run just before orderly exits after a clean shutdown.

### \-all-lifecycle-hooks BIN

Shorthand for setting all lifecycle hooks to the same script, in this case env
variables can disambiguate the action to
take.

### \-{start-complete,on-restart,on-failure,on-shutdown}-timeout SECONDS (default=120)\`

The number of seconds to wait for a given hook before giving up and
triggering a restart. A negative value means no timeout.

### \-check-delay SECONDS (default=5)

The amount of time in seconds to wait between health check loops.

## PROCESS SPEC FLAGS

### \-name NAME

The name of the service, passed to all callbacks under the env variable
as 'ORDERLY\_SERVICE\_NAME'.

### \-run BIN

The command invoked by **orderly** to launch a supervised process. If
this program exits, it will trigger a restart.

### \-wait-started BIN

An optional command invoked concurrently with the service, it should
exit with a 0 exit code when this process is ready and the next process
can be started.

### \-check BIN

An optional command invoked periodically as a health check. If this
commands times out or returns an unsuccessful exit code, a restart will
be triggered. This check is in addition to ensuring the run process has
not exited.

### \-shutdown BIN

An optional command to shutdown the supervised process. If not
specified, **orderly** will send SIGTERM to terminate the supervised
process.

The shutdown command may be run if a command start times out, a sibling process
dies and the server needs to restart, or orderly is shutting down.

### \-clean BIN

An optional command to cleanup any resources the running process may
have left. If it exits with an unsuccessful exit code, a restart will be
triggered. Process cleaning should be idempotent, and always happens in
reverse order to process startup.

### \-all-commands BIN

Shorthand for setting all commands to the same script, in this case env
variables can disambiguate the action to
take.

### \-{wait-started,check,shutdown,clean}-timeout SECONDS (default=120)\`

The number of seconds to wait for a given command before giving up. A negative value means no timeout.

### \-terminate-timeout SECONDS (default=10)\`

The amount of time to wait after a shutdown command before terminating the child
with a SIGKILL if it does not exit on it's own.

## PROCESS SPEC ENV VARIABLES

The following env variables are passed to any specified process scripts.

### ORDERLY\_SERVICE\_NAME

The name of the process being managed.

### ORDERLY\_ACTION

One of START_COMPLETE, RESTART, FAILURE, RUN, WAIT\_STARTED, CHECK, SHUTDOWN, CLEANUP depending on which
action **orderly** is requesting.

### ORDERLY\_SUPERVISOR\_PID

The pid of the orderly process.

### ORDERLY\_RUN\_PID

The pid of the supervised process, if it is running.

## SIGNALS

### SIGINT SIGTERM

**orderly** shuts all processes down with the provided or default shutdown commands
in reverse order.

## EXIT CODE

**orderly** exits with a zero exit code only if shutdown after a SIGINT or SIGTERM
occured with no errors.

## EXAMPLE

Given the executable service script 'sv':

``` 
  #! /usr/bin/env bash

  set -eu

  p () {
    echo "$ORDERLY_SERVICE_NAME $ORDERLY_ACTION"
  }

  case $ORDERLY_ACTION in
    RUN)
      p
      exec sleep 9999
    ;;
    WAIT_STARTED)
      sleep 0.1
      p
    ;;
    CHECK)
      p
    ;;
    SHUTDOWN)
      p
      kill -9 $ORDERLY_RUN_PID
    ;;
    CLEANUP)
      p
    ;;
    *)
      echo "unknown action: $ORDERLY_ACTION"
      exit 1
    ;;
  esac
```

And the invocation:

``` 
  orderly -- \
    -name sv1 -all-commands ./sv \
      -- \
    -name sv2 -all-commands ./sv \
      -- \
    -name sv3 -all-commands ./sv &

  pid="$!"
  sleep 1
  kill -SIGINT "$pid"
  wait
```

You will see output like:

``` 
  2019-03-28 12:23:10 INFO  [orderly] (re)starting all procs.
  2019-03-28 12:23:10 INFO  [orderly] running sv3 cleanup.
  sv3 CLEANUP
  2019-03-28 12:23:10 INFO  [orderly] running sv2 cleanup.
  sv2 CLEANUP
  2019-03-28 12:23:10 INFO  [orderly] running sv1 cleanup.
  sv1 CLEANUP
  2019-03-28 12:23:10 INFO  [orderly] starting sv1.
  sv1 RUN
  sv1 WAIT_STARTED
  2019-03-28 12:23:10 INFO  [orderly] starting sv2.
  sv2 RUN
  sv2 WAIT_STARTED
  2019-03-28 12:23:10 INFO  [orderly] starting sv3.
  sv3 RUN
  sv3 WAIT_STARTED
  2019-03-28 12:23:10 INFO  [orderly] checking sv1.
  sv1 CHECK
  2019-03-28 12:23:10 INFO  [orderly] checking sv2.
  sv2 CHECK
  2019-03-28 12:23:10 INFO  [orderly] checking sv3.
  sv3 CHECK
  2019-03-28 12:23:11 INFO  [orderly] supervisor shutting down gracefully.
  2019-03-28 12:23:11 INFO  [orderly] shutting down sv3.
  sv3 SHUTDOWN
  2019-03-28 12:23:11 INFO  [orderly] running sv3 cleanup.
  sv3 CLEANUP
  2019-03-28 12:23:11 INFO  [orderly] shutting down sv2.
  sv2 SHUTDOWN
  2019-03-28 12:23:11 INFO  [orderly] running sv2 cleanup.
  sv2 CLEANUP
  2019-03-28 12:23:11 INFO  [orderly] shutting down sv1.
  sv1 SHUTDOWN
  2019-03-28 12:23:11 INFO  [orderly] running sv1 cleanup.
  sv1 CLEANUP
```

## NOTES

Logging facilities may be added in the future, though currently a
logging process can simply be part of the process list, and can be sent
input via named pipes or any other mechanism.

## COPYRIGHT

orderly is Copyright (C) 2019 Andrew Chambers <https://acha.ninja/>

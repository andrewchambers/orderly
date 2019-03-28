# Example

This is a simple example showing how you might perform:

  - Running collecting and rotating logs.
  - Running redis.
  - Running a simple (but fake) web server.

The example depends on svlogd from runit, go and redis.

To run the example, add orderly to your PATH and run ./run\_example,
feel free to experiment with killing services and watch the restart
logic handle the failures.

Press ctrl-c to cleanly close the example down.

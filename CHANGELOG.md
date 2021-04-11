# 0.3.0

* [#6](https://github.com/DaGenix/netrange/pull/6): Require a filename for the `cloud merge` and `cloud read` subcommands.
  The previous behavior of defaulting to STDIN when no filename was provided was confusing as it
  could result in the subcommand seemingly hanging for no reason. "-" can be used as the filename if the
  user really wants to read from STDIN.

# 0.2.0

* [#2](https://github.com/DaGenix/netrange/pull/2): Significant improvements to help text in `cloud filter-help` subcommand.
  Also, add support for networkFeatures attribute for Azure source.
* [#3](https://github.com/DaGenix/netrange/pull/3): Support added for Google source which lists all Google advertised
  IP addressed ranges (which includes all of the GCP ranges)

# 0.1.0

Initial release

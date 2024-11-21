# gen

Generate random data on the commandline

## Installation

Currently you have to compile locally.

- Clone the repo
  `git clone https://github.com/Skyppex/gen.git`
  `gh repo clone Skyppex/gen`
- Build using cargo
  `cargo build --release`

## Usage

Do `gen --help` to see command documentation. Also works for subcommands

The root command has a `--destination` flag which takes a path to a file.
Use this to output the random data to that file instead of `stdout`.
This is useful especially for the `ascii` and `unicode` subcommands as they can
bog down your terminal significantly when generating excessive amounts of data.

### Subcommands

- `int`: Generate a random integer within a range.
- `float`: Generate a random floating-point number within a range.
- `uuid`: Generate a random uuid.
- `ascii`: Generate random ascii characters.
- `unicode`: Generate random unicode characters.

## Contributing

Issues and PRs are welcome!

# rsh: a Unix shell in Rust

A small Unix shell written in Rust, following Stephen Brennan's tutorial
[Write a Shell in C](https://brennan.io/2015/01/16/write-a-shell-in-c/).
The C design is translated into Rust by hand, and then extended with
pipelines and `$?`.

Processes and pipes are managed with `fork`, `execvp`, `waitpid`, `pipe`,
`dup2` and `close` through **hand-written FFI bindings**. The project uses
no `libc` crate and no dependencies at all.

## Features

- An interactive prompt that shows the user and the current folder:
  `numeez@rsh rshell $`
- Builtin commands: `cd`, `help` and `exit`
- Runs any program on your `PATH`, with arguments
- Pipelines of any length: `ls | grep rs | wc -l`
- `$?`: the exit status of the last command
- Reports commands that were killed by a signal
- Exits cleanly on `exit` or Ctrl-D, with the last command's status
- Errors go to stderr; a bad command never crashes the shell

## Example

```
numeez@rsh rshell $ ls src | wc -l
4
numeez@rsh rshell $ echo a b c | tr a-z A-Z | rev
C B A
numeez@rsh rshell $ false
numeez@rsh rshell $ echo $?
1
numeez@rsh rshell $ cd /tmp
numeez@rsh tmp $ pwd
/private/tmp
numeez@rsh tmp $ nosuchcommand
error: nosuchcommand: No such file or directory (os error 2)
numeez@rsh tmp $ ls |
error: syntax error near `|`
numeez@rsh tmp $ exit
```

## Getting started

Requires Rust 1.85 or newer (edition 2024) on macOS or Linux.

```
git clone https://github.com/Numeez/rshell
cd rshell
cargo run
```

Or install it straight from GitHub:

```
cargo install --git https://github.com/Numeez/rshell
```

Run the tests with:

```
cargo test
```

## How it works

Each time round the loop, the shell:

1. **Reads** a line from stdin. EOF (Ctrl-D) ends the shell.
2. **Splits** it into words on whitespace. The words borrow from the line,
   so nothing is copied.
3. **Expands** `$?`: every word that is exactly `$?` is replaced by the last
   command's status.
4. **Parses** the words into a `Command`: `Exit`, `Cd`, `Help`, `Pipeline`
   or `SyntaxError`. Words are split into pipeline stages on `|`, and an
   empty stage (`ls |`, `| wc`) is a syntax error.
5. **Executes** it and gets back a `Status`: `Continue(code)` or `Stop`. The
   loop keeps the code for `$?` and stops only on `Stop`, so `exit` and
   Ctrl-D leave through the same path.

Builtins run inside the shell process. A child process can't change its
parent's directory or end its parent, so `cd` and `exit` have to.

### Running a pipeline

Every external command is a pipeline; a single command is a pipeline with
one stage.

1. All stages are converted to `CString`s, and a null-terminated `argv` array
   is built for each. The `CString`s stay alive until every child has called
   `execvp`, so the pointers never dangle.
2. For each stage, the shell creates a pipe (unless it's the last stage),
   then forks the stage. Every stage is a child of the shell, so the shell
   can wait for all of them.
3. **The child** plugs the previous pipe's read end into stdin and the new
   pipe's write end into stdout with `dup2`, closes every other pipe end,
   then calls `execvp`. If `execvp` fails, it prints an error and calls
   `_exit(1)`, so it can never return to the shell loop.
4. **The shell** closes its copies of those pipe ends right after the fork.
   A reader only sees end of file when every copy of the write end is
   closed, so this is what stops pipelines from hanging.
5. The shell waits for every stage. The pipeline's status is the status of
   the last stage.

### Decoding the `waitpid` status

C programs use the `WIFEXITED`, `WEXITSTATUS`, `WIFSIGNALED` and `WTERMSIG`
macros. Macros can't be called through FFI, so `wait.rs` reimplements them
from macOS's `sys/wait.h` as one function that returns an enum:

```rust
pub enum WaitStatus {
    Exited(c_int),   // exit code: bits 8-15
    Signaled(c_int), // signal number: the low 7 bits
    Stopped(c_int),  // low 7 bits are 0x7F
}
```

A command killed by a signal gets the status `128 + signal`, as in bash, so
`kill -9` gives `137`.

## Project structure

| File | Contents |
|---|---|
| `src/main.rs` | The prompt, the read loop, word splitting and `$?` expansion |
| `src/command.rs` | Parsing, builtins, and running pipelines |
| `src/ffi.rs` | `unsafe extern "C"` bindings for `fork`, `execvp`, `waitpid`, `_exit`, `pipe`, `dup2` and `close` |
| `src/wait.rs` | Decoding the `waitpid` status into `WaitStatus` |

## Limitations

- No quoting or escaping: `echo "a b"` passes `"a` and `b"` as two words
- `|` must be surrounded by spaces: `ls|wc` is read as one word
- No redirection (`>`, `<`), globbing (`*`), or variables other than `$?`
- Builtins in a pipeline (`cd /tmp | ls`) run as external programs, so they
  don't affect the shell
- An early-exiting reader (`ls | head -1`) can make the writer print
  "Broken pipe", because Rust ignores `SIGPIPE` and children inherit that
- Builtins ignore extra arguments, and `cd` needs a directory
- Unix only: `fork` doesn't exist on Windows

## Planned

- [x] Pipes
- [x] `$?`
- [ ] Reset `SIGPIPE` in children
- [ ] Ctrl-C handling: the shell survives and the running command stops
- [ ] Redirection with `>` and `<`
- [ ] Command history

## Credits

Based on [Write a Shell in C](https://brennan.io/2015/01/16/write-a-shell-in-c/)
by Stephen Brennan.

# rsh: a Unix shell in Rust

A small Unix shell written in Rust, following Stephen Brennan's tutorial
[Write a Shell in C](https://brennan.io/2015/01/16/write-a-shell-in-c/).
The C design is translated into Rust by hand.

Programs are launched with `fork`, `execvp` and `waitpid` through
**hand-written FFI bindings**. The project uses no `libc` crate and no
dependencies at all.

## Features

- An interactive prompt that shows the user and the current folder:
  `numeez@rsh rshell $`
- Builtin commands: `cd`, `help` and `exit`
- Runs any program on your `PATH`, with arguments
- Reports commands that were killed by a signal
- Exits cleanly on `exit` or Ctrl-D
- Errors go to stderr; a bad command never crashes the shell

## Example

```
numeez@rsh rshell $ echo hello world
hello world
numeez@rsh rshell $ cd /tmp
numeez@rsh tmp $ pwd
/private/tmp
numeez@rsh tmp $ nosuchcommand
error: nosuchcommand: No such file or directory (os error 2)
numeez@rsh tmp $ exit
```

## Getting started

Requires Rust 1.85 or newer (edition 2024) on macOS or Linux.

```
git clone https://github.com/Numeez/rshell
cd rshell
cargo run
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
3. **Parses** the words into a `Command`: `Exit`, `Cd`, `Help` or `External`.
4. **Executes** it and gets back a `Status` (`Continue` or `Stop`). The loop
   stops only on `Stop`, so `exit` and Ctrl-D leave through the same path.

Builtins run inside the shell process. A child process can't change its
parent's directory or end its parent, so `cd` and `exit` have to.

External commands are launched like this:

1. The words are converted to `CString`s, and a null-terminated array of
   pointers to them (`argv`) is built. The `CString`s stay alive until
   `execvp` is called, so the pointers never dangle.
2. `fork()` creates a copy of the shell.
3. **The child** calls `execvp`, which replaces it with the program. If
   `execvp` fails, the child prints an error and calls `_exit(1)`, so it can
   never return to the shell loop.
4. **The parent** waits with `waitpid`, then decodes the status.

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

## Project structure

| File | Contents |
|---|---|
| `src/main.rs` | The prompt, the read loop and word splitting |
| `src/command.rs` | Parsing, builtins, and launching external commands |
| `src/ffi.rs` | `unsafe extern "C"` bindings for `fork`, `execvp`, `waitpid` and `_exit` |
| `src/wait.rs` | Decoding the `waitpid` status into `WaitStatus` |

## Limitations

These are the same as in the tutorial's shell:

- No quoting or escaping: `echo "a b"` passes `"a` and `b"` as two words
- No pipes (`|`), redirection (`>`, `<`), globbing (`*`) or variables (`$HOME`)
- Builtins ignore extra arguments, and `cd` needs a directory
- Unix only: `fork` doesn't exist on Windows

## Planned

- [ ] Ctrl-C handling: the shell survives and the running command stops
- [ ] Redirection with `>` and `<`
- [ ] Pipes
- [ ] Command history

## Credits

Based on [Write a Shell in C](https://brennan.io/2015/01/16/write-a-shell-in-c/)
by Stephen Brennan.
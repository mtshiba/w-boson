# w-boson

Windows port of [backtrace-on-stack-overflow](https://crates.io/crates/backtrace-on-stack-overflow)

## Usage

```rust
use w_boson::enable;
// or use w_boson::enable_backtrace_on_stack_overflow;

fn recursive(n: usize) {
    print!("{n}");
    recursive(n+1);
}

fn main() {
    unsafe { enable(); }
    recursive(0);
}
```

## Notes

To get the correct function names even in the release build, add the following settings to `Cargo.toml`.

```toml
# Cargo.toml
[profile.release]
debug = true
```

It has been found that some applications using rayon etc. may not display backtraces correctly.
In this case, we recommend using [WinDbg](https://learn.microsoft.com/en-us/windows-hardware/drivers/debuggercmds/windbg-overview).

## License

MIT & Apache-2.0

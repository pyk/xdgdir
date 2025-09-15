# xdgdir

`xdgdir` helps you to resolves paths according to the
[XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/).

- **Zero I/O**: The library performs no filesystem operations. It is a pure path
  resolver, making it fast, predictable, and suitable for any context, including
  async runtimes.
- **Spec Compliant**: Correctly handles environment variables, empty variables,
  and default fallbacks as defined by the spec.
- **Simple API:** Provides a minimal, ergonomic API for the most common use
  cases.

## Getting started

Add `xdgdir` to your project's dependencies:

```sh
cargo add xdgdir
```

## Usage

To get the set of directories for your specific application, use
`BaseDir::new()`.

```rust
use xdgdir::BaseDir;

fn main() {
    let dirs = BaseDir::new("my-app").unwrap();

    println!("Config file should be in: {}", dirs.config.display());
    // -> /home/user/.config/my-app

    println!("Data files should be in: {}", dirs.data.display());
    // -> /home/user/.local/share/my-app

    Ok(())
}
```

To get the raw, non-application-specific base directories, use
`BaseDir::global()`.

```rust
use xdgdir::BaseDir;

fn main() {
    let global_dirs = BaseDir::global().unwrap();

    println!("Default user config dir: {}", global_dirs.config.display());
    // -> /home/user/.config

    println!("User executables dir: {}", global_dirs.bin.display());
    // -> /home/user/.local/bin

    Ok(())
}
```

## License

This project is licensed under the MIT License.

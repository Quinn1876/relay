# Relay
The relay service is designed to run on a raspberry pi in a Waterloop Hyperloop Pod. It's goal is to enable remote communication between the desktop controller and the Pod's subsystems.
The relay service also enables remote flashing supported embedded devices.

## Testing on Windows
Portions of the relay service rely on having access to the `socketcan` crate which is only for linux. These portions of the code are only important for connecting to the canbus.
For the purposes of testing the connection with the desktop, you can run the relay crate on windows with `cargo run` but it will not have any CAN functionality.

# Crate: canota-sys
The canota-sys crate provides bindings to a C library which is used for ota flashing through the CAN bus.
The bindings are generated and stored in the repository. After they are generated, some manual work is needed
to clean up the generated bindings. This includes adding some markers for functions which indicate errors in the form of boolean values. Outside of the markers, the C library is provided as is with unsafe functions. For a safe wrapper api, see the `canota` crate.s

# Crate: canota
The goal of the canota crate is to provide a safe, application ready implementation of the canota-sys library.



## Generating the bindings
Creating the bindings should be done on a linux device or through WSL2.

To create the bindings, first install the CLI tool `bindgen` via `cargo install bindgen`

You may be prompted to install other packages which bindgen relies on if they are not already installed on your system.

Once bindgen is installed, run `make generate-bindings-canota` to generate the bindings file.


# Useful Cargo Commands
-   cargo run  : Runs a binary crate. For the purpose of this repo, run this command from the root to run the relay service
- `cargo build` : Builds a binary or library crate without running it if it is a binary crate. Useful for checking if code will compile
- `cargo test` : Builds and runs the tests for a crate
- `cargo doc --open` : Generates docs for the crate that this is called in.


# References that were helpful building this repository

[Using C Libraries in Rust](https://medium.com/dwelo-r-d/using-c-libraries-in-rust-13961948c72a)

[Wrapping Unsafe C Libraries in Rust](https://medium.com/dwelo-r-d/wrapping-unsafe-c-libraries-in-rust-d75aeb283c65)

Rust for Rustaceans by Jon GJENGSET

The Rust Book

[Socket Can Documentation](https://www.kernel.org/doc/html/latest/networking/can.html)

[Setting Up VCAN on WSL2](https://github.com/microsoft/WSL/issues/5533)

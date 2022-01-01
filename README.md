# Relay
The relay service is designed to run on a raspberry pi in a Waterloop Hyperloop Pod. It's goal is to enable remote communication between the desktop controller and the Pod's subsystems.
The relay service also enables remote flashing supported embedded devices.

# CRATE: canota-sys
THe canota-sys crate provides bindings to a C library which is used for ota flashing through the CAN bus.
The bindings are generated and stored in the repository. After they are generated, some manual work is needed
to clean up the generated bindings. This includes adding some markers for functions which indicate errors in the form of boolean values.

## Generating the bindings
Creating the bindings should be done on a linux device or through WSL2.

To create the bindings, first install the CLI tool `bindgen` via `cargo install bindgen`

You may be prompted to install other packages which bindgen relies on if they are not already installed on your system.

Once bindgen is installed, run `make generate-bindings-canota` to generate the bindings file.



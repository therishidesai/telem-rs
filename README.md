# tracing-mcap

Experimenting with a higher level logging implementation for MCAP files.

My end goal is to leverage the rust Tracing crate and add a `Layer`
that will log structs to an MCAP file and also log spans.


Currently, this crate contains an `McapLogger` struct and
`McapMessage` trait. A user will implement the `McapMessage` trait for
any struct they want logged into an MCAP file. The logger then takes
any struct that impl's `McapMessage`.

See `examples/tracing.rs`



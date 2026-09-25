//! The `rustyecon` binary: a thin frontend over `rustyecon-engine`. It parses arguments, reads
//! and writes files, and turns results into exit codes. The tick loop, resume and the
//! shadow-replay audit live in the engine (docs/ENGINE.md §7, §8). Behaviour comes from the tape,
//! never from a command-line switch (R4, E1).

fn main() {}

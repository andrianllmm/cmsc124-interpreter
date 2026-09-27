//! Stages the pipeline can stop at.

/// Where to stop the pipeline and print its output.
#[derive(Clone, Copy)]
pub enum Stage {
    Tokenize,
    Parse,
}

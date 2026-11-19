//! Cell mode, the backend for the Jupyter kernel in `editors/jupyter`.
//!
//! Unlike the REPL, which ends an entry once its brackets close or on a blank line,
//! it runs each cell whole, however it's split into lines. Cells share one interpreter.

use crate::evaluator::Interpreter;
use crate::pipeline;
use crate::stage::Stage;
use std::cell::RefCell;
use std::io::{self, BufRead, Write};
use std::rc::Rc;

/// Ends each cell on stdin and each part of its result on stdout.
/// Source and output are text, so it never appears in either.
const END: u8 = b'\0';

/// Runs cells from stdin until EOF. Each cell ends with `END`.
///
/// After each cell, stdout gets its output, `END`, its errors
/// (empty if it succeeded), then `END` again.
pub fn run() {
    let err = Buffer::default();
    let mut interpreter = Interpreter::with_output(Box::new(io::stdout()), Box::new(err.clone()));

    for cell in io::stdin().lock().split(END) {
        let Ok(cell) = cell else { break };
        // Like the REPL: echo each expression's value.
        let _ = pipeline::run(
            &mut interpreter,
            &String::from_utf8_lossy(&cell),
            Some(Stage::Eval),
        );

        let mut stdout = io::stdout().lock();
        let _ = stdout.write_all(&[END]);
        let _ = stdout.write_all(&err.take());
        let _ = stdout.write_all(&[END]);
        let _ = stdout.flush();
    }
}

/// Collects errors so they're sent after the cell's output, not mixed into it.
#[derive(Clone, Default)]
struct Buffer(Rc<RefCell<Vec<u8>>>);

impl Buffer {
    /// Returns everything written so far and empties the buffer.
    fn take(&self) -> Vec<u8> {
        std::mem::take(&mut *self.0.borrow_mut())
    }
}

impl Write for Buffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

use std::{cell::RefCell, io::Write, rc::Rc};

pub fn green_blink() {
    const ESC: &str = "\x1B[";
    const RESET: &str = "\x1B[0m";
    eprint!("\r{}42m{}K{}\r", ESC, ESC, RESET);
    std::io::stdout().flush().unwrap();
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(50));
        eprint!("\r{}40m{}K{}\r", ESC, ESC, RESET);
        std::io::stdout().flush().unwrap();
    });
}

pub(crate) trait RcWrap: Sized {
    fn wrap(self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self))
    }
}
impl<T> RcWrap for T where T: Sized {}

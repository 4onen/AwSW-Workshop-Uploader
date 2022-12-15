pub fn error_dialog(msg: &str) {
    let _ = native_dialog::MessageDialog::new()
        .set_type(native_dialog::MessageType::Error)
        .set_title("Error")
        .set_text(msg)
        .show_alert();
}

pub fn confirm_dialog(msg: &str) -> bool {
    let ans = native_dialog::MessageDialog::new()
        .set_type(native_dialog::MessageType::Warning)
        .set_title("Warning")
        .set_text(msg)
        .show_confirm();

    match ans {
        Ok(b) => b,
        Err(e) => {
            error_dialog(
                format!("Error retrieving confirmation: {:?}\nAssuming False...", e).as_str(),
            );
            false
        }
    }
}

pub trait ErrorDialogUnwrapper<T> {
    fn expect_or_dialog(self, msg: &str) -> T;
}

impl<T> ErrorDialogUnwrapper<T> for Option<T> {
    #[track_caller]
    fn expect_or_dialog(self, msg: &str) -> T {
        self.unwrap_or_else(|| {
            error_dialog(msg);
            None.expect(msg)
        })
    }
}

impl<T, E: std::fmt::Debug> ErrorDialogUnwrapper<T> for Result<T, E> {
    #[track_caller]
    fn expect_or_dialog(self, msg: &str) -> T {
        self.unwrap_or_else(|e| {
            error_dialog(format!("{} {:?}", msg, &e).as_str());
            Err(e).expect(msg)
        })
    }
}

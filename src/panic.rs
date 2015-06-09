use terminal;

//pub const GLOBAL_WRITER: &'static terminal::Terminal = ;

#[lang = "panic_fmt"] #[inline(never)] #[cold]
pub extern fn panic_impl(msg: ::core::fmt::Arguments,
                         file: &'static str,
                         line: usize) -> !
{
  unsafe {
    use io::Writer;
    let _ = write!(terminal::get_terminal(), "{}:{} {}", file, line, msg);
    ::core::intrinsics::abort();
  }
}

pub unsafe fn init() {
  terminal::get_terminal().clear_screen()
}

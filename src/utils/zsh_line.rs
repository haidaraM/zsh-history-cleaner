use std::io::BufRead;

struct Guard<'a> {
    buf: &'a mut Vec<u8>,
    len: usize,
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        unsafe {
            self.buf.set_len(self.len);
        }
    }
}

/// An extension of [BufRead] which handles Zsh's metafied format.
/// See: <https://www.zsh.org/mla/users/2011/msg00154.html>
///
/// This trait provides methods to read lines from a buffer while
/// decoding these metafied characters.
///
/// The implementation follows the unsafe pattern used in the standard library
/// to manipulate the underlying buffer directly for performance reasons.
pub(crate) trait ZshLineRead: BufRead {
    fn read_zsh_line(&mut self, buf: &mut String) -> Result<usize, std::io::Error> {
        let mut g = Guard {
            len: buf.len(),
            buf: unsafe { buf.as_mut_vec() },
        };
        let read = self.read_until(b'\n', g.buf)?;

        let mut src = g.len;
        let mut dst = g.len;
        while src < g.buf.len() {
            if g.buf[src] == 0x83 && src + 1 < g.buf.len() {
                g.buf[dst] = g.buf[src + 1] ^ 0x20;
                src += 2;
            } else {
                g.buf[dst] = g.buf[src];
                src += 1;
            }
            dst += 1;
        }
        g.buf.truncate(dst);

        let appended = unsafe { g.buf.get_unchecked(g.len..) };
        if str::from_utf8(appended).is_err() {
            // copied from INVALID_UTF8
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "stream did not contain valid UTF-8",
            ))
        } else {
            g.len = g.buf.len();
            Ok(read)
        }
    }

    fn zsh_lines(self) -> ZshLines<Self>
    where
        Self: Sized,
    {
        ZshLines { buf: self }
    }
}

impl<B: BufRead + ?Sized> ZshLineRead for B {}

pub struct ZshLines<B> {
    buf: B,
}

impl<B: BufRead> Iterator for ZshLines<B> {
    type Item = Result<String, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = String::new();
        match self.buf.read_zsh_line(&mut buf) {
            Ok(0) => None,
            Ok(_n) => {
                if buf.ends_with('\n') {
                    buf.pop();
                    if buf.ends_with('\r') {
                        buf.pop();
                    }
                }
                Some(Ok(buf))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

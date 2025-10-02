use std::io::Write;

use anyhow::Context;
use term::Terminal;

pub struct TestTerminal {
    buffer: Vec<u8>,
}

impl Default for TestTerminal {
    fn default() -> Self {
        Self::new()
    }
}

impl TestTerminal {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }
}

impl TryFrom<TestTerminal> for String {
    type Error = anyhow::Error;

    fn try_from(value: TestTerminal) -> Result<Self, Self::Error> {
        String::from_utf8(value.buffer).context("Not UTF-8")
    }
}

impl Write for TestTerminal {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.buffer.flush()
    }
}

impl Terminal for TestTerminal {
    type Output = Vec<u8>;

    fn fg(&mut self, _: term::color::Color) -> term::Result<()> {
        Ok(())
    }

    fn bg(&mut self, _: term::color::Color) -> term::Result<()> {
        Ok(())
    }

    fn attr(&mut self, _: term::Attr) -> term::Result<()> {
        Ok(())
    }

    fn supports_attr(&self, _: term::Attr) -> bool {
        true
    }

    fn reset(&mut self) -> term::Result<()> {
        Ok(())
    }

    fn supports_reset(&self) -> bool {
        true
    }

    fn supports_color(&self) -> bool {
        true
    }

    fn cursor_up(&mut self) -> term::Result<()> {
        Ok(())
    }

    fn delete_line(&mut self) -> term::Result<()> {
        Ok(())
    }

    fn carriage_return(&mut self) -> term::Result<()> {
        Ok(())
    }

    fn get_ref(&self) -> &Self::Output {
        &self.buffer
    }

    fn get_mut(&mut self) -> &mut Self::Output {
        &mut self.buffer
    }

    fn into_inner(self) -> Self::Output
    where
        Self: Sized,
    {
        self.buffer
    }
}

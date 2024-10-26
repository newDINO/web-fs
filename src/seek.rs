use std::{
    io::{Error, Result, SeekFrom},
    pin::Pin,
    task::{Context, Poll},
};

use futures_lite::AsyncSeek;

use crate::File;

const SEEK_ERROR: &str = "More cursor to negative value";

impl AsyncSeek for File {
    /// File System API dosen't fully expose the cursor of the file, so this is a simulated one and does not actually require async.
    fn poll_seek(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        pos: SeekFrom,
    ) -> Poll<Result<u64>> {
        match pos {
            SeekFrom::Current(offset) => {
                self.cursor = self
                    .cursor
                    .checked_add_signed(offset)
                    .ok_or(Error::other(SEEK_ERROR))?
            }
            SeekFrom::End(offset) => {
                self.cursor = self
                    .size
                    .checked_add_signed(offset)
                    .ok_or(Error::other(SEEK_ERROR))?
            }
            SeekFrom::Start(offset) => self.cursor += offset,
        }
        Poll::Ready(Ok(self.cursor))
    }
}

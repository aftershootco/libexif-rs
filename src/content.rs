use std::slice;

use crate::bindings::*;

use crate::bits::*;
use crate::entry::Entry;
use crate::internal::*;

/// Container for all EXIF data in a single [IFD](enum.IFD.html).
pub struct Content<'a> {
    inner: &'a mut ExifContent,
}

impl<'a> Content<'a> {
    /// Return the IFD for the content.
    pub fn ifd(&self) -> IFD {
        IFD::from_libexif(unsafe { exif_content_get_ifd(self.inner as *const _ as *mut _) })
    }

    /// Return the number of [entries](struct.Entry.html) in the IFD.
    pub fn len(&self) -> usize {
        self.inner.count as usize
    }

    pub fn is_empty(&self) -> bool {
        self.inner.count == 0
    }

    /// Iterate over the [entries](struct.Entry.html) in the IFD.
    pub fn entries(&self) -> impl ExactSizeIterator<Item = Entry> {
        Entries {
            entries: unsafe {
                slice::from_raw_parts(self.inner.entries, self.inner.count as usize)
            },
            index: 0,
        }
    }
}

impl<'a> FromLibExif<&'a mut ExifContent> for Content<'a> {
    fn from_libexif(content: &'a mut ExifContent) -> Content<'a> {
        Content { inner: content }
    }
}

struct Entries<'a> {
    entries: &'a [*mut ExifEntry],
    index: usize,
}

impl<'a> Iterator for Entries<'a> {
    type Item = Entry<'a>;

    fn next(&mut self) -> Option<Entry<'a>> {
        if self.index < self.entries.len() {
            let entry = self.entries[self.index];
            self.index += 1;

            Some(Entry::from_libexif(unsafe { &mut *entry }))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.entries.len() - self.index;

        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for Entries<'a> {
    fn len(&self) -> usize {
        self.entries.len()
    }
}

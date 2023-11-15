pub struct Writer<'a> {
    pub buf: &'a mut [u8],
    pub index: usize,
}

impl<'a> Writer<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, index: 0 }
    }

    pub fn push(&mut self, item: u8) {
        self.buf[self.index] = item;
        self.index += 1;
    }

    pub fn extend_from_slice(&mut self, src: &[u8]) {
        assert!(src.len() <= self.buf.len() - self.index);
        self.buf[self.index..self.index + src.len()].copy_from_slice(src);
        self.index += src.len();
    }

    pub fn to_bytes(&self) -> &[u8] {
        &self.buf[..self.index]
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Reader {
    pub index: usize,
    pub end: usize,
}

impl Reader {
    pub fn eof(&self) -> bool {
        self.index >= self.end
    }

    pub fn new_with_len(len: usize) -> Self {
        Self { index: 0, end: len }
    }

    pub fn set_len(&mut self, len: usize) {
        self.end = len;
    }

    pub fn read_byte(&mut self, buf: &[u8]) -> u8 {
        if self.eof() {
            panic!("read_byte attempt to read past end of buffer");
        } else {
            let byte = buf[self.index];
            self.index += 1;
            byte
        }
    }

    pub fn read_bytes<const COUNT: usize>(&mut self, buf: &[u8]) -> [u8; COUNT] {
        if self.index + COUNT > self.end {
            panic!("read_bytes attempt to read past end of buffer");
        } else {
            let mut tmp: [u8; COUNT] = [0; COUNT];
            tmp.copy_from_slice(&buf[self.index..self.index + COUNT]);
            self.index += COUNT;
            tmp
        }
    }

    pub fn read_slice<'a>(&mut self, len: usize, buf: &'a [u8]) -> &'a [u8] {
        if self.index + len > self.end {
            panic!("read_slice attempt to read past end of buffer");
        } else {
            let slice = &buf[self.index..self.index + len];
            self.index += len;
            slice
        }
    }
}

impl Default for Reader {
    fn default() -> Self {
        Self {
            index: 0,
            end: usize::MAX - 1000,
        }
    }
}

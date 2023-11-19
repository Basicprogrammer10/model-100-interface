pub struct BinParser<'a> {
    raw: &'a [u8],
    idx: usize,
}

#[allow(dead_code)]
impl<'a> BinParser<'a> {
    pub fn new(raw: &'a [u8]) -> Self {
        Self { raw, idx: 0 }
    }

    pub fn read_u8(&mut self) -> u8 {
        let val = self.raw[self.idx];
        self.idx += 1;
        val
    }

    pub fn read_array<const N: usize>(&mut self) -> [u8; N] {
        let mut out = [0; N];
        out.copy_from_slice(&self.raw[self.idx..self.idx + N]);
        self.idx += N;
        out
    }

    pub fn skip(&mut self, n: usize) {
        self.idx += n;
    }

    pub fn get(&self, idx: usize) -> u8 {
        self.raw[idx]
    }
}

const HIGH_BITS: usize = 5;
const SPLIT: usize = 1 << HIGH_BITS;

pub struct Intset {
    items: [Vec<u64>; SPLIT],
}

impl Intset {
    #[inline]
    pub fn insert(&mut self, value: u64) {
        let bucket = (value >> (64 - HIGH_BITS)) as usize;
        match self.items[bucket].binary_search(&value) {
            Ok(_) => {} // element already in vector @ `pos`
            Err(pos) => self.items[bucket].insert(pos, value),
        }
    }

    pub fn clear(&mut self) {
        for i in 0..SPLIT {
            self.items[i as usize].clear();
        }
    }

    #[inline]
    pub fn contains(&self, value: &u64) -> bool {
        let bucket = value >> (64 - HIGH_BITS);
        self.items[bucket as usize].binary_search(value).is_ok()
    }

    pub fn new(cap: usize) -> Self {
        let mut ret = Self {
            items: Default::default(),
        };
        for i in 0..SPLIT {
            ret.items[i as usize].reserve(cap);
        }
        ret
    }
}

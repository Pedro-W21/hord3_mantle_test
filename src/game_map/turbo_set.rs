use to_from_bytes_derive::{FromBytes, ToBytes};



#[derive(Clone, ToBytes, FromBytes, Debug, PartialEq)]
pub struct TurboSet {
    highest_id:usize,
    data:Vec<u64>,
    batches:Vec<usize>,
    len:usize,
}

pub const BATCH_POW:u32 = 10;
pub const BATCH_SIZE:usize = 2_usize.pow(BATCH_POW);
pub const BATCH_INDEX_POW:u32 = BATCH_POW - 6;
pub const BATCH_INDEX_SIZE:usize = 2_usize.pow(BATCH_INDEX_POW);

impl TurboSet {
    pub fn new(start_capa:usize, max_capa:usize) -> Self {
        Self { highest_id: 0, data: Vec::with_capacity(start_capa), batches: Vec::with_capacity(max_capa/BATCH_SIZE), len:0 }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn contains(&self, id:usize) -> bool {
        let index = id >> 6;
        let sub_index = id & 0b111111;
        if index < self.data.len() {
            let mask = BIT_MASKS[sub_index];
            //dbg!(id, index, sub_index);
            self.data[index] & mask == mask
        }
        else {
            false
        }
    }

    pub fn get_data(&self) -> &Vec<u64> {
        &self.data
    }

    pub fn insert(&mut self, id:usize) {
        let index = id >> 6;
        let sub_index = id & 0b111111;
        let mask = BIT_MASKS[sub_index];
        let batch_index = id >> BATCH_POW;
        if index < self.data.len() {
            let mut target = self.data[index];
            if target & mask == 0 {
                self.len += 1;
                self.batches[batch_index] += 1;
            }
            //dbg!(id, index, sub_index);
            self.data[index] = target | mask;
        }
        else {
            self.highest_id = id;
            for i in self.data.len()..=index {
                self.data.push(0);
            }
            for i in self.batches.len()..=batch_index {
                self.batches.push(0);
            }

            let mut target = self.data[index];
            if target & mask == 0 {
                self.batches[batch_index] += 1;
                self.len += 1;
            }
            //dbg!(id, index, sub_index);
            self.data[index] = target | mask;
        }
    }

    pub fn remove(&mut self, id:usize) {
        let index = id >> 6;
        let sub_index = id & 0b111111;
        let mask = BIT_MASKS[sub_index];
        let batch_index = id >> BATCH_POW;
        if index < self.data.len() {
            let target = self.data[index];
            if target & mask > 0 {
                self.batches[batch_index] -= 1;
                self.len -= 1;
                self.data[index] = target ^ mask;
            }
        }
    }

    pub fn iter<'a>(&'a self) -> TurboSetIterator<'a> {
        TurboSetIterator { set: self, current_index: 0, current_subindex: 0, current_id: 0, current_batch: 0 }
    }


}

const BIT_MASKS:[u64 ; 64] = generate_bitmasks();


const fn generate_bitmasks() -> [u64 ; 64] {
    let mut masks = [0 ; 64];
    let mut i = 0;
    while i < 64 {
        masks[i] = 2_u64.pow(i as u32);
        i += 1;
    }
    masks
}

pub struct TurboSetIterator<'a> {
    set:&'a TurboSet,
    current_index:usize,
    current_subindex:usize,
    current_id:usize,
    current_batch:usize,
}

impl<'a> TurboSetIterator<'a> {
    fn advance_indices(&mut self) {
        self.current_id += 1;
        self.current_subindex = self.current_id & 0b111111;
        if self.current_subindex == 0 {
            let next_index = self.current_index + 1;
            if next_index >> BATCH_INDEX_POW != self.current_batch {
                for batch in &self.set.batches[(next_index >> BATCH_INDEX_POW)..] {
                    self.current_batch += 1;
                    if *batch == 0 {
                        self.current_index += BATCH_INDEX_SIZE;
                        self.current_id += BATCH_SIZE;
                    }
                    else {
                        break;
                    }
                }
                self.current_index += 1;
            }
            else if next_index < self.set.data.len() && self.set.data[next_index] == 0 {
                //dbg!(self.current_id);
                for d in &self.set.data[next_index..] {
                    if *d == 0 {
                        self.current_index += 1;
                        self.current_id = self.current_index << 6;
                        //dbg!(self.current_id);
                    }
                    else {
                        break;
                    }
                }
            }
            else {
                self.current_index = next_index;
            }
        }
    }
    fn get_current(&self) -> Option<usize> {
        let mask = BIT_MASKS[self.current_subindex];
        if self.set.data[self.current_index] & mask == mask {
            Some(self.current_id)
        }
        else {
            None
        }
    }
}

impl<'a> Iterator for TurboSetIterator<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index < self.set.data.len() {
            if let Some(id) = self.get_current() {
                //dbg!("a");
                //dbg!(self.current_index, self.current_subindex, self.current_id, self.current_batch);
                //dbg!(self.set.data[self.current_index]);
                self.advance_indices();
                Some(id)
            }
            else {
                //dbg!("b");
                //dbg!(self.current_index, self.current_subindex, self.current_id, self.current_batch);
                //dbg!(self.set.data[self.current_index], BIT_MASKS[self.current_subindex]);
                while let None = self.get_current() {
                    self.advance_indices();
                    if self.current_index >= self.set.data.len() {
                        return None;
                    }
                }
                //dbg!("c");
                //dbg!(self.current_index, self.current_subindex, self.current_id, self.current_batch);
                //dbg!(self.set.data[self.current_index], BIT_MASKS[self.current_subindex]);
                let curr = self.get_current();
                self.advance_indices();
                curr
            }
        }
        else {
            None
        }
    }
}
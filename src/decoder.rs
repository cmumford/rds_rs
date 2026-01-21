pub enum BlockErrorCount {
    None = 0,        // No block errors.
    OneToTwo = 1,    // 1-2 block errors.
    ThreeToFive = 2, // 3-5 block errors.
    SixPlus = 3,     // 6+ block errors.
}

pub struct RdsBlock {
    pub block: u16,                  // The RDS block data.
    pub num_errors: BlockErrorCount, // Number of bit errors in the block.
}

pub struct RdsBlocks {
    pub a: Option<RdsBlock>,
    pub b: Option<RdsBlock>,
    pub c: Option<RdsBlock>,
    pub d: Option<RdsBlock>,
}

pub struct Decoder {
    // decoder fields
}

impl Decoder {
    pub fn new() -> Self {
        Decoder {
            // initialize fields
        }
    }

    pub fn decode(&self, blocks: &RdsBlocks) {
        // decoding logic
    }
}

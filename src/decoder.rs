pub enum BlockErrorCount {
    None = 0,        // No block errors.
    OneToTwo = 1,    // 1-2 block errors.
    ThreeToFive = 2, // 3-5 block errors.
    SixPlus = 3,     // 6+ block errors.
}

pub struct RdsBlock {
    block: u16,                  // The RDS block data.
    num_errors: BlockErrorCount, // Number of bit errors in the block.
}

pub struct RdsBlocks {
    a: RdsBlock,
    b: RdsBlock,
    c: RdsBlock,
    d: RdsBlock,
}

pub struct Decoder {
    // decoder fields
}

impl Decoder {
    fn new() -> Self {
        Decoder {
            // initialize fields
        }
    }

    fn decode(&self, blocks: &RdsBlocks) {
        // decoding logic
    }
}

// Section 3.2.1.6.1 describes how 8-bit values are mapped to either
// UHF frequencies, LF/MF frequencies, or othe special codes. These
// are described in three different tables:
//
// Table 10: VHF code table and
// Table 11: Special meanings code table
// Table 12: LF/MF code table - for ITU regions 1 and 3 (9 kHz spacing)
//
// They essentially correspond to these categories.
enum AfCodeEntryType {
    Unassigned,    // Unused/unassigned/filler.
    UhfFrequency,  // A UHF frequency value.
    LmMfFrequency, // A LM/MF frequency value.
    AltFreqCount,  // Num. of AF's to follow.
    LmMfFollows,   // Next entry is a LM/MF freq.
}

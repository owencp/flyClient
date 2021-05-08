#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

// Import from `core` instead of from `std` since we are in no-std mode
use core::result::Result;

// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
use alloc::{collections::btree_set::BTreeSet, vec::Vec};

use crate::store::{Batch, Error, IteratorDirection, Store};
use ckb_types::{
    bytes::Bytes,
    core::{BlockNumber, HeaderView, ScriptHashType, packed::Byte32},
    h256, packed,
    prelude::*,
    utilities::compact_to_difficulty,
    H256, U256,
};
use ckb_hash::{blake2b_256, new_blake2b};


entry!(entry);
default_alloc!();

/// Program entry
fn entry() -> i8 {
    // Call main function and return error code
    match main() {
        Ok(_) => 0,
        Err(err) => err as i8,
    }
}

/// Error
#[repr(i8)]
enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    // Add customized errors here...
    InvalidArgument,
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}

struct  MmrPeak {
    //current MMR root
    peakValue:Byte32,
    //accumulated difficulty of MMR nodes below peak
    accumDifficulty:uint128,
}

struct MmrPeaks {
    //the highest peak of the mmrs
    highestPeak:uint8,
    //previous block processed by cell
    previousBlock:BlockNumber,
    //current MMR root
    mmrRoot:Byte32,
    //all the mmr peaks
    mmrPeaks:Vec<MmrPeak>,
}

struct header {
    headerHash:Byte32
    difficulty:uint128
    blockNumber:BlockNumber
}

impl MmrPeaks {
	pub fn new(prev_block:BlockNumber) -> Self {
        Self {
            highestPeak:0 as u8,
            previousBlock:prev_block,
            mmrRoot:Byte32::default(),
            mmrPeaks:Vec::with_capacity(64),
        }
    }
	
	fn add_to_mmr(&mut self, block_hash:Byte32, difficulty:uint128, height:uint8){

        if let Some(m_peak) = self.mmrPeaks.get_mut(height.clone()) {
            //peakValue != 0, then value exists at this height
            if m_peak.peakValue != 0  {
                //combine the peak with new leaf
                //hash together existing peak and new leaf, add their difficulty values
                let memBlockHash = {
                    let mut blake2b = new_blake2b();
                    blake2b.update(m_peak.peakValue.as_bytes());
                    blake2b.update(m_peak[height].accumDifficulty.to_le_bytes());
                    blake2b.update(block_hash.as_bytes());
                    blake2b.update(difficulty.to_le_bytes());
                    let mut ret = [0; 32];
                    blake2b.finalize(&mut ret);
                    Bytes::from(ret.to_vec())
                }
                let memAccumDifficulty = m_peak.accumDifficulty + difficulty;

                //clear existing peak data at this height
                m_peak.peakValue = 0;
                m_peak.accumulatedDifficulty = 0;

                //call recursively up the tree
                add_to_mmr(memBlockHash, memAccumDifficulty, height++);

            } else {
                //store input value at this height
                m_peak.peakValue = blockHash;
                m_peak.accumDifficulty = difficulty;

                //update HIGHEST_PEAK
                if height > self.highestPeak {
                    self.highestPeak = height;
                }
            }
        }
    }
	
	//get mmrRoot
    fn bag_Peaks(&mut self){
        let memMMRvalue:Byte32 = Byte32::default();
        self
            .mmrPeaks
            .iter()
            .filter(|mPeak| mPeak.peakValue != 0)
            .for_each(|mPeak| memMMRvalue = {
                    let mut blake2b = new_blake2b();
                    blake2b.update(memMMRvalue.as_bytes());
                    blake2b.update(mPeak.peakValue.to_le_bytes());
                    blake2b.update(mPeak.accumDifficulty.to_le_bytes());
                    let mut ret = [0; 32];
                    blake2b.finalize(&mut ret);
                    Bytes::from(ret.to_vec())
                });

        self.mmrRoot = memMMRvalue;
    }
	
}

main()-> Result<(), Error> { 
    //load header
	//calc mmr
}

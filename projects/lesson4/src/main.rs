//! Substrate Node Template CLI library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;

pub use sc_cli::{VersionInfo, IntoExit, error};

fn main() -> Result<(), cli::error::Error> {
	let version = VersionInfo {
		name: "Substrate Node",
		commit: env!("VERGEN_SHA_SHORT"),
		version: env!("CARGO_PKG_VERSION"),
		executable_name: "substrate-kitties",
		author: "Bryan Chen",
		description: "substrate-kitties",
		support_url: "support.anonymous.an",
	};

	cli::run(std::env::args(), cli::Exit, version)
}

/*

fn combine_dna(kitty1_dna: u8, kitty2_dna: u8, selector: u8) -> u8 {
		let temp1:u8;
		let temp2:u8;
		temp1 =  kitty1_dna & selector;
		temp2 =  kitty2_dna & !selector;
		let out:u8;
		out = temp1 | temp2;
		return out;
}


fn main() {
	let dna1:u8 = 0b11110000;
	let dna2:u8 = 0b11001100;
	let selector:u8 = 0b10101010;
	let out = combine_dna(dna1, dna2, selector);
	println!("0b{:08b}", out);
}
*/
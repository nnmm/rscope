use getopts::Options;
use std::env;
use std::process;

#[derive(StructOpt, Debug)]
#[structopt(name = "rscope", about = "A simple software oscilloscope that reads from JACK", author = "")]
pub struct OscOpts {
	#[structopt(long = "magnification", short = "m", default_value = "1.0")]
	pub magnification: f32,
	#[structopt(long = "samples", short = "s", default_value = "1024")]
	pub samples_per_frame: usize
}

pub fn get_options() -> OscOpts {
	let args: Vec<String> = env::args().collect();
	let program = args[0].clone();

	let mut opts = Options::new();
	opts.optopt("m", "magnification", "set magnification", "DOUBLE");
	opts.optopt("s", "samples", "set samples per frame", "INT");
	opts.optflag("h", "help", "print this help menu");
	let matches = match opts.parse(&args[1..]) {
	    Ok(m) => { m }
	    Err(f) => { panic!(f.to_string()) }
	};

	if matches.opt_present("h") {
	    print_usage(&program, opts);
	    process::exit(0);
	}
	let mag = match matches.opt_str("m") {
	    Some(m) => { match m.parse() {
	        Ok(m) => { m }
	        Err(_) => { panic!("Error: magnification not a double.") }
	    } }
	    None => { 1.0 }
	};
	let spf = match matches.opt_str("s") {
	    Some(s) => { match s.parse() {
	        Ok(s) => { s }
	        Err(_) => { panic!("Error: samples not an int.") }
	    } }
	    None => { 1024 }
	};

	OscOpts { magnification: mag, samples_per_frame: spf }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

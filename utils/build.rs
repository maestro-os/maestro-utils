fn main() {
	// Building the C code
	println!("cargo:rerun-if-changed=src/termios.c");
	cc::Build::new()
		.file("src/termios.c")
		.compile("utils")
}

fn main() {
    println!("cargo:rustc-link-lib=crypt");

	println!("cargo:rerun-if-changed=src/hash.c");
	println!("cargo:rerun-if-changed=src/termios.c");

	cc::Build::new()
        .static_flag(true)
		.file("src/hash.c")
		.file("src/termios.c")
		.compile("utils")
}

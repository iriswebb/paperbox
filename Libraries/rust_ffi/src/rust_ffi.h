extern "C" {
	/// Rust functions called from C++
	extern void rust_setup();
	extern void rust_loop(int fc);
}

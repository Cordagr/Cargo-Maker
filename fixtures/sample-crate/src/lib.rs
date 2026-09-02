#[no_mangle]
pub extern "C" fn sample_crate_add(a: i32, b: i32) -> i32 {
    a + b
}

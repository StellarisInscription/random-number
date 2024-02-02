mod service;

#[allow(dead_code)]
#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    use candid::Nat;
    candid::export_service!();
    std::fs::write("random-num/random_number.did", __export_service()).unwrap()
}



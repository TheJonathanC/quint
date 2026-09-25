fn main() {
    let err = ureq::get("https://httpbin.org/status/404").call().unwrap_err();
    println!("{:?}", err);
}

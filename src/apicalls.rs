pub fn get_proxy_url()-> String {
    let url = std::env::var("PROXY_SERVICE").expect("Not Declared PROXY_SERVICE");
    url.trim().to_string()
}

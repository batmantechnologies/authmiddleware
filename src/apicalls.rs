
pub fn get_token_url()-> String {
    let url = std::env::var("TOKENSERVICE_URL").expect("Not Declared TOKENSERVICE_URL");
    url
}

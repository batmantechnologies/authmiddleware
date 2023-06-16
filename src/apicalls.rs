
pub fn get_commonservices_url()-> String {
    let url = std::env::var("COMMON_SERVICES").expect("Not Declared COMMON_SERVICES");
    url
}

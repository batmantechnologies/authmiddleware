use commonfunctions;

pub fn get_proxy_url()-> String {
    commonfunctions::get_desirialised("PROXY_SERVICE")
}

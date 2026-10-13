use std::str::FromStr;

pub enum Api_Type {
    SOAP,
    REST,
    GRAPHQL,
}
impl FromStr for Api_Type {
    pub fn from_str(string: &str) -> Result<Self, Error> {
        match string {
            "SOAP" => Ok(Api_Type::SOAP),
            "REST" => Ok(Api_Type::REST),
            "GRAPHQL" => Ok(Api_Type::GRAPHQL),
            _ => Err("Not a valid type")
        }
    }
}
pub struct RequestBuilder {
    query_type: Api_Type,
    url: String,
    payload: Optional<String>
}
pub impl RequestBuilder {
    pub fn new()
}
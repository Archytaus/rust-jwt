use std::collections::BTreeMap;
use rustc_serialize::Decodable;
use rustc_serialize::base64::{
    FromBase64,
    ToBase64,
};
use rustc_serialize::json::{
    self,
    Decoder,
    Json,
};
use Component;
use error::Error;
use BASE_CONFIG;

#[derive(Debug, Default, PartialEq)]
pub struct Claims {
    pub reg: Registered,
    pub private: BTreeMap<String, Json>,
}

#[derive(Debug, Default, PartialEq, RustcDecodable, RustcEncodable)]
pub struct Registered {
    pub iss: Option<String>,
    pub sub: Option<String>,
    pub aud: Option<String>,
    pub exp: Option<u64>,
    pub nbf: Option<u64>,
    pub iat: Option<u64>,
    pub jti: Option<String>,
}

/// JWT Claims. Registered claims are directly accessible via the `Registered`
/// struct embedded, while private fields are a map that contains `Json`
/// values.
impl Claims {
    pub fn new(reg: Registered) -> Claims {
        Claims {
            reg: reg,
            private: BTreeMap::new(),
        }
    }
}

impl Component for Claims {
    fn from_base64(raw: &str) -> Result<Claims, Error> {
        let data = try!(raw.from_base64());
        let s = try!(String::from_utf8(data));
        let tree = match try!(Json::from_str(&*s)) {
            Json::Object(x) => x,
            _ => return Err(Error::Format),
        };

        const FIELDS: [&'static str; 7] = [
            "iss", "sub", "aud",
            "exp", "nbf", "iat",
            "jti",
        ];

        let (reg, pri): (BTreeMap<_, _>, BTreeMap<_, _>) = tree.into_iter()
            .partition(|&(ref key, _)| {
                FIELDS.iter().any(|f| f == key)
            });

        let mut decoder = Decoder::new(Json::Object(reg));
        let reg_claims: Registered = try!(Decodable::decode(&mut decoder));

        Ok(Claims{
            reg: reg_claims,
            private: pri,
        })
    }

    fn to_base64(&self) -> Result<String, Error> {
        // Extremely inefficient
        let s = try!(json::encode(&self.reg));
        let mut tree = match try!(Json::from_str(&*s)) {
            Json::Object(x) => x,
            _ => return Err(Error::Format),
        };

        tree.extend(self.private.clone());

        let s = try!(json::encode(&tree));
        let enc = (&*s).as_bytes().to_base64(BASE_CONFIG);
        Ok(enc)
    }
}

#[cfg(test)]
mod tests {
    use std::default::Default;
    use claims::{Claims, Registered};
    use Component;

    #[test]
    fn from_base64() {
        let enc = "ew0KICAiaXNzIjogIm1pa2t5YW5nLmNvbSIsDQogICJleHAiOiAxMzAyMzE5MTAwLA0KICAibmFtZSI6ICJNaWNoYWVsIFlhbmciLA0KICAiYWRtaW4iOiB0cnVlDQp9";
        let claims = Claims::from_base64(enc).unwrap();

        assert_eq!(claims.reg.iss.unwrap(), "mikkyang.com");
        assert_eq!(claims.reg.exp.unwrap(), 1302319100);
    }

    #[test]
    fn multiple_types() {
        let enc = "ew0KICAiaXNzIjogIm1pa2t5YW5nLmNvbSIsDQogICJleHAiOiAxMzAyMzE5MTAwLA0KICAibmFtZSI6ICJNaWNoYWVsIFlhbmciLA0KICAiYWRtaW4iOiB0cnVlDQp9";
        let claims = Registered::from_base64(enc).unwrap();

        assert_eq!(claims.iss.unwrap(), "mikkyang.com");
        assert_eq!(claims.exp.unwrap(), 1302319100);
    }

    #[test]
    fn roundtrip() {
        let mut claims: Claims = Default::default();
        claims.reg.iss = Some("mikkyang.com".into());
        claims.reg.exp = Some(1302319100);
        let enc = claims.to_base64().unwrap();
        assert_eq!(claims, Claims::from_base64(&*enc).unwrap());
    }
}

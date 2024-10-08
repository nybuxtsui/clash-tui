use std::collections::HashMap;

use url::Url;

pub struct SubsUrl {
    url: Url,
    query: HashMap<String, String>,
}

impl SubsUrl {
    pub fn new<T: AsRef<str>>(url: T) -> anyhow::Result<Self> {
        let url = Url::parse(url.as_ref())?;
        let query: HashMap<String, String> = url.query_pairs().into_owned().collect();
        Ok(SubsUrl { url, query })
    }

    pub fn scheme(&self) -> String {
        self.url.scheme().to_owned()
    }

    pub fn server(&self) -> String {
        match self.url.host_str() {
            None => "",
            Some(v) => v,
        }
        .to_owned()
    }

    pub fn port(&self) -> u16 {
        self.url.port().unwrap_or(0)
    }

    pub fn username(&self) -> String {
        self.url.username().to_owned()
    }

    pub fn fragment(&self) -> String {
        match self.url.fragment() {
            None => "",
            Some(v) => v,
        }
        .to_owned()
    }

    pub fn password(&self) -> String {
        match self.url.password() {
            None => "",
            Some(v) => v,
        }
        .to_owned()
    }

    pub fn query(&self, name: &str) -> String {
        let v = self.query.get(name);
        match v {
            None => "",
            Some(v) => v,
        }
        .to_owned()
    }
}

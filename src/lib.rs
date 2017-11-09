extern crate url;

#[macro_use]
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate error_chain;

use std::borrow::Cow;
use serde_json::Value;
use err::{ErrorKind};

pub mod err {
    error_chain! {
        errors { 
            Parse {
                description("Couldn't parse input")
            }
        }

        foreign_links {
            StdFmt(::std::fmt::Error);
            Io(::std::io::Error);
            SerdeJson(::serde_json::Error);
        }
    }
}

mod deserializer;

trait ValueExtensions {
    fn find_or_insert_object<I: Into<String>>(&mut self, key: I) -> err::Result<&mut Self>;
    fn find_or_insert_array(&mut self) -> err::Result<&mut Self>;
    fn collect_keys<'a>(&'a self, pairs: &mut Vec<(String, Cow<'a, str>)>, buf: String) -> err::Result<()>;
    fn parse_key(&mut self, key: &str, outside: bool) -> err::Result<&mut Self>;
}

impl ValueExtensions for Value {
    fn find_or_insert_object<I: Into<String>>(&mut self, key: I) -> err::Result<&mut Self> {
        use serde_json::map::Map;
        match *self {
            Value::Object(ref mut map) => Ok(map.entry(key.into()).or_insert_with(|| Value::Null)),
            Value::Null => {
                *self = Value::Object(Map::new());
                self.find_or_insert_object(key)
            },
            _ => { Err(ErrorKind::Parse.into()) }
        }
    }
    fn find_or_insert_array(&mut self) -> err::Result<&mut Self> {
        match *self {
            Value::Array(ref mut arr) => {
                arr.push(Value::Null);
                Ok(arr.last_mut().ok_or_else(|| ErrorKind::Parse)?)
            },
            Value::Null => {
                *self = Value::Array(Vec::new());
                self.find_or_insert_array()
            },
            _ => { Err(ErrorKind::Parse.into()) }
        }
    }
    fn parse_key(&mut self, key: &str, outside: bool) -> err::Result<&mut Self> {
        if outside {
            if key.is_empty() {
                return Ok(self);
            }
            let mut iter = key.char_indices();
            loop {
                match iter.next() {
                    Some((0, '[')) => {
                        return self.parse_key(&key[1..], false);
                    },
                    Some((idx, '[')) => {
                        let pre = &key[..idx];
                        let post = &key[idx + 1..];
                        let new_node = self.find_or_insert_object(pre)?;
                        return new_node.parse_key(post, false);
                    },
                    None => { break; },
                    _ => { continue; }
                }
            }
            return Ok(self.find_or_insert_object(key)?)
        } else {
            if key.is_empty() {
                return Err(ErrorKind::Parse.into());
            }
            let mut iter = key.char_indices();
            loop {
                match iter.next() {
                    Some((0, ']')) => {
                        let post = &key[1..];
                        let new_node = self.find_or_insert_array()?;
                        return new_node.parse_key(post, true);
                    },
                    Some((idx, ']')) => {
                        let pre = &key[..idx];
                        let post = &key[idx + 1..];
                        let new_node = self.find_or_insert_object(pre)?;
                        return new_node.parse_key(post, true);
                    },
                    None => { break; },
                    _ => { continue; }
                }
            }
            return Err(ErrorKind::Parse.into());
        }
    }

    fn collect_keys<'a>(&'a self, pairs: &mut Vec<(String, Cow<'a, str>)>, mut buf: String) -> err::Result<()> {
        use std::fmt::Write as FmtWrite;
        match *self {
            Value::Null => {},
            Value::Bool(b) => { pairs.push((buf, b.to_string().into())); },
            Value::Number(ref num) => { pairs.push((buf, num.to_string().into())); },
            Value::String(ref s) => { pairs.push((buf, s.as_str().into())); },
            Value::Array(ref arr) => {
                write!(&mut buf, "[]")?;
                for node in arr {
                    node.collect_keys(pairs, buf.clone())?;
                }
            },
            Value::Object(ref map) => {
                for (k, node) in map {
                    let mut buf = buf.clone();
                    if buf.is_empty() {
                        write!(&mut buf, "{}", k)?;
                    } else {
                        write!(&mut buf, "[{}]", k)?;
                    }
                    node.collect_keys(pairs, buf)?;
                }
            }
        }
        Ok(())
    }
}

pub fn from_slice<D: ::serde::de::DeserializeOwned>(slice: &[u8]) -> err::Result<D> {
    let mut root = Value::Object(::serde_json::map::Map::new());
        
    for (key, value) in ::url::form_urlencoded::parse(slice) {
        let node = root.parse_key(key.as_ref(), true)?;
        *node = value.into_owned().into();
    }

    Ok(D::deserialize(deserializer::Deserializer(root))?)
}

pub fn from_bytes<D: ::serde::de::DeserializeOwned>(slice: &[u8]) -> err::Result<D> {
    from_slice(slice)
}

pub fn from_str<D: ::serde::de::DeserializeOwned>(s: &str) -> err::Result<D> {
    from_slice(s.as_ref())
}

pub fn from_reader<R: ::std::io::Read, D: ::serde::de::DeserializeOwned>(mut s: R) -> err::Result<D> {
    let mut buf = Vec::new();
    ::std::io::Read::read_to_end(&mut s, &mut buf)?;
    from_slice(buf.as_ref())
}

pub fn to_string<S: ::serde::Serialize>(s: &S) -> err::Result<String> {
    let value = ::serde_json::to_value(s)?;
    let mut pairs = Vec::new();
    value.collect_keys(&mut pairs, String::new())?;
    let out = ::url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(&pairs)
        .finish();
    Ok(out)
}


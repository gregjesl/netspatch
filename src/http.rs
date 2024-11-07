use std::collections::HashMap;

#[derive(Clone, PartialEq)]
pub enum HTTPMethod {
    GET,
    POST,
}

#[derive(Clone)]
pub struct RequestLine {
    pub method: HTTPMethod,
    pub path: String,
    pub version: String
}

impl RequestLine {
    pub fn parse(input: &String) -> Result<Self, HTTPResponseCode> {
        // Parse the method
        if !input.contains(' ') {
            return Err(HTTPResponseCode::BadRequest);
        }
        let (method_str, rem) =  input.split_once(' ').unwrap();
        let method = match method_str {
            "GET" => HTTPMethod::GET,
            "POST" => HTTPMethod::POST,
            &_ => return Err(HTTPResponseCode::MethodNotAllowed)
        };

        // Parse path
        if !rem.contains(' ') {
            return Err(HTTPResponseCode::BadRequest);
        }
        let (fullpath, version) = rem.split_once(' ').unwrap();
        assert!(fullpath.len() > 0);
        if fullpath.chars().next().unwrap() != '/' {
            return Err(HTTPResponseCode::BadRequest);
        }
        let (_, path) = fullpath.split_once('/').unwrap();
        return Ok(Self {
            method,
            path: path.to_string(),
            version: version.to_string(),
        });
    }
}

pub struct HTTPRequest {
    pub method: HTTPMethod,
    pub uri: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HTTPRequest {
    pub fn parse(raw: Vec<String>) -> Result<Self, HTTPResponseCode> {
        if raw.len() == 0 {
            return Err(HTTPResponseCode::BadRequest);
        }
        let line = RequestLine::parse(raw.first().unwrap())?;
        let mut headers: HashMap<String, String> = HashMap::new();
        for i in 1..raw.len() {
            let header = raw.get(i).unwrap();
            if !header.contains(": ") {
                return Err(HTTPResponseCode::BadRequest);
            }
            let (key, value) = header.split_once(": ").unwrap();
            if headers.contains_key(key) {
                return Err(HTTPResponseCode::BadRequest);
            }
            headers.insert(key.to_string(), value.to_string());
        }
        return Ok(Self {
            method: line.method,
            uri: line.path,
            version: line.version,
            headers,
            body: String::new(),
        })
    }
}

#[derive(Clone)]
pub enum HTTPResponseCode {
    OK,
    NoContent,
    BadRequest,
    NotFound,
    MethodNotAllowed,
    Conflict,
    InternalServerError,
    HTTPVersionNotSupported
}

impl HTTPResponseCode {
    fn to_string(&self) -> String {
        return match self {
            Self::OK => "OK".to_string(),
            Self::NoContent => "No Content".to_string(),
            Self::BadRequest => "Bad Request".to_string(),
            Self::NotFound => "Not Found".to_string(),
            Self::MethodNotAllowed => "Method Not Allowed".to_string(),
            Self::Conflict => "Conflict".to_string(),
            Self::InternalServerError => "Internal Server Error".to_string(),
            Self::HTTPVersionNotSupported => "HTTP Version Not Supported".to_string(),
        }
    }

    fn to_code(&self) -> i32 {
        return match self {
            Self::OK => 200,
            Self::NoContent => 204,
            Self::BadRequest => 400,
            Self::NotFound => 404,
            Self::MethodNotAllowed => 405,
            Self::Conflict => 409,
            Self::InternalServerError => 500,
            Self::HTTPVersionNotSupported => 505
        }
    }
}

#[derive(Clone)]
pub struct HTTPResponse {
    version: String,
    status: HTTPResponseCode,
    headers: HashMap<String, String>,
    pub content: String,
}

impl HTTPResponse {
    pub fn new(code: HTTPResponseCode) -> Self {
        return Self {
            version: "HTTP/1.1".to_string(),
            status: code,
            headers: HashMap::new(),
            content: String::new()
        };
    }

    pub fn as_string(&self) -> String {
        let mut result = format!("{} {} {}\r\n", self.version, self.status.to_code(), self.status.to_string());
        let mut headers = self.headers.clone();
        if !headers.contains_key("Content-Length") {
            headers.insert("Content-Length".to_string(), self.content.len().to_string());
        } else {
            *headers.get_mut("Content-Length").unwrap() = self.content.len().to_string();
        }
        for (key, value) in headers {
            let header = format!("{key}: {value}\r\n").to_string();
            result.push_str(&header);
        }
        if self.content.len() > 0 {
            result.push_str("\r\n");
            result.push_str(&self.content);
        }
        return result;
    }
}
use std::{collections::HashMap, io::{self, BufRead, BufReader, Read}, net::TcpStream, str};

#[derive(Clone, PartialEq)]
pub enum HTTPMethod {
    GET,
    POST,
}

impl HTTPMethod {
    fn to_string(&self) -> String {
        return match self {
            Self::GET => "GET".to_string(),
            Self::POST => "POST".to_string(),
        }
    }
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

pub trait HTTPMessage {
    fn headers(&self) -> &HashMap<String, String>;

    fn expected_body_length(&self) -> usize {
        let key: String = "Content-Length".to_string();
        if !self.headers().contains_key(&key) {
            return 0
        };
        let length_str = self.headers().get(&key).unwrap();
        let err = format!("Could not parse Content-Length: {}", length_str);
        return length_str.parse::<usize>().expect(&err);
    }

    fn read_body(&self, mut reader: BufReader<&TcpStream>) -> Result<String, io::Error> {
        let mut result = String::new();
        let body_len = self.expected_body_length();
        while result.len() < body_len {
            let mut buf  = [0_u8; 1];
            reader.read_exact(&mut buf)?;
            result.push_str(str::from_utf8(&buf).expect("UTF8 parsing error"));
        }
        return Ok(result);
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
    pub fn new(method: HTTPMethod, uri: String) -> Self {
        return Self {
            method,
            uri,
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            body: String::new()
        };
    }

    pub fn to_string(&self) -> String {
        let mut result = format!("{} /{} {}\r\n", self.method.to_string(), self.uri, self.version);
        let mut headers = self.headers.clone();
        if !self.body.is_empty() {
            if self.method == HTTPMethod::GET {
                panic!("Attempting to send a body in a GET request");
            }
            headers.insert("Content-Length".to_string(), self.body.len().to_string());
        }
        for (key, value) in headers {
            let header = format!("{key}: {value}\r\n").to_string();
            result.push_str(&header);
        }
        result.push_str("\r\n");
        if !self.body.is_empty() {
            result.push_str(&self.body);
        }
        result.push_str("\r\n");
        return result;
    }

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

    pub fn read(mut reader: BufReader<&TcpStream>) -> Result<HTTPRequest, HTTPResponseCode> {
        let mut raw = Vec::new();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).expect("Could not read line");
            if line == "\r\n" {
                break;
            } else if line.len() == 0 {
                return Err(HTTPResponseCode::BadRequest);
            }
            line.pop();
            line.pop();
            raw.push(line);
        }
        let mut request = HTTPRequest::parse(raw)?;

        // Get the body
        request.body = match request.read_body(reader) {
            Ok(value) => value,
            Err(_) => return Err(HTTPResponseCode::BadRequest),
        };
        return Ok(request);
    }
}

impl HTTPMessage for HTTPRequest {
    fn headers(&self) -> &HashMap<String, String> {
        return &self.headers;
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
    fn from_code(code: u32) -> Option<HTTPResponseCode> {
        return match code {
            200 => Some(HTTPResponseCode::OK),
            204 => Some(HTTPResponseCode::NoContent),
            400 => Some(HTTPResponseCode::BadRequest),
            404 => Some(HTTPResponseCode::NotFound),
            405 => Some(HTTPResponseCode::MethodNotAllowed),
            409 => Some(HTTPResponseCode::Conflict),
            500 => Some(HTTPResponseCode::InternalServerError),
            505 => Some(HTTPResponseCode::HTTPVersionNotSupported),
            _ => None,
        };
    }

    fn from_string(code: String) -> Option<HTTPResponseCode> {
        let ucode = code.parse::<u32>();
        if ucode.is_err() {
            return None;
        }
        return Self::from_code(ucode.unwrap());
    }

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
    pub status: HTTPResponseCode,
    pub headers: HashMap<String, String>,
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

    pub fn parse(raw: Vec<String>) -> Option<Self> {
        if raw.len() == 0 { 
            return None; 
        }
        let startline = raw.first()?;
        let (version, rem) = startline.split_once(' ')?;
        let (code, text) = rem.split_once(' ')?;
        let mut headers = HashMap::new();
        for i in 1..raw.len() {
            let (key, value) = raw.get(i)?.split_once(": ")?;
            let mut value_string = value.to_string();
            headers.insert(key.to_string(), value_string);
        }
        return Some(Self {
            version: version.to_string(),
            status: HTTPResponseCode::from_string(code.to_string())?,
            headers,
            content: String::new()
        });
    }

    pub fn read(mut reader: BufReader<&TcpStream>) -> Option<HTTPResponse> {
        let mut raw = Vec::new();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).expect("Could not read line");
            if line == "\r\n" {
                break;
            } else if line.len() == 0 {
                return None;
            }
            line.pop();
            line.pop();
            raw.push(line);
        }
        let mut response = HTTPResponse::parse(raw)?;

        // Get the body
        response.content = match response.read_body(reader) {
            Ok(value) => value,
            Err(_) => return None,
        };

        return Some(response);
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
        result.push_str("\r\n");
        return result;
    }
}

impl HTTPMessage for HTTPResponse {
    fn headers(&self) -> &HashMap<String, String> {
        return &self.headers;
    }
}
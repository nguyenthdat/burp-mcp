use base64::Engine;
use flate2::Compression;
use flate2::read::{
    DeflateDecoder, DeflateEncoder, GzDecoder, GzEncoder, ZlibDecoder, ZlibEncoder,
};
use hmac::{Hmac, Mac};
use percent_encoding::{NON_ALPHANUMERIC, percent_decode_str, utf8_percent_encode};
use regex::Regex;
use serde_json::Value;
use std::io::{Cursor, Read};
use url::form_urlencoded;
pub use utility_core::{DataValue, MAX_BATCH_ITEMS, OperationInfo};
use utility_core::{MAX_UTILITY_INPUT_BYTES, RecipeStep};

const MAX_REGEX_MATCHES: usize = 1_000;
const MAX_SPLIT_PARTS: usize = 10_000;
const MAX_DECOMPRESSION_RATIO: usize = 1_000;
const MAX_JWT_SEGMENTS: usize = 3;

macro_rules! operation {
    ($id:literal, $name:literal, $description:literal, $input:literal, $output:literal) => {
        op($id, $name, $description, $input, $output, false)
    };
    ($id:literal, $name:literal, $description:literal, $input:literal, $output:literal, weak) => {
        op($id, $name, $description, $input, $output, true)
    };
}

const OPERATIONS: &[OperationInfo] = &[
    operation!(
        "base64.encode",
        "Base64 Encode",
        "Encode bytes as Base64",
        "any",
        "text"
    ),
    operation!(
        "base64.decode",
        "Base64 Decode",
        "Decode Base64 into bytes",
        "text",
        "bytes"
    ),
    operation!(
        "base64url.encode",
        "Base64URL Encode",
        "Encode bytes as unpadded Base64URL",
        "any",
        "text"
    ),
    operation!(
        "base64url.decode",
        "Base64URL Decode",
        "Decode Base64URL into bytes",
        "text",
        "bytes"
    ),
    operation!(
        "hex.encode",
        "Hex Encode",
        "Encode bytes as lowercase hexadecimal",
        "any",
        "text"
    ),
    operation!(
        "hex.decode",
        "Hex Decode",
        "Decode hexadecimal into bytes",
        "text",
        "bytes"
    ),
    operation!(
        "url.encode",
        "URL Encode",
        "Percent encode all non-alphanumeric UTF-8 bytes",
        "text",
        "text"
    ),
    operation!(
        "url.decode",
        "URL Decode",
        "Decode percent encoded UTF-8 text",
        "text",
        "text"
    ),
    operation!(
        "html.encode",
        "HTML Encode",
        "Encode HTML special characters",
        "text",
        "text"
    ),
    operation!(
        "html.decode",
        "HTML Decode",
        "Decode HTML named and numeric entities",
        "text",
        "text"
    ),
    operation!(
        "unicode.escape",
        "Unicode Escape",
        "Escape Unicode scalar values",
        "text",
        "text"
    ),
    operation!(
        "unicode.unescape",
        "Unicode Unescape",
        "Decode Rust/JavaScript-style Unicode escapes",
        "text",
        "text"
    ),
    operation!(
        "json.pretty",
        "JSON Pretty",
        "Pretty print JSON",
        "text_or_json",
        "text"
    ),
    operation!(
        "json.minify",
        "JSON Minify",
        "Minify JSON",
        "text_or_json",
        "text"
    ),
    operation!(
        "json.query",
        "JSON Query",
        "Select JSON with a bounded dotted path or JSON Pointer",
        "text_or_json",
        "json"
    ),
    operation!(
        "text.uppercase",
        "Uppercase",
        "Convert text to uppercase",
        "text",
        "text"
    ),
    operation!(
        "text.lowercase",
        "Lowercase",
        "Convert text to lowercase",
        "text",
        "text"
    ),
    operation!(
        "text.reverse",
        "Reverse",
        "Reverse Unicode scalar values",
        "text",
        "text"
    ),
    operation!(
        "text.split",
        "Split",
        "Split text by a literal delimiter",
        "text",
        "json"
    ),
    operation!(
        "text.join",
        "Join",
        "Join a JSON string array",
        "json",
        "text"
    ),
    operation!(
        "regex.extract",
        "Regex Extract",
        "Extract bounded regex matches",
        "text",
        "json"
    ),
    operation!(
        "regex.replace",
        "Regex Replace",
        "Replace bounded regex matches",
        "text",
        "text"
    ),
    operation!(
        "entropy",
        "Entropy",
        "Calculate Shannon entropy in bits per byte",
        "any",
        "json"
    ),
    operation!(
        "strings.extract",
        "Printable Strings",
        "Extract bounded printable byte strings",
        "any",
        "json"
    ),
    operation!("length", "Length", "Return byte length", "any", "json"),
    operation!("md5", "MD5", "Compute MD5 digest", "any", "text", weak),
    operation!("sha1", "SHA-1", "Compute SHA-1 digest", "any", "text", weak),
    operation!("sha256", "SHA-256", "Compute SHA-256 digest", "any", "text"),
    operation!("sha512", "SHA-512", "Compute SHA-512 digest", "any", "text"),
    operation!("blake3", "BLAKE3", "Compute BLAKE3 digest", "any", "text"),
    operation!(
        "hmac.sha256",
        "HMAC SHA-256",
        "Compute keyed SHA-256 MAC",
        "any",
        "text"
    ),
    operation!(
        "hmac.sha512",
        "HMAC SHA-512",
        "Compute keyed SHA-512 MAC",
        "any",
        "text"
    ),
    operation!(
        "gzip.compress",
        "Gzip Compress",
        "Compress bytes with gzip",
        "any",
        "bytes"
    ),
    operation!(
        "gzip.decompress",
        "Gzip Decompress",
        "Decompress bounded gzip bytes",
        "bytes",
        "bytes"
    ),
    operation!(
        "zlib.compress",
        "Zlib Compress",
        "Compress bytes with zlib framing",
        "any",
        "bytes"
    ),
    operation!(
        "zlib.decompress",
        "Zlib Decompress",
        "Decompress bounded zlib bytes",
        "bytes",
        "bytes"
    ),
    operation!(
        "deflate.compress",
        "Deflate Compress",
        "Compress bytes with raw DEFLATE",
        "any",
        "bytes"
    ),
    operation!(
        "deflate.decompress",
        "Deflate Decompress",
        "Decompress bounded raw DEFLATE bytes",
        "bytes",
        "bytes"
    ),
    operation!(
        "brotli.compress",
        "Brotli Compress",
        "Compress bytes with Brotli",
        "any",
        "bytes"
    ),
    operation!(
        "brotli.decompress",
        "Brotli Decompress",
        "Decompress bounded Brotli bytes",
        "bytes",
        "bytes"
    ),
    operation!(
        "jwt.decode",
        "JWT Decode",
        "Decode JWT header and payload without verification",
        "text",
        "json"
    ),
    operation!(
        "jwt.verify_hs256",
        "JWT Verify HS256",
        "Verify a JWT HS256 signature with a caller key",
        "text",
        "json"
    ),
    operation!(
        "cookie.parse",
        "Cookie Parse",
        "Parse a Cookie header without logging values",
        "text",
        "json"
    ),
    operation!(
        "query.parse",
        "Query Parse",
        "Parse a query string preserving repeated keys",
        "text",
        "json"
    ),
    operation!(
        "query.build",
        "Query Build",
        "Build a query string from an object or pair array",
        "json",
        "text"
    ),
    operation!(
        "http.parse",
        "HTTP Parse",
        "Parse an HTTP request or response",
        "any",
        "json"
    ),
    operation!(
        "http.set_body",
        "HTTP Set Body",
        "Replace the body and update Content-Length",
        "any",
        "bytes"
    ),
    operation!(
        "http.update_content_length",
        "HTTP Content-Length",
        "Recalculate Content-Length for an HTTP message",
        "any",
        "bytes"
    ),
];

const fn op(
    id: &'static str,
    name: &'static str,
    description: &'static str,
    input_kind: &'static str,
    output_kind: &'static str,
    cryptographically_weak: bool,
) -> OperationInfo {
    OperationInfo {
        id,
        name,
        description,
        input_kind,
        output_kind,
        deterministic: true,
        pure: true,
        cryptographically_weak,
    }
}

pub fn search(query: &str) -> Vec<OperationInfo> {
    let query = query.to_ascii_lowercase();
    OPERATIONS
        .iter()
        .copied()
        .filter(|operation| {
            operation.id.contains(&query)
                || operation.name.to_ascii_lowercase().contains(&query)
                || operation.description.to_ascii_lowercase().contains(&query)
        })
        .collect()
}

pub fn describe(id: &str) -> Option<OperationInfo> {
    OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.id == id)
}

pub fn run(id: &str, input: DataValue, args: &Value) -> Result<DataValue, String> {
    input.ensure_bounded("input")?;
    let output = match id {
        "base64.encode" => {
            DataValue::Text(base64::engine::general_purpose::STANDARD.encode(input.as_bytes()?))
        }
        "base64.decode" => {
            decode_base64(input.as_text()?, &base64::engine::general_purpose::STANDARD)?
        }
        "base64url.encode" => DataValue::Text(
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(input.as_bytes()?),
        ),
        "base64url.decode" => decode_base64_url(input.as_text()?)?,
        "hex.encode" => DataValue::Text(hex_encode(input.as_bytes()?)),
        "hex.decode" => DataValue::Bytes(hex_decode(input.as_text()?)?),
        "url.encode" => {
            DataValue::Text(utf8_percent_encode(input.as_text()?, NON_ALPHANUMERIC).to_string())
        }
        "url.decode" => DataValue::Text(
            percent_decode_str(input.as_text()?)
                .decode_utf8()
                .map_err(|error| error.to_string())?
                .into_owned(),
        ),
        "html.encode" => DataValue::Text(html_escape::encode_safe(input.as_text()?).into_owned()),
        "html.decode" => {
            DataValue::Text(html_escape::decode_html_entities(input.as_text()?).into_owned())
        }
        "unicode.escape" => DataValue::Text(unicode_escape(input.as_text()?)),
        "unicode.unescape" => DataValue::Text(unicode_unescape(input.as_text()?)?),
        "json.pretty" => DataValue::Text(
            serde_json::to_string_pretty(&input.parse_json()?)
                .map_err(|error| error.to_string())?,
        ),
        "json.minify" => DataValue::Text(
            serde_json::to_string(&input.parse_json()?).map_err(|error| error.to_string())?,
        ),
        "json.query" => DataValue::Json(json_query(
            &input.parse_json()?,
            required_str(args, "path")?,
        )?),
        "text.uppercase" => DataValue::Text(input.as_text()?.to_uppercase()),
        "text.lowercase" => DataValue::Text(input.as_text()?.to_lowercase()),
        "text.reverse" => DataValue::Text(input.as_text()?.chars().rev().collect()),
        "text.split" => text_split(input.as_text()?, args)?,
        "text.join" => text_join(&input.parse_json()?, args)?,
        "regex.extract" => regex_extract(input.as_text()?, args)?,
        "regex.replace" => regex_replace(input.as_text()?, args)?,
        "entropy" => entropy(input.as_bytes()?),
        "strings.extract" => printable_strings(input.as_bytes()?, args)?,
        "length" => DataValue::Json(serde_json::json!({"bytes": input.as_bytes()?.len()})),
        "md5" => DataValue::Text(digest::<md5::Md5>(input.as_bytes()?)),
        "sha1" => DataValue::Text(digest::<sha1::Sha1>(input.as_bytes()?)),
        "sha256" => DataValue::Text(digest::<sha2::Sha256>(input.as_bytes()?)),
        "sha512" => DataValue::Text(digest::<sha2::Sha512>(input.as_bytes()?)),
        "blake3" => DataValue::Text(blake3::hash(input.as_bytes()?).to_hex().to_string()),
        "hmac.sha256" => hmac_sha256(input.as_bytes()?, key(args)?)?,
        "hmac.sha512" => hmac_sha512(input.as_bytes()?, key(args)?)?,
        "gzip.compress" => {
            compress_reader(GzEncoder::new(input.as_bytes()?, Compression::default()))?
        }
        "gzip.decompress" => {
            decompress_reader(GzDecoder::new(input.as_bytes()?), input.as_bytes()?.len())?
        }
        "zlib.compress" => {
            compress_reader(ZlibEncoder::new(input.as_bytes()?, Compression::default()))?
        }
        "zlib.decompress" => {
            decompress_reader(ZlibDecoder::new(input.as_bytes()?), input.as_bytes()?.len())?
        }
        "deflate.compress" => compress_reader(DeflateEncoder::new(
            input.as_bytes()?,
            Compression::default(),
        ))?,
        "deflate.decompress" => decompress_reader(
            DeflateDecoder::new(input.as_bytes()?),
            input.as_bytes()?.len(),
        )?,
        "brotli.compress" => brotli_compress(input.as_bytes()?)?,
        "brotli.decompress" => brotli_decompress(input.as_bytes()?)?,
        "jwt.decode" => jwt_decode(input.as_text()?)?,
        "jwt.verify_hs256" => jwt_verify_hs256(input.as_text()?, key(args)?)?,
        "cookie.parse" => cookie_parse(input.as_text()?),
        "query.parse" => query_parse(input.as_text()?),
        "query.build" => query_build(&input.parse_json()?)?,
        "http.parse" => http_parse(input.as_bytes()?)?,
        "http.set_body" => http_set_body(input.as_bytes()?, args)?,
        "http.update_content_length" => http_update_content_length(input.as_bytes()?)?,
        _ => return Err(format!("unknown utility operation: {id}")),
    };
    output.ensure_bounded("output")?;
    Ok(output)
}

pub fn run_recipe(value: DataValue, steps: &[(String, Value)]) -> Result<DataValue, String> {
    let steps = steps
        .iter()
        .map(|(operation, args)| RecipeStep {
            operation: operation.clone(),
            args: args.clone(),
        })
        .collect::<Vec<_>>();
    utility_core::run_recipe(value, &steps, run)
}

fn required_str<'a>(args: &'a Value, name: &str) -> Result<&'a str, String> {
    args.get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("operation requires string argument '{name}'"))
}

fn key(args: &Value) -> Result<&[u8], String> {
    required_str(args, "key").map(str::as_bytes)
}

fn decode_base64(input: &str, engine: &impl Engine) -> Result<DataValue, String> {
    engine
        .decode(input)
        .map(DataValue::Bytes)
        .map_err(|error| error.to_string())
}

fn decode_base64_url(input: &str) -> Result<DataValue, String> {
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(input)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(input))
        .map(DataValue::Bytes)
        .map_err(|error| error.to_string())
}

fn hex_encode(input: &[u8]) -> String {
    use std::fmt::Write;
    let mut output = String::with_capacity(input.len().saturating_mul(2));
    for byte in input {
        write!(output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

fn hex_decode(input: &str) -> Result<Vec<u8>, String> {
    if !input.len().is_multiple_of(2) {
        return Err("hex input must contain an even number of digits".to_owned());
    }
    (0..input.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&input[index..index + 2], 16).map_err(|error| error.to_string())
        })
        .collect()
}

fn unicode_escape(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            c if c.is_ascii_graphic() || c == ' ' => output.push(c),
            c => output.extend(c.escape_unicode()),
        }
    }
    output
}

fn unicode_unescape(input: &str) -> Result<String, String> {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(character) = chars.next() {
        if character != '\\' {
            output.push(character);
            continue;
        }
        match chars
            .next()
            .ok_or_else(|| "trailing Unicode escape delimiter".to_owned())?
        {
            '\\' => output.push('\\'),
            'n' => output.push('\n'),
            'r' => output.push('\r'),
            't' => output.push('\t'),
            '"' => output.push('"'),
            '0' => output.push('\0'),
            'u' => {
                let mut digits = String::new();
                if chars.clone().next() == Some('{') {
                    chars.next();
                    for digit in chars.by_ref() {
                        if digit == '}' {
                            break;
                        }
                        digits.push(digit);
                        if digits.len() > 6 {
                            return Err("Unicode escape has too many digits".to_owned());
                        }
                    }
                } else {
                    for _ in 0..4 {
                        digits.push(
                            chars
                                .next()
                                .ok_or_else(|| "incomplete Unicode escape".to_owned())?,
                        );
                    }
                }
                let scalar = u32::from_str_radix(&digits, 16).map_err(|error| error.to_string())?;
                output.push(
                    char::from_u32(scalar)
                        .ok_or_else(|| "invalid Unicode scalar value".to_owned())?,
                );
            }
            escaped => return Err(format!("unsupported escape sequence: \\{escaped}")),
        }
    }
    Ok(output)
}
#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct MagicSuggestion {
    pub operation: &'static str,
    pub confidence: f32,
}

pub fn magic(input: &DataValue) -> Vec<MagicSuggestion> {
    const MAX_MAGIC_CANDIDATES: usize = 5;
    let mut suggestions = Vec::with_capacity(MAX_MAGIC_CANDIDATES);
    match input {
        DataValue::Text(value) => {
            if value.len().is_multiple_of(2)
                && !value.is_empty()
                && value.bytes().all(|byte| byte.is_ascii_hexdigit())
            {
                suggestions.push(MagicSuggestion {
                    operation: "hex.decode",
                    confidence: 0.95,
                });
            }
            if base64::engine::general_purpose::STANDARD
                .decode(value)
                .is_ok()
            {
                suggestions.push(MagicSuggestion {
                    operation: "base64.decode",
                    confidence: 0.9,
                });
            }
            if value.contains('%') && percent_decode_str(value).decode_utf8().is_ok() {
                suggestions.push(MagicSuggestion {
                    operation: "url.decode",
                    confidence: 0.85,
                });
            }
            if value.contains('&') && value.contains(';') {
                suggestions.push(MagicSuggestion {
                    operation: "html.decode",
                    confidence: 0.7,
                });
            }
            if serde_json::from_str::<Value>(value).is_ok() {
                suggestions.push(MagicSuggestion {
                    operation: "json.pretty",
                    confidence: 1.0,
                });
            }
        }
        DataValue::Bytes(value) if value.starts_with(&[0x1f, 0x8b]) => {
            suggestions.push(MagicSuggestion {
                operation: "gzip.decompress",
                confidence: 1.0,
            })
        }
        _ => {}
    }
    suggestions.truncate(MAX_MAGIC_CANDIDATES);
    suggestions
}

fn json_query(value: &Value, path: &str) -> Result<Value, String> {
    if path.is_empty() {
        return Ok(value.clone());
    }
    if path.starts_with('/') {
        return value
            .pointer(path)
            .cloned()
            .ok_or_else(|| "JSON pointer not found".to_owned());
    }
    let mut current = value;
    for segment in path.split('.') {
        current = match current {
            Value::Object(object) => object.get(segment),
            Value::Array(array) => segment
                .parse::<usize>()
                .ok()
                .and_then(|index| array.get(index)),
            _ => None,
        }
        .ok_or_else(|| format!("JSON path not found at '{segment}'"))?;
    }
    Ok(current.clone())
}

fn text_split(input: &str, args: &Value) -> Result<DataValue, String> {
    let delimiter = required_str(args, "delimiter")?;
    if delimiter.is_empty() {
        return Err("split delimiter must not be empty".to_owned());
    }
    let values = input
        .split(delimiter)
        .take(MAX_SPLIT_PARTS + 1)
        .map(Value::from)
        .collect::<Vec<_>>();
    if values.len() > MAX_SPLIT_PARTS {
        return Err(format!("split exceeds {MAX_SPLIT_PARTS} parts"));
    }
    Ok(DataValue::Json(Value::Array(values)))
}

fn text_join(value: &Value, args: &Value) -> Result<DataValue, String> {
    let delimiter = required_str(args, "delimiter")?;
    let values = value
        .as_array()
        .ok_or_else(|| "join requires a JSON array".to_owned())?;
    if values.len() > MAX_SPLIT_PARTS {
        return Err(format!("join exceeds {MAX_SPLIT_PARTS} parts"));
    }
    let strings = values
        .iter()
        .map(|item| {
            item.as_str()
                .ok_or_else(|| "join requires an array of strings".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(DataValue::Text(strings.join(delimiter)))
}

fn regex_from_args(args: &Value) -> Result<Regex, String> {
    Regex::new(required_str(args, "pattern")?).map_err(|error| error.to_string())
}

fn regex_extract(input: &str, args: &Value) -> Result<DataValue, String> {
    let regex = regex_from_args(args)?;
    let limit = args
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(MAX_REGEX_MATCHES as u64)
        .min(MAX_REGEX_MATCHES as u64) as usize;
    let matches = regex
        .find_iter(input)
        .take(limit + 1)
        .map(|item| Value::from(item.as_str()))
        .collect::<Vec<_>>();
    if matches.len() > limit {
        return Err(format!("regex results exceed limit {limit}"));
    }
    Ok(DataValue::Json(Value::Array(matches)))
}

fn regex_replace(input: &str, args: &Value) -> Result<DataValue, String> {
    let regex = regex_from_args(args)?;
    let replacement = required_str(args, "replacement")?;
    let limit = args
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(MAX_REGEX_MATCHES as u64)
        .min(MAX_REGEX_MATCHES as u64) as usize;
    Ok(DataValue::Text(
        regex.replacen(input, limit, replacement).into_owned(),
    ))
}

fn entropy(input: &[u8]) -> DataValue {
    if input.is_empty() {
        return DataValue::Json(serde_json::json!({"bits_per_byte": 0.0}));
    }
    let mut counts = [0usize; 256];
    for byte in input {
        counts[usize::from(*byte)] += 1;
    }
    let length = input.len() as f64;
    let value = counts
        .into_iter()
        .filter(|count| *count > 0)
        .fold(0.0, |sum, count| {
            let probability = count as f64 / length;
            sum - probability * probability.log2()
        });
    DataValue::Json(serde_json::json!({"bits_per_byte": value}))
}

fn printable_strings(input: &[u8], args: &Value) -> Result<DataValue, String> {
    let minimum = args
        .get("minimum")
        .and_then(Value::as_u64)
        .unwrap_or(4)
        .clamp(1, 1024) as usize;
    let limit = args
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(MAX_REGEX_MATCHES as u64)
        .min(MAX_REGEX_MATCHES as u64) as usize;
    let mut strings = Vec::new();
    let mut start = None;
    for (index, byte) in input.iter().copied().chain(std::iter::once(0)).enumerate() {
        if byte.is_ascii_graphic() || byte == b' ' {
            start.get_or_insert(index);
        } else if let Some(begin) = start.take()
            && index - begin >= minimum
        {
            if strings.len() == limit {
                return Err(format!("printable strings exceed limit {limit}"));
            }
            strings.push(Value::from(
                String::from_utf8_lossy(&input[begin..index]).into_owned(),
            ));
        }
    }
    Ok(DataValue::Json(Value::Array(strings)))
}

fn digest<D: sha1::Digest + Default>(input: &[u8]) -> String {
    hex_encode(&D::digest(input))
}

fn hmac_sha256(input: &[u8], key: &[u8]) -> Result<DataValue, String> {
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(key).map_err(|error| error.to_string())?;
    mac.update(input);
    Ok(DataValue::Text(hex_encode(&mac.finalize().into_bytes())))
}

fn hmac_sha512(input: &[u8], key: &[u8]) -> Result<DataValue, String> {
    let mut mac = Hmac::<sha2::Sha512>::new_from_slice(key).map_err(|error| error.to_string())?;
    mac.update(input);
    Ok(DataValue::Text(hex_encode(&mac.finalize().into_bytes())))
}

fn compress_reader(mut reader: impl Read) -> Result<DataValue, String> {
    let mut output = Vec::new();
    reader
        .read_to_end(&mut output)
        .map_err(|error| error.to_string())?;
    Ok(DataValue::Bytes(output))
}

fn decompress_reader(reader: impl Read, compressed_len: usize) -> Result<DataValue, String> {
    let mut output = Vec::new();
    reader
        .take((MAX_UTILITY_INPUT_BYTES + 1) as u64)
        .read_to_end(&mut output)
        .map_err(|error| error.to_string())?;
    validate_decompressed_size(compressed_len, output.len())?;
    Ok(DataValue::Bytes(output))
}

fn validate_decompressed_size(compressed_len: usize, output_len: usize) -> Result<(), String> {
    if output_len > MAX_UTILITY_INPUT_BYTES {
        return Err("decompressed output exceeds byte limit".to_owned());
    }
    let ratio_limit = compressed_len
        .saturating_mul(MAX_DECOMPRESSION_RATIO)
        .max(1024);
    if output_len > ratio_limit {
        return Err(format!(
            "decompression ratio exceeds {MAX_DECOMPRESSION_RATIO}:1"
        ));
    }
    Ok(())
}

fn brotli_compress(input: &[u8]) -> Result<DataValue, String> {
    let mut output = Vec::new();
    brotli::BrotliCompress(
        &mut Cursor::new(input),
        &mut output,
        &brotli::enc::BrotliEncoderParams::default(),
    )
    .map_err(|error| error.to_string())?;
    Ok(DataValue::Bytes(output))
}

fn brotli_decompress(input: &[u8]) -> Result<DataValue, String> {
    let mut output = Vec::new();
    brotli::BrotliDecompress(&mut Cursor::new(input), &mut output)
        .map_err(|error| error.to_string())?;
    validate_decompressed_size(input.len(), output.len())?;
    Ok(DataValue::Bytes(output))
}

fn jwt_segments(token: &str) -> Result<[&str; MAX_JWT_SEGMENTS], String> {
    let segments = token.split('.').collect::<Vec<_>>();
    segments
        .try_into()
        .map_err(|_| "JWT must contain exactly three segments".to_owned())
}

fn jwt_decode(token: &str) -> Result<DataValue, String> {
    let [header, payload, _] = jwt_segments(token)?;
    let decode_json = |segment: &str| -> Result<Value, String> {
        let bytes = match decode_base64_url(segment)? {
            DataValue::Bytes(bytes) => bytes,
            _ => unreachable!(),
        };
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())
    };
    Ok(DataValue::Json(
        serde_json::json!({"header": decode_json(header)?, "payload": decode_json(payload)?}),
    ))
}

fn jwt_verify_hs256(token: &str, key: &[u8]) -> Result<DataValue, String> {
    let [header, payload, signature] = jwt_segments(token)?;
    let decoded = match jwt_decode(token)? {
        DataValue::Json(value) => value,
        _ => unreachable!(),
    };
    if decoded["header"]["alg"] != "HS256" {
        return Err("JWT alg must be HS256".to_owned());
    }
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(key).map_err(|error| error.to_string())?;
    mac.update(format!("{header}.{payload}").as_bytes());
    let signature = match decode_base64_url(signature)? {
        DataValue::Bytes(bytes) => bytes,
        _ => unreachable!(),
    };
    let valid = mac.verify_slice(&signature).is_ok();
    Ok(DataValue::Json(
        serde_json::json!({"valid": valid, "header": decoded["header"], "payload": decoded["payload"]}),
    ))
}

fn cookie_parse(input: &str) -> DataValue {
    let pairs = input
        .split(';')
        .filter_map(|part| {
            let (name, value) = part.trim().split_once('=')?;
            Some(Value::Array(vec![Value::from(name), Value::from(value)]))
        })
        .collect::<Vec<_>>();
    DataValue::Json(Value::Array(pairs))
}

fn query_parse(input: &str) -> DataValue {
    let pairs = form_urlencoded::parse(input.trim_start_matches('?').as_bytes())
        .map(|(name, value)| {
            Value::Array(vec![
                Value::from(name.into_owned()),
                Value::from(value.into_owned()),
            ])
        })
        .collect();
    DataValue::Json(Value::Array(pairs))
}

fn query_build(value: &Value) -> Result<DataValue, String> {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    match value {
        Value::Object(object) => {
            for (name, value) in object {
                if let Some(values) = value.as_array() {
                    for value in values {
                        serializer.append_pair(name, scalar_text(value)?);
                    }
                } else {
                    serializer.append_pair(name, scalar_text(value)?);
                }
            }
        }
        Value::Array(pairs) => {
            for pair in pairs {
                let pair = pair
                    .as_array()
                    .filter(|pair| pair.len() == 2)
                    .ok_or_else(|| "query pair must be a two-element array".to_owned())?;
                serializer.append_pair(scalar_text(&pair[0])?, scalar_text(&pair[1])?);
            }
        }
        _ => return Err("query build requires a JSON object or pair array".to_owned()),
    }
    Ok(DataValue::Text(serializer.finish()))
}

fn scalar_text(value: &Value) -> Result<&str, String> {
    value
        .as_str()
        .ok_or_else(|| "query names and values must be strings".to_owned())
}

struct HttpMessage<'a> {
    start_line: &'a [u8],
    headers: Vec<&'a [u8]>,
    body: &'a [u8],
}

fn parse_http(input: &[u8]) -> Result<HttpMessage<'_>, String> {
    let (head, body) = split_once_bytes(input, b"\r\n\r\n")
        .or_else(|| split_once_bytes(input, b"\n\n"))
        .ok_or_else(|| "HTTP message requires a header/body separator".to_owned())?;
    let mut lines = head
        .split(|byte| *byte == b'\n')
        .map(|line| line.strip_suffix(b"\r").unwrap_or(line));
    let start_line = lines
        .next()
        .filter(|line| !line.is_empty())
        .ok_or_else(|| "HTTP start line is missing".to_owned())?;
    Ok(HttpMessage {
        start_line,
        headers: lines.collect(),
        body,
    })
}

fn split_once_bytes<'a>(input: &'a [u8], separator: &[u8]) -> Option<(&'a [u8], &'a [u8])> {
    input
        .windows(separator.len())
        .position(|window| window == separator)
        .map(|index| (&input[..index], &input[index + separator.len()..]))
}

fn http_parse(input: &[u8]) -> Result<DataValue, String> {
    let message = parse_http(input)?;
    let start_line = String::from_utf8_lossy(message.start_line).into_owned();
    let headers = message.headers.iter().filter_map(|line| split_once_bytes(line, b":")).map(|(name, value)| serde_json::json!({"name": String::from_utf8_lossy(name), "value": String::from_utf8_lossy(value).trim()})).collect::<Vec<_>>();
    let kind = if message.start_line.starts_with(b"HTTP/") {
        "response"
    } else {
        "request"
    };
    Ok(DataValue::Json(
        serde_json::json!({"kind": kind, "start_line": start_line, "headers": headers, "body_base64": base64::engine::general_purpose::STANDARD.encode(message.body), "body_length": message.body.len()}),
    ))
}

fn http_set_body(input: &[u8], args: &Value) -> Result<DataValue, String> {
    let body = if let Some(base64) = args.get("body_base64").and_then(Value::as_str) {
        base64::engine::general_purpose::STANDARD
            .decode(base64)
            .map_err(|error| error.to_string())?
    } else {
        required_str(args, "body")?.as_bytes().to_vec()
    };
    rewrite_http(input, &body)
}

fn http_update_content_length(input: &[u8]) -> Result<DataValue, String> {
    let message = parse_http(input)?;
    rewrite_http(input, message.body)
}

fn rewrite_http(input: &[u8], body: &[u8]) -> Result<DataValue, String> {
    let message = parse_http(input)?;
    let mut output = Vec::with_capacity(input.len().saturating_add(body.len()));
    output.extend_from_slice(message.start_line);
    output.extend_from_slice(b"\r\n");
    let mut replaced = false;
    for header in message.headers {
        let name = split_once_bytes(header, b":")
            .map(|(name, _)| name)
            .unwrap_or(header);
        if name.eq_ignore_ascii_case(b"content-length") {
            if !replaced {
                output.extend_from_slice(format!("Content-Length: {}\r\n", body.len()).as_bytes());
                replaced = true;
            }
        } else {
            output.extend_from_slice(header);
            output.extend_from_slice(b"\r\n");
        }
    }
    if !replaced {
        output.extend_from_slice(format!("Content-Length: {}\r\n", body.len()).as_bytes());
    }
    output.extend_from_slice(b"\r\n");
    output.extend_from_slice(body);
    Ok(DataValue::Bytes(output))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_text(id: &str, input: &str, args: Value) -> DataValue {
        run(id, DataValue::Text(input.to_owned()), &args).unwrap()
    }

    #[test]
    fn recipe_preserves_binary_without_utf8_loss() {
        let value = run_recipe(
            DataValue::Text("AP8=".to_owned()),
            &[
                ("base64.decode".to_owned(), Value::Null),
                ("hex.encode".to_owned(), Value::Null),
            ],
        )
        .unwrap();
        assert_eq!(value, DataValue::Text("00ff".to_owned()));
    }

    #[test]
    fn expanding_operation_rejects_oversized_output() {
        let error = run(
            "base64.encode",
            DataValue::Bytes(vec![0; MAX_UTILITY_INPUT_BYTES]),
            &Value::Null,
        )
        .unwrap_err();
        assert_eq!(
            format!("output exceeds {MAX_UTILITY_INPUT_BYTES} bytes"),
            error
        );
    }

    #[test]
    fn operations_are_unique_pure_and_described() {
        let mut ids = OPERATIONS
            .iter()
            .map(|operation| operation.id)
            .collect::<Vec<_>>();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), OPERATIONS.len());
        assert!(OPERATIONS.iter().all(|operation| operation.pure
            && operation.deterministic
            && !operation.description.is_empty()));
    }

    #[test]
    fn implements_encoding_text_and_web_operations() {
        assert_eq!(
            run_text("html.decode", "&lt;a&gt;", Value::Null),
            DataValue::Text("<a>".to_owned())
        );
        assert_eq!(
            run_text("unicode.unescape", "\\u{1f680}", Value::Null),
            DataValue::Text("🚀".to_owned())
        );
        assert_eq!(
            run_text(
                "json.query",
                r#"{"user":{"id":7}}"#,
                serde_json::json!({"path":"user.id"})
            ),
            DataValue::Json(Value::from(7))
        );
        assert_eq!(
            run_text("query.parse", "a=1&a=2", Value::Null),
            DataValue::Json(serde_json::json!([["a", "1"], ["a", "2"]]))
        );
        assert_eq!(
            run_text("cookie.parse", "a=1; b=two", Value::Null),
            DataValue::Json(serde_json::json!([["a", "1"], ["b", "two"]]))
        );
    }

    #[test]
    fn compression_round_trips_every_supported_format() {
        for (compress, decompress) in [
            ("gzip.compress", "gzip.decompress"),
            ("zlib.compress", "zlib.decompress"),
            ("deflate.compress", "deflate.decompress"),
            ("brotli.compress", "brotli.decompress"),
        ] {
            let compressed = run(
                compress,
                DataValue::Bytes(b"bounded decoder".to_vec()),
                &Value::Null,
            )
            .unwrap();
            let decompressed = run(decompress, compressed, &Value::Null).unwrap();
            assert_eq!(
                decompressed,
                DataValue::Bytes(b"bounded decoder".to_vec()),
                "{compress}"
            );
        }
    }

    #[test]
    fn http_body_update_preserves_binary_and_framing() {
        let message =
            b"POST / HTTP/1.1\r\nHost: example.test\r\nContent-Length: 3\r\n\r\nold".to_vec();
        let output = run(
            "http.set_body",
            DataValue::Bytes(message),
            &serde_json::json!({"body_base64":"AP8="}),
        )
        .unwrap();
        let DataValue::Bytes(output) = output else {
            panic!("expected bytes")
        };
        assert!(
            output
                .windows(b"Content-Length: 2".len())
                .any(|window| window == b"Content-Length: 2")
        );
        assert!(output.ends_with(&[0, 255]));
    }
    #[test]
    fn decompression_ratio_is_bounded() {
        let compressed = run(
            "gzip.compress",
            DataValue::Bytes(vec![b'A'; 2 * 1024 * 1024]),
            &Value::Null,
        )
        .unwrap();
        let error = run("gzip.decompress", compressed, &Value::Null).unwrap_err();
        assert_eq!(error, "decompression ratio exceeds 1000:1");
    }

    #[test]
    fn regex_and_split_result_counts_are_bounded() {
        let regex_error = run(
            "regex.extract",
            DataValue::Text("aaaa".to_owned()),
            &serde_json::json!({"pattern": "a", "limit": 2}),
        )
        .unwrap_err();
        assert_eq!(regex_error, "regex results exceed limit 2");

        let oversized = "x,".repeat(MAX_SPLIT_PARTS);
        let error = run(
            "text.split",
            DataValue::Text(oversized),
            &serde_json::json!({"delimiter": ","}),
        )
        .unwrap_err();
        assert_eq!(error, format!("split exceeds {MAX_SPLIT_PARTS} parts"));
    }

    #[test]
    fn catalog_has_no_network_filesystem_or_code_execution_operations() {
        let prohibited = [
            "http.fetch",
            "file.",
            "filesystem",
            "javascript",
            "shell",
            "process.spawn",
        ];
        assert!(
            OPERATIONS
                .iter()
                .all(|operation| prohibited.iter().all(|term| !operation.id.contains(term)))
        );
    }

    #[test]
    fn magic_is_deterministic_confident_and_bounded() {
        let input = DataValue::Text("7b226f6b223a747275657d".to_owned());
        let first = magic(&input);
        let second = magic(&input);
        assert_eq!(
            serde_json::to_value(&first).unwrap(),
            serde_json::to_value(&second).unwrap()
        );
        assert!(second.len() <= 5);
        assert!(
            second
                .iter()
                .all(|suggestion| (0.0..=1.0).contains(&suggestion.confidence))
        );
    }

    #[derive(serde::Deserialize)]
    struct Fixture {
        operation: String,
        input: FixtureValue,
        output: FixtureValue,
    }

    #[derive(serde::Deserialize, PartialEq, Debug)]
    #[serde(tag = "kind", rename_all = "snake_case")]
    enum FixtureValue {
        Text { value: String },
        Bytes { base64: String },
    }

    #[test]
    fn matches_cyberchef_differential_fixtures() {
        let document: Value = serde_json::from_str(include_str!(
            "../../../test-fixtures/utility-cyberchef-v2.json"
        ))
        .unwrap();
        let fixtures: Vec<Fixture> = serde_json::from_value(document["cases"].clone()).unwrap();
        for fixture in fixtures {
            let input = match fixture.input {
                FixtureValue::Text { value } => DataValue::Text(value),
                FixtureValue::Bytes { base64 } => DataValue::Bytes(
                    base64::engine::general_purpose::STANDARD
                        .decode(base64)
                        .unwrap(),
                ),
            };
            let actual = match run(&fixture.operation, input, &Value::Null).unwrap() {
                DataValue::Text(value) => FixtureValue::Text { value },
                DataValue::Bytes(value) => FixtureValue::Bytes {
                    base64: base64::engine::general_purpose::STANDARD.encode(value),
                },
                DataValue::Json(value) => FixtureValue::Text {
                    value: value.to_string(),
                },
            };
            assert_eq!(actual, fixture.output, "{}", fixture.operation);
        }
    }
}

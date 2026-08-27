use crate::error::{UtilityError, UtilityResult};
use crate::registry::{Operation, OperationInfo, ValueKind, run_from_registry};
use crate::value::{DataValue, MAX_UTILITY_INPUT_BYTES};
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

const MAX_REGEX_MATCHES: usize = 1_000;
const MAX_SPLIT_PARTS: usize = 10_000;
const MAX_DECOMPRESSION_RATIO: usize = 1_000;
const MAX_JWT_SEGMENTS: usize = 3;

macro_rules! operation {
    ($id:literal, $name:literal, $description:literal, $input:ident, $output:ident) => {
        op(
            $id,
            $name,
            $description,
            ValueKind::$input,
            ValueKind::$output,
            false,
        )
    };
    ($id:literal, $name:literal, $description:literal, $input:ident, $output:ident, weak) => {
        op(
            $id,
            $name,
            $description,
            ValueKind::$input,
            ValueKind::$output,
            true,
        )
    };
}

const OPERATIONS: &[Operation] = &[
    operation!(
        "base64.encode",
        "Base64 Encode",
        "Encode bytes as Base64",
        Any,
        Text
    ),
    operation!(
        "base64.decode",
        "Base64 Decode",
        "Decode Base64 into bytes",
        Text,
        Bytes
    ),
    operation!(
        "base64url.encode",
        "Base64URL Encode",
        "Encode bytes as unpadded Base64URL",
        Any,
        Text
    ),
    operation!(
        "base64url.decode",
        "Base64URL Decode",
        "Decode Base64URL into bytes",
        Text,
        Bytes
    ),
    operation!(
        "base64.mime.encode",
        "MIME Base64 Encode",
        "Encode bytes as MIME Base64 with CRLF line wrapping",
        Any,
        Text
    ),
    operation!(
        "base64.mime.decode",
        "MIME Base64 Decode",
        "Decode MIME Base64 while ignoring ASCII whitespace",
        Text,
        Bytes
    ),
    operation!(
        "hex.encode",
        "Hex Encode",
        "Encode bytes as lowercase hexadecimal",
        Any,
        Text
    ),
    operation!(
        "hex.decode",
        "Hex Decode",
        "Decode hexadecimal into bytes",
        Text,
        Bytes
    ),
    operation!(
        "url.encode",
        "URL Encode",
        "Percent encode all non-alphanumeric UTF-8 bytes",
        Text,
        Text
    ),
    operation!(
        "url.decode",
        "URL Decode",
        "Decode percent encoded UTF-8 text",
        Text,
        Text
    ),
    operation!(
        "url.encode.key",
        "URL Encode Key",
        "Encode a URL key while preserving Burp-style safe characters",
        Text,
        Text
    ),
    operation!(
        "url.encode.value",
        "URL Encode Value",
        "Encode a URL value while preserving Burp-style safe characters",
        Text,
        Text
    ),
    operation!(
        "url.encode.all",
        "URL Encode All",
        "Percent encode every UTF-8 byte",
        Text,
        Text
    ),
    operation!(
        "url.encode.java",
        "URL Encode Java",
        "Form URL encode text using plus for spaces",
        Text,
        Text
    ),
    operation!(
        "html.encode",
        "HTML Encode",
        "Encode HTML special characters",
        Text,
        Text
    ),
    operation!(
        "html.decode",
        "HTML Decode",
        "Decode HTML named and numeric entities",
        Text,
        Text
    ),
    operation!(
        "html.encode.decimal",
        "HTML Decimal Encode",
        "Encode each Unicode scalar as a decimal HTML entity",
        Text,
        Text
    ),
    operation!(
        "html.encode.hex",
        "HTML Hex Encode",
        "Encode each Unicode scalar as a hexadecimal HTML entity",
        Text,
        Text
    ),
    operation!(
        "unicode.escape",
        "Unicode Escape",
        "Escape Unicode scalar values",
        Text,
        Text
    ),
    operation!(
        "unicode.unescape",
        "Unicode Unescape",
        "Decode Rust/JavaScript-style Unicode escapes",
        Text,
        Text
    ),
    operation!(
        "json.pretty",
        "JSON Pretty",
        "Pretty print JSON",
        TextOrJson,
        Text
    ),
    operation!(
        "json.minify",
        "JSON Minify",
        "Minify JSON",
        TextOrJson,
        Text
    ),
    operation!(
        "json.query",
        "JSON Query",
        "Select JSON with a bounded dotted path or JSON Pointer",
        TextOrJson,
        Json
    ),
    operation!(
        "text.uppercase",
        "Uppercase",
        "Convert text to uppercase",
        Text,
        Text
    ),
    operation!(
        "text.lowercase",
        "Lowercase",
        "Convert text to lowercase",
        Text,
        Text
    ),
    operation!(
        "text.reverse",
        "Reverse",
        "Reverse Unicode scalar values",
        Text,
        Text
    ),
    operation!(
        "text.split",
        "Split",
        "Split text by a literal delimiter",
        Text,
        Json
    ),
    operation!("text.join", "Join", "Join a JSON string array", Json, Text),
    operation!(
        "regex.extract",
        "Regex Extract",
        "Extract bounded regex matches",
        Text,
        Json
    ),
    operation!(
        "regex.replace",
        "Regex Replace",
        "Replace bounded regex matches",
        Text,
        Text
    ),
    operation!(
        "entropy",
        "Entropy",
        "Calculate Shannon entropy in bits per byte",
        Any,
        Json
    ),
    operation!(
        "strings.extract",
        "Printable Strings",
        "Extract bounded printable byte strings",
        Any,
        Json
    ),
    operation!("length", "Length", "Return byte length", Any, Json),
    operation!(
        "bytes.index_of",
        "Byte Index Of",
        "Find the first bounded byte or text search term",
        Any,
        Json
    ),
    operation!(
        "bytes.count",
        "Byte Match Count",
        "Count non-overlapping byte or text search terms",
        Any,
        Json
    ),
    operation!(
        "bytes.to_latin1",
        "Bytes To Latin-1",
        "Map each byte losslessly to the same Unicode code point",
        Any,
        Text
    ),
    operation!(
        "bytes.from_latin1",
        "Latin-1 To Bytes",
        "Map each Unicode code point low byte to bytes",
        Text,
        Bytes
    ),
    operation!(
        "number.convert",
        "Number Base Convert",
        "Convert an arbitrary-width unsigned integer between bases 2, 8, 10, and 16",
        Text,
        Text
    ),
    operation!("md5", "MD5", "Compute MD5 digest", Any, Text, weak),
    operation!("sha1", "SHA-1", "Compute SHA-1 digest", Any, Text, weak),
    operation!("sha256", "SHA-256", "Compute SHA-256 digest", Any, Text),
    operation!("sha512", "SHA-512", "Compute SHA-512 digest", Any, Text),
    operation!("blake3", "BLAKE3", "Compute BLAKE3 digest", Any, Text),
    operation!("sha224", "SHA-224", "Compute SHA-224 digest", Any, Text),
    operation!("sha384", "SHA-384", "Compute SHA-384 digest", Any, Text),
    operation!(
        "sha512_224",
        "SHA-512/224",
        "Compute SHA-512/224 digest",
        Any,
        Text
    ),
    operation!(
        "sha512_256",
        "SHA-512/256",
        "Compute SHA-512/256 digest",
        Any,
        Text
    ),
    operation!("sha3_224", "SHA3-224", "Compute SHA3-224 digest", Any, Text),
    operation!("sha3_256", "SHA3-256", "Compute SHA3-256 digest", Any, Text),
    operation!("sha3_384", "SHA3-384", "Compute SHA3-384 digest", Any, Text),
    operation!("sha3_512", "SHA3-512", "Compute SHA3-512 digest", Any, Text),
    operation!(
        "hmac.sha256",
        "HMAC SHA-256",
        "Compute keyed SHA-256 MAC",
        Any,
        Text
    ),
    operation!(
        "hmac.sha512",
        "HMAC SHA-512",
        "Compute keyed SHA-512 MAC",
        Any,
        Text
    ),
    operation!(
        "gzip.compress",
        "Gzip Compress",
        "Compress bytes with gzip",
        Any,
        Bytes
    ),
    operation!(
        "gzip.decompress",
        "Gzip Decompress",
        "Decompress bounded gzip bytes",
        Bytes,
        Bytes
    ),
    operation!(
        "zlib.compress",
        "Zlib Compress",
        "Compress bytes with zlib framing",
        Any,
        Bytes
    ),
    operation!(
        "zlib.decompress",
        "Zlib Decompress",
        "Decompress bounded zlib bytes",
        Bytes,
        Bytes
    ),
    operation!(
        "deflate.compress",
        "Deflate Compress",
        "Compress bytes with raw DEFLATE",
        Any,
        Bytes
    ),
    operation!(
        "deflate.decompress",
        "Deflate Decompress",
        "Decompress bounded raw DEFLATE bytes",
        Bytes,
        Bytes
    ),
    operation!(
        "brotli.compress",
        "Brotli Compress",
        "Compress bytes with Brotli",
        Any,
        Bytes
    ),
    operation!(
        "brotli.decompress",
        "Brotli Decompress",
        "Decompress bounded Brotli bytes",
        Bytes,
        Bytes
    ),
    operation!(
        "jwt.decode",
        "JWT Decode",
        "Decode JWT header and payload without verification",
        Text,
        Json
    ),
    operation!(
        "jwt.verify_hs256",
        "JWT Verify HS256",
        "Verify a JWT HS256 signature with a caller key",
        Text,
        Json
    ),
    operation!(
        "cookie.parse",
        "Cookie Parse",
        "Parse a Cookie header without logging values",
        Text,
        Json
    ),
    operation!(
        "query.parse",
        "Query Parse",
        "Parse a query string preserving repeated keys",
        Text,
        Json
    ),
    operation!(
        "query.build",
        "Query Build",
        "Build a query string from an object or pair array",
        Json,
        Text
    ),
    operation!(
        "http.parse",
        "HTTP Parse",
        "Parse an HTTP request or response",
        Any,
        Json
    ),
    operation!(
        "http.set_body",
        "HTTP Set Body",
        "Replace the body and update Content-Length",
        Any,
        Bytes
    ),
    operation!(
        "http.update_content_length",
        "HTTP Content-Length",
        "Recalculate Content-Length for an HTTP message",
        Any,
        Bytes
    ),
];

const fn op(
    id: &'static str,
    name: &'static str,
    description: &'static str,
    input_kind: ValueKind,
    output_kind: ValueKind,
    cryptographically_weak: bool,
) -> Operation {
    Operation::new(
        OperationInfo {
            id,
            name,
            description,
            input_kind,
            output_kind,
            deterministic: true,
            pure: true,
            cryptographically_weak,
        },
        execute_operation,
    )
}

pub fn search(query: &str) -> Vec<OperationInfo> {
    let query = query.to_ascii_lowercase();
    OPERATIONS
        .iter()
        .map(|operation| operation.info)
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
        .find(|operation| operation.info.id == id)
        .map(|operation| operation.info)
}

pub fn run(id: &str, input: DataValue, args: &Value) -> UtilityResult<DataValue> {
    run_from_registry(OPERATIONS, id, input, args)
}

fn execute_operation(id: &str, input: DataValue, args: &Value) -> UtilityResult<DataValue> {
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
        "base64.mime.encode" => DataValue::Text(mime_base64_encode(input.as_bytes()?)),
        "base64.mime.decode" => DataValue::Bytes(
            base64::engine::general_purpose::STANDARD
                .decode(
                    input
                        .as_text()?
                        .bytes()
                        .filter(|byte| !byte.is_ascii_whitespace())
                        .collect::<Vec<_>>(),
                )
                .map_err(|error| UtilityError::message(error.to_string()))?,
        ),
        "hex.encode" => DataValue::Text(hex_encode(input.as_bytes()?)),
        "hex.decode" => DataValue::Bytes(hex_decode(input.as_text()?)?),
        "url.encode" => {
            DataValue::Text(utf8_percent_encode(input.as_text()?, NON_ALPHANUMERIC).to_string())
        }
        "url.encode.key" => DataValue::Text(url_encode_selected(input.as_text()?, false)),
        "url.encode.value" => DataValue::Text(url_encode_selected(input.as_text()?, true)),
        "url.encode.all" => DataValue::Text(percent_encode_all(input.as_text()?.as_bytes())),
        "url.encode.java" => {
            DataValue::Text(form_urlencoded::byte_serialize(input.as_text()?.as_bytes()).collect())
        }
        "url.decode" => DataValue::Text(
            percent_decode_str(input.as_text()?)
                .decode_utf8()
                .map_err(|error| UtilityError::message(error.to_string()))?
                .into_owned(),
        ),
        "html.encode" => DataValue::Text(html_escape::encode_safe(input.as_text()?).into_owned()),
        "html.decode" => {
            DataValue::Text(html_escape::decode_html_entities(input.as_text()?).into_owned())
        }
        "html.encode.decimal" => DataValue::Text(
            input
                .as_text()?
                .chars()
                .map(|value| format!("&#{};", value as u32))
                .collect(),
        ),
        "html.encode.hex" => DataValue::Text(
            input
                .as_text()?
                .chars()
                .map(|value| format!("&#x{:x};", value as u32))
                .collect(),
        ),
        "unicode.escape" => DataValue::Text(unicode_escape(input.as_text()?)),
        "unicode.unescape" => DataValue::Text(unicode_unescape(input.as_text()?)?),
        "json.pretty" => DataValue::Text(
            serde_json::to_string_pretty(&input.parse_json()?)
                .map_err(|error| UtilityError::message(error.to_string()))?,
        ),
        "json.minify" => DataValue::Text(
            serde_json::to_string(&input.parse_json()?)
                .map_err(|error| UtilityError::message(error.to_string()))?,
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
        "bytes.index_of" => byte_index_of(input.as_bytes()?, args)?,
        "bytes.count" => byte_count(input.as_bytes()?, args)?,
        "bytes.to_latin1" => DataValue::Text(
            input
                .as_bytes()?
                .iter()
                .map(|byte| char::from(*byte))
                .collect(),
        ),
        "bytes.from_latin1" => DataValue::Bytes(
            input
                .as_text()?
                .chars()
                .map(|value| (value as u32 & 0xff) as u8)
                .collect(),
        ),
        "number.convert" => number_convert(input.as_text()?, args)?,
        "md5" => DataValue::Text(digest::<md5::Md5>(input.as_bytes()?)),
        "sha1" => DataValue::Text(digest::<sha1::Sha1>(input.as_bytes()?)),
        "sha224" => DataValue::Text(digest::<sha2::Sha224>(input.as_bytes()?)),
        "sha256" => DataValue::Text(digest::<sha2::Sha256>(input.as_bytes()?)),
        "sha384" => DataValue::Text(digest::<sha2::Sha384>(input.as_bytes()?)),
        "sha512" => DataValue::Text(digest::<sha2::Sha512>(input.as_bytes()?)),
        "sha512_224" => DataValue::Text(digest::<sha2::Sha512_224>(input.as_bytes()?)),
        "sha512_256" => DataValue::Text(digest::<sha2::Sha512_256>(input.as_bytes()?)),
        "sha3_224" => DataValue::Text(digest::<sha3::Sha3_224>(input.as_bytes()?)),
        "sha3_256" => DataValue::Text(digest::<sha3::Sha3_256>(input.as_bytes()?)),
        "sha3_384" => DataValue::Text(digest::<sha3::Sha3_384>(input.as_bytes()?)),
        "sha3_512" => DataValue::Text(digest::<sha3::Sha3_512>(input.as_bytes()?)),
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
        _ => return Err(UtilityError::message(format!("unknown operation: {id}"))),
    };
    Ok(output)
}

fn mime_base64_encode(input: &[u8]) -> String {
    let encoded = base64::engine::general_purpose::STANDARD.encode(input);
    encoded
        .as_bytes()
        .chunks(76)
        .map(String::from_utf8_lossy)
        .collect::<Vec<_>>()
        .join("\r\n")
}

fn percent_encode_all(input: &[u8]) -> String {
    input.iter().map(|byte| format!("%{byte:02X}")).collect()
}

fn url_encode_selected(value: &str, preserve_slash: bool) -> String {
    value
        .bytes()
        .map(|byte| {
            let safe = byte.is_ascii_alphanumeric()
                || b"-_.~".contains(&byte)
                || (preserve_slash && byte == b'/');
            if safe {
                (byte as char).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect()
}

fn byte_search_term(args: &Value) -> UtilityResult<Vec<u8>> {
    if let Some(value) = args.get("term").and_then(Value::as_str) {
        return Ok(value.as_bytes().to_vec());
    }
    args.get("term_base64")
        .and_then(Value::as_str)
        .ok_or_else(|| UtilityError::message("operation requires 'term' or 'term_base64'"))
        .and_then(|value| {
            base64::engine::general_purpose::STANDARD
                .decode(value)
                .map_err(|error| UtilityError::message(error.to_string()))
        })
}

fn byte_index_of(data: &[u8], args: &Value) -> UtilityResult<DataValue> {
    let term = byte_search_term(args)?;
    require_nonempty_bytes(&term)?;
    let index = data
        .windows(term.len())
        .position(|window| window == term)
        .map(|value| value as i64)
        .unwrap_or(-1);
    Ok(DataValue::Json(serde_json::json!({"index": index})))
}

fn byte_count(data: &[u8], args: &Value) -> UtilityResult<DataValue> {
    let term = byte_search_term(args)?;
    require_nonempty_bytes(&term)?;
    Ok(DataValue::Json(
        serde_json::json!({"count": data.windows(term.len()).filter(|window| *window == term.as_slice()).count()}),
    ))
}

fn require_nonempty_bytes(value: &[u8]) -> UtilityResult<()> {
    if value.is_empty() {
        Err(UtilityError::message("search term must not be empty"))
    } else {
        Ok(())
    }
}

fn number_convert(value: &str, args: &Value) -> UtilityResult<DataValue> {
    let from = args
        .get("from")
        .and_then(Value::as_u64)
        .ok_or_else(|| UtilityError::message("number.convert requires 'from'"))?;
    let to = args
        .get("to")
        .and_then(Value::as_u64)
        .ok_or_else(|| UtilityError::message("number.convert requires 'to'"))?;
    if !matches!(from, 2 | 8 | 10 | 16) || !matches!(to, 2 | 8 | 10 | 16) {
        return Err(UtilityError::message(
            "number bases must be one of 2, 8, 10, or 16",
        ));
    }
    let number = num_bigint::BigUint::parse_bytes(value.trim().as_bytes(), from as u32)
        .ok_or_else(|| UtilityError::message("invalid unsigned number"))?;
    Ok(DataValue::Text(number.to_str_radix(to as u32)))
}

fn required_str<'a>(args: &'a Value, name: &str) -> UtilityResult<&'a str> {
    args.get(name).and_then(Value::as_str).ok_or_else(|| {
        UtilityError::message(format!("operation requires string argument '{name}'"))
    })
}

fn key(args: &Value) -> UtilityResult<&[u8]> {
    required_str(args, "key").map(str::as_bytes)
}

fn decode_base64(input: &str, engine: &impl Engine) -> UtilityResult<DataValue> {
    engine
        .decode(input)
        .map(DataValue::Bytes)
        .map_err(|error| UtilityError::message(error.to_string()))
}

fn decode_base64_url(input: &str) -> UtilityResult<DataValue> {
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(input)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(input))
        .map(DataValue::Bytes)
        .map_err(|error| UtilityError::message(error.to_string()))
}

fn hex_encode(input: &[u8]) -> String {
    hex::encode(input)
}

fn hex_decode(input: &str) -> UtilityResult<Vec<u8>> {
    hex::decode(input)
        .map_err(|error| UtilityError::with_source("invalid hexadecimal input", error))
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

fn unicode_unescape(input: &str) -> UtilityResult<String> {
    unescaper::unescape(input)
        .map_err(|error| UtilityError::with_source("invalid Unicode escape", error))
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

fn json_query(value: &Value, path: &str) -> UtilityResult<Value> {
    if path.is_empty() {
        return Ok(value.clone());
    }
    if path.starts_with('/') {
        return value
            .pointer(path)
            .cloned()
            .ok_or_else(|| UtilityError::message("JSON pointer not found"));
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

fn text_split(input: &str, args: &Value) -> UtilityResult<DataValue> {
    let delimiter = required_str(args, "delimiter")?;
    if delimiter.is_empty() {
        return Err(UtilityError::message("split delimiter must not be empty"));
    }
    let values = input
        .split(delimiter)
        .take(MAX_SPLIT_PARTS + 1)
        .map(Value::from)
        .collect::<Vec<_>>();
    if values.len() > MAX_SPLIT_PARTS {
        return Err(UtilityError::message(format!(
            "split exceeds {MAX_SPLIT_PARTS} parts"
        )));
    }
    Ok(DataValue::Json(Value::Array(values)))
}

fn text_join(value: &Value, args: &Value) -> UtilityResult<DataValue> {
    let delimiter = required_str(args, "delimiter")?;
    let values = value
        .as_array()
        .ok_or_else(|| UtilityError::message("join requires a JSON array"))?;
    if values.len() > MAX_SPLIT_PARTS {
        return Err(UtilityError::message(format!(
            "join exceeds {MAX_SPLIT_PARTS} parts"
        )));
    }
    let strings = values
        .iter()
        .map(|item| {
            item.as_str()
                .ok_or_else(|| UtilityError::message("join requires an array of strings"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(DataValue::Text(strings.join(delimiter)))
}

fn regex_from_args(args: &Value) -> UtilityResult<Regex> {
    Regex::new(required_str(args, "pattern")?)
        .map_err(|error| UtilityError::message(error.to_string()))
}

fn regex_extract(input: &str, args: &Value) -> UtilityResult<DataValue> {
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
        return Err(UtilityError::message(format!(
            "regex results exceed limit {limit}"
        )));
    }
    Ok(DataValue::Json(Value::Array(matches)))
}

fn regex_replace(input: &str, args: &Value) -> UtilityResult<DataValue> {
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

fn printable_strings(input: &[u8], args: &Value) -> UtilityResult<DataValue> {
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
                return Err(UtilityError::message(format!(
                    "printable strings exceed limit {limit}"
                )));
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

fn hmac_sha256(input: &[u8], key: &[u8]) -> UtilityResult<DataValue> {
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(key)
        .map_err(|error| UtilityError::message(error.to_string()))?;
    mac.update(input);
    Ok(DataValue::Text(hex_encode(&mac.finalize().into_bytes())))
}

fn hmac_sha512(input: &[u8], key: &[u8]) -> UtilityResult<DataValue> {
    let mut mac = Hmac::<sha2::Sha512>::new_from_slice(key)
        .map_err(|error| UtilityError::message(error.to_string()))?;
    mac.update(input);
    Ok(DataValue::Text(hex_encode(&mac.finalize().into_bytes())))
}

fn compress_reader(mut reader: impl Read) -> UtilityResult<DataValue> {
    let mut output = Vec::new();
    reader
        .read_to_end(&mut output)
        .map_err(|error| UtilityError::message(error.to_string()))?;
    Ok(DataValue::Bytes(output))
}

fn decompress_reader(reader: impl Read, compressed_len: usize) -> UtilityResult<DataValue> {
    let mut output = Vec::new();
    reader
        .take((MAX_UTILITY_INPUT_BYTES + 1) as u64)
        .read_to_end(&mut output)
        .map_err(|error| UtilityError::message(error.to_string()))?;
    validate_decompressed_size(compressed_len, output.len())?;
    Ok(DataValue::Bytes(output))
}

fn validate_decompressed_size(compressed_len: usize, output_len: usize) -> UtilityResult<()> {
    if output_len > MAX_UTILITY_INPUT_BYTES {
        return Err(UtilityError::message(
            "decompressed output exceeds byte limit",
        ));
    }
    let ratio_limit = compressed_len
        .saturating_mul(MAX_DECOMPRESSION_RATIO)
        .max(1024);
    if output_len > ratio_limit {
        return Err(UtilityError::message(format!(
            "decompression ratio exceeds {MAX_DECOMPRESSION_RATIO}:1"
        )));
    }
    Ok(())
}

fn brotli_compress(input: &[u8]) -> UtilityResult<DataValue> {
    let mut output = Vec::new();
    brotli::BrotliCompress(
        &mut Cursor::new(input),
        &mut output,
        &brotli::enc::BrotliEncoderParams::default(),
    )
    .map_err(|error| UtilityError::message(error.to_string()))?;
    Ok(DataValue::Bytes(output))
}

fn brotli_decompress(input: &[u8]) -> UtilityResult<DataValue> {
    decompress_reader(
        brotli::Decompressor::new(Cursor::new(input), 4096),
        input.len(),
    )
}

fn jwt_segments(token: &str) -> UtilityResult<[&str; MAX_JWT_SEGMENTS]> {
    let segments = token.split('.').collect::<Vec<_>>();
    segments
        .try_into()
        .map_err(|_| UtilityError::message("JWT must contain exactly three segments"))
}

fn jwt_decode(token: &str) -> UtilityResult<DataValue> {
    let [header, payload, _] = jwt_segments(token)?;
    let decode_json = |segment: &str| -> UtilityResult<Value> {
        let bytes = match decode_base64_url(segment)? {
            DataValue::Bytes(bytes) => bytes,
            _ => unreachable!(),
        };
        serde_json::from_slice(&bytes).map_err(|error| UtilityError::message(error.to_string()))
    };
    Ok(DataValue::Json(
        serde_json::json!({"header": decode_json(header)?, "payload": decode_json(payload)?}),
    ))
}

fn jwt_verify_hs256(token: &str, key: &[u8]) -> UtilityResult<DataValue> {
    let [header, payload, signature] = jwt_segments(token)?;
    let decoded = match jwt_decode(token)? {
        DataValue::Json(value) => value,
        _ => unreachable!(),
    };
    if decoded["header"]["alg"] != "HS256" {
        return Err(UtilityError::message("JWT alg must be HS256"));
    }
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(key)
        .map_err(|error| UtilityError::message(error.to_string()))?;
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

fn query_build(value: &Value) -> UtilityResult<DataValue> {
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
                    .ok_or_else(|| {
                        UtilityError::message("query pair must be a two-element array")
                    })?;
                serializer.append_pair(scalar_text(&pair[0])?, scalar_text(&pair[1])?);
            }
        }
        _ => {
            return Err(UtilityError::message(
                "query build requires a JSON object or pair array",
            ));
        }
    }
    Ok(DataValue::Text(serializer.finish()))
}

fn scalar_text(value: &Value) -> UtilityResult<&str> {
    value
        .as_str()
        .ok_or_else(|| UtilityError::message("query names and values must be strings"))
}

struct HttpMessage<'a> {
    start_line: &'a [u8],
    headers: Vec<&'a [u8]>,
    body: &'a [u8],
}

fn parse_http(input: &[u8]) -> UtilityResult<HttpMessage<'_>> {
    let (head, body) = split_once_bytes(input, b"\r\n\r\n")
        .or_else(|| split_once_bytes(input, b"\n\n"))
        .ok_or_else(|| UtilityError::message("HTTP message requires a header/body separator"))?;
    let mut lines = head
        .split(|byte| *byte == b'\n')
        .map(|line| line.strip_suffix(b"\r").unwrap_or(line));
    let start_line = lines
        .next()
        .filter(|line| !line.is_empty())
        .ok_or_else(|| UtilityError::message("HTTP start line is missing"))?;
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

fn http_parse(input: &[u8]) -> UtilityResult<DataValue> {
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

fn http_set_body(input: &[u8], args: &Value) -> UtilityResult<DataValue> {
    let body = if let Some(base64) = args.get("body_base64").and_then(Value::as_str) {
        base64::engine::general_purpose::STANDARD
            .decode(base64)
            .map_err(|error| UtilityError::message(error.to_string()))?
    } else {
        required_str(args, "body")?.as_bytes().to_vec()
    };
    rewrite_http(input, &body)
}

fn http_update_content_length(input: &[u8]) -> UtilityResult<DataValue> {
    let message = parse_http(input)?;
    rewrite_http(input, message.body)
}

fn rewrite_http(input: &[u8], body: &[u8]) -> UtilityResult<DataValue> {
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

    fn text_value(id: &str, input: &str, args: Value) -> String {
        match run_text(id, input, args) {
            DataValue::Text(value) => value,
            other => panic!("expected text, got {other:?}"),
        }
    }

    fn json_value(id: &str, input: &str, args: Value) -> Value {
        match run_text(id, input, args) {
            DataValue::Json(value) => value,
            other => panic!("expected JSON, got {other:?}"),
        }
    }

    #[test]
    fn recipe_preserves_binary_without_utf8_loss() {
        let steps = [
            crate::RecipeStep {
                operation: "base64.decode".to_owned(),
                args: Value::Null,
            },
            crate::RecipeStep {
                operation: "hex.encode".to_owned(),
                args: Value::Null,
            },
        ];
        let value = crate::run_recipe(DataValue::Text("AP8=".to_owned()), &steps, run).unwrap();
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
            error.to_string()
        );
    }
    #[test]
    fn montoya_style_utility_operations_cover_mime_bytes_and_number_bases() {
        assert_eq!(
            text_value("base64.mime.encode", "hello", Value::Null),
            "aGVsbG8="
        );
        assert_eq!(
            text_value("url.encode.all", "A B", Value::Null),
            "%41%20%42"
        );
        assert_eq!(text_value("html.encode.hex", "A", Value::Null), "&#x41;");
        assert_eq!(
            text_value(
                "number.convert",
                "ff",
                serde_json::json!({"from": 16, "to": 10})
            ),
            "255"
        );
        assert_eq!(
            json_value(
                "bytes.index_of",
                "abcabc",
                serde_json::json!({"term": "bc"})
            )["index"],
            1
        );
        assert!(
            run(
                "bytes.index_of",
                DataValue::Text("abc".to_owned()),
                &serde_json::json!({"term": ""}),
            )
            .is_err()
        );
        assert!(
            run(
                "hex.decode",
                DataValue::Text("aéb".to_owned()),
                &Value::Null,
            )
            .is_err()
        );
        assert_eq!(
            json_value("bytes.count", "abcabc", serde_json::json!({"term": "bc"}))["count"],
            2
        );
    }

    #[test]
    fn operations_are_unique_pure_and_described() {
        let mut ids = OPERATIONS
            .iter()
            .map(|operation| operation.info.id)
            .collect::<Vec<_>>();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), OPERATIONS.len());
        assert!(OPERATIONS.iter().all(|operation| operation.info.pure
            && operation.info.deterministic
            && !operation.info.description.is_empty()));
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
        assert!(
            run(
                "unicode.unescape",
                DataValue::Text("\\u{123".to_owned()),
                &Value::Null,
            )
            .is_err()
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
        for (compress, decompress) in [
            ("gzip.compress", "gzip.decompress"),
            ("brotli.compress", "brotli.decompress"),
        ] {
            let compressed = run(
                compress,
                DataValue::Bytes(vec![b'A'; 2 * 1024 * 1024]),
                &Value::Null,
            )
            .unwrap();
            let error = run(decompress, compressed, &Value::Null).unwrap_err();
            assert_eq!(error.to_string(), "decompression ratio exceeds 1000:1");
        }
    }

    #[test]
    fn regex_and_split_result_counts_are_bounded() {
        let regex_error = run(
            "regex.extract",
            DataValue::Text("aaaa".to_owned()),
            &serde_json::json!({"pattern": "a", "limit": 2}),
        )
        .unwrap_err();
        assert_eq!(regex_error.to_string(), "regex results exceed limit 2");

        let oversized = "x,".repeat(MAX_SPLIT_PARTS);
        let error = run(
            "text.split",
            DataValue::Text(oversized),
            &serde_json::json!({"delimiter": ","}),
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            format!("split exceeds {MAX_SPLIT_PARTS} parts")
        );
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
        assert!(OPERATIONS.iter().all(|operation| {
            prohibited
                .iter()
                .all(|term| !operation.info.id.contains(term))
        }));
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
}

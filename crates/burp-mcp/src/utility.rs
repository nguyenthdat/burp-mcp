use base64::Engine;
use flate2::Compression;
use flate2::read::{GzDecoder, GzEncoder};
use hmac::{Hmac, Mac};
use percent_encoding::{NON_ALPHANUMERIC, percent_decode_str, utf8_percent_encode};
use serde::Serialize;
use serde_json::Value;
use std::io::Read;

pub const MAX_UTILITY_INPUT_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_BATCH_ITEMS: usize = 100;
pub const MAX_RECIPE_STEPS: usize = 64;

#[derive(Clone, Debug, PartialEq)]
pub enum DataValue {
    Text(String),
    Bytes(Vec<u8>),
    Json(Value),
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct OperationInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub input_kind: &'static str,
    pub output_kind: &'static str,
    pub deterministic: bool,
    pub pure: bool,
    pub cryptographically_weak: bool,
}

const OPERATIONS: &[OperationInfo] = &[
    op(
        "base64.encode",
        "Base64 Encode",
        "Encode bytes as Base64",
        "any",
        "text",
        false,
    ),
    op(
        "base64.decode",
        "Base64 Decode",
        "Decode Base64 into bytes",
        "text",
        "bytes",
        false,
    ),
    op(
        "base64url.encode",
        "Base64URL Encode",
        "Encode bytes as unpadded Base64URL",
        "any",
        "text",
        false,
    ),
    op(
        "base64url.decode",
        "Base64URL Decode",
        "Decode Base64URL into bytes",
        "text",
        "bytes",
        false,
    ),
    op(
        "hex.encode",
        "Hex Encode",
        "Encode bytes as lowercase hexadecimal",
        "any",
        "text",
        false,
    ),
    op(
        "hex.decode",
        "Hex Decode",
        "Decode hexadecimal into bytes",
        "text",
        "bytes",
        false,
    ),
    op(
        "url.encode",
        "URL Encode",
        "Percent encode UTF-8 text",
        "text",
        "text",
        false,
    ),
    op(
        "url.decode",
        "URL Decode",
        "Decode percent encoded UTF-8 text",
        "text",
        "text",
        false,
    ),
    op(
        "json.pretty",
        "JSON Pretty",
        "Pretty print JSON",
        "text",
        "text",
        false,
    ),
    op(
        "json.minify",
        "JSON Minify",
        "Minify JSON",
        "text",
        "text",
        false,
    ),
    op(
        "text.uppercase",
        "Uppercase",
        "Convert text to uppercase",
        "text",
        "text",
        false,
    ),
    op(
        "text.lowercase",
        "Lowercase",
        "Convert text to lowercase",
        "text",
        "text",
        false,
    ),
    op(
        "text.reverse",
        "Reverse",
        "Reverse Unicode scalar values",
        "text",
        "text",
        false,
    ),
    op(
        "length",
        "Length",
        "Return byte length",
        "any",
        "json",
        false,
    ),
    op("md5", "MD5", "Compute MD5 digest", "any", "text", true),
    op("sha1", "SHA-1", "Compute SHA-1 digest", "any", "text", true),
    op(
        "sha256",
        "SHA-256",
        "Compute SHA-256 digest",
        "any",
        "text",
        false,
    ),
    op(
        "sha512",
        "SHA-512",
        "Compute SHA-512 digest",
        "any",
        "text",
        false,
    ),
    op(
        "blake3",
        "BLAKE3",
        "Compute BLAKE3 digest",
        "any",
        "text",
        false,
    ),
    op(
        "hmac.sha256",
        "HMAC SHA-256",
        "Compute keyed SHA-256 MAC",
        "any",
        "text",
        false,
    ),
    op(
        "hmac.sha512",
        "HMAC SHA-512",
        "Compute keyed SHA-512 MAC",
        "any",
        "text",
        false,
    ),
    op(
        "gzip.compress",
        "Gzip Compress",
        "Compress bytes with gzip",
        "any",
        "bytes",
        false,
    ),
    op(
        "gzip.decompress",
        "Gzip Decompress",
        "Decompress gzip bytes",
        "bytes",
        "bytes",
        false,
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
    ensure_size(&input, "input")?;
    let output = match id {
        "base64.encode" => Ok(DataValue::Text(
            base64::engine::general_purpose::STANDARD.encode(bytes(&input)?),
        )),
        "base64.decode" => decode_base64(text(&input)?, &base64::engine::general_purpose::STANDARD),
        "base64url.encode" => Ok(DataValue::Text(
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes(&input)?),
        )),
        "base64url.decode" => decode_base64(
            text(&input)?,
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        ),
        "hex.encode" => Ok(DataValue::Text(hex_encode(bytes(&input)?))),
        "hex.decode" => Ok(DataValue::Bytes(hex_decode(text(&input)?)?)),
        "url.encode" => Ok(DataValue::Text(
            utf8_percent_encode(text(&input)?, NON_ALPHANUMERIC).to_string(),
        )),
        "url.decode" => Ok(DataValue::Text(
            percent_decode_str(text(&input)?)
                .decode_utf8()
                .map_err(|error| error.to_string())?
                .into_owned(),
        )),
        "json.pretty" => Ok(DataValue::Text(
            serde_json::to_string_pretty(&parse_json(&input)?)
                .map_err(|error| error.to_string())?,
        )),
        "json.minify" => Ok(DataValue::Text(
            serde_json::to_string(&parse_json(&input)?).map_err(|error| error.to_string())?,
        )),
        "text.uppercase" => Ok(DataValue::Text(text(&input)?.to_uppercase())),
        "text.lowercase" => Ok(DataValue::Text(text(&input)?.to_lowercase())),
        "text.reverse" => Ok(DataValue::Text(text(&input)?.chars().rev().collect())),
        "length" => Ok(DataValue::Json(
            serde_json::json!({"bytes": bytes(&input)?.len()}),
        )),
        "md5" => Ok(DataValue::Text(digest::<md5::Md5>(bytes(&input)?))),
        "sha1" => Ok(DataValue::Text(digest::<sha1::Sha1>(bytes(&input)?))),
        "sha256" => Ok(DataValue::Text(digest::<sha2::Sha256>(bytes(&input)?))),
        "sha512" => Ok(DataValue::Text(digest::<sha2::Sha512>(bytes(&input)?))),
        "blake3" => Ok(DataValue::Text(
            blake3::hash(bytes(&input)?).to_hex().to_string(),
        )),
        "hmac.sha256" => hmac_sha256(bytes(&input)?, key(args)?),
        "hmac.sha512" => hmac_sha512(bytes(&input)?, key(args)?),
        "gzip.compress" => gzip_compress(bytes(&input)?),
        "gzip.decompress" => gzip_decompress(bytes(&input)?),
        _ => Err(format!("unknown utility operation: {id}")),
    }?;
    ensure_size(&output, "output")?;
    Ok(output)
}

pub fn run_recipe(mut value: DataValue, steps: &[(String, Value)]) -> Result<DataValue, String> {
    if steps.len() > MAX_RECIPE_STEPS {
        return Err(format!(
            "recipe must contain at most {MAX_RECIPE_STEPS} steps"
        ));
    }
    for (id, args) in steps {
        value = run(id, value, args)?;
    }
    Ok(value)
}

fn ensure_size(value: &DataValue, label: &str) -> Result<(), String> {
    let size = match value {
        DataValue::Text(value) => value.len(),
        DataValue::Bytes(value) => value.len(),
        DataValue::Json(value) => serde_json::to_vec(value)
            .map_err(|error| error.to_string())?
            .len(),
    };
    if size > MAX_UTILITY_INPUT_BYTES {
        Err(format!("{label} exceeds {MAX_UTILITY_INPUT_BYTES} bytes"))
    } else {
        Ok(())
    }
}
fn bytes(value: &DataValue) -> Result<&[u8], String> {
    match value {
        DataValue::Text(value) => Ok(value.as_bytes()),
        DataValue::Bytes(value) => Ok(value),
        DataValue::Json(_) => Err("operation does not accept JSON input".to_owned()),
    }
}
fn text(value: &DataValue) -> Result<&str, String> {
    match value {
        DataValue::Text(value) => Ok(value),
        _ => Err("operation requires text input".to_owned()),
    }
}
fn parse_json(value: &DataValue) -> Result<Value, String> {
    match value {
        DataValue::Json(value) => Ok(value.clone()),
        DataValue::Text(value) => serde_json::from_str(value).map_err(|error| error.to_string()),
        DataValue::Bytes(_) => Err("JSON operation requires text or JSON input".to_owned()),
    }
}
fn decode_base64(input: &str, engine: &impl Engine) -> Result<DataValue, String> {
    engine
        .decode(input)
        .map(DataValue::Bytes)
        .map_err(|error| error.to_string())
}
fn hex_encode(input: &[u8]) -> String {
    use std::fmt::Write;
    let mut output = String::with_capacity(input.len() * 2);
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
fn digest<D: sha1::Digest + Default>(input: &[u8]) -> String {
    hex_encode(&D::digest(input))
}
fn key(args: &Value) -> Result<&[u8], String> {
    args.get("key")
        .and_then(Value::as_str)
        .map(str::as_bytes)
        .ok_or_else(|| "HMAC requires string argument 'key'".to_owned())
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
fn gzip_compress(input: &[u8]) -> Result<DataValue, String> {
    let mut encoder = GzEncoder::new(input, Compression::default());
    let mut output = Vec::new();
    encoder
        .read_to_end(&mut output)
        .map_err(|error| error.to_string())?;
    Ok(DataValue::Bytes(output))
}
fn gzip_decompress(input: &[u8]) -> Result<DataValue, String> {
    let decoder = GzDecoder::new(input);
    let mut output = Vec::new();
    decoder
        .take((MAX_UTILITY_INPUT_BYTES + 1) as u64)
        .read_to_end(&mut output)
        .map_err(|error| error.to_string())?;
    if output.len() > MAX_UTILITY_INPUT_BYTES {
        return Err("decompressed output exceeds limit".to_owned());
    }
    Ok(DataValue::Bytes(output))
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let input = DataValue::Bytes(vec![0; MAX_UTILITY_INPUT_BYTES]);
        let error = run("base64.encode", input, &Value::Null).unwrap_err();
        assert_eq!(
            format!("output exceeds {MAX_UTILITY_INPUT_BYTES} bytes"),
            error
        );
    }

    #[test]
    fn operations_are_unique_and_pure() {
        let mut ids = OPERATIONS
            .iter()
            .map(|operation| operation.id)
            .collect::<Vec<_>>();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), OPERATIONS.len());
        assert!(
            OPERATIONS
                .iter()
                .all(|operation| operation.pure && operation.deterministic)
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

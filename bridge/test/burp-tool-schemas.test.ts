import { describe, expect, test } from "bun:test"
import { getBurpToolInputSchema } from "../src/burp-tool-schemas"
import { isJsonObject, type JsonObject, type JsonValue } from "../src/types"

function propertiesOf(tool: string): JsonObject {
  const properties = getBurpToolInputSchema(tool)["properties"]
  if (!isJsonObject(properties)) {
    throw new Error(`${tool} schema has no properties object`)
  }
  return properties
}

function requiredOf(tool: string): readonly JsonValue[] {
  const required = getBurpToolInputSchema(tool)["required"]
  return Array.isArray(required) ? required : []
}

function schemaOf(tool: string): JsonObject {
  return getBurpToolInputSchema(tool)
}

describe("Burp tool input schemas", () => {
  test("describes scan URL and mode", () => {
    // Given / When
    const properties = propertiesOf("scan")

    // Then
    expect(properties).toHaveProperty("url")
    expect(properties).toHaveProperty("mode")
    expect(requiredOf("scan")).toContain("url")
  })

  test("describes parallel request objects", () => {
    // Given / When
    const requests = propertiesOf("send_request_parallel")["requests"]

    // Then
    expect(requests).toEqual({
      type: "array",
      items: {
        type: "object",
        properties: {
          method: { type: "string" },
          url: { type: "string" },
          body: { type: "string" },
        },
        required: ["method", "url"],
      },
    })
    expect(requiredOf("send_request_parallel")).toContain("requests")
  })

  test("describes transport options read by active tools", () => {
    // Given
    const expectedProperties = {
      race_condition: ["port", "https"],
      inline_fuzzer: ["port", "https", "marker"],
      access_control_sweep: ["port", "https"],
      websocket_create: ["https", "path"],
    } as const

    // When / Then
    for (const [tool, fields] of Object.entries(expectedProperties)) {
      const properties = propertiesOf(tool)
      for (const field of fields) {
        expect(properties).toHaveProperty(field)
      }
    }
  })

  test("marks backend-required tool fields as required", () => {
    // Given
    const expectedRequired = {
      send_request: ["url"],
      race_condition: ["request", "host"],
      inline_fuzzer: ["template", "host", "wordlist"],
      access_control_sweep: ["request", "host", "auth_headers"],
      websocket_create: ["host"],
      injection_probe: ["url", "param"],
      proxy_detail: ["index"],
      send_to_intruder: ["request"],
      encode: ["input"],
      decode: ["input"],
      convert_request: ["request"],
      generate_csrf_poc: ["request"],
      payload_process: ["input", "operation"],
      add_to_scope: ["url"],
      remove_from_scope: ["url"],
      compare: ["index1", "index2"],
      import_config: ["config"],
      set_upstream_proxy: ["proxy_host", "proxy_port"],
      set_dns_override: ["hostname", "ip"],
      token_analysis: ["tokens"],
      sequencer: ["tokens"],
      add_issue: ["name", "url"],
      register_proxy_rule: ["url_contains"],
      log: ["message"],
      websocket_send_text: ["id", "text"],
      websocket_send_binary: ["id", "data"],
      websocket_close: ["id"],
      session_create_rule: ["find", "replace"],
      jwt_decode: ["token"],
    } as const

    // When / Then
    for (const [tool, fields] of Object.entries(expectedRequired)) {
      expect(requiredOf(tool)).toEqual(fields)
    }
  })

  test("describes optional fields read by backend handlers", () => {
    // Given
    const expectedProperties = {
      intruder_battering_ram: ["body_template"],
      add_issue: ["remediation"],
      register_proxy_rule: ["intercept"],
    } as const

    // When / Then
    for (const [tool, fields] of Object.entries(expectedProperties)) {
      const properties = propertiesOf(tool)
      for (const field of fields) {
        expect(properties).toHaveProperty(field)
      }
    }
  })

  test("requires one complete HTTP handler operation", () => {
    // Given / When / Then
    expect(schemaOf("register_http_handler")["anyOf"]).toEqual([
      { required: ["header_name", "header_value"] },
      { required: ["match", "replace"] },
    ])
  })

  test("describes bounded response extraction", () => {
    // Given / When
    const properties = propertiesOf("extract_from_response")

    // Then
    expect(properties["limit"]).toEqual({
      type: "number",
      description: "Maximum matches returned (default 100, max 500)",
    })
  })
})

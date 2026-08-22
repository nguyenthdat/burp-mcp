set shell := ["bash", "-euo", "pipefail", "-c"]

lms_model := "qwen3.8-27b-abliterated"
lms_context := "262144"

goose_recipe := ".goose/recipe/burp-mcp-validation.recipe.yaml"
goose_report := "target/goose-burp-mcp-report.json"
goose_target := env("BURP_MCP_TARGET_URL", "http://127.0.0.1:3000")
goose_grpc_port := env("BURP_MCP_GRPC_PORT", "9877")
goose_proxy_port := env("BURP_PROXY_PORT", "8080")
juice_shop_image := env("JUICE_SHOP_IMAGE", "bkimminich/juice-shop:v20.1.1")
juice_shop_name := env("JUICE_SHOP_CONTAINER", "burp-mcp-juice-shop")
juice_shop_port := env("JUICE_SHOP_PORT", "3000")

# Start LM Studio server and load the local Qwen model.
lms: lms-server lms-load

# Start LM Studio's OpenAI-compatible server on port 1234.
lms-server:
    lms server start

# Load Qwen with full GPU offload and one inference slot.
# Requires the LM Studio llama.cpp runtime to be installed and selected.
lms-load:
    @if ! lms runtime ls | grep -q '^llama.cpp-'; then \
        echo 'LM Studio llama.cpp runtime is not installed or not visible to lms.' >&2; \
        echo 'Install/select llama.cpp in LM Studio: Settings > Runtime.' >&2; \
        echo 'Then retry: just lms' >&2; \
        exit 1; \
    fi
    lms load "{{ lms_model }}" \
      --identifier "{{ lms_model }}" \
      --gpu max \
      --context-length "{{ lms_context }}" \
      --parallel 1 \
      -y

# Show server, runtime, and loaded-model state.
lms-status:
    lms status
    lms runtime ls
    lms ps

# Stop the LM Studio server.
lms-stop:
    lms server stop

# Start the intentionally vulnerable Juice Shop fixture on loopback only.
juice-shop-start:
    @command -v docker >/dev/null || { echo 'docker is required' >&2; exit 1; }
    @docker info >/dev/null 2>&1 || { echo 'Docker daemon is not running' >&2; exit 1; }
    @if docker container inspect "{{ juice_shop_name }}" >/dev/null 2>&1; then \
        docker start "{{ juice_shop_name }}" >/dev/null; \
      else \
        docker run --detach \
          --name "{{ juice_shop_name }}" \
          --publish "127.0.0.1:{{ juice_shop_port }}:3000" \
          --pull missing \
          --restart no \
          "{{ juice_shop_image }}" >/dev/null; \
      fi
    @for attempt in $$(seq 1 60); do \
        curl --fail --silent --show-error --max-time 2 "http://127.0.0.1:{{ juice_shop_port }}/" >/dev/null && { \
          echo 'Juice Shop ready at http://127.0.0.1:{{ juice_shop_port }}/#/'; \
          exit 0; \
        }; \
        sleep 1; \
      done; \
      echo 'Juice Shop did not become ready within 60 seconds' >&2; \
      docker logs --tail 50 "{{ juice_shop_name }}" >&2; \
      exit 1

# Stop the local Juice Shop fixture without deleting its container.
juice-shop-stop:
    @docker container inspect "{{ juice_shop_name }}" >/dev/null 2>&1 || exit 0
    @docker stop "{{ juice_shop_name }}" >/dev/null

# Delete the local Juice Shop fixture container.
juice-shop-clean:
    @docker container inspect "{{ juice_shop_name }}" >/dev/null 2>&1 || exit 0
    @docker rm --force "{{ juice_shop_name }}" >/dev/null

# Start Juice Shop, then run the bounded Burp/Goose validation against it.
goose-test-juice-shop: juice-shop-start
    @just goose-test "http://127.0.0.1:{{ juice_shop_port }}/#/"

# Run the bounded Goose validation. Prints a shareable result only when code may need fixing.
goose-test target_url=goose_target grpc_port=goose_grpc_port proxy_port=goose_proxy_port report_path=goose_report:
    @python3 -c 'import sys, urllib.parse; u=urllib.parse.urlsplit(sys.argv[1]); assert u.scheme in ("http", "https") and u.hostname, "target_url must be an absolute HTTP(S) URL"' "{{ target_url }}"
    @command -v goose >/dev/null || { echo 'goose is required' >&2; exit 1; }
    @command -v jq >/dev/null || { echo 'jq is required' >&2; exit 1; }
    @command -v bunx >/dev/null || { echo 'bunx is required for Playwright MCP' >&2; exit 1; }
    @bunx --bun @playwright/mcp@0.0.79 --version >/dev/null
    @test -x target/debug/burp-mcp || { echo 'Run: cargo build -p burp-mcp' >&2; exit 1; }
    @nc -z 127.0.0.1 "{{ grpc_port }}" || { echo 'Burp MCP gRPC is not listening on 127.0.0.1:{{ grpc_port }}' >&2; exit 1; }
    @nc -z 127.0.0.1 1234 || { echo 'LM Studio is not listening on 127.0.0.1:1234; run: just lms' >&2; exit 1; }
    @nc -z 127.0.0.1 "{{ proxy_port }}" || { echo 'Burp Proxy is not listening on 127.0.0.1:{{ proxy_port }}' >&2; exit 1; }
    @goose recipe validate "{{ goose_recipe }}"
    @mkdir -p "$$(dirname "{{ report_path }}")"
    @printf '' > "{{ report_path }}"
    @target_origin=$$(python3 -c 'import sys, urllib.parse; u=urllib.parse.urlsplit(sys.argv[1]); print(u.scheme + chr(58) + chr(47) * 2 + u.netloc)' "{{ target_url }}"); \
      target_hostname=$$(python3 -c 'import sys, urllib.parse; print(urllib.parse.urlsplit(sys.argv[1]).hostname)' "{{ target_url }}"); \
      goose run \
        --recipe "{{ goose_recipe }}" \
        --params "target_url={{ target_url }}" \
        --params "target_origin=$$target_origin" \
        --params "target_hostname=$$target_hostname" \
        --params "grpc_port={{ grpc_port }}" \
        --params "proxy_port={{ proxy_port }}" \
        --params "report_path={{ report_path }}" \
        --params "fix_mode=report-only" \
        --params "active_checks=false" \
        --no-session \
        --quiet \
        --output-format json >/dev/null
    @jq -e 'type == "object" and (.summary.failed | type == "number") and (.confirmed_defects | type == "array")' "{{ report_path }}" >/dev/null || { echo 'Goose did not write a valid validation report' >&2; exit 1; }
    @failed="$$(jq -r '.summary.failed' "{{ report_path }}")"; \
      defects="$$(jq -r '.confirmed_defects | length' "{{ report_path }}")"; \
      if (( failed > 0 || defects > 0 )); then \
        echo 'NEEDS_FIX — send the JSON below for remediation:'; \
        jq '{overall_status, summary, confirmed_defects, blockers, report_path}' "{{ report_path }}"; \
        exit 1; \
      fi; \
      jq -r '"\(.overall_status) — failed=\(.summary.failed), blocked=\(.summary.blocked), skipped=\(.summary.skipped). No code-fix report to send."' "{{ report_path }}"

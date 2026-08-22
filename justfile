set shell := ["bash", "-euo", "pipefail", "-c"]

lms_model := "qwen3.8-27b-abliterated"
lms_context := "262144"

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

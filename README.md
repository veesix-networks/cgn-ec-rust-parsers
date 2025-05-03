# CGN Event Consumer Rust Parser

A fast Rust implementation for regex pattern matching in CGN event parsing.

## Installation

```bash
cd cgn-ec-rust-parsers
pip install maturin
maturin develop
```

## Usage Example

```python
from cgn_ec_rust_parsers import RegexMatcher

# Create a new matcher
matcher = RegexMatcher()

# Add patterns with their event types
matcher.add_pattern(r"([AD]) VRF (\d+) (\d+) INT (\d+\.\d+\.\d+\.\d+):(\d+) EXT (\d+\.\d+\.\d+\.\d+):(\d+) DST (\d+\.\d+\.\d+\.\d+):(\d+) DIR (OUT|IN)$", "session_mapping")
matcher.add_pattern(r"([AD]) VRF (\d+) (\d+) INT (\d+\.\d+\.\d+\.\d+):(\d+) EXT (\d+\.\d+\.\d+\.\d+):(\d+)$", "port_mapping")

# Match a message
message = "A VRF 0 6 INT 10.0.0.1:57938 EXT 100.64.0.1:28475 DST 185.165.123.206:443 DIR OUT"
result = matcher.match_message(message)

if result:
    event_type, captures = result
    print(f"Event type: {event_type}")
    print(f"Captures: {captures}")
else:
    print("No match found")
```
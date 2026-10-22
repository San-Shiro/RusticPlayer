use engine::api::{API_VERSION, Command, Event, PROTOCOL_VERSION};
use schemars::schema_for;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command_schema = schema_for!(Command);
    let event_schema = schema_for!(Event);

    let command_json = serde_json::to_string_pretty(&command_schema)?;
    let event_json = serde_json::to_string_pretty(&event_schema)?;

    let content = format!(
        r#"# Rustic Player — Frozen Engine API Contract

**API Version:** `{}`  
**Protocol Version:** `{}`  
**Milestone Gate:** S0 (Frozen)

This document contains the frozen Command & Event contract schemas that govern all communication between Track A (Core Engine) and Track B (UI).

---

## 1. Command Schema

```json
{}
```

---

## 2. Event Schema

```json
{}
```
"#,
        API_VERSION, PROTOCOL_VERSION, command_json, event_json
    );

    let output_path = PathBuf::from("plans/engine-api-contract.md");
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&output_path, content)?;
    println!("Exported engine API contract to {}", output_path.display());
    Ok(())
}

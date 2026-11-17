# Verdigris

A security scanner for LLM applications, written in pure Rust.

Verdigris sends adversarial prompts (prompt injection, jailbreaks, system-prompt extraction) to a language-model application, decides whether each attack leaked a hidden secret, and reports leak rates and severity, much like a vulnerability scanner does for networks.

Course project for Advanced Software Paradigms (CSCI 6221), George Washington University.

**Team:** Apurva Tiwari, Abdulla M Alneyadi Alneyadi

## How it works

A hidden canary secret (PINEAPPLE-42) is placed in the target's system prompt. Verdigris sends 6 attack types, each rewritten by 6 mutations (plain, story, poem, json, reversed, base64), repeats each pair several times, and checks every reply for the canary in plain, reversed or base64 form. A reply that mentions the canary only while refusing is classified as Refused and does not count as a leak.

## Status

Done:
- [x] Local Ollama target and a mock target, behind a `Target` trait
- [x] Attack library: 6 attacks across 6 categories
- [x] Mutation engine: plain, story, poem, json, reversed, base64
- [x] `Detector` trait with a canary detector (plain, reversed, base64) and refusal awareness
- [x] Repeated trials, leak rate per attack and mutation, severity (LOW / MEDIUM / HIGH)
- [x] JSON report and self-contained HTML report with heatmap and reproduction evidence
- [x] Unit tests (`cargo test`)

Planned: command-line options, OpenAI-compatible target, judge-model detector, parallel requests.

## Requirements

- Rust (https://rustup.rs)
- Ollama (https://ollama.com) with a model pulled: `ollama pull llama3.2:3b`

## Run

Scan a local model (takes several minutes):

    cargo run

Scan a built-in fake model, no Ollama needed (instant):

    VERDIGRIS_MOCK=1 cargo run

Run the tests:

    cargo test

Each scan writes `verdigris-report.json` and `verdigris-report.html`. An example is in `sample-report/`.

## Code layout

| File | Role |
|---|---|
| `src/target.rs` | `Target` trait, Ollama target, mock target |
| `src/attacks.rs` | attack payloads |
| `src/mutations.rs` | prompt rewrites and the base64 encoder |
| `src/detector.rs` | `Detector` trait, canary detector, refusal check |
| `src/report.rs` | severity, JSON and HTML reports, HTML escaping |
| `src/main.rs` | scan loop |

## Limitations

- Small samples (4 trials per cell) on one small model, so results are a signal, not proof.
- The refusal check is a heuristic: it looks at how a reply opens.
- Requests run one at a time.

## Ethics

Verdigris is intended only for models and applications you own or are authorized to test.

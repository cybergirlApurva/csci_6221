# Verdigris

A security scanner for LLM applications, written in pure Rust.

Verdigris sends adversarial prompts (prompt injection, jailbreaks, system-prompt extraction) to a language-model application, decides whether each attack worked, and reports leak rates, much like a vulnerability scanner does for networks.

Course project for Advanced Software Paradigms (CSCI 6221), George Washington University.

**Team:** Apurva Tiwari, Abdulla M Alneyadi Alneyadi

## Status

Done:
- [x] Request/response with a local Ollama model
- [x] Attack library: 5 attacks across 5 categories
- [x] Mutation engine: plain, story, poem, json, reversed, base64
- [x] Canary-based detector (plain, reversed and base64 forms) behind a `Detector` trait
- [x] Repeated trials with leak rates per attack and mutation

Planned: JSON and HTML reports, OpenAI-compatible adapter, judge-model detector, parallel requests.

## Requirements

- Rust (https://rustup.rs)
- Ollama (https://ollama.com) with a model pulled: `ollama pull llama3.2:3b`

## Run

Start Ollama, then:

    cargo run

A full scan takes several minutes.

## Ethics

Verdigris is intended only for models and applications you own or are authorized to test.

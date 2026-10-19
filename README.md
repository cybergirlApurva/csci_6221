# Verdigris

A security scanner for LLM applications, written in pure Rust.

Verdigris sends adversarial prompts (prompt injection, jailbreaks, system-prompt extraction) to a language-model application, decides whether each attack worked, and produces a scored report, much like a vulnerability scanner does for networks.

Course project for Advanced Software Paradigms (CSCI 6221), George Washington University.

**Team:** Apurva Tiwari, Abdulla M Alneyadi Alneyadi

## Status

Work in progress. Done so far:

- [x] Project setup and first request to a local Ollama model

Planned: attack library, mutation engine, detection, JSON and HTML reports, OpenAI-compatible adapter.

## Requirements

- Rust (install from https://rustup.rs)
- Ollama (https://ollama.com) with a model pulled: `ollama pull llama3.2:3b`

## Run

```
cargo run
```

Ollama must be running first.

## Ethics

Verdigris is intended only for models and applications you own or are authorized to test.

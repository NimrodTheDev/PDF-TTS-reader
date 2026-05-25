# PDF Speech Reader

A browser-native PDF text-to-speech reader built with Rust, WASM, and Leptos.

This project extracts text from uploaded PDF files, cleans and chunks the content into speech-friendly segments, and narrates the document using the browser Speech Synthesis API.

---

# Features

- PDF upload + extraction
- Reactive transcript rendering
- Browser Text-To-Speech playback
- Voice selection
- Playback speed control
- Voice pitch control
- Click-to-jump narration
- Auto page progression
- Text cleanup pipeline for improved speech quality
- Built fully in Rust + WASM

---

# Stack

- Rust
- Leptos
- WASM
- `web_sys`
- Browser Speech Synthesis API
- Regex-based preprocessing

---

# What This Project Taught Me

This project turned into a deep dive into:

- Rust frontend architecture
- Browser speech synthesis limitations
- WASM/browser interoperability
- Reactive ownership lifetimes
- Event-driven systems
- Cross-browser inconsistencies

The app originally started as a simple recursive speech reader and gradually evolved into a more reactive architecture.

---

# Initial Speech Architecture

The original playback system used recursive speech chaining.

Flow:

```text
Speak()
    ↓
SpeechSynthesisUtterance created
    ↓
Speech ends
    ↓
onend callback calls Speak() again
```

Example:

```rust
let on_end = Closure::wrap(Box::new(move || {
    engine.Speak(rate, pitch);
}) as Box<dyn FnMut()>);
```

This worked surprisingly well initially.

Until browsers started breaking.

---

# Browser Problems Encountered

The biggest challenge came from Chromium-based browsers.

The app:

- worked in Chrome
- later stopped speaking consistently in Brave
- still worked in Comet

Despite all being Chromium-based browsers.

---

## 1. Recursive `onend` Chains Became Unstable

Over time, recursive speech chaining appeared to destabilize the browser speech engine.

Symptoms:

- speech silently stopping
- no audio output
- speech queue desync
- playback randomly failing

Especially in Brave.

---

## 2. `speech.cancel()` Caused Queue Problems

The project relied heavily on:

```rust
speech.cancel();
```

to stop playback and switch pages.

After repeated:

- cancel
- pause
- resume
- recursive speak

the browser speech queue sometimes became corrupted.

Result:

- `speak()` still executed
- no sound played
- browser needed refresh

---

## 3. User Gesture Restrictions

Modern browsers aggressively protect media APIs.

Initial speech started from a user button click.

But recursive playback happened from callbacks:

```text
button click
    ↓
Speak()
    ↓
onend callback
    ↓
Speak()
```

Some browsers eventually stopped treating this as user-initiated audio.

Brave was especially strict here.

---

## 4. Browser Voice Initialization Issues

Some browsers lazily initialize voices.

This meant:

```rust
speechSynthesis.getVoices()
```

had to be called after user interaction before playback became reliable.

---

# Reactive Ownership Problems

Another major issue came from storing `RwSignal`s inside long-lived structs.

Example:

```rust
struct SpeakStruct {
    chunks: RwSignal<Vec<String>>,
    index: RwSignal<usize>,
}
```

This caused disposed-owner panics:

```text
you tried to access a reactive value ... but it has already been disposed
```

The issue was that browser callbacks outlived the reactive owner that created the signals.

---

# Architectural Refactor

The project is now moving toward a cleaner reactive architecture.

---

# Centralized Speech State

Instead of scattering speech state across multiple closures and signals, the new architecture centralizes everything into one state structure.

Example:

```rust
struct SpeechState {
    chunks: Vec<String>,
    index: usize,
    playing: bool,
    paused: bool,
    voice: Option<String>,
    rate: f32,
    pitch: f32,
}
```

Benefits:

- cleaner ownership
- simpler debugging
- easier persistence
- fewer lifecycle issues
- more predictable reactivity

---

# Replacing Recursive Speech Calls

The recursive pattern:

```text
onend -> Speak() -> onend -> Speak()
```

is being replaced with reactive effects.

---

# Speaking Through Effects

The new architecture uses an effect that reacts to `index` changes.

Flow:

```text
index changes
    ↓
effect reacts
    ↓
effect speaks current chunk
    ↓
speech ends
    ↓
onend increments index
    ↓
effect reacts again
```

This removes recursive speech chaining entirely.

---

# New Intended Playback Flow

```text
User presses play
    ↓
playing = true
    ↓
Effect notices:
- playing == true
- index changed
    ↓
Effect speaks chunk[index]
    ↓
Speech ends
    ↓
onend increments index
    ↓
Effect automatically speaks next chunk
```

Benefits:

- more stable browser behavior
- easier pause/resume handling
- cleaner control flow
- simpler page transitions
- fewer race conditions
- no recursive callback loops

---

# Auto Page Progression

Another effect can monitor when the current chunk reaches the end:

```text
if index >= chunks.len()
    ↓
move to next page
    ↓
reload chunks
    ↓
reset index
```

This keeps the entire playback system state-driven.

---

# Text Cleanup Pipeline

Before narration, PDF text is cleaned to improve speech quality.

The preprocessing pipeline currently handles:

- broken PDF hyphenation
- newline collapsing
- excessive whitespace
- symbol replacement
- control character removal

Example:

```text
inter-
national
```

becomes:

```text
international
```

This dramatically improves speech quality.

---

# Future Improvements

- state machine-based speech engine
- deterministic callback cleanup
- persistent reader settings
- search + bookmarks
- virtualized transcript rendering
- lazy PDF loading
- offline support
- better mobile browser compatibility
- background playback controls

---

# Final Thoughts

This project became far more than a simple PDF reader.

It turned into a deep exploration of:

- Rust frontend systems
- reactive architecture
- browser media APIs
- WASM interoperability
- cross-browser behavior

One of the biggest takeaways was realizing that browser APIs often behave unpredictably under recursive event flows, and that state-driven reactive systems are significantly more stable than callback-driven orchestration.

The architecture is now evolving toward a fully reactive playback engine centered around effects and centralized state management.
//Track a speech when it has ended
//clear speech on page change+
//switch values used in struct to a RWSignal+
// - Speak action uses this RWSignal+
// - Jump back/Move forward on click of a paragraph+
// - Listen to the signal for when an index reaches the end of the chunks use a memo
//Add voice selection


mod bind;

use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use leptos::*;
use leptos::leptos_dom::logging::console_log;
use leptos::prelude::*;
use serde_json::*;
use leptos::task::spawn_local;
use regex::Regex;
use web_sys::*;
use crate::bind::extract_pdf_book;

fn main() {
    console_error_panic_hook::set_once();
    println!("Hello, world!");
    mount_to_body(App);
}

use serde::{Deserialize, Serialize};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};


pub fn chunk_paragraph(text: &str) -> Vec<String> {
    let re =
        Regex::new(r#"[^.!?]+(?:[.!?]+["')\]]*)+"#)
                .unwrap();

    re.find_iter(text)
            .map(|m| m.as_str().trim().to_string())
            .collect()
}

pub fn clean_pdf_text(text: &str) -> String {
    let mut t = text.to_string();

    // 1. Fix broken hyphenation: "inter-\national" → "international"
    t = regex::Regex::new(r"(\w)-\s*\n\s*(\w)")
        .unwrap()
        .replace_all(&t, "$1$2")
        .to_string();

    // 2. Remove soft line breaks → join lines
    t = regex::Regex::new(r"\n+")
        .unwrap()
        .replace_all(&t, " ")
        .to_string();

    // 3. Collapse multiple spaces
    t = regex::Regex::new(r"\s{2,}")
        .unwrap()
        .replace_all(&t, " ")
        .to_string();

    t = t.replace("&", " and ");
    t = t.replace("%", " percent ");
    t = t.replace("$", " dollars ");

    t = regex::Regex::new(r"\b\d{1,4}\b")
        .unwrap()
        .replace_all(&t, "")
        .to_string();

    t = regex::Regex::new(r"(?m)^\s*\d+\s*$")
    .unwrap()
    .replace_all(&t, "")
    .to_string();

    // 4. Remove weird control/symbol chars
    t = regex::Regex::new(r"[^\x09\x0A\x0D\x20-\x7E]")
        .unwrap()
        .replace_all(&t, "")
        .to_string();

    // 5. Trim
    t.trim().to_string()
}

#[derive(Clone, Copy)]
struct SpeakStruct {
    pub chunks: RwSignal<Vec<String>>,
    pub index: RwSignal<usize>,
}

impl SpeakStruct {
    pub  fn clone_func(&self) -> Self {
        Self{
            chunks: self.chunks.clone(),
            index: self.index.clone(),
        }
    }
    pub fn new() -> Self {
        SpeakStruct { chunks: RwSignal::new(vec![]), index: RwSignal::new(0) }
    }
    pub fn Speak(&self, rate: f32, pitch: f32, selected_voice: Option<String>) {
        let chunks = self.chunks.get();
        let index = self.index.get();
        console_log(format!("{index}, {},{:?}", chunks.len(),chunks).as_str());
        if index >= chunks.len() {
            // web_sys::window().unwrap().speech_synthesis().unwrap().cancel();
            return;
        }

        let text = chunks[index].clone();
        self.index.set(index + 1);




        let window_object = web_sys::window();
        let speech = window_object.map(|e| e.speech_synthesis());
        let speech = match speech {
            Some(Ok(speech)) => speech,
            _ => return,
        };
        let engine = self.clone_func();
        let utterance = match SpeechSynthesisUtterance::new_with_text(text.as_str()) {
            Ok(u) => u,
            Err(e) => {
                console_log(format!("Failed to create utterance: {:?}", e).as_str());
                return;
            }
        };
        let selected_voice_clone = selected_voice.clone();
        utterance.set_rate(rate);
        utterance.set_pitch(pitch);
        if let Some(selected_uri) = selected_voice {
            let voices = speech.get_voices();

            for i in 0..voices.length() {
                let v = voices.get(i);

                if let Ok(v) = v.dyn_into::<SpeechSynthesisVoice>() {

                    if v.voice_uri() == selected_uri {
                        utterance.set_voice(Some(&v));
                        break;
                    }
                }
            }
        }

        console_log(format!("Speak done!, {:?}", text).as_str());

        let on_end = Closure::wrap(Box::new(move || {

            engine.Speak(rate, pitch, selected_voice_clone.clone());
        }) as Box<dyn FnMut()>);
        utterance.set_onend(Some(on_end.as_ref().unchecked_ref()));
        on_end.forget();
        speech.speak(&utterance);

    }
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Book {
    pub metadata: Metadata,
    pub total_pages: u32,
    pub pages: Vec<Page>,
}
impl Book {
    pub fn from_js(value: JsValue) -> Book {
        let book= serde_wasm_bindgen::from_value::<Book>(value).map_err(|e| e);
        book.unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all="PascalCase")]
pub struct Metadata {
    pub title: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Stats {
    pub total_pages: usize,
    pub total_words: Option<usize>,
    pub estimated_minutes: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Page {
    pub page_number: usize,
    pub text: String,
    pub char_count: usize,
}

#[component]
pub fn App() -> impl IntoView {
    let (active_chapter, set_active_chapter) = signal(0);
    let (book, set_book) = signal::<Option<Book>>(None);
    let (speech_state, set_speech_state) = signal::<Option<SpeakStruct>>(None);
    let (is_speaking, set_is_speaking) = signal(false);
    let speed_ref = NodeRef::<leptos::html::Input>::new();
    let pitch_ref = NodeRef::<leptos::html::Input>::new();
    let chunks = RwSignal::new(vec![]);
    let index = RwSignal::new(0);
    let voices = RwSignal::new(Vec::<SpeechSynthesisVoice>::new());
    let selected_voice = RwSignal::new(None::<String>);


    Effect::new(move |_| {
        let synth = web_sys::window()
            .unwrap()
            .speech_synthesis()
            .unwrap();

        let load_voices = {
            let synth = synth.clone();

            move || {
                let list = synth.get_voices();

                let mut out = vec![];

                for i in 0..list.length() {
                    let v = list.get(i);

                    if let Ok(v) = v.dyn_into::<SpeechSynthesisVoice>() {
                        out.push(v);
                    }
                }

                voices.set(out);
            }
        };

        load_voices();

        let closure = Closure::wrap(Box::new(move || {
            load_voices();
        }) as Box<dyn FnMut()>);

        synth.set_onvoiceschanged(Some(
            closure.as_ref().unchecked_ref()
        ));

        closure.forget();
    });


    Effect::new(move|_|{
        let book = book.get()
            .and_then(|e| e.pages.get(active_chapter.get()).map(|p| p.text.clone()));
        let chunks = chunk_paragraph(
            clean_pdf_text(book.unwrap_or_default().as_str()).as_str()
        );
        let speech_state = speech_state.read().deref().clone();
        if speech_state.is_some() {
            speech_state.unwrap().chunks.set(chunks);
            speech_state.unwrap().index.set(0);
            console_log("value changed");
        }

    });


    Effect::new(move |_| {

        let speech = speech_state.get();

        if let Some(speech) = speech {

            let index = speech.index.get();
            let len = speech.chunks.get().len();

            // reached end
            if len > 0 && index >= len {

                let total_pages = book
                    .get()
                    .map(|b| b.pages.len())
                    .unwrap_or(0);

                let current = active_chapter.get();

                // move to next page if possible
                if current + 1 < total_pages {

                    web_sys::window()
                        .unwrap()
                        .speech_synthesis()
                        .unwrap()
                        .cancel();

                    set_active_chapter.set(current + 1);

                    // reload chunks automatically
                    let next_text = book.get()
                        .and_then(|b| {
                            b.pages
                                .get(current + 1)
                                .map(|p| p.text.clone())
                        })
                        .unwrap_or_default();

                    let next_chunks = chunk_paragraph(
                        clean_pdf_text(&next_text).as_str()
                    );

                    speech.chunks.set(next_chunks);
                    speech.index.set(0);

                    let pitch = pitch_ref.get()
                        .map(|e| e.value_as_number())
                        .unwrap_or(1.0);

                    let rate = speed_ref.get()
                        .map(|e| e.value_as_number())
                        .unwrap_or(1.0);

                    // speech.Speak(
                    //     rate as f32,
                    //     pitch as f32,
                    //     selected_voice.get(),
                    // );

                } else {

                    // finished entire book
                    set_is_speaking.set(false);

                    web_sys::window()
                        .unwrap()
                        .speech_synthesis()
                        .unwrap()
                        .cancel();
                }
            }
        }
    });


    view! {
        <ErrorBoundary fallback=move|e|"An error occured">
        <div class="min-h-screen flex items-center justify-center bg-zinc-950 text-white">

            // App container
            <div class="w-[420px] h-[820px] bg-zinc-900 rounded-3xl border border-zinc-800 flex flex-col overflow-hidden">

                // Upload section
                <div class="p-4 border-b border-zinc-800 flex flex-col gap-1">
                    <p class="text-xs uppercase tracking-widest text-zinc-400 mb-3">
                        {move || if let Some(book) = book.get(){
                            book.metadata.title.unwrap_or("untitled".to_string())
                        }else {
                            "Upload pdf".into()
                        }}
                    </p>

                    <label class="block border-2 border-dashed border-zinc-700 rounded-xl p-6 text-center cursor-pointer hover:bg-zinc-800 transition">
                        <input type="file" class="hidden" on:change:target=move|e|{
                            let files = e.target().files();
                            if let Some(files) = files {
                                let file = files.get(0);
                                if let Some(file) = file {
                                    spawn_local(async move {
                                       let file = extract_pdf_book(file).await;
                                        match file {
                                            Ok(r)=> {
                                                console_log(format!("{:?}", r).as_str());
                                                let book = Book::from_js(r);
                                                set_book.set(Some(book));
                                            }
                                            Err(r)=> {console_log(format!("{:?}", r).as_str())}
                                        }
                                    });
                                }
                            }
                        }/>
                        <div class="space-y-1">
                            <h3 class="font-medium">"Select PDF"</h3>
                            <p class="text-sm text-zinc-400">"Tap to upload pdf"</p>
                        </div>
                    </label>
                    {move||book.get().map(|_| view! {
                        <button class="text-[red] ml-auto" on:click=move|_|set_book.set(None)>x</button>
                    })}
                </div>

                // Chapters
                <div class="p-4 border-b border-zinc-800">
                    <p class="text-xs uppercase tracking-widest text-zinc-400 mb-3">
                        "Pages"
                    </p>

                    <div class="space-y-2 max-h-40 overflow-y-auto">
                        {move ||
                            book.get().and_then(move |e| {
                              Some(e.pages.into_iter().enumerate().map(|(i, ch)| {
                                let is_active = move || active_chapter.get() == i;

                                view! {
                                    <div
                                        class=move || format!(
                                            "p-3 rounded-xl cursor-pointer transition {}",
                                            if is_active() {
                                                "bg-indigo-600"
                                            } else {
                                                "bg-zinc-800 hover:bg-zinc-700"
                                            }
                                        )
                                        on:click=move |_| set_active_chapter.set(i)
                                    >
                                        {ch.page_number}
                                    </div>
                                }
                            }).collect_view())
                            })
                        }
                    </div>
                </div>

                // Transcript
                <div class="p-4 flex-1 overflow-y-auto border-b border-zinc-800">
                    <p class="text-xs uppercase tracking-widest text-zinc-400 mb-3">
                        "Current Text"
                    </p>

                    <div class="space-y-4 text-zinc-300 leading-relaxed text-sm">
                        {
                            let speak = speech_state.read().deref().clone();
                            let speak_method = move|index: usize|{
                                    if speak.is_some() {
                                        speak.unwrap().index.set(index);
                                        web_sys::window().unwrap().speech_synthesis().unwrap().cancel();
                                        let pitch = pitch_ref.get().map(|e|{
                                            e.value_as_number()
                                        }).unwrap_or( 1f64 );
                                        let rate = speed_ref.get().map(|e|{
                                            e.value_as_number()
                                        }).unwrap_or( 1f64 );
                                        speak.unwrap().Speak(rate as f32, pitch as f32, selected_voice.get());
                                    }
                            };
                            move || {
                            let book =book.get().and_then(move|e| Some(e.pages.get(active_chapter.get()).unwrap().text.clone()));
                            let speak_method = speak_method.clone();
                            chunk_paragraph(clean_pdf_text(book.unwrap_or_default().as_str()).as_str()).into_iter().enumerate().map(|s| view! {
                                <p on:click:target=move|_|{
                                    (speak_method.clone())(s.0);
                                } class=move||format!("my-2 p-1 cursor-pointer {}", if (speak.is_some() && speak.unwrap().index.get() == (s.0 )) {"bg-yellow border-b border-zinc-300"} else {""})>{s.1}</p>
                            }).collect_view()
                        }}
                    </div>
                </div>


                // Controls
                <div class="p-4 space-y-4">

                    <div class="flex gap-2">
                        // Load — only shown when nothing is playing
                        {
                            let speak = speech_state.read().deref().clone();
                            let speak_method = move||{
                                if !speak.is_some() && book.get().is_some() {
                                    let pitch = pitch_ref.get().map(|e|{
                                        e.value_as_number()
                                    }).unwrap_or( 1f64 );
                                    let rate = speed_ref.get().map(|e|{
                                        e.value_as_number()
                                    }).unwrap_or( 1f64 );

                                    let book = book.get()
                                        .and_then(|e| e.pages.get(active_chapter.get()).map(|p| p.text.clone()));
                                    let chunk = chunk_paragraph(
                                        clean_pdf_text(book.unwrap_or_default().as_str()).as_str()
                                    );
                                    chunks.set(chunk);

                                    let speech = SpeakStruct {
                                        chunks,
                                        index,
                                    };
                                    speech.Speak(rate as f32, pitch as f32, selected_voice.get());
                                    set_speech_state.set(Some(speech));
                                    set_is_speaking.set(true);
                                }
                            };
                            move || (!speech_state.get().is_some()).then(||
                            {
                            let speak_method = speak_method.clone();

                                view! {
                            <button
                                class="flex-1 bg-zinc-800 hover:bg-zinc-700 py-3 rounded-xl transition text-sm"
                                on:click=move |_| {
                                    (speak_method.clone())();
                                }
                            >
                                "▶ Load"
                            </button>
                        }})}

                        // Pause / Resume — only shown while speech is loaded
                        {move || speech_state.get().map(|_| view! {
                            <button
                                class="flex-1 bg-zinc-800 hover:bg-zinc-700 py-3 rounded-xl transition text-sm"
                                on:click=move |_| {
                                    let synth = web_sys::window().unwrap().speech_synthesis().unwrap();
                                    if synth.paused() {
                                        synth.resume();
                                        set_is_speaking.set(true);
                                    } else {
                                        synth.pause();
                                        set_is_speaking.set(false);
                                    }
                                }
                            >
                                {move || if is_speaking.get() { "⏸ Pause" } else { "▶ Resume" }}
                            </button>
                        })}

                        // Stop — only shown while speech is loaded
                        {move || speech_state.get().map(|_| view! {
                            <button
                                class="flex-1 bg-zinc-800 hover:bg-zinc-700 py-3 rounded-xl transition text-sm"
                                on:click=move |_| {
                                    web_sys::window().unwrap().speech_synthesis().unwrap().cancel();
                                    set_speech_state.set(None);
                                    set_is_speaking.set(false);  // was missing before
                                }
                            >
                                "■ Stop"
                            </button>
                        })}

                    </div>

                    // Speed slider with live readout
                    <div class="space-y-2">
                        <div class="flex justify-between">
                            <p class="text-sm text-zinc-400">"Playback speed"</p>
                            <p class="text-sm text-zinc-400" id="speed-out">"1.0×"</p>
                        </div>
                        <input type="range" min="0.5" max="2.0" step="0.1" value="1.0"
                            class="w-full accent-indigo-500"
                            node_ref=speed_ref
                            on:input=move |e| {
                                // update readout; wire into SpeechSynthesisUtterance.set_rate() when speaking
                                let val = event_target_value(&e);
                                if let Some(el) = web_sys::window()
                                    .and_then(|w| w.document())
                                    .and_then(|d| d.get_element_by_id("speed-out")) {
                                    el.set_text_content(Some(&format!("{}×", val)));
                                }
                            }
                        />
                    </div>

                    // Pitch slider with live readout
                    <div class="space-y-2">
                        <div class="flex justify-between">
                            <p class="text-sm text-zinc-400">"Voice pitch"</p>
                            <p class="text-sm text-zinc-400" id="pitch-out">"1.0"</p>
                        </div>
                        <input type="range" min="0.5" max="2.0" step="0.1" value="1.0"
                            class="w-full accent-indigo-500"
                            node_ref=pitch_ref
                            on:input=move |e| {
                                let val = event_target_value(&e);
                                if let Some(el) = web_sys::window()
                                    .and_then(|w| w.document())
                                    .and_then(|d| d.get_element_by_id("pitch-out")) {
                                    el.set_text_content(Some(&val));
                                }
                            }
                        />
                    </div>
                    <div class="space-y-2">
    <div class="flex justify-between">
        <p class="text-sm text-zinc-400">"Voice"</p>
    </div>

    <select
        class="w-full bg-zinc-800 p-3 rounded-xl text-sm"
        on:change=move |ev| {
            let value = event_target_value(&ev);

            if value.is_empty() {
                selected_voice.set(None);
            } else {
                selected_voice.set(Some(value));
            }
        }
    >
        <option value="">
            "Default voice"
        </option>

        {
            move || {
                voices.get()
                    .into_iter()
                    .map(|voice| {

                        let uri = voice.voice_uri();

                        let label = format!(
                            "{} - {}{}{}",
                            voice.name(),
                            voice.lang(),
                            if voice.local_service() {
                                " [local]"
                            } else {
                                " [remote]"
                            },
                            if voice.default() {
                                " [default]"
                            } else {
                                ""
                            }
                        );

                        view! {
                            <option value=uri>
                                {label}
                            </option>
                        }
                    })
                    .collect_view()
            }
        }
    </select>
</div>

                </div>
            </div>
        </div>
        </ErrorBoundary>
    }
}


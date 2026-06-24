//! A `duat` [`Mode`] for searching for character sequences
//!
//! This is a plugin inspired by [`vim-sneak`], which is a kind of
//! extension to the regular `f`/`t` key bindings in vim. This one is
//! similar to it, but implemented for Duat instead
//!
//! # Installation
//!
//! Just like other Duat plugins, this one can be installed by calling
//! `cargo add` in the config directory:
//!
//! ```bash
//! cargo add duat-sneak@"*" --rename sneak
//! ```
//!
//! Or, if you are using a `--git-deps` version of duat, do this:
//!
//! ```bash
//! cargo add --git https://github.com/AhoyISki/duat-sneak --rename sneak
//! ```
//!
//! # Usage
//!
//! In order to make use of it, just add the following to your `setup`
//! function:
//!
//! ```rust
//! setup_duat!(setup);
//! use duat::prelude::*;
//!
//! fn setup() {
//!     plug(duat_sneak::Sneak::new());
//! }
//! ```
//!
//! With the above call, you will map the `s` key in [`User`] [`Mode`]
//! to the [`Sneak`] mode, you can also do that manually:
//!
//! ```rust
//! setup_duat!(setup);
//! use duat::prelude::*;
//!
//! fn setup() {
//!     map::<User>("s", duat_sneak::Sneak::new());
//! }
//! ```
//!
//! In the [`Sneak`] mode, these are the available key sequences:
//!
//! - `{char0}{char1}`: Highlight any instance of the string
//!   `{char0}{char1}` on screen. If there is only one instance, it
//!   will be selected immediately, returning to the [default mode].
//!   If there are multiple instances, one entry will be selected, and
//!   typing does the following:
//!
//!   - `n` for the next entry
//!   - `N` for the previous entry if [`mode::alt_is_reverse()`] is
//!     `false`
//!   - `<A-n>` for the previous entry if [`mode::alt_is_reverse()`]
//!     is `true`
//!   - Any other key will select and return to the [default mode]
//!
//! - Any other key will pick the last `{char0}{char1}` sequence and
//!   use that. If there was no previous sequence, just returns to the
//!   [default mode].
//!
//! # More Options
//!
//! Note: The following options can be used when plugging the mode as
//! well.
//!
//! ```rust
//! setup_duat!(setup);
//! use duat::prelude::*;
//! use duat_sneak::Sneak;
//!
//! fn setup() {
//!     map::<User>("s", Sneak::new().select_keys(',', ';').with_len(3));
//! }
//! ```
//!
//! Instead of switching with the regular keys, `;` selects the
//! previous entry and `,` selects the next. Additionally, this will
//! select three characters instead of just two.
//!
//! # Labels
//!
//! If there are too many matches, switching to a far away match could
//! be tedious, so you can do the following instead:
//!
//! ```rust
//! setup_duat!(setup);
//! use duat::prelude::*;
//! use duat_sneak::Sneak;
//!
//! fn setup() {
//!     map::<User>("s", Sneak::new().min_for_labels(8));
//! }
//! ```
//!
//! Now, if there are 8 or more matches, instead of switching to them
//! via `n` and `N`, labels with one character will show up on each
//! match. If you type the character in a label, all other labels will
//! be filtered out, until there is only one label left, at which
//! point it will be selected and you'll return to the [default mode].
//!
//! # Forms
//!
//! When plugging [`Sneak`] this crate sets two [`Form`]s:
//!
//! - `"sneak.match"`, which is set to `"default.info"`
//! - `"sneak.label"`, which is set to `"accent.info"`
//!
//! [`Mode`]: duat::mode::Mode
//! [`vim-sneak`]: https://github.com/justinmk/vim-sneak
//! [`Cargo.toml`'s `dependencies` section]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html
//! [map]: https://docs.rs/duat/latest/duat/prelude/map
//! [`User`]: duat::mode::User
//! [default mode]: mode::reset
use std::{
    ops::Range,
    sync::{LazyLock, Mutex},
};

use duat::{
    Plugin, Plugins,
    mode::{KeyCode::*, KeyMod},
    prelude::*,
};

static NS: LazyLock<Ns> = Ns::new_lazy();
static CUR_NS: LazyLock<Ns> = Ns::new_lazy();
static LAST: Mutex<String> = Mutex::new(String::new());

/// A [`Mode`] used for jumping to sequences of characters
#[derive(Clone)]
pub struct Sneak {
    step: Step,
    len: usize,
    prev_key: KeyEvent,
    next_key: KeyEvent,
    min_for_labels: usize,
    is_case_sensitive: bool,
}

impl Sneak {
    /// Create a new instance of the [`Sneak`] [`Mode`]
    pub fn new() -> Self {
        Self {
            step: Step::Start,
            len: 2,
            next_key: KeyCode::Char('n').into(),
            prev_key: if mode::alt_is_reverse() {
                KeyEvent::new(KeyCode::Char('n'), KeyMod::ALT)
            } else {
                Char('N').into()
            },
            min_for_labels: usize::MAX,
            is_case_sensitive: false,
        }
    }

    /// Which `char`s to select the previous and next matches,
    /// respectively
    ///
    /// By default, they are:
    ///
    /// - `n` for the next entry
    /// - `N` for the previous entry if [`mode::alt_is_reverse()`] is
    ///   `false`
    /// - `<A-n>` for the previous entry if [`mode::alt_is_reverse()`]
    ///   is `true`
    pub fn select_keys(self, prev: char, next: char) -> Self {
        Self {
            prev_key: Char(prev).into(),
            next_key: Char(next).into(),
            ..self
        }
    }

    /// Sneaks with `len` chars, as opposed to just 2
    #[track_caller]
    pub fn with_len(self, len: usize) -> Self {
        assert!(len >= 1, "Can't match on 0 characters");
        Self { len, ..self }
    }

    /// Sneak with or without caring for case-insensitivity
    pub fn with_case_sensitivity(self, case_sensitivity: bool) -> Self {
        Self {
            is_case_sensitive: case_sensitivity,
            ..self
        }
    }

    /// Sets a minimum number of matches to enable labels
    ///
    /// Instead of getting to a specific match with [the selection
    /// keys], a label will appear in front of each match, if you type
    /// the character in the label, [`Sneak`] will filter out all non
    /// matching labels until there are only at most 26 left, in which
    /// case the next character will finish sneaking.
    ///
    /// This feature is disabled by default (i.e. `min_for_labels ==
    /// usize::MAX`).
    ///
    /// [the selection keys]: Self::select_keys
    pub fn min_for_labels(self, min_for_labels: usize) -> Self {
        Self { min_for_labels, ..self }
    }
}

impl Plugin for Sneak {
    fn plug(self, _: &mut Opts, _: &Plugins) {
        use Step::*;
        let cloak_ns = Ns::new();

        hook::add::<ModeSwitched>(move |pa, mut switch| {
            if switch.new.is::<Sneak>() {
                let buffer = context::current_buffer(pa);

                let id = form::id_of!("cloak");
                buffer.text_mut(pa).insert_tag(cloak_ns, .., id.to_tag(239));
            } else if let Some(sneak) = switch.old.get_as::<Sneak>() {
                let buffer = context::current_buffer(pa);

                if let Filter(pat) | MatchedMove(pat, ..) | MatchedLabels(pat, _) = &sneak.step {
                    *LAST.lock().unwrap() = pat.clone();
                }

                let mut text = buffer.text_mut(pa);
                text.remove_tags(*NS, ..);
                text.remove_tags(*CUR_NS, ..);
                text.remove_tags(cloak_ns, ..);
            }
        });

        mode::map::<mode::User>("s", move |pa: &mut Pass| mode::set(pa, self.clone()))
            .doc(txt!("Enter [mode]Sneak[] mode"));

        form::set_weak("sneak.match", Form::mimic("default.info"));
        form::set_weak("sneak.label", Form::mimic("accent.info"));
        form::set_weak("sneak.current", Form::new().underlined());
    }
}

impl Mode for Sneak {
    fn bindings() -> mode::Bindings {
        mode::bindings!(match _ {
            unmod!(Char(..)) => txt!("Filter by [key.char]{{char}}"),
        })
    }

    fn send_key(&mut self, pa: &mut Pass, key: mode::KeyEvent) {
        let widget = context::current_widget(pa);

        match &mut self.step {
            Step::Start => {
                let (pat, finished_filtering) = if let unmod!(Char(char)) = key {
                    (char.to_string(), self.len == 1)
                } else {
                    let last = LAST.lock().unwrap();

                    if last.is_empty() {
                        context::error!("mode hasn't been set to [a]Sneak[] yet");
                        mode::reset::<Buffer>(pa);
                        return;
                    } else {
                        (last.clone(), true)
                    }
                };

                let should_ci = if self.is_case_sensitive { "" } else { "(?i)" };
                let regex = format!(
                    "{should_ci}{pat}[^\n]{{{}}}",
                    self.len - pat.chars().count()
                );
                let (matches, cur) = hi_matches(pa, &regex, &widget);

                let Some(cur) = cur else {
                    context::error!("No matches found for [a]{pat}");
                    mode::reset::<Buffer>(pa);
                    return;
                };

                self.step = if finished_filtering {
                    // Stop immediately if there is only one match
                    if matches.len() == 1 {
                        let range = matches[0].clone();
                        widget.edit_main(pa, |mut c| c.move_to(range));

                        mode::reset::<Buffer>(pa);

                        Step::MatchedMove(pat, matches, cur)
                    } else if matches.len() >= self.min_for_labels {
                        hi_labels(pa, &widget, &matches);

                        Step::MatchedLabels(pat, matches)
                    } else {
                        hi_cur(pa, &widget, matches[cur].clone(), matches[cur].clone());

                        Step::MatchedMove(pat, matches, cur)
                    }
                } else {
                    Step::Filter(pat)
                }
            }
            Step::Filter(pat) => {
                widget.text_mut(pa).remove_tags(*NS, ..);

                let (regex, finished_filtering) = if let unmod!(Char(char)) = key {
                    pat.push(char);

                    let should_ci = if self.is_case_sensitive { "" } else { "(?i)" };

                    let regex = format!(
                        "{should_ci}{pat}[^\n]{{{}}}",
                        self.len - pat.chars().count()
                    );
                    (regex, pat.chars().count() >= self.len)
                } else {
                    (pat.clone(), true)
                };

                let (matches, cur) = hi_matches(pa, &regex, &widget);

                let Some(cur) = cur else {
                    context::error!("No matches found for [a]{pat}");
                    mode::reset::<Buffer>(pa);
                    return;
                };

                hi_cur(pa, &widget, matches[cur].clone(), matches[cur].clone());

                if finished_filtering {
                    // Stop immediately if there is only one match
                    self.step = if matches.len() == 1 {
                        let range = matches[0].clone();
                        widget.edit_main(pa, |mut c| c.move_to(range));

                        mode::reset::<Buffer>(pa);

                        Step::MatchedMove(pat.clone(), matches, cur)
                    } else if matches.len() >= self.min_for_labels {
                        hi_labels(pa, &widget, &matches);

                        Step::MatchedLabels(pat.clone(), matches)
                    } else {
                        hi_cur(pa, &widget, matches[cur].clone(), matches[cur].clone());

                        Step::MatchedMove(pat.clone(), matches, cur)
                    };
                }
            }
            Step::MatchedMove(_, matches, cur) => {
                let prev = *cur;
                let last = matches.len() - 1;

                if key == self.next_key {
                    *cur = if *cur == last { 0 } else { *cur + 1 };
                    hi_cur(pa, &widget, matches[*cur].clone(), matches[prev].clone());
                } else if key == self.prev_key {
                    *cur = if *cur == 0 { last } else { *cur - 1 };
                    hi_cur(pa, &widget, matches[*cur].clone(), matches[prev].clone());
                } else {
                    let range = matches[*cur].clone();
                    widget.edit_main(pa, |mut c| c.move_to(range));

                    mode::reset::<Buffer>(pa);
                }
            }
            Step::MatchedLabels(_, matches) => {
                widget.text_mut(pa).remove_tags(*NS, ..);

                let filtered_label = if let unmod!(Char(char)) = key
                    && iter_labels(matches.len()).any(|label| char == label)
                {
                    char
                } else {
                    if let unmod!(Char(char)) = key {
                        context::error!("[a]{char}[] is not a valid label");
                    } else {
                        context::error!("[a]{key.code:?}[] is not a valid label");
                    }
                    mode::reset::<Buffer>(pa);
                    return;
                };

                let mut iter = iter_labels(matches.len());
                matches.retain(|_| iter.next() == Some(filtered_label));

                if matches.len() == 1 {
                    let range = matches[0].clone();
                    widget.edit_main(pa, |mut c| c.move_to(range));

                    mode::reset::<Buffer>(pa);
                } else {
                    hi_labels(pa, &widget, matches);
                }
            }
        }
    }
}

fn hi_labels(pa: &mut Pass, handle: &Handle<dyn Widget>, matches: &Vec<Range<usize>>) {
    let mut text = handle.text_mut(pa);

    text.remove_tags(*NS, ..);
    text.remove_tags(*CUR_NS, ..);

    for (label, range) in iter_labels(matches.len()).zip(matches) {
        let ghost = Overlay::new(txt!("[sneak.label:239]{label}"));
        text.insert_tag(*NS, range.start, ghost);
    }
}

fn hi_matches(pa: &mut Pass, pat: &str, widget: &Handle<dyn Widget>) -> (Vec<Range<usize>>, Option<usize>) {
    let popts = widget.read(pa).print_opts();
    let (text, area) = pa.write_many((widget.rw_text(), widget.area()));

    let start = area.start_points(&text, popts).real;
    let end = area.end_points(&text, popts).real;
    let cursor = text.main_sel().cursor().byte();

    let mut parts = text.parts();

    let matches: Vec<_> = parts.strs.search(pat).range(start..end).collect();

    let id = form::id_of!("sneak.match");

    let ns = *NS;
    let mut next = None;
    for (i, range) in matches.iter().enumerate() {
        if range.start > cursor && next.is_none() {
            next = Some(i);
        }
        parts.tags.insert(ns, range.clone(), id.to_tag(239));
    }

    let last = matches.len().checked_sub(1);
    (matches, next.or(last))
}

fn hi_cur(pa: &mut Pass, handle: &Handle<dyn Widget>, cur: Range<usize>, prev: Range<usize>) {
    let cur_id = form::id_of!("sneak.current");

    let mut text = handle.text_mut(pa);
    text.remove_tags(*CUR_NS, prev.start);
    text.insert_tag(*CUR_NS, cur, cur_id.to_tag(240));
}

fn iter_labels(total: usize) -> impl Iterator<Item = char> {
    const LETTERS: &str = "abcdefghijklmnopqrstuvwxyz";

    let multiple = total / LETTERS.len();

    let singular = LETTERS.chars().skip(multiple);

    singular
        .chain(
            LETTERS
                .chars()
                .take(multiple)
                .flat_map(|c| std::iter::repeat_n(c, 26)),
        )
        .take(total)
}

#[derive(Clone)]
enum Step {
    Start,
    Filter(String),
    MatchedMove(String, Vec<Range<usize>>, usize),
    MatchedLabels(String, Vec<Range<usize>>),
}

impl Default for Sneak {
    fn default() -> Self {
        Self::new()
    }
}

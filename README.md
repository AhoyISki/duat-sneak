# duat-sneak ![License: AGPL-3.0-or-later](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue) [![duat-sneak on crates.io](https://img.shields.io/crates/v/duat-sneak)](https://crates.io/crates/duat-sneak) [![duat-sneak on docs.rs](https://docs.rs/duat-sneak/badge.svg)](https://docs.rs/duat-sneak) [![Source Code Repository](https://img.shields.io/badge/Code-On%20GitHub-blue?logo=GitHub)](https://github.com/AhoyISki/duat-sneak)

![sneak demonstration](./assets/sneak-demonstration.gif)

A `duat` [`Mode`][__link0] for searching for character sequences

This is a plugin inspired by [`vim-sneak`][__link1], which is a kind of
extension to the regular `f`/`t` key bindings in vim. This one is
similar to it, but implemented for Duat instead

## Installation

Just like other Duat plugins, this one can be installed by calling
`cargo add` in the config directory:

```bash
cargo add duat-sneak@"*" --rename sneak
```

Or, if you are using a `--git-deps` version of duat, do this:

```bash
cargo add --git https://github.com/AhoyISki/duat-sneak --rename sneak
```

## Usage

In order to make use of it, just add the following to your `setup`
function:

```rust
setup_duat!(setup);
use duat::prelude::*;
use sneak::*;

fn setup() {
    plug!(Sneak::new());
}
```

With the above call, you will map the `s` key in [`User`][__link2] [`Mode`][__link3]
to the [`Sneak`][__link4] mode, you can also do that manually:

```rust
setup_duat!(setup);
use duat::prelude::*;
use sneak::*;

fn setup() {
    map::<User>("s", Sneak::new());
}
```

In the [`Sneak`][__link5] mode, these are the available key sequences:

* `{char0}{char1}`: Highlight any instance of the string
  `{char0}{char1}` on screen. If there is only one instance, it
  will be selected immediately, returning to the [default mode][__link6].
  If there are multiple instances, one entry will be selected, and
  typing does the following:
  
  * `n` for the next entry
  * `N` for the previous entry if [`mode::alt_is_reverse()`][__link7] is
    `false`
  * `<A-n>` for the previous entry if [`mode::alt_is_reverse()`][__link8]
    is `true`
  * Any other key will select and return to the [default mode][__link9]
* Any other key will pick the last `{char0}{char1}` sequence and
  use that. If there was no previous sequence, just returns to the
  [default mode][__link10].

## More Options

Note: The following options can be used when plugging the mode as
well.

```rust
map::<User>("s", Sneak::new().select_keys(',', ';').with_len(3));
```

Instead of switching with the regular keys, `;` selects the
previous entry and `,` selects the next. Additionally, this will
select three characters instead of just two.

## Labels

If there are too many matches, switching to a far away match could
be tedious, so you can do the following instead:

```rust
map::<User>("s", Sneak::new().min_for_labels(8));
```

Now, if there are 8 or more matches, instead of switching to them
via `n` and `N`, labels with one character will show up on each
match. If you type the character in a label, all other labels will
be filtered out, until there is only one label left, at which
point it will be selected and you’ll return to the [default mode][__link11].

## Forms

When plugging [`Sneak`][__link12] this crate sets two [`Form`][__link13]s:

* `"sneak.match"`, which is set to `"default.info"`
* `"sneak.label"`, which is set to `"accent.info"`


 [__cargo_doc2readme_dependencies_info]: ggGkYW0BYXSEG7OnKS_PkemFG56IpJ_QmFWVGx-ENAHaG7xgG84zvdQl8TPsYXKEG3uRJ8rCRKsGGzgzvxuuY1UmG8EKjuoYLaW6G6vQMjiIL1FIYWSEgmRGb3Jt9oNqZHVhdC1zbmVha2UwLjEuMWpkdWF0X3NuZWFrgmlkdWF0X2NvcmVlMC41LjOCZG1vZGX2
 [__link0]: https://docs.rs/duat_core/0.5.3/duat_core/?search=mode::Mode
 [__link1]: https://github.com/justinmk/vim-sneak
 [__link10]: https://docs.rs/mode/latest/mode/?search=reset
 [__link11]: https://docs.rs/mode/latest/mode/?search=reset
 [__link12]: https://docs.rs/duat-sneak/0.1.1/duat_sneak/struct.Sneak.html
 [__link13]: https://crates.io/crates/Form
 [__link2]: https://docs.rs/duat_core/0.5.3/duat_core/?search=mode::User
 [__link3]: https://docs.rs/duat_core/0.5.3/duat_core/?search=mode::Mode
 [__link4]: https://docs.rs/duat-sneak/0.1.1/duat_sneak/struct.Sneak.html
 [__link5]: https://docs.rs/duat-sneak/0.1.1/duat_sneak/struct.Sneak.html
 [__link6]: https://docs.rs/mode/latest/mode/?search=reset
 [__link7]: `mode::alt_is_reverse()`
 [__link8]: `mode::alt_is_reverse()`
 [__link9]: https://docs.rs/mode/latest/mode/?search=reset

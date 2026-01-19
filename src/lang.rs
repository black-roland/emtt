// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DefaultLocalizer, LanguageLoader, Localizer,
};
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n"]
struct Localizations;

pub static LANGUAGE_LOADER: Lazy<FluentLanguageLoader> = Lazy::new(|| {
    let loader: FluentLanguageLoader = fluent_language_loader!();

    loader
        .load_fallback_language(&Localizations)
        .expect("Error while loading fallback language");

    loader
});

// Define the macro in the lang module where LANGUAGE_LOADER is defined
#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!(crate::lang::LANGUAGE_LOADER, $message_id)
    }};
    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!(crate::lang::LANGUAGE_LOADER, $message_id, $($args), *)
    }};
}

pub fn init_localizer() {
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    DefaultLocalizer::new(&*LANGUAGE_LOADER, &Localizations)
        .select(&requested_languages)
        .unwrap();
    LANGUAGE_LOADER.with_bundles_mut(|bundle| {
        bundle.set_use_isolating(false);
    });
}

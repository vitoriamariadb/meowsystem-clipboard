use std::sync::LazyLock;

use i18n_embed::{
    DefaultLocalizer, LanguageLoader, Localizer,
    fluent::{FluentLanguageLoader, fluent_language_loader},
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

pub static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader: FluentLanguageLoader = fluent_language_loader!();

    loader
        .load_fallback_language(&Localizations)
        .expect("Error while loading fallback language");

    loader
});

#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::localize::LANGUAGE_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::localize::LANGUAGE_LOADER, $message_id, $($args), *)
    }};
}

// Get the `Localizer` to be used for localizing this library.
pub fn localizer() -> Box<dyn Localizer> {
    Box::from(DefaultLocalizer::new(&*LANGUAGE_LOADER, &Localizations))
}

pub fn localize() {
    let localizer = localizer();
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    if let Err(error) = localizer.select(&requested_languages) {
        error!("Error while loading language for App List {}", error);
    }
}

#[cfg(test)]
mod tests {
    fn chaves(ftl: &str) -> Vec<&str> {
        ftl.lines()
            .filter_map(|linha| linha.split_once('=').map(|(chave, _)| chave.trim()))
            .filter(|chave| !chave.is_empty() && !chave.starts_with('#'))
            .collect()
    }

    #[test]
    fn pt_br_traz_toda_chave_do_ingles() {
        let en = chaves(include_str!("../i18n/en/meowsystem_clipboard.ftl"));
        let pt = chaves(include_str!("../i18n/pt-BR/meowsystem_clipboard.ftl"));
        let faltam: Vec<_> = en.iter().filter(|chave| !pt.contains(chave)).collect();
        assert!(faltam.is_empty(), "faltam em pt-BR: {faltam:?}");
    }

    #[test]
    fn sobe_em_pt_br_com_lang_pt_br() {
        unsafe {
            std::env::set_var("LANG", "pt_BR.UTF-8");
            for nome in ["LANGUAGE", "LC_ALL", "LC_MESSAGES"] {
                std::env::remove_var(nome);
            }
        }
        super::localize();
        assert_eq!(crate::fl!("search_entries"), "Buscar");
        assert_eq!(crate::fl!("incognito"), "Modo anônimo");
        assert_eq!(crate::fl!("data_control_missing"), "O compositor não oferece o protocolo de área de transferência (data control)");
    }
}

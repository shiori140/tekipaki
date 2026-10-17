//! メニューバー（Windows はタスクトレイ）のアイコンとメニュー。
//!
//! メニューは Rust 側で作るため、画面側の辞書（i18n.js）は使えず、文言をここに持つ。

use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, Wry};

use crate::settings::{Lang, LanguagePref, Settings};

const TRAY_ID: &str = "main";

struct Labels {
    open: &'static str,
    language: &'static str,
    auto: &'static str,
    quit: &'static str,
}

fn labels(lang: Lang) -> Labels {
    match lang {
        Lang::Ja => Labels {
            open: "一覧を開く",
            // 読めない言語を選んでしまっても戻せるよう、英語も併記する。
            language: "言語 (Language)",
            auto: "自動（OS に合わせる）",
            quit: "終了",
        },
        Lang::En => Labels {
            open: "Open tekipaki",
            language: "Language",
            auto: "Automatic (system)",
            quit: "Quit",
        },
    }
}

/// 言語の選択肢。自分の言語名はどの言語で表示していても同じ表記にする。
const LANGUAGE_ITEMS: [(&str, LanguagePref, Option<&str>); 3] = [
    ("lang:auto", LanguagePref::Auto, None),
    ("lang:ja", LanguagePref::Ja, Some("日本語")),
    ("lang:en", LanguagePref::En, Some("English")),
];

fn build_menu(app: &AppHandle, pref: LanguagePref) -> tauri::Result<Menu<Wry>> {
    let l = labels(pref.resolve());
    let open = MenuItem::with_id(app, "open", l.open, true, None::<&str>)?;
    let languages = LANGUAGE_ITEMS
        .iter()
        .map(|(id, item, name)| {
            CheckMenuItem::with_id(app, *id, name.unwrap_or(l.auto), true, *item == pref, None::<&str>)
        })
        .collect::<tauri::Result<Vec<_>>>()?;
    let language_refs: Vec<&dyn tauri::menu::IsMenuItem<Wry>> =
        languages.iter().map(|i| i as &dyn tauri::menu::IsMenuItem<Wry>).collect();
    let language = Submenu::with_items(app, l.language, true, &language_refs)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", l.quit, true, None::<&str>)?;
    Menu::with_items(app, &[&open, &language, &separator, &quit])
}

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let pref = app.state::<Settings>().language();
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("tekipaki")
        .menu(&build_menu(app, pref)?)
        .on_menu_event(|app, event| {
            let id = event.id.as_ref();
            if let Some((_, pref, _)) = LANGUAGE_ITEMS.iter().find(|(item, _, _)| *item == id) {
                set_language(app, *pref);
                return;
            }
            match id {
                "open" => crate::show_main(app),
                "quit" => app.exit(0),
                _ => {}
            }
        })
        .build(app)?;
    Ok(())
}

fn set_language(app: &AppHandle, pref: LanguagePref) {
    if let Err(e) = app.state::<Settings>().set_language(pref) {
        eprintln!("言語の設定を保存できませんでした: {e}");
    }
    // 選んだ項目にだけチェックが付くよう、メニューごと作り直す。
    match (app.tray_by_id(TRAY_ID), build_menu(app, pref)) {
        (Some(tray), Ok(menu)) => {
            let _ = tray.set_menu(Some(menu));
        }
        (_, Err(e)) => eprintln!("メニューを作り直せませんでした: {e}"),
        _ => {}
    }
    let _ = app.emit("language:changed", pref.resolve());
}

# tekipaki

> [!WARNING]
> **開発中です。** まだ正式なリリースはなく、配布用のビルドもありません。保存データの形式や操作方法は、予告なく変わることがあります。
>
> **Work in progress.** There is no release or installer yet. The data format and controls may change without notice.

突発タスクを、てきぱき登録・てきぱき片付けるためのツールです。どのアプリで選んだ文字でも、ショートカットを押す回数で優先度を決めて、ローカルの ToDo に登録できます。サーバーや外部サービスは使わず、データは自分の PC にだけ保存されます。

[English](#english)

## いまの状態

| 機能 | 状態 |
|---|---|
| 選択した文字をショートカットで登録（1回: 今すぐ / 2回: 今日中 / 3回: あとで） | macOS で動作確認済み |
| 何も選択していないときの手入力欄 | macOS で動作確認済み |
| 整理画面（4列表示、キーボード操作、元に戻す） | macOS で動作確認済み |
| 日本語 / English の切り替え | macOS で動作確認済み |
| Windows 対応 | 実装済み・**未検証** |
| 振り返り、設定画面、ショートカットの変更 | 未実装 |

## 使い方

1. 好きなアプリで文字を選択します。
2. `⌘⇧Space`（Windows は `Ctrl+Shift+Space`）を押します。押す回数で優先度が決まり、最後に押してから約0.4秒で登録されます。`Esc` で取り消せます。
3. メニューバー（Windows はタスクトレイ）のアイコンから「一覧を開く」を選ぶと、整理画面が開きます。

整理画面では、矢印キーで移動、`1` `2` `3` で優先度の付け替え、`Space` で完了、`⌘Z`（`Ctrl+Z`）で元に戻す、の操作ができます。

## 開発環境で動かす

[Node.js](https://nodejs.org/) と [Rust](https://www.rust-lang.org/) が必要です。そのほかの準備は [Tauri の前提条件](https://tauri.app/start/prerequisites/) を参照してください。

```bash
npm install
npm run tauri dev
```

テストは次のコマンドで実行できます。

```bash
npm test
cd src-tauri && cargo test
```

### macOS での注意

選択した文字を取得するためにキー操作を送るので、初回は「システム設定 → プライバシーとセキュリティ → アクセシビリティ」で許可が必要です。開発中に作り直すと許可が外れることがあるので、そのときは一度オフにしてからオンに戻してください。

## データの保存場所

- macOS: `~/Library/Application Support/io.github.tekipaki/`
- Windows: `%APPDATA%\io.github.tekipaki\`

タスクは `tasks.json`、設定は `settings.json` に保存されます。外部への送信は一切しません。

## ライセンス

[MIT](LICENSE)

---

## English

tekipaki is a small tool for capturing interruptions quickly. Select text in any app, press a shortcut, and the number of presses sets the priority. Everything stays on your computer: no server, no account, no external service.

### Status

| Feature | Status |
|---|---|
| Add selected text with a shortcut (once: Now / twice: Today / 3 times: Later) | Works on macOS |
| Manual input when nothing is selected | Works on macOS |
| Task board (4 columns, keyboard controls, undo) | Works on macOS |
| Japanese / English | Works on macOS |
| Windows | Implemented, **not yet tested** |
| Review stats, settings screen, custom shortcut | Not yet |

### Usage

1. Select text in any app.
2. Press `⌘⇧Space` (`Ctrl+Shift+Space` on Windows). The number of presses sets the priority, and the task is added about 0.4 seconds after the last press. Press `Esc` to cancel.
3. Choose "Open tekipaki" from the menu bar (system tray on Windows) icon to open the board.

On the board, use the arrow keys to move, `1` `2` `3` to change priority, `Space` to mark as done, and `⌘Z` (`Ctrl+Z`) to undo.

### Running in development

You need [Node.js](https://nodejs.org/) and [Rust](https://www.rust-lang.org/). See [Tauri prerequisites](https://tauri.app/start/prerequisites/) for the rest.

```bash
npm install
npm run tauri dev
```

To run the tests:

```bash
npm test
cd src-tauri && cargo test
```

On macOS, tekipaki sends a copy keystroke to read the selected text, so you need to allow it in System Settings → Privacy & Security → Accessibility. Rebuilding during development can reset this permission; if so, turn it off and on again.

### Data

Tasks are saved to `tasks.json` and settings to `settings.json` in:

- macOS: `~/Library/Application Support/io.github.tekipaki/`
- Windows: `%APPDATA%\io.github.tekipaki\`

Nothing is ever sent anywhere.

### License

[MIT](LICENSE)

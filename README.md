# 二代目タコ八

## コードの構造

- `src/main.rs`: メインのエントリーポイント。Botの初期化とイベントハンドリングを行う
- `src/commands/`: 各スラッシュコマンドの実装を格納するディレクトリ
  - `mod.rs`: コマンドモジュールのエントリーポイント
    - 新しくモジュール (ファイル) を追加した場合、ここに `pub mod <module_name>;` を追加する
  - `ping.rs`: `/ping`
  - `count.rs`: `/count`
  - `marimo.rs`: `/mt`
    - まりもタイム


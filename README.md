# 二代目タコ八

## 開発

prek という pre-commit の仕組みを提供するツールを使うのでインストールしてください: https://prek.j178.dev/installation/

```bash
# このコマンドを実行すると commit 前に fmt と lint が実行されます。
# 警告があった場合は commit されないので、修正してから commit し直してください。
prek install
```

## コードの構造

- `src/main.rs`: メインのエントリーポイント。Botの初期化とイベントハンドリングを行う
- `src/commands/`: 各スラッシュコマンドの実装を格納するディレクトリ
  - `mod.rs`: コマンドモジュールのエントリーポイント
    - 新しくモジュール (ファイル) を追加した場合、ここに `pub mod <module_name>;` を追加する
  - `ping.rs`: `/ping`
  - `count.rs`: `/count`
  - `marimo.rs`: `/mt`
    - まりもタイム


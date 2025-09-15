# Nidaime Takohachi - systemd セットアップガイド

このドキュメントでは、Nidaime Takohachi Discord Bot を Linux サーバー上で systemd サービスとして実行する方法を説明します。

## 前提条件

- Linux サーバー（Ubuntu, Debian, CentOS, RHEL など）
- Rust ツールチェーン（rustc, cargo）がインストール済み
- root または sudo 権限
- Git がインストール済み

## セットアップ手順

### 1. リポジトリのクローン

```bash
git clone https://github.com/pistachiostudio/nidaime-takohachi.git
cd nidaime-takohachi
```

### 2. 設定ファイルの準備

`config.example.json` をコピーして `config.json` を作成し、必要な情報を設定します：

```bash
cp config.example.json config.json
vim config.json  # または好みのエディタで編集
```

以下の項目を設定してください：
- `discord_token`: Discord Bot のトークン
- `guild_id`: Bot を使用するサーバーの ID
- その他の設定項目（必要に応じて）

### 3. アプリケーションのビルド

リリースビルドを作成します：

```bash
cargo build --release
```

### 4. 自動セットアップスクリプトの実行

提供されているセットアップスクリプトを使用して、systemd サービスを自動的に設定できます：

```bash
sudo ./setup-systemd.sh
```

このスクリプトは以下の作業を自動的に行います：
- 専用ユーザー `takohachi` の作成
- `/opt/takohachi/` ディレクトリの作成
- 実行ファイルと設定ファイルのコピー
- 適切な権限の設定
- systemd サービスファイルのインストール
- サービスの自動起動設定

### 5. 手動セットアップ（オプション）

自動セットアップスクリプトを使用しない場合は、以下の手順で手動セットアップを行います：

#### 5.1. 専用ユーザーの作成

```bash
sudo useradd -r -s /bin/false -m -d /var/lib/takohachi takohachi
```

#### 5.2. アプリケーションディレクトリの作成

```bash
sudo mkdir -p /opt/takohachi
```

#### 5.3. ファイルのコピー

```bash
sudo cp target/release/nidaime-takohachi /opt/takohachi/
sudo cp config.json /opt/takohachi/
```

#### 5.4. 権限の設定

```bash
sudo chown -R takohachi:takohachi /opt/takohachi
sudo chmod 755 /opt/takohachi/nidaime-takohachi
sudo chmod 600 /opt/takohachi/config.json
```

#### 5.5. systemd サービスファイルのインストール

```bash
sudo cp takohachi.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable takohachi.service
```

## サービスの管理

### サービスの起動

```bash
sudo systemctl start takohachi
```

### サービスの停止

```bash
sudo systemctl stop takohachi
```

### サービスの再起動

```bash
sudo systemctl restart takohachi
```

### サービスの状態確認

```bash
sudo systemctl status takohachi
```

### ログの確認

リアルタイムでログを確認：

```bash
sudo journalctl -u takohachi -f
```

過去のログを確認：

```bash
sudo journalctl -u takohachi --since "1 hour ago"
sudo journalctl -u takohachi --since today
```

## トラブルシューティング

### サービスが起動しない場合

1. 設定ファイルを確認：
   ```bash
   sudo nano /opt/takohachi/config.json
   ```
   - Discord トークンが正しく設定されているか確認
   - JSON の構文エラーがないか確認

2. 権限を確認：
   ```bash
   ls -la /opt/takohachi/
   ```

3. ログを確認：
   ```bash
   sudo journalctl -u takohachi -n 50
   ```

### ネットワーク接続の問題

ファイアウォールが Discord API への接続をブロックしていないか確認：

```bash
# UFW の場合
sudo ufw status

# firewalld の場合
sudo firewall-cmd --list-all
```

### メモリ使用量の調整

サービスファイルでメモリ制限を設定しています（デフォルト: 512MB）。必要に応じて調整してください：

```bash
sudo nano /etc/systemd/system/takohachi.service
# MemoryLimit=512M を変更
sudo systemctl daemon-reload
sudo systemctl restart takohachi
```

## アップデート手順

1. サービスを停止：
   ```bash
   sudo systemctl stop takohachi
   ```

2. 最新のコードを取得：
   ```bash
   git pull origin main
   ```

3. 再ビルド：
   ```bash
   cargo build --release
   ```

4. 実行ファイルを更新：
   ```bash
   sudo cp target/release/nidaime-takohachi /opt/takohachi/
   sudo chown takohachi:takohachi /opt/takohachi/nidaime-takohachi
   sudo chmod 755 /opt/takohachi/nidaime-takohachi
   ```

5. 必要に応じて設定ファイルを更新：
   ```bash
   sudo nano /opt/takohachi/config.json
   ```

6. サービスを再起動：
   ```bash
   sudo systemctl start takohachi
   ```

## セキュリティに関する注意事項

- `config.json` には Discord Bot トークンが含まれているため、ファイル権限を `600` に設定しています
- 専用ユーザー `takohachi` で実行することで、システムへの影響を最小限に抑えています
- systemd サービス設定でセキュリティ強化オプション（`NoNewPrivileges`, `ProtectSystem` など）を有効にしています

## アンインストール

Bot を完全に削除する場合：

```bash
# サービスの停止と無効化
sudo systemctl stop takohachi
sudo systemctl disable takohachi

# サービスファイルの削除
sudo rm /etc/systemd/system/takohachi.service
sudo systemctl daemon-reload

# アプリケーションディレクトリの削除
sudo rm -rf /opt/takohachi

# ユーザーの削除（オプション）
sudo userdel -r takohachi
```

## サポート

問題が発生した場合は、以下の情報を含めて Issue を作成してください：

- OS とバージョン（`cat /etc/os-release`）
- エラーログ（`sudo journalctl -u takohachi -n 100`）
- systemd のステータス（`sudo systemctl status takohachi`）
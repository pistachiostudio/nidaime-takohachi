#!/bin/bash

# Nidaime Takohachi 自動アップデートスクリプト
# このスクリプトはsystemdで管理されているtakohachiサービスをアップデートします

set -e

# 色付き出力用の定数
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# ログ関数
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# rootまたはsudo権限の確認
if [[ $EUID -ne 0 ]]; then
   log_error "このスクリプトはroot権限またはsudoで実行する必要があります"
   exit 1
fi

# スクリプトが実行されているディレクトリの確認
if [ ! -f "Cargo.toml" ]; then
    log_error "このスクリプトはプロジェクトのルートディレクトリから実行してください"
    exit 1
fi

log_info "Nidaime Takohachi アップデートを開始します..."

# 1. サービスの停止
log_info "takohachiサービスを停止しています..."
systemctl stop takohachi
if [ $? -eq 0 ]; then
    log_info "サービスが正常に停止しました"
else
    log_error "サービスの停止に失敗しました"
    exit 1
fi

# 2. 最新のコードを取得
log_info "最新のコードを取得しています..."
sudo -u $(logname) git pull origin main
if [ $? -eq 0 ]; then
    log_info "コードが正常に更新されました"
else
    log_error "gitプルに失敗しました"
    log_info "サービスを再起動しています..."
    systemctl start takohachi
    exit 1
fi

# 3. ビルド
log_info "アプリケーションをビルドしています..."
sudo -u $(logname) cargo build --release
if [ $? -eq 0 ]; then
    log_info "ビルドが正常に完了しました"
else
    log_error "ビルドに失敗しました"
    log_info "サービスを再起動しています..."
    systemctl start takohachi
    exit 1
fi

# 4. 実行ファイルの更新
log_info "実行ファイルを更新しています..."
cp target/release/nidaime-takohachi /opt/takohachi/
if [ $? -eq 0 ]; then
    chown takohachi:takohachi /opt/takohachi/nidaime-takohachi
    chmod 755 /opt/takohachi/nidaime-takohachi
    log_info "実行ファイルが正常に更新されました"
else
    log_error "実行ファイルのコピーに失敗しました"
    exit 1
fi

# 5. 設定ファイルの確認
read -p "設定ファイル (config.json) を更新しますか? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    log_info "設定ファイルエディタを開いています..."
    vim /opt/takohachi/config.json
fi

# 6. サービスの再起動
log_info "takohachiサービスを開始しています..."
systemctl start takohachi
if [ $? -eq 0 ]; then
    log_info "サービスが正常に開始されました"
else
    log_error "サービスの開始に失敗しました"
    log_error "ログを確認してください: journalctl -u takohachi -n 50"
    exit 1
fi

# 7. サービスのステータス確認
log_info "サービスのステータスを確認しています..."
sleep 2
if systemctl is-active --quiet takohachi; then
    log_info "✅ アップデートが正常に完了しました！"
    echo
    systemctl status takohachi --no-pager | head -n 10
    echo
    log_info "最新のログ:"
    journalctl -u takohachi -n 5 --no-pager
else
    log_error "サービスが正常に動作していません"
    log_error "詳細なログ:"
    journalctl -u takohachi -n 20 --no-pager
    exit 1
fi

echo
log_info "アップデートプロセスが完了しました！"

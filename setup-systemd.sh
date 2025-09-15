#!/bin/bash

# Nidaime Takohachi systemd セットアップスクリプト
# このスクリプトは root 権限で実行する必要があります

set -e

# 色付き出力用の変数
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# エラーハンドリング
error_exit() {
    echo -e "${RED}エラー: $1${NC}" >&2
    exit 1
}

# 成功メッセージ
success_msg() {
    echo -e "${GREEN}✓ $1${NC}"
}

# 警告メッセージ
warning_msg() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# root権限チェック
if [[ $EUID -ne 0 ]]; then
   error_exit "このスクリプトは root 権限で実行する必要があります。\n  使用方法: sudo $0"
fi

echo "======================================"
echo "Nidaime Takohachi systemd セットアップ"
echo "======================================"
echo ""

# 1. ビルドの確認
echo "1. リリースビルドの確認..."
if [ ! -f "./target/release/nidaime-takohachi" ]; then
    warning_msg "リリースビルドが見つかりません。ビルドを実行します..."
    cargo build --release || error_exit "ビルドに失敗しました"
fi
success_msg "リリースビルドを確認しました"

# 2. 専用ユーザーの作成
echo ""
echo "2. 専用ユーザーの作成..."
if id "takohachi" &>/dev/null; then
    warning_msg "ユーザー 'takohachi' は既に存在します"
else
    useradd -r -s /bin/false -m -d /var/lib/takohachi takohachi || error_exit "ユーザーの作成に失敗しました"
    success_msg "ユーザー 'takohachi' を作成しました"
fi

# 3. ディレクトリの作成
echo ""
echo "3. アプリケーションディレクトリの作成..."
if [ -d "/opt/takohachi" ]; then
    warning_msg "/opt/takohachi は既に存在します"
    read -p "既存のディレクトリを削除して再作成しますか？ (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf /opt/takohachi
        mkdir -p /opt/takohachi
    fi
else
    mkdir -p /opt/takohachi
fi
success_msg "ディレクトリを作成しました"

# 4. ファイルのコピー
echo ""
echo "4. ファイルのコピー..."
cp ./target/release/nidaime-takohachi /opt/takohachi/ || error_exit "実行ファイルのコピーに失敗しました"
success_msg "実行ファイルをコピーしました"

# 5. config.json の処理
echo ""
echo "5. 設定ファイルの準備..."
if [ -f "./config.json" ]; then
    cp ./config.json /opt/takohachi/ || error_exit "config.json のコピーに失敗しました"
    success_msg "config.json をコピーしました"
else
    if [ -f "./config.example.json" ]; then
        cp ./config.example.json /opt/takohachi/config.json || error_exit "config.example.json のコピーに失敗しました"
        warning_msg "config.example.json を config.json としてコピーしました"
        warning_msg "必ず /opt/takohachi/config.json を編集して Discord トークンを設定してください"
    else
        error_exit "config.json または config.example.json が見つかりません"
    fi
fi

# 6. 権限の設定
echo ""
echo "6. ファイル権限の設定..."
chown -R takohachi:takohachi /opt/takohachi || error_exit "所有者の変更に失敗しました"
chmod 755 /opt/takohachi/nidaime-takohachi || error_exit "実行権限の設定に失敗しました"
chmod 600 /opt/takohachi/config.json || error_exit "config.json の権限設定に失敗しました"
success_msg "権限を設定しました"

# 7. systemd サービスファイルのインストール
echo ""
echo "7. systemd サービスファイルのインストール..."
if [ ! -f "./takohachi.service" ]; then
    error_exit "takohachi.service が見つかりません"
fi
cp ./takohachi.service /etc/systemd/system/ || error_exit "サービスファイルのコピーに失敗しました"
success_msg "サービスファイルをインストールしました"

# 8. systemd のリロード
echo ""
echo "8. systemd の設定をリロード..."
systemctl daemon-reload || error_exit "systemd のリロードに失敗しました"
success_msg "systemd の設定をリロードしました"

# 9. サービスの自動起動設定
echo ""
echo "9. サービスの自動起動設定..."
systemctl enable takohachi.service || error_exit "自動起動の設定に失敗しました"
success_msg "自動起動を設定しました"

echo ""
echo "======================================"
echo -e "${GREEN}セットアップが完了しました！${NC}"
echo "======================================"
echo ""
echo "次のステップ:"
echo "1. 設定ファイルを確認・編集:"
echo "   sudo nano /opt/takohachi/config.json"
echo ""
echo "2. サービスを起動:"
echo "   sudo systemctl start takohachi"
echo ""
echo "3. サービスの状態を確認:"
echo "   sudo systemctl status takohachi"
echo ""
echo "4. ログを確認:"
echo "   sudo journalctl -u takohachi -f"
echo ""

# config.json の編集が必要かチェック
if grep -q "YOUR_DISCORD_BOT_TOKEN" /opt/takohachi/config.json 2>/dev/null; then
    echo ""
    warning_msg "重要: config.json に Discord Bot トークンを設定する必要があります！"
    read -p "今すぐ編集しますか？ (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        nano /opt/takohachi/config.json
    fi
fi
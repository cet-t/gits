# gits

インタラクティブなコミット・ブランチセレクタ付き git ラッパー。
よく使う git 操作をキーボードだけで素早く実行できます。

## インストール

```sh
cargo install --path .
```

`~/.cargo/bin` にパスが通っていれば `gits <subcommand>` で使えます。

### パスの確認

```sh
# パスが通っているか確認
gits --help
```

通っていない場合は `~/.cargo/bin` を `PATH` に追加してください。

```sh
# .bashrc / .zshrc に追記
export PATH="$HOME/.cargo/bin:$PATH"
```

## 使い方

```
gits <subcommand> [options]
```

セレクタは `j`/`k` または矢印キーで移動、`Enter` または `l` で確定、`Esc` / `h` / `Ctrl-C` でキャンセルします。

---

## サブコマンド

### `show` — コミット内容を表示

コミット一覧からインタラクティブに選択して `git show` を実行します。

```sh
gits show
gits show --stat
gits show -p --name-only
```

### `diff` — コミット間の差分を表示

ベースとターゲットの 2 つのコミットを順に選択して `git diff` を実行します。

```sh
gits diff
gits diff --stat
gits diff --base HEAD         # ベースを固定してターゲットだけ選択
```

### `switch` — ブランチを切り替え

ブランチ一覧から選択して `git switch` を実行します。

```sh
gits switch
gits switch --detach
```

### `merge` — ブランチをマージ

ブランチ一覧から選択して `git merge` を実行します。

```sh
gits merge
gits merge --no-ff
```

### `rebase` — ブランチをリベース

ブランチ一覧から選択して `git rebase` を実行します。

```sh
gits rebase
gits rebase --interactive     # 選択したブランチで git rebase -i を実行
```

### `commit` — コミットハッシュを出力

コミット一覧から選択してハッシュを標準出力に出力します。
他のコマンドと組み合わせて使います。

```sh
git rebase -i $(gits commit)
git diff $(gits commit) HEAD
git cherry-pick $(gits commit)
```

### `branch` — ブランチ名を出力

ブランチ一覧から選択してブランチ名を標準出力に出力します。

```sh
git switch $(gits branch)
git diff $(gits branch)
git merge --no-ff $(gits branch)
```

---

## キーバインド

| キー                   | 動作                         |
| ---------------------- | ---------------------------- |
| `j` / `↓`              | 次の項目へ                   |
| `k` / `↑`              | 前の項目へ                   |
| `Enter` / `l`          | 選択して確定                 |
| `Esc` / `h` / `Ctrl-C` | キャンセル（終了コード 130） |

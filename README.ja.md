# formatdiff - フィールドベースのバイナリ比較ツール

2つのファイルを、フォーマットのパース後におけるフィールド単位で比較するツールです。

[ImHex](https://github.com/WerWolv/ImHex)のパターン定義([ImHex-Patterns](https://github.com/WerWolv/ImHex-Patterns))を使用しています。

## ビルド

Rust 2024 Edition、C++23対応コンパイラ、CMakeが必要です。

```sh
git clone --recurse https://github.com/couyoh/formatdiff
cd formatdiff
cargo build --release
```

## 使い方

`a.bmp`と`b.bmp`を比較したい場合:

```
formatdiff --patterns-root vendor/ImHex-Patterns/patterns \
    -I vendor/ImHex-Patterns/includes a.bmp b.bmp
```

使用する`.hexpat`ファイルは、デフォルトでは拡張子を基に判断します。
明示的に指定する場合は`--format`を使用します。

該当の`.hexpat`ファイルは、カレントディレクトリから探索します。
明示的に指定する場合は、`--patterns-root`を使用します。多くの場合、ImHex-Patternsの`patterns/`ディレクトリを指定します。

明示的にパターンを指定する場合は、`--pattern`を使用します。

パターンライブラリを指定する場合は、`-I`を使用します。

## ライセンス

GPL-2.0-only
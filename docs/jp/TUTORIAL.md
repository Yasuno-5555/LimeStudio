# Squeezer IDE チュートリアル：可視化された開発体験

このチュートリアルでは、LimeStudio の統合開発環境 **Squeezer** を使用して、シンプルなレゾナント・フィルターを構築するプロセスを学びます。

---

## 1. Squeezer の起動

まず、IDE をビルドして起動します。

```bash
cargo run -p squeezer
```

起動すると、左側にプロジェクトツリー、中央に VPL (Visual Programming Language) キャンバスが表示されます。

---

## 2. DSP グラフの構築 (The Reality)

Squeezer の中央キャンバスで、右クリックメニューからノードを追加して接続します。

1.  **Input ノード** を追加します。
2.  **Filter ノード** を追加し、Input から接続します。
3.  **Output ノード** を追加し、Filter から接続します。
4.  **Knob ノード** を 2 つ追加し、Filter の `Frequency` と `Resonance` に接続します。

この時点で、音がリアルタイムで変化するのを確認できます。これが **現実層 (Reality Layer)** の構築です。

---

## 3. UI のプロジェクション (The Perception)

DSP グラフが完成したら、ユーザーに見せるための **知覚層 (Perception Layer)** を定義します。Squeezer の `UI Layout` ビューに切り替えるか、`src/lib.rs` で以下のように記述します。

```rust
ui: |params| {
    view! {
        vbox![
            Label::new("SQUEEZER FILTER").secondary(),
            hbox![
                Knob::new(params.cutoff).label("CUTOFF"),
                Knob::new(params.resonance).label("RESONANCE"),
            ],
            Lens::new("output_viz", "spectrum"),
        ]
    }
}
```

Squeezer IDE 内で、この記述に基づいた UI が即座にプレビューされます。

---

## 4. リアルタイム監査 (Forensic Inspection)

Squeezer の特徴は、開発中に「何が起きているか」を完全に可視化できることです。

- **Causality Link**: ノブを動かしたとき、どの DSP パラメータが影響を受けているかを視覚的なラインで表示。
- **Confidence Visualizer**: コンパイラが DSP 命令を生成する際の「確信度」を表示し、潜在的なエラーを未然に防ぎます。

---

## 5. 出荷 (Exporting)

IDE 内でプロトタイプが完成したら、`BUILD` ボタンを押すか、以下のコマンドで製品版バイナリ (VST3/CLAP) をエクスポートします。

```bash
cargo lime export
```

エクスポートされたプラグインは、Squeezer で構築したグラフと UI をそのまま保持し、高い信頼性を持って DAW 上で動作します。

---

## 次のステップ

より詳細な UI コンポーネントのカスタマイズについては、[UI チュートリアル](file:///Users/yasuno/projects/LimeStudio/docs/jp/UI_TUTORIAL.md) を参照してください。

# LimeStudio: オーディオロジックのための可視化されたコンパイラ

> 「確信こそが製品であり、視覚的な明快さこそが武器である。」

LimeStudio は、プロフェッショナル・グレードの信頼性、透明性、そしてシグナルプロセッシング・チェーンにおける「信頼の UI (Trust UI)」を求めるミュージシャンやデベロッパーのために設計された高性能オーディオフレームワークです。

**DirtyData Kernel** を搭載した LimeStudio は、高レベルなビジュアルデザインと低レベルなフォレンジック実行の間の溝を埋めます。

---

## 1. アーキテクチャ：現実 vs 知覚

LimeStudio は、完全なリアルタイム安全性を確保するために厳格な **Execution Boundary Controller（実行境界コントローラー）** アーキテクチャを採用しています。

1.  **知覚層 (Shell)**: UI (`limestudio_surface`, `limestudio_vpl`)。人間の相互作用と視覚的状態を管理します。通信は **意図 (Intent)** のみを通じて行われます。
2.  **トランザクション層 (Bridge)**: UI の Intent をカーネルの Patch に変換し、ID 解決（ULID <-> u64）と楽観的な更新を管理します。
3.  **現実層 (Kernel)**: **DirtyData** エンジン。生のグラフを実行します。オーディオ、DSP、および構造的な状態に関する唯一の真実のソースです。
4.  **コントラクト層 (SDK)**: `dirtydata-sdk`。外部プラグインのための「カーネルの法」を定義し、安定した ABI と DSP 開発用の `plugin!` マクロを提供します。

---

## 2. コア・ピラー（核心となる柱）

### 2.1 Trust UI (運用の明快さ)
私たちは単なる「綺麗な」ノブは作りません。私たちは **Trust UI** を作ります。
- **モジュレーションリング**: パラメータがリアルタイムでどのように影響を受けているかを正確に可視化。
- **安全性モニター**: レイテンシ、デノーマル、数学的な不安定性への即時の可視性。
- **プロバナンストレース**: パラメータ値がなぜその値なのか（Macro -> LFO -> Velocity）を正確に追跡。

### 2.2 Lime Surface (物理主導の UI ランタイム)
オーディオソフトウェア専用に設計された、カスタム `wgpu` + `SDF` レンダリングエンジン。
- **線形補間の禁止**: すべての UI 遷移は **臨界減衰スプリング (Critically Damped Springs)** を使用します。
- **Oklab の義務化**: 知覚的な一貫性のため、すべてのカラー補間は Oklab 空間で行われます。
- **SDF レンダリング**: コストゼロのスケーリングと完璧なアンチエイリアシング。

### 2.3 三種の神器
「Lime の法」を維持するため、3 つの義務的な監査ツールを提供します：
- **`cargo lime lint` (静的な裁判官)**: HIG v3.0 への準拠を強制。装飾的なライティング（グロー/シャドウ）を拒否し、DSP パス内の非リアルタイム安全なコードを検出します。
- **`cargo lime doctor` (生存監査)**: ホスト固有の安定性問題（Logic, FL Studio 等）を診断し、「JUCE 臭（JUCE-smell）」を検出します。
- **`cargo lime verify` (真実の証明)**: `.lime` アーティファクトのフォレンジックな検証。音の生成がソースロジックと同一であることを保証します。

---

## 3. コンポーネント

- **`limestudio_vpl`**: ビジュアルパッチング環境。
- **`limestudio_surface`**: 高性能な Matte & Precision UI ランタイム (SDF ベース)。
- **`limestudio_lint`**: HIG 執行のための「信頼のコンパイラ」エンジン。
- **`limestudio_cli`**: `lint`, `verify`, `testify`, `release` のためのコマンドセンター。
- **`DirtyData/dirtydata-runtime`**: 現実層。高性能な JIT 搭載オーディオエンジン。

---

## 4. はじめに

### 学習パス
1.  **メンタルモデルを読む**: 私たちの [コア原則](file:///Users/yasuno/projects/LimeStudio/docs/jp/MENTAL_MODELS.md) を理解する。
2.  **技術ガイドを読む**: [アーキテクチャと実装](file:///Users/yasuno/projects/LimeStudio/docs/jp/TECHNICAL_GUIDE.md) の詳細を学ぶ。
3.  **チュートリアルに従う**: [ステップバイステップ・ガイド](file:///Users/yasuno/projects/LimeStudio/docs/jp/TUTORIAL.md) で最初のプラグインを作成する。
4.  **内部構造を学ぶ**: Rust エキスパートのための [エンジン・アーキテクチャ](file:///Users/yasuno/projects/LimeStudio/docs/jp/ADVANCED_INTERNALS.md) 深掘り。
4.  **テンプレートを探索する**: `templates/` ディレクトリでインスピレーションを得る。

### サウンドロジックの出力
LimeStudio はライブパッチングのためだけのものではありません。プロフェッショナルなプラグインのためのエクスポーターでもあります：
```bash
# あなたのグラフをビルド可能な Rust プロジェクトに変換（錬成）する
dirty-cli export my_patch.lime --target vst3 --output ./MyPlugin
```

---

## 5. ライセンス
MIT - 透明なロジック。プロフェッショナルなサウンド。

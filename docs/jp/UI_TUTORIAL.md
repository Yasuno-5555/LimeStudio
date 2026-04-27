# LimeSurface チュートリアル：宣言的 UI の構築

このチュートリアルでは、LimeStudio の UI フレームワークである LimeSurface を使用して、宣言的で美しいプラグイン UI を構築する方法を学びます。

## 1. 基本的な考え方：UI は「投影」である

LimeSurface において、UI は状態を直接変更するものではありません。代わりに、**「現在のパラメータ状態から、どのような UI が見えるべきか」**というルールを記述します。

## 2. はじめての UI 定義

プラグインの `build_ui` メソッド内で、`view!` マクロを使用して UI を構築します。

```rust
fn build_ui(params: Arc<MyParams>, obs: ObservationConsumer) -> Box<dyn Widget> {
    Box::new(view! {
        vbox![
            Label::new("Master Volume"),
            Knob::new(params.gain).label("Gain"),
            hbox![
                Toggle::new(params.mute).label("Mute"),
                Badge::new("Active")
            ]
        ]
    })
}
```

- `vbox!`: ウィジェットを垂直方向に並べます。
- `hbox!`: ウィジェットを水平方向に並べます。
- `view!`: 宣言的な記述を、エンジンが理解できる中間表現に変換します。

## 3. コンポーネントの共通化 (Custom Widgets)

UI が複雑になってきたら、独自のウィジェット構造体を作成して共通化できます。これは Flutter の `StatelessWidget` に相当します。

```rust
struct ControlGroup<'a> {
    title: &'static str,
    param: UiParam<'a>,
}

impl<'a> Widget for ControlGroup<'a> {
    fn build(&self) -> WidgetIR {
        vbox![
            Label::new(self.title).secondary(),
            Padding::new(16.0, Knob::new(self.param)),
        ]
    }
}
```

作成したコンポーネントは、他のウィジェットと同様に `vbox!` などの中で使用できます。

## 4. レイアウトの調整：Padding と Spacer

HIG v3.0 に準拠するため、余白は常に 8px の倍数である必要があります。

- `Padding`: 子ウィジェットの周囲に余白を追加します。
- `Spacer`: Flex レイアウト内で、利用可能な空間を埋めます。

```rust
vbox![
    HeaderWidget::new(),
    Spacer::new(1.0), // 下に押し下げる
    FooterWidget::new(),
]
```

## 5. 物理アニメーションの恩恵

LimeSurface では、アニメーションを明示的に記述する必要はありません。
`Knob` の値が変わったり、`Toggle` が切り替わったりする際、エンジンは自動的に **Critically Damped Spring** モデルを適用し、滑らかで物理的に正しい遷移を描画します。

## 6. 次のステップ

さらに高度なビジュアライザー（スペクトラムアナライザーや波形表示など）を実装したい場合は、`CustomWidget` を使用して独自の SDF プリミティブを定義する方法について [TECHNICAL_GUIDE.md](file:///Users/yasuno/projects/LimeStudio/docs/jp/TECHNICAL_GUIDE.md) を参照してください。

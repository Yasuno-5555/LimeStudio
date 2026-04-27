# LimeSurface Tutorial: Building Declarative UIs

This tutorial teaches you how to build beautiful, declarative plugin UIs using LimeSurface, the UI framework for LimeStudio.

## 1. Core Concept: UI as a Projection

In LimeSurface, the UI does not change state directly. Instead, you describe the rules for **"what the UI should look like given the current parameter state."**

## 2. Defining Your First UI

Use the `view!` macro inside your plugin's `build_ui` method:

```rust
fn build_ui(params: Arc<MyParams>, obs: ObservationConsumer) -> Box<dyn Widget> {
    Box::new(view! {
        vbox![
            Label::new("LIME FILTER").secondary(),
            Knob::new(params.cutoff).label("CUTOFF"),
            hbox![
                Toggle::new(params.mute).label("MUTE"),
                Badge::new("ACTIVE")
            ]
        ]
    })
}
```

- `vbox!`: Arranges widgets vertically.
- `hbox!`: Arranges widgets horizontally.
- `view!`: Converts declarative descriptions into the Intermediate Representation (IR) understood by the engine.

## 3. Creating Custom Components

As your UI grows, you can create your own widget structs for reusability. This is equivalent to Flutter's `StatelessWidget`.

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

## 4. Layout Helpers: Padding and Spacer

To comply with HIG v3.0, spacing must always be a multiple of 8px.

- `Padding`: Adds space around a child widget.
- `Spacer`: Fills available space in a Flex layout.

```rust
vbox![
    HeaderWidget::new(),
    Spacer::new(1.0), // Pushes the footer to the bottom
    FooterWidget::new(),
]
```

## 5. The Benefits of Physical Motion

You don't need to write explicit animations. When a knob value changes or a toggle flips, the engine automatically applies a **Critically Damped Spring** model for smooth, physically accurate transitions.

## 6. Next Steps

To build advanced visualizers (spectrum analyzers, waveforms), refer to the [Technical Guide](file:///Users/yasuno/projects/LimeStudio/docs/en/TECHNICAL_GUIDE.md) on how to define your own SDF primitives using `CustomWidget`.

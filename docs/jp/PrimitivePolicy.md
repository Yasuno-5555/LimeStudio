# Lime Studio: Primitive Policy & Doctrine of Overlap

Lime Studio における描画命令（Primitive）は、単なる幾何学ではなく、データの「出所」と「責任」を証明するための「法」に従わなければなりません。

## 1. Primitive Admission Policy (加入条件)

新しいプリミティブを追加するには、以下の5条件をすべて満たす必要があります。

1.  **Semantic Independence**: それ自体が「値」「接続」「瞬間」などの固有の意味を持つこと。
2.  **Reuse Across Domains**: 特定のウィジェット専用ではなく、汎用的なセマンティクスであること。
3.  **Interaction Contract**: Hit test / Snap / Focus など、操作系との契約を持つこと。
4.  **Temporal Contract**: 「いつ嘘をついてよいか（補完戦略）」を定義できること。
5.  **Provenance Visibility**: データの出所を監査可能であり、Trust UI に寄与すること。

## 2. DensityMap Truth Policy (ラスタデータの真実)

スペクトログラム等の高密度データは、以下の原則で投影されます。

- **Freshness > Completeness**: 時間の連続性よりも、現在の真実（最新性）を優先する。
- **Batch Accumulation**: 前回描画時からの全スライスを一括で提出する。
- **Trace Lost (欠落の自白)**: バッファ溢れ（Overrun）が発生した際は、それを隠蔽せず、視覚的なマーカー（赤い線等）で「証拠欠落」を正直に描画する。
- **Resolution Scaling**: 負荷時はフレームレートを落とさず、解像度（Bins）を落として対応する。

## 3. Doctrine of Overlap (重なりの法)

アルファ合成（透かし）による混色を禁止し、幾何学的な「関係」を描画します。

### 1. 合算の法 (Law of Total Response)
パラメトリックEQのように、論理的に加算されるべき要素は、個別の領域を透かして重ねるのではなく、**合算された一つの結果（Union）**をソリッドに描画する。個別の構成要素は輪郭線（Path）のみで示す。

### 2. 排他の法 (Semantic Occlusion)
独立したオブジェクトが重なる場合、前面のオブジェクトは背後をソリッドに隠蔽する。ただし、境界には **SDF減算による「ネガティブスペース（隙間）」** を設け、前後関係という「事実」を明示する。

### 3. 交差の法 (Boolean Intersection)
選択範囲などの「論理積」を示す必要がある場合、混色ではなく **Oklab空間における明度（L）のステップ移動** を行い、第三のソリッドな領域として描画する。

## 4. Interaction & Transform Laws (操作と変換の法)

幾何学的な複雑さに対し、操作の整合性を保つためのルールです。

- **Inverse Transform Law (逆変換の法)**:
  描画階層（TransformNode）は「Local -> Screen」の変換を行うが、Interaction Kernel は常にその逆行列を保持し、Screen 座標を各階層の Local 座標へ引き戻して判定しなければならない。
- **Interaction Clip Sovereignty (操作クリップの主権)**:
  `ClipMask` は描画と操作の両方に作用するが、その適用範囲は独立して定義可能とする。「見えているのに触れない」等の不整合を避けるため、セマンティクスによる明示を必須とする。
- **Silent Honesty (静かなる誠実)**:
  `Trace Lost` 等のシステムエラーの警告はユーザーの集中を削いではならない。スペクトログラム端の微かなノッチ等の控えめな表現に留め、詳細な証拠は Forensic Mode（法廷モード）でのみ開示する。

## 5. Causal Replay Policy (因果再生の法)

過去の因果を遡るための、ハイブリッドな記録政策です。

- **Layer 1: Responsibility Events (主権)**:
  ランク交代、閾値越え、接続変更等の「事件」のみを 1ms 精度で記録する。これがリプレイの主軸（裁判記録）となる。
- **Layer 2: Sparse Snapshots (補助)**:
  なだらかな変調を再現するため、10Hz 程度の頻度で主要なパラメータの動きをスナップショットとして保持する。
- **Layer 3: Forensic Freeze (例外)**:
  ユーザー要求時のみ、特定区間を 60Hz 以上のフル解像度で凍結保存する。
- **Law of Read-Only Evidence (証拠保全の法)**:
  リプレイ中、過去の証拠（パラメータ状態）を編集することは文明の崩壊を招くため、物理的に禁止する。リプレイは常に「読み取り専用」である。

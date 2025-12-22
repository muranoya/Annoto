# app.rs リファクタリング計画

## 現在の問題点

### 1. AnnotoApp構造体が肥大化（59フィールド）
- 描画状態、UI状態、選択状態が混在
- 責任が多すぎて保守困難

### 2. 重複コード
- `handle_drawing_mode()` と `render_drag_preview()` で図形作成ロジックが重複
- 各図形タイプのマッチング処理が複数箇所に存在

### 3. メソッドが大きすぎる
- `render_central_panel()`: 約120行
- `handle_drawing_mode()`: 約110行
- `render_drag_preview()`: 約100行

### 4. テスト困難
- 全機能が1つの構造体に依存
- UI、ロジック、状態が密結合

---

## 推奨リファクタリング戦略

**責任分離 + 機能モジュール化**

### 新しいディレクトリ構造

```
src/
├── app.rs (メインアプリ、UI統合 - 約200行に削減)
├── main.rs (既存)
├── drawing_tool.rs (既存)
├── canvas_items/ (既存)
├── state/
│   ├── mod.rs (状態管理の統合)
│   ├── drawing_state.rs (描画状態: zoom, drag_start, current_tool)
│   ├── ui_state.rs (UI状態: cursor_pos, show_export_dialog)
│   └── selection_state.rs (選択状態: selected_item, selected_handle)
├── ui/
│   ├── mod.rs (UI統合)
│   ├── top_panel.rs (トップパネル: 約40行)
│   ├── side_panel.rs (サイドパネル: 約130行)
│   ├── central_panel.rs (セントラルパネル: 約120行)
│   └── export_dialog.rs (エクスポートダイアログ: 約40行)
├── drawing/
│   ├── mod.rs (描画ロジック統合)
│   ├── shape_factory.rs (図形生成ロジック)
│   ├── preview_renderer.rs (プレビュー描画)
│   └── item_renderer.rs (アイテム描画)
└── export/
    ├── mod.rs (エクスポート統合)
    ├── image_exporter.rs (画像エクスポート: 約80行)
    └── download_handler.rs (ダウンロード処理: 約25行)
```

---

## 各モジュールの詳細

### state/ - 状態管理の分離

**drawing_state.rs** (約30行)
```rust
pub struct DrawingState {
    pub zoom: f32,
    pub current_tool: DrawingTool,
    pub drag_start: Option<egui::Pos2>,
    pub stroke_width: f32,
    pub stroke_color: egui::Color32,
    pub fill_color: egui::Color32,
    pub rounding: u8,
    pub mosaic_granularity: u8,
}
```

**ui_state.rs** (約20行)
```rust
pub struct UiState {
    pub cursor_pos: Option<egui::Pos2>,
    pub show_export_dialog: bool,
    pub export_format: String,
    pub mode: AppMode,
    pub pan_offset: egui::Vec2,
}
```

**selection_state.rs** (約15行)
```rust
pub struct SelectionState {
    pub selected_item: Option<usize>,
    pub selected_handle: Option<crate::canvas_items::Handle>,
}
```

### ui/ - UI描画ロジックの分離

各パネルを独立したモジュールに分割。
- `top_panel.rs`: トップメニューバー
- `side_panel.rs`: ツール選択とプロパティ設定
- `central_panel.rs`: 画像表示と描画キャンバス
- `export_dialog.rs`: エクスポートダイアログ

### drawing/ - 描画ロジックの分離

**shape_factory.rs** (約110行)
- 図形生成ロジックを集約
- `create_shape_from_drag()` メソッドで重複を排除

**preview_renderer.rs** (約100行)
- ドラッグプレビュー描画

**item_renderer.rs** (約10行)
- 既存アイテムの描画

### export/ - エクスポート機能の分離

**image_exporter.rs** (約80行)
- 画像エクスポート処理

**download_handler.rs** (約25行)
- ブラウザダウンロード処理

---

## 実装ステップ

### Phase 1: 状態管理の分離
1. `src/state/mod.rs` を作成
2. `src/state/drawing_state.rs` を実装
3. `src/state/ui_state.rs` を実装
4. `src/state/selection_state.rs` を実装
5. `app.rs` で新しい状態構造体を使用

### Phase 2: UI描画の分離
1. `src/ui/mod.rs` を作成
2. `src/ui/top_panel.rs` を実装
3. `src/ui/side_panel.rs` を実装
4. `src/ui/central_panel.rs` を実装
5. `src/ui/export_dialog.rs` を実装

### Phase 3: 描画ロジックの分離
1. `src/drawing/mod.rs` を作成
2. `src/drawing/shape_factory.rs` を実装
3. `src/drawing/preview_renderer.rs` を実装
4. `src/drawing/item_renderer.rs` を実装

### Phase 4: エクスポート機能の分離
1. `src/export/mod.rs` を作成
2. `src/export/image_exporter.rs` を実装
3. `src/export/download_handler.rs` を実装

### Phase 5: app.rsの統合
1. 新しいモジュールをインポート
2. `AnnotoApp` を簡潔化
3. イベント処理を統合

### Phase 6: テストと検証
1. 既存機能の動作確認
2. ビルドエラーの修正
3. 動作テスト

---

## メリット

✅ **保守性向上**: 各モジュールが単一責任を持つ  
✅ **テスト容易**: 各機能を独立してテスト可能  
✅ **再利用性**: UI要素や描画ロジックを再利用可能  
✅ **拡張性**: 新機能追加時の影響範囲が限定的  
✅ **可読性**: 各ファイルが小さく、理解しやすい  
✅ **コード削減**: app.rsが979行から約200行に削減

---

## 注意点

- 各モジュール間の依存関係を最小化する
- 共通の型定義は `mod.rs` で管理
- テクスチャハンドルなどのリソースは慎重に管理
- 既存の `canvas_items/` との連携を保つ

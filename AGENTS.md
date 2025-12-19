# Annoto

## 概要

Annotoは、画像ペインティングツール。画像に矢印、四角形、直線、テキストなどの図形を描画し、画像を編集できる簡易的なツール。WebAssemblyで動作。

## 主要コンポーネント

### src/main.rs

アプリのエントリーポイント。eframeを使ってWebAssemblyアプリを起動。AnnotoAppを初期化。

### src/app.rs

AnnotoApp構造体。eframe::Appを実装。

- 画像のロードと表示。
- UIパネル: トップパネル（ファイルオープン、ズーム）、サイドパネル（ツール選択、プロパティ設定）、セントラルパネル（画像と図形の描画）。
- 描画処理: ドラッグで図形を作成、既存図形のレンダリング、テキスト入力。

### src/drawing_tool.rs

DrawingTool enum: StrokeRect, FilledRect, Arrow, Line, Text。

### src/canvas_items/

各図形の構造体とrenderメソッド。

- mod.rs: CanvasItem enum、各図形をバリアントとして持つ。
- arrow.rs: Arrow構造体。start/end座標、色。renderで矢印を描画。
- filled_rect.rs: FilledRect。座標、塗りつぶし色、丸め。
- line.rs: Line。start/end、太さ、色。
- stroke_rect.rs: StrokeRect。座標、太さ、色、丸め。
- text.rs: Text。位置、内容、フォントサイズ、色。

## 機能

- 画像ロード: ファイルダイアログで画像を選択、ロード。
- ズーム: 1-500%。
- ツール選択: サイドパネルでツールを選ぶ。
- プロパティ設定: 線の太さ、色、塗りつぶし色、角の丸め。
- 描画: ドラッグで図形を作成。テキストはクリックで入力ウィンドウ。
- レンダリング: 画像上に図形を描画。スケーリング対応。
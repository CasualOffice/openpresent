chrome-info 156.0.8078.80 ANGLE (Apple, ANGLE Metal Renderer: Apple M4, Unspecified Version) {'canvas': 'Canvas: Hardware accelerated*   Direct Rendering Display Compositor: Disabled*   Composi', 'raster': 'Rasterization: Hardware accelerated*   Raw Draw: Disabled*   Skia Graphite: Enabled*   TreesInV'} idle rAF 16.664999999999964 

chrome-gpu-info 156.0.8078.80 ANGLE (Apple, ANGLE Metal Renderer: Apple M4, Unspecified Version) {'canvas': 'Canvas: Hardware accelerated*   Direct Rendering Display Compositor: Disabled*   Composi', 'raster': 'Rasterization: Hardware accelerated on all pages*   Raw Draw: Disabled*   Skia Graphite: Enable'} idle rAF 16.664999999999964 

shell-info 158.0.8127.18 ANGLE (Google, Vulkan 1.3.0 (SwiftShader Device (LLVM 10.0.0) (0x0000C0DE)), SwiftShader driver) {'error': 'Error: page.goto: net::ERR_FAILED at chrome://gpu\nCall log:\n\x1b[2m  - navigating to "chrome://gpu", waiting until "load"\x1b[22m\n'} idle rAF 16.664999999999964 

linux-info 158.0.8127.18 ANGLE (Google, Vulkan 1.3.0 (SwiftShader Device (LLVM 10.0.0) (0x0000C0DE)), SwiftShader driver) {'error': 'Error: page.goto: net::ERR_FAILED at chrome://gpu\nCall log:\n  - navigating to "chrome://gpu", waiting until "load"\n'} idle rAF 16.66500000000002 

### Sizes (KiB)

| Build | Profile | wasm raw | gzip -9 | Brotli q11 | JS glue raw / Brotli |
| --- | --- | ---: | ---: | ---: | ---: |
| baseline (bindgen only) | release, no wasm-opt | 17 | 8 | 7 | 8.0 / 2.0 |
| baseline (bindgen only) | release, wasm-opt -O | 16 | 7 | 6 | 8.0 / 2.0 |
| baseline (bindgen only) | small (opt-level s, fat LTO), wasm-opt -Oz | 17 | 8 | 7 | 8.2 / 2.0 |
| baseline (bindgen only) | smallest (opt-level z, fat LTO, overflow checks off), wasm-opt -Oz | 17 | 8 | 7 | 8.2 / 2.0 |
| + skrifa (outlines) | release, no wasm-opt | 397 | 130 | 104 | 8.8 / 2.1 |
| + skrifa (outlines) | release, wasm-opt -O | 324 | 121 | 99 | 8.8 / 2.1 |
| + skrifa (outlines) | small (opt-level s, fat LTO), wasm-opt -Oz | 458 | 181 | 144 | 9.0 / 2.1 |
| + skrifa (outlines) | smallest (opt-level z, fat LTO, overflow checks off), wasm-opt -Oz | 419 | 166 | 134 | 9.0 / 2.1 |
| + harfrust (shaping) | release, no wasm-opt | 790 | 269 | 208 | 9.5 / 2.2 |
| + harfrust (shaping) | release, wasm-opt -O | 739 | 276 | 215 | 9.5 / 2.2 |
| + harfrust (shaping) | small (opt-level s, fat LTO), wasm-opt -Oz | 653 | 257 | 204 | 9.7 / 2.2 |
| + harfrust (shaping) | smallest (opt-level z, fat LTO, overflow checks off), wasm-opt -Oz | 563 | 229 | 184 | 9.7 / 2.2 |
| + unicode-bidi, unicode-script | release, no wasm-opt | 863 | 297 | 225 | 9.8 / 2.2 |
| + unicode-bidi, unicode-script | release, wasm-opt -O | 810 | 303 | 233 | 9.8 / 2.2 |
| + unicode-bidi, unicode-script | small (opt-level s, fat LTO), wasm-opt -Oz | 725 | 286 | 222 | 10.0 / 2.2 |
| + unicode-bidi, unicode-script | smallest (opt-level z, fat LTO, overflow checks off), wasm-opt -Oz | 632 | 257 | 202 | 10.0 / 2.2 |
| + unicode-linebreak = full stack | release, no wasm-opt | 1,228 | 414 | 313 | 23.7 / 3.1 |
| + unicode-linebreak = full stack | release, wasm-opt -O | 1,103 | 409 | 315 | 23.7 / 3.1 |
| + unicode-linebreak = full stack | small (opt-level s, fat LTO), wasm-opt -Oz | 1,134 | 449 | 343 | 23.7 / 3.1 |
| + unicode-linebreak = full stack | smallest (opt-level z, fat LTO, overflow checks off), wasm-opt -Oz | 998 | 405 | 313 | 23.7 / 3.1 |
| full stack, icu_segmenter instead (no complex scripts) | release, no wasm-opt | 1,268 | 431 | 326 | 23.7 / 3.1 |
| full stack, icu_segmenter instead (no complex scripts) | release, wasm-opt -O | 1,139 | 427 | 329 | 23.7 / 3.1 |
| full stack, icu_segmenter instead (no complex scripts) | small (opt-level s, fat LTO), wasm-opt -Oz | 1,167 | 465 | 356 | 23.7 / 3.1 |
| full stack, icu_segmenter instead (no complex scripts) | smallest (opt-level z, fat LTO, overflow checks off), wasm-opt -Oz | 1,027 | 418 | 325 | 23.7 / 3.1 |
| full stack, icu_segmenter instead (auto: Thai etc.) | release, no wasm-opt | 1,586 | 721 | 610 | 23.7 / 3.1 |
| full stack, icu_segmenter instead (auto: Thai etc.) | release, wasm-opt -O | 1,456 | 717 | 613 | 23.7 / 3.1 |
| full stack, icu_segmenter instead (auto: Thai etc.) | small (opt-level s, fat LTO), wasm-opt -Oz | 1,482 | 755 | 642 | 23.7 / 3.1 |
| full stack, icu_segmenter instead (auto: Thai etc.) | smallest (opt-level z, fat LTO, overflow checks off), wasm-opt -Oz | 1,342 | 709 | 609 | 23.7 / 3.1 |
| full stack + Carlito Regular embedded | release, no wasm-opt | 1,841 | 677 | 524 | 24.2 / 3.1 |
| full stack + Carlito Regular embedded | release, wasm-opt -O | 1,656 | 672 | 529 | 24.2 / 3.1 |
| full stack + Carlito Regular embedded | small (opt-level s, fat LTO), wasm-opt -Oz | 1,688 | 712 | 557 | 24.2 / 3.1 |
| full stack + Carlito Regular embedded | smallest (opt-level z, fat LTO, overflow checks off), wasm-opt -Oz | 1,552 | 667 | 525 | 24.2 / 3.1 |
| full stack + Carlito and Liberation Sans Regular embedded | release, no wasm-opt | 2,243 | 883 | 692 | 24.7 / 3.1 |
| full stack + Carlito and Liberation Sans Regular embedded | release, wasm-opt -O | 2,057 | 878 | 699 | 24.7 / 3.1 |
| full stack + Carlito and Liberation Sans Regular embedded | small (opt-level s, fat LTO), wasm-opt -Oz | 2,088 | 919 | 726 | 24.7 / 3.1 |
| full stack + Carlito and Liberation Sans Regular embedded | smallest (opt-level z, fat LTO, overflow checks off), wasm-opt -Oz | 1,952 | 873 | 695 | 24.7 / 3.1 |

| Font file | raw | gzip -9 | Brotli q11 |
| --- | ---: | ---: | ---: |
| Carlito-Bold.ttf | 666 | 273 | 215 |
| Carlito-BoldItalic.ttf | 790 | 314 | 238 |
| Carlito-Italic.ttf | 601 | 274 | 222 |
| Carlito-Regular.ttf | 613 | 262 | 207 |
| LiberationSans-Bold.ttf | 405 | 207 | 166 |
| LiberationSans-BoldItalic.ttf | 399 | 209 | 170 |
| LiberationSans-Italic.ttf | 406 | 212 | 172 |
| LiberationSans-Regular.ttf | 401 | 206 | 165 |
| NotoSansArabic.ttf | 825 | 443 | 361 |
| NotoSansSC.otf | 8,136 | 7,034 | 5,865 |

### Throughput, installed Chrome, new headless, GPU: 2,000 characters, 16px, 600px measure, ms

| Case | Bundle | Glyphs | Lines | Bidi + itemize | Shape incl. itemize | Line break opportunities | Full layout median | p95 | Layout + JS copy median | chars/ms | First layout, fresh page, median of 5 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Latin (Carlito) | o3 | 1910 | 23 | 0.135 | 0.299 | 0.009 | 0.455 | 0.524 | 0.380 | 4,396 | 6.19 |
| Latin (Liberation Sans) | o3 | 2000 | 25 | 0.100 | 0.177 | 0.008 | 0.253 | 0.255 | 0.265 | 7,905 | 5.39 |
| Arabic (Noto Sans Arabic) | o3 | 2478 | 22 | 0.166 | 0.673 | 0.008 | 0.695 | 0.769 | 0.755 | 2,876 | 8.80 |
| CJK (Noto Sans SC, CFF) | o3 | 2000 | 54 | 0.129 | 0.225 | 0.012 | 0.327 | 0.359 | 0.340 | 6,107 | 6.71 |
| Mixed bidi + fallback | o3 | 2146 | 27 | 0.165 | 0.428 | 0.008 | 0.562 | 0.564 | 0.535 | 3,562 | 8.36 |
| Latin (Carlito) | small | 1910 | 23 | 0.220 | 0.410 | 0.010 | 0.494 | 0.588 | 0.455 | 4,044 | 6.68 |
| Latin (Liberation Sans) | small | 2000 | 25 | 0.179 | 0.256 | 0.010 | 0.323 | 0.327 | 0.320 | 6,197 | 5.47 |
| Arabic (Noto Sans Arabic) | small | 2478 | 22 | 0.168 | 0.685 | 0.010 | 0.724 | 0.777 | 0.745 | 2,761 | 8.36 |
| CJK (Noto Sans SC, CFF) | small | 2000 | 54 | 0.121 | 0.245 | 0.015 | 0.324 | 0.329 | 0.335 | 6,182 | 6.45 |
| Mixed bidi + fallback | small | 2146 | 27 | 0.208 | 0.531 | 0.011 | 0.575 | 0.576 | 0.600 | 3,481 | 10.87 |

o3: instantiate median 5.8 ms; register_font median carlito 1.77 ms, liberation 0.14 ms, arabic 0.12 ms, cjk 0.81 ms
small: instantiate median 6.0 ms; register_font median carlito 1.41 ms, liberation 0.16 ms, arabic 0.19 ms, cjk 0.88 ms

### Throughput, Playwright headless shell, software: 2,000 characters, 16px, 600px measure, ms

| Case | Bundle | Glyphs | Lines | Bidi + itemize | Shape incl. itemize | Line break opportunities | Full layout median | p95 | Layout + JS copy median | chars/ms | First layout, fresh page, median of 5 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Latin (Carlito) | o3 | 1910 | 23 | 0.085 | 0.214 | 0.006 | 0.274 | 0.338 | 0.275 | 7,306 | 7.75 |
| Latin (Liberation Sans) | o3 | 2000 | 25 | 0.079 | 0.146 | 0.006 | 0.198 | 0.202 | 0.195 | 10,088 | 10.22 |
| Arabic (Noto Sans Arabic) | o3 | 2478 | 22 | 0.130 | 0.556 | 0.007 | 0.554 | 0.601 | 0.710 | 3,608 | 14.74 |
| CJK (Noto Sans SC, CFF) | o3 | 2000 | 54 | 0.114 | 0.206 | 0.011 | 0.289 | 0.307 | 0.285 | 6,926 | 11.61 |
| Mixed bidi + fallback | o3 | 2146 | 27 | 0.132 | 0.358 | 0.007 | 0.451 | 0.458 | 0.460 | 4,430 | 15.70 |
| Latin (Carlito) | small | 1910 | 23 | 0.209 | 0.379 | 0.010 | 0.481 | 0.549 | 0.410 | 4,156 | 11.22 |
| Latin (Liberation Sans) | small | 2000 | 25 | 0.167 | 0.237 | 0.009 | 0.293 | 0.302 | 0.295 | 6,826 | 9.89 |
| Arabic (Noto Sans Arabic) | small | 2478 | 22 | 0.210 | 0.902 | 0.012 | 0.791 | 0.848 | 1.020 | 2,527 | 13.46 |
| CJK (Noto Sans SC, CFF) | small | 2000 | 54 | 0.195 | 0.379 | 0.020 | 0.498 | 0.504 | 0.460 | 4,016 | 8.88 |
| Mixed bidi + fallback | small | 2146 | 27 | 0.319 | 0.613 | 0.012 | 0.893 | 0.997 | 0.650 | 2,241 | 10.24 |

o3: instantiate median 6.8 ms; register_font median carlito 3.11 ms, liberation 0.24 ms, arabic 0.22 ms, cjk 1.30 ms
small: instantiate median 5.7 ms; register_font median carlito 2.11 ms, liberation 0.22 ms, arabic 0.24 ms, cjk 1.20 ms

### Throughput, Node v24.22.1 (same bundle, V8 only), ms

| Case | Bundle | Full layout median | p95 | chars/ms |
| --- | --- | ---: | ---: | ---: |
| Latin (Carlito) | o3 | 0.329 | 0.365 | 6,071 |
| Latin (Liberation Sans) | o3 | 0.240 | 0.242 | 8,329 |
| Arabic (Noto Sans Arabic) | o3 | 0.709 | 0.727 | 2,822 |
| CJK (Noto Sans SC, CFF) | o3 | 0.330 | 0.337 | 6,065 |
| Mixed bidi + fallback | o3 | 0.528 | 0.540 | 3,790 |
| Latin (Carlito) | small | 0.483 | 0.595 | 4,145 |
| Latin (Liberation Sans) | small | 0.379 | 0.381 | 5,283 |
| Arabic (Noto Sans Arabic) | small | 0.873 | 0.901 | 2,290 |
| CJK (Noto Sans SC, CFF) | small | 0.381 | 0.382 | 5,251 |
| Mixed bidi + fallback | small | 0.683 | 0.689 | 2,926 |

### Frame time by readback, installed Chrome, new headless, Metal GPU, ms

Each row is 5 fresh browsers. Cold is that browser's first frame; cached is 40 later frames in each, median of medians. Scene layout is not in the frame.

| Text | DPR | Painter | Glyphs | Distinct | Scene layout | Outline to Path2D | Cold frame median (min to max) | Cached frame median | Cached p95 | Cached submit |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| latin | 1 | Path2D per glyph | 4750 | 50 | 15.5 | 3.0 | 26.1 (23.8 to 28.4) | 8.24 | 8.93 | 0.98 |
| latin | 1 | Path2D per shape | 4750 | 50 | 15.4 | 3.1 | 25.8 (25.6 to 27.6) | 12.20 | 13.24 | 0.13 |
| latin | 1 | fillText per cluster | 4750 | 50 | 13.2 | 0.0 | 24.3 (23.6 to 25.3) | 3.14 | 3.48 | 1.38 |
| latin | 1 | fillText per line | 4750 | 50 | 16.6 | 0.0 | 23.2 (20.0 to 25.3) | 1.43 | 1.66 | 0.47 |
| arabic | 1 | Path2D per glyph | 6244 | 85 | 15.9 | 3.1 | 24.9 (23.5 to 26.9) | 8.71 | 9.14 | 1.07 |
| arabic | 1 | Path2D per shape | 6244 | 85 | 17.1 | 3.3 | 26.0 (25.4 to 29.0) | 11.29 | 12.12 | 0.11 |
| arabic | 1 | fillText per cluster | 6244 | 85 | 18.9 | 0.0 | 30.3 (28.4 to 36.2) | 4.04 | 4.45 | 1.96 |
| arabic | 1 | fillText per line | 6244 | 85 | 18.1 | 0.0 | 27.7 (27.6 to 27.8) | 1.73 | 1.95 | 0.69 |
| cjk | 1 | Path2D per glyph | 5000 | 136 | 13.1 | 4.4 | 28.5 (27.3 to 29.8) | 14.39 | 15.10 | 0.90 |
| cjk | 1 | Path2D per shape | 5000 | 136 | 12.7 | 4.1 | 27.4 (27.2 to 29.5) | 21.38 | 22.56 | 0.11 |
| cjk | 1 | fillText per cluster | 5000 | 136 | 19.0 | 0.0 | 51.2 (45.8 to 61.8) | 4.84 | 5.43 | 2.00 |
| cjk | 1 | fillText per line | 5000 | 136 | 19.5 | 0.0 | 49.0 (45.7 to 53.2) | 2.75 | 3.13 | 0.97 |
| han | 1 | Path2D per glyph | 5000 | 4999 | 16.9 | 105.8 | 141.2 (127.8 to 160.4) | 24.76 | 28.14 | 1.21 |
| han | 1 | Path2D per shape | 5000 | 4999 | 11.8 | 72.5 | 104.6 (102.5 to 105.2) | 28.55 | 30.94 | 0.12 |
| han | 1 | fillText per cluster | 5000 | 4999 | 11.9 | 0.0 | 304.2 (293.9 to 317.0) | 33.96 | 40.03 | 1.57 |
| han | 1 | fillText per line | 5000 | 4999 | 17.3 | 0.0 | 445.7 (299.5 to 692.2) | 50.64 | 75.15 | 1.06 |
| latin | 2 | Path2D per glyph | 4750 | 50 | 18.2 | 3.7 | 37.2 (28.7 to 41.5) | 12.07 | 15.70 | 0.98 |
| latin | 2 | Path2D per shape | 4750 | 50 | 13.4 | 2.6 | 30.5 (27.9 to 41.5) | 17.59 | 19.09 | 0.15 |
| latin | 2 | fillText per cluster | 4750 | 50 | 14.2 | 0.0 | 33.8 (33.0 to 39.3) | 4.72 | 5.41 | 1.64 |
| latin | 2 | fillText per line | 4750 | 50 | 14.4 | 0.0 | 29.3 (27.5 to 34.6) | 2.44 | 3.35 | 0.46 |
| arabic | 2 | Path2D per glyph | 6244 | 85 | 16.7 | 3.3 | 31.4 (30.1 to 40.1) | 13.16 | 14.70 | 1.13 |
| arabic | 2 | Path2D per shape | 6244 | 85 | 15.9 | 3.1 | 29.7 (29.3 to 32.3) | 16.43 | 17.78 | 0.14 |
| arabic | 2 | fillText per cluster | 6244 | 85 | 18.3 | 0.0 | 38.7 (33.3 to 41.3) | 5.20 | 5.91 | 2.02 |
| arabic | 2 | fillText per line | 6244 | 85 | 17.3 | 0.0 | 34.7 (33.0 to 35.8) | 2.74 | 3.13 | 0.70 |
| cjk | 2 | Path2D per glyph | 5000 | 136 | 13.1 | 4.2 | 33.7 (33.1 to 40.9) | 21.80 | 22.89 | 0.91 |
| cjk | 2 | Path2D per shape | 5000 | 136 | 21.0 | 7.0 | 48.3 (47.0 to 55.8) | 64.05 | 72.24 | 0.25 |
| cjk | 2 | fillText per cluster | 5000 | 136 | 17.5 | 0.0 | 66.2 (55.4 to 66.4) | 16.35 | 18.13 | 2.01 |
| cjk | 2 | fillText per line | 5000 | 136 | 17.0 | 0.0 | 53.3 (51.6 to 57.6) | 13.08 | 14.20 | 0.90 |
| han | 2 | Path2D per glyph | 5000 | 4999 | 17.1 | 105.8 | 150.3 (128.2 to 158.5) | 38.12 | 41.80 | 1.35 |
| han | 2 | Path2D per shape | 5000 | 4999 | 15.2 | 97.1 | 139.0 (102.4 to 185.1) | 45.75 | 61.06 | 0.17 |
| han | 2 | fillText per cluster | 5000 | 4999 | 11.4 | 0.0 | 305.7 (302.7 to 436.2) | 86.29 | 90.71 | 1.77 |
| han | 2 | fillText per line | 5000 | 4999 | 12.5 | 0.0 | 348.8 (297.0 to 383.9) | 101.59 | 129.89 | 0.90 |

### Frame time by readback, Playwright headless shell, SwiftShader, ms

Each row is 5 fresh browsers. Cold is that browser's first frame; cached is 40 later frames in each, median of medians. Scene layout is not in the frame.

| Text | DPR | Painter | Glyphs | Distinct | Scene layout | Outline to Path2D | Cold frame median (min to max) | Cached frame median | Cached p95 | Cached submit |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| latin | 1 | Path2D per glyph | 4750 | 50 | 15.7 | 3.1 | 15.2 (13.4 to 15.4) | 8.50 | 9.49 | 1.02 |
| latin | 1 | Path2D per shape | 4750 | 50 | 14.1 | 2.9 | 23.5 (23.1 to 31.1) | 11.72 | 12.86 | 0.12 |
| latin | 1 | fillText per cluster | 4750 | 50 | 18.4 | 0.0 | 14.9 (13.5 to 17.8) | 4.50 | 4.86 | 1.97 |
| latin | 1 | fillText per line | 4750 | 50 | 18.6 | 0.0 | 12.1 (10.1 to 12.4) | 1.58 | 1.89 | 0.51 |
| arabic | 1 | Path2D per glyph | 6244 | 85 | 21.0 | 4.2 | 19.7 (19.1 to 23.4) | 11.32 | 12.65 | 1.34 |
| arabic | 1 | Path2D per shape | 6244 | 85 | 15.6 | 3.1 | 22.7 (21.1 to 26.6) | 10.09 | 10.91 | 0.10 |
| arabic | 1 | fillText per cluster | 6244 | 85 | 18.3 | 0.0 | 14.5 (13.3 to 14.8) | 3.68 | 4.33 | 1.80 |
| arabic | 1 | fillText per line | 6244 | 85 | 18.3 | 0.0 | 15.4 (15.3 to 16.2) | 1.72 | 2.23 | 0.69 |
| cjk | 1 | Path2D per glyph | 5000 | 136 | 14.2 | 4.9 | 25.1 (23.6 to 26.4) | 15.24 | 16.27 | 0.96 |
| cjk | 1 | Path2D per shape | 5000 | 136 | 13.8 | 4.7 | 37.1 (36.9 to 39.1) | 22.82 | 23.98 | 0.12 |
| cjk | 1 | fillText per cluster | 5000 | 136 | 14.6 | 0.0 | 25.7 (25.5 to 25.7) | 3.68 | 4.30 | 1.55 |
| cjk | 1 | fillText per line | 5000 | 136 | 15.7 | 0.0 | 26.4 (24.6 to 28.9) | 2.25 | 2.66 | 0.79 |
| han | 1 | Path2D per glyph | 5000 | 4999 | 12.2 | 79.8 | 104.4 (84.8 to 130.7) | 20.82 | 22.11 | 1.03 |
| han | 1 | Path2D per shape | 5000 | 4999 | 10.8 | 63.9 | 99.0 (98.1 to 109.3) | 26.18 | 27.27 | 0.11 |
| han | 1 | fillText per cluster | 5000 | 4999 | 10.5 | 0.0 | 195.0 (190.2 to 198.1) | 31.36 | 37.25 | 1.47 |
| han | 1 | fillText per line | 5000 | 4999 | 17.8 | 0.0 | 332.2 (224.6 to 397.7) | 55.58 | 80.63 | 1.25 |
| latin | 2 | Path2D per glyph | 4750 | 50 | 14.6 | 2.9 | 18.5 (16.9 to 22.9) | 12.64 | 13.81 | 1.04 |
| latin | 2 | Path2D per shape | 4750 | 50 | 12.6 | 2.5 | 27.9 (26.8 to 31.1) | 17.80 | 18.12 | 0.16 |
| latin | 2 | fillText per cluster | 4750 | 50 | 14.1 | 0.0 | 13.8 (13.3 to 14.9) | 4.73 | 5.22 | 1.60 |
| latin | 2 | fillText per line | 4750 | 50 | 14.3 | 0.0 | 11.7 (11.5 to 11.9) | 2.47 | 2.67 | 0.42 |
| arabic | 2 | Path2D per glyph | 6244 | 85 | 17.6 | 3.5 | 21.0 (20.8 to 21.8) | 13.53 | 14.47 | 1.16 |
| arabic | 2 | Path2D per shape | 6244 | 85 | 17.3 | 3.4 | 33.3 (30.2 to 34.7) | 18.57 | 19.25 | 0.16 |
| arabic | 2 | fillText per cluster | 6244 | 85 | 14.4 | 0.0 | 13.2 (13.0 to 13.3) | 3.87 | 4.32 | 1.46 |
| arabic | 2 | fillText per line | 6244 | 85 | 14.4 | 0.0 | 14.2 (14.1 to 14.4) | 2.23 | 2.51 | 0.56 |
| cjk | 2 | Path2D per glyph | 5000 | 136 | 11.6 | 3.8 | 26.2 (25.9 to 26.4) | 19.30 | 19.87 | 0.81 |
| cjk | 2 | Path2D per shape | 5000 | 136 | 11.7 | 3.9 | 44.1 (44.0 to 44.4) | 32.69 | 33.00 | 0.15 |
| cjk | 2 | fillText per cluster | 5000 | 136 | 11.8 | 0.0 | 26.4 (25.9 to 27.6) | 10.68 | 11.26 | 1.33 |
| cjk | 2 | fillText per line | 5000 | 136 | 11.6 | 0.0 | 25.1 (24.7 to 25.8) | 9.13 | 9.73 | 0.61 |
| han | 2 | Path2D per glyph | 5000 | 4999 | 11.4 | 70.8 | 102.3 (97.3 to 117.5) | 27.90 | 28.91 | 0.96 |
| han | 2 | Path2D per shape | 5000 | 4999 | 12.0 | 77.1 | 137.9 (114.6 to 146.8) | 49.26 | 51.47 | 0.18 |
| han | 2 | fillText per cluster | 5000 | 4999 | 10.5 | 0.0 | 199.8 (198.0 to 204.3) | 81.70 | 85.50 | 1.73 |
| han | 2 | fillText per line | 5000 | 4999 | 12.2 | 0.0 | 248.3 (200.3 to 261.0) | 98.62 | 100.51 | 0.90 |

### Frame time, fresh canvas per sample, installed Chrome, new headless, Metal GPU, ms

3 fresh browsers per row; 25 cached samples each, median of medians. Each sample includes a new canvas and one readback, given as Empty.

| Text | DPR | Painter | Glyphs | Distinct | Empty | Outline to Path2D | Cold frame median (min to max) | Cached median | Cached p95 | Cached minus empty |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| latin | 1 | Path2D per glyph | 4750 | 50 | 1.14 | 2.5 | 16.8 (15.8 to 17.3) | 9.11 | 9.74 | 7.97 |
| latin | 1 | Path2D per shape | 4750 | 50 | 1.27 | 2.5 | 16.5 (15.5 to 18.3) | 5.67 | 6.28 | 4.40 |
| latin | 1 | fillText per cluster | 4750 | 50 | 1.26 | 0.0 | 15.2 (15.1 to 16.4) | 6.94 | 8.75 | 5.67 |
| latin | 1 | fillText per line | 4750 | 50 | 1.27 | 0.0 | 10.9 (10.9 to 11.0) | 3.13 | 3.63 | 1.87 |
| arabic | 1 | Path2D per glyph | 6244 | 85 | 1.25 | 2.9 | 17.3 (17.3 to 22.3) | 10.33 | 11.05 | 9.08 |
| arabic | 1 | Path2D per shape | 6244 | 85 | 0.99 | 2.8 | 16.9 (15.4 to 17.4) | 5.73 | 7.98 | 4.75 |
| arabic | 1 | fillText per cluster | 6244 | 85 | 1.22 | 0.0 | 17.6 (17.6 to 21.3) | 7.32 | 7.78 | 6.10 |
| arabic | 1 | fillText per line | 6244 | 85 | 1.21 | 0.0 | 16.6 (16.6 to 17.1) | 4.29 | 5.09 | 3.08 |
| cjk | 1 | Path2D per glyph | 5000 | 136 | 1.26 | 4.0 | 20.0 (19.7 to 24.2) | 11.41 | 12.69 | 10.14 |
| cjk | 1 | Path2D per shape | 5000 | 136 | 1.21 | 3.8 | 20.8 (20.7 to 21.0) | 7.64 | 8.12 | 6.43 |
| cjk | 1 | fillText per cluster | 5000 | 136 | 1.21 | 0.0 | 27.2 (26.8 to 28.3) | 7.86 | 8.40 | 6.65 |
| cjk | 1 | fillText per line | 5000 | 136 | 1.22 | 0.0 | 23.5 (23.2 to 24.6) | 4.50 | 5.28 | 3.29 |
| han | 1 | Path2D per glyph | 5000 | 4999 | 2.09 | 67.7 | 88.9 (87.5 to 95.2) | 15.00 | 16.15 | 12.91 |
| han | 1 | Path2D per shape | 5000 | 4999 | 1.29 | 69.8 | 93.5 (88.7 to 94.8) | 11.63 | 14.97 | 10.34 |
| han | 1 | fillText per cluster | 5000 | 4999 | 1.20 | 0.0 | 295.1 (287.0 to 298.5) | 33.42 | 36.69 | 32.22 |
| han | 1 | fillText per line | 5000 | 4999 | 1.19 | 0.0 | 295.5 (294.9 to 296.9) | 28.95 | 33.49 | 27.76 |
| latin | 2 | Path2D per glyph | 4750 | 50 | 2.09 | 2.4 | 17.1 (16.0 to 18.1) | 9.50 | 10.07 | 7.41 |
| latin | 2 | Path2D per shape | 4750 | 50 | 2.08 | 2.2 | 15.8 (15.6 to 15.8) | 5.88 | 8.92 | 3.80 |
| latin | 2 | fillText per cluster | 4750 | 50 | 2.16 | 0.0 | 17.4 (17.2 to 20.1) | 7.61 | 9.56 | 5.45 |
| latin | 2 | fillText per line | 4750 | 50 | 2.11 | 0.0 | 13.5 (13.0 to 14.5) | 3.67 | 4.03 | 1.56 |
| arabic | 2 | Path2D per glyph | 6244 | 85 | 1.92 | 3.1 | 16.9 (16.8 to 19.0) | 10.38 | 10.88 | 8.46 |
| arabic | 2 | Path2D per shape | 6244 | 85 | 2.00 | 3.0 | 17.8 (17.7 to 17.9) | 5.93 | 7.17 | 3.93 |
| arabic | 2 | fillText per cluster | 6244 | 85 | 2.12 | 0.0 | 20.2 (19.7 to 20.3) | 7.85 | 8.45 | 5.72 |
| arabic | 2 | fillText per line | 6244 | 85 | 2.14 | 0.0 | 19.6 (18.9 to 23.4) | 4.95 | 7.78 | 2.81 |
| cjk | 2 | Path2D per glyph | 5000 | 136 | 2.08 | 3.7 | 21.2 (20.5 to 23.1) | 12.21 | 13.24 | 10.13 |
| cjk | 2 | Path2D per shape | 5000 | 136 | 2.04 | 3.8 | 21.5 (21.3 to 21.9) | 8.35 | 8.67 | 6.31 |
| cjk | 2 | fillText per cluster | 5000 | 136 | 2.00 | 0.0 | 34.1 (33.0 to 36.1) | 13.33 | 16.95 | 11.32 |
| cjk | 2 | fillText per line | 5000 | 136 | 2.12 | 0.0 | 29.6 (29.2 to 29.9) | 9.33 | 10.61 | 7.20 |
| han | 2 | Path2D per glyph | 5000 | 4999 | 2.05 | 73.9 | 96.6 (96.2 to 96.7) | 16.24 | 17.65 | 14.19 |
| han | 2 | Path2D per shape | 5000 | 4999 | 2.01 | 85.4 | 113.2 (102.4 to 113.7) | 12.22 | 16.83 | 10.21 |
| han | 2 | fillText per cluster | 5000 | 4999 | 2.27 | 0.0 | 448.5 (372.4 to 516.7) | 87.95 | 110.68 | 85.68 |
| han | 2 | fillText per line | 5000 | 4999 | 2.32 | 0.0 | 377.7 (372.0 to 389.2) | 84.93 | 94.73 | 82.60 |

### Frame time, fresh canvas per sample, Playwright headless shell, SwiftShader, ms

3 fresh browsers per row; 25 cached samples each, median of medians. Each sample includes a new canvas and one readback, given as Empty.

| Text | DPR | Painter | Glyphs | Distinct | Empty | Outline to Path2D | Cold frame median (min to max) | Cached median | Cached p95 | Cached minus empty |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| latin | 1 | Path2D per glyph | 4750 | 50 | 0.48 | 2.9 | 13.3 (13.3 to 16.8) | 8.78 | 9.41 | 8.30 |
| latin | 1 | Path2D per shape | 4750 | 50 | 0.50 | 3.0 | 25.0 (24.6 to 25.1) | 13.41 | 14.11 | 12.91 |
| latin | 1 | fillText per cluster | 4750 | 50 | 0.50 | 0.0 | 11.4 (11.4 to 13.4) | 4.13 | 4.67 | 3.62 |
| latin | 1 | fillText per line | 4750 | 50 | 0.48 | 0.0 | 9.2 (9.1 to 9.3) | 2.11 | 2.68 | 1.63 |
| arabic | 1 | Path2D per glyph | 6244 | 85 | 0.48 | 3.6 | 17.0 (16.5 to 17.7) | 11.64 | 12.64 | 11.16 |
| arabic | 1 | Path2D per shape | 6244 | 85 | 0.62 | 5.2 | 38.7 (34.4 to 44.5) | 19.89 | 22.45 | 19.27 |
| arabic | 1 | fillText per cluster | 6244 | 85 | 0.58 | 0.0 | 18.0 (17.8 to 19.4) | 6.24 | 6.86 | 5.66 |
| arabic | 1 | fillText per line | 6244 | 85 | 0.57 | 0.0 | 19.9 (19.2 to 19.9) | 4.54 | 5.11 | 3.97 |
| cjk | 1 | Path2D per glyph | 5000 | 136 | 0.65 | 6.9 | 35.1 (31.3 to 36.6) | 27.20 | 28.15 | 26.55 |
| cjk | 1 | Path2D per shape | 5000 | 136 | 0.62 | 6.8 | 56.9 (51.0 to 57.6) | 35.22 | 44.60 | 34.60 |
| cjk | 1 | fillText per cluster | 5000 | 136 | 0.65 | 0.0 | 36.3 (35.9 to 36.6) | 7.49 | 8.38 | 6.84 |
| cjk | 1 | fillText per line | 5000 | 136 | 0.64 | 0.0 | 34.0 (31.0 to 34.3) | 4.80 | 5.89 | 4.16 |
| han | 1 | Path2D per glyph | 5000 | 4999 | 0.73 | 125.3 | 166.9 (151.0 to 182.6) | 37.03 | 38.46 | 36.30 |
| han | 1 | Path2D per shape | 5000 | 4999 | 0.65 | 118.7 | 187.2 (173.2 to 194.9) | 51.17 | 54.54 | 50.52 |
| han | 1 | fillText per cluster | 5000 | 4999 | 0.64 | 0.0 | 379.1 (365.2 to 379.6) | 234.46 | 254.89 | 233.82 |
| han | 1 | fillText per line | 5000 | 4999 | 0.63 | 0.0 | 357.2 (336.0 to 358.1) | 209.21 | 236.73 | 208.57 |
| latin | 2 | Path2D per glyph | 4750 | 50 | 2.21 | 3.6 | 23.6 (21.4 to 24.9) | 17.04 | 19.07 | 14.83 |
| latin | 2 | Path2D per shape | 4750 | 50 | 2.38 | 4.1 | 46.5 (41.6 to 47.1) | 30.54 | 32.52 | 28.16 |
| latin | 2 | fillText per cluster | 4750 | 50 | 2.35 | 0.0 | 18.9 (18.9 to 20.2) | 8.75 | 9.45 | 6.39 |
| latin | 2 | fillText per line | 4750 | 50 | 2.06 | 0.0 | 14.3 (13.4 to 15.5) | 5.80 | 6.35 | 3.75 |
| arabic | 2 | Path2D per glyph | 6244 | 85 | 2.35 | 4.7 | 29.6 (29.5 to 31.2) | 23.13 | 24.40 | 20.78 |
| arabic | 2 | Path2D per shape | 6244 | 85 | 1.95 | 4.6 | 44.4 (37.2 to 48.2) | 28.61 | 31.83 | 26.66 |
| arabic | 2 | fillText per cluster | 6244 | 85 | 2.27 | 0.0 | 23.0 (22.2 to 23.1) | 9.58 | 10.35 | 7.31 |
| arabic | 2 | fillText per line | 6244 | 85 | 2.32 | 0.0 | 24.1 (22.2 to 24.3) | 7.59 | 8.33 | 5.27 |
| cjk | 2 | Path2D per glyph | 5000 | 136 | 2.23 | 6.1 | 42.9 (34.2 to 44.4) | 29.32 | 38.14 | 27.09 |
| cjk | 2 | Path2D per shape | 5000 | 136 | 2.30 | 6.1 | 76.3 (75.1 to 76.9) | 57.90 | 59.98 | 55.60 |
| cjk | 2 | fillText per cluster | 5000 | 136 | 1.98 | 0.0 | 39.7 (36.7 to 43.7) | 20.31 | 22.52 | 18.33 |
| cjk | 2 | fillText per line | 5000 | 136 | 2.04 | 0.0 | 33.8 (33.3 to 37.5) | 18.26 | 19.43 | 16.22 |
| han | 2 | Path2D per glyph | 5000 | 4999 | 1.94 | 93.8 | 138.1 (136.3 to 156.0) | 40.81 | 46.99 | 38.87 |
| han | 2 | Path2D per shape | 5000 | 4999 | 2.15 | 104.4 | 192.3 (175.1 to 251.5) | 78.44 | 88.66 | 76.29 |
| han | 2 | fillText per cluster | 5000 | 4999 | 2.13 | 0.0 | 350.7 (239.6 to 387.4) | 274.54 | 322.06 | 272.40 |
| han | 2 | fillText per line | 5000 | 4999 | 1.65 | 0.0 | 240.7 (236.5 to 328.4) | 188.65 | 198.49 | 187.00 |

### Frame time by rAF interval, installed Chrome, new headless, Metal GPU, ms (no readback, uncapped flags)

| Text | DPR | Painter | rAF interval median | p95 | JS submit median | Cold: submit | Cold: until 2 rAFs later |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: |
| latin | 1 | Path2D per glyph | 6.00 | 7.61 | 1.42 | 6.9 | 25.8 |
| latin | 1 | Path2D per shape | 2.59 | 3.88 | 0.19 | 13.6 | 14.6 |
| latin | 1 | fillText per cluster | 5.18 | 5.75 | 2.31 | 4.4 | 17.0 |
| latin | 1 | fillText per line | 1.29 | 1.44 | 0.56 | 3.1 | 13.2 |
| cjk | 1 | Path2D per glyph | 7.65 | 12.59 | 1.57 | 9.5 | 11.5 |
| cjk | 1 | Path2D per shape | 3.39 | 6.52 | 0.20 | 14.7 | 16.6 |
| cjk | 1 | fillText per cluster | 5.38 | 5.77 | 2.29 | 18.0 | 35.4 |
| cjk | 1 | fillText per line | 2.02 | 2.27 | 0.98 | 17.1 | 32.4 |
| latin | 2 | Path2D per glyph | 5.82 | 7.45 | 1.40 | 6.9 | 25.9 |
| latin | 2 | Path2D per shape | 2.57 | 4.38 | 0.21 | 13.6 | 14.5 |
| latin | 2 | fillText per cluster | 5.12 | 5.67 | 2.23 | 4.6 | 18.7 |
| latin | 2 | fillText per line | 1.40 | 1.62 | 0.65 | 3.4 | 15.2 |
| cjk | 2 | Path2D per glyph | 7.21 | 9.51 | 1.43 | 8.1 | 27.1 |
| cjk | 2 | Path2D per shape | 4.33 | 7.47 | 0.24 | 17.2 | 18.9 |
| cjk | 2 | fillText per cluster | 6.22 | 7.23 | 2.59 | 18.4 | 42.0 |
| cjk | 2 | fillText per line | 2.80 | 8.58 | 1.15 | 17.2 | 38.0 |

### Frame time by rAF interval, Playwright headless shell, SwiftShader, ms (no readback, uncapped flags)

| Text | DPR | Painter | rAF interval median | p95 | JS submit median | Cold: submit | Cold: until 2 rAFs later |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: |
| latin | 1 | Path2D per glyph | 6.71 | 7.40 | 0.76 | 4.7 | 13.2 |
| latin | 1 | Path2D per shape | 9.65 | 10.29 | 0.12 | 8.6 | 20.2 |
| latin | 1 | fillText per cluster | 3.27 | 3.45 | 1.31 | 3.0 | 11.4 |
| latin | 1 | fillText per line | 1.32 | 1.49 | 0.39 | 2.1 | 9.2 |
| cjk | 1 | Path2D per glyph | 14.66 | 15.70 | 0.92 | 6.3 | 21.0 |
| cjk | 1 | Path2D per shape | 22.15 | 23.79 | 0.15 | 11.0 | 32.7 |
| cjk | 1 | fillText per cluster | 3.14 | 3.32 | 1.25 | 11.6 | 22.1 |
| cjk | 1 | fillText per line | 1.62 | 1.78 | 0.56 | 10.4 | 21.0 |
| latin | 2 | Path2D per glyph | 11.03 | 12.32 | 0.84 | 5.1 | 18.1 |
| latin | 2 | Path2D per shape | 20.95 | 25.61 | 0.19 | 9.7 | 29.2 |
| latin | 2 | fillText per cluster | 4.63 | 4.89 | 1.38 | 3.8 | 15.9 |
| latin | 2 | fillText per line | 2.83 | 3.06 | 0.45 | 3.1 | 14.3 |
| cjk | 2 | Path2D per glyph | 24.80 | 26.18 | 1.01 | 6.9 | 30.2 |
| cjk | 2 | Path2D per shape | 39.22 | 54.53 | 0.18 | 11.8 | 49.2 |
| cjk | 2 | fillText per cluster | 12.78 | 13.73 | 1.62 | 12.7 | 31.1 |
| cjk | 2 | fillText per line | 11.62 | 12.41 | 0.87 | 12.1 | 30.8 |

### Appearance metrics, chrome

| Panel | Font | Size | Device px | Ink fillText / Path2D | Mean abs lum diff (0-255) | Peak contrast Path2D, fillText | LCD pixels Path2D, fillText (opaque canvas) |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| light | Carlito | 10pt 1x | 13.3 | 1.16 | 7.8 | 1.00, 1.00 | 0, 0 |
| light | Carlito | 12pt 1x | 16.0 | 1.15 | 5.9 | 1.00, 1.00 | 0, 0 |
| light | Carlito | 18pt 1x | 24.0 | 1.16 | 5.8 | 1.00, 1.00 | 0, 0 |
| light | Carlito | 10pt 2x | 26.7 | 1.16 | 5.9 | 1.00, 1.00 | 0, 0 |
| light | Carlito | 12pt 2x | 32.0 | 1.16 | 6.0 | 1.00, 1.00 | 0, 0 |
| light | Carlito | 18pt 2x | 48.0 | 1.17 | 6.0 | 1.00, 1.00 | 0, 0 |
| light | Carlito | 10pt 0.5x | 6.7 | 1.11 | 5.8 | 0.85, 0.91 | 0, 0 |
| light | Liberation Sans | 10pt 1x | 13.3 | 1.14 | 6.4 | 1.00, 1.00 | 0, 0 |
| light | Liberation Sans | 12pt 1x | 16.0 | 1.16 | 7.3 | 1.00, 1.00 | 0, 0 |
| light | Liberation Sans | 18pt 1x | 24.0 | 1.16 | 6.3 | 1.00, 1.00 | 0, 0 |
| light | Liberation Sans | 10pt 2x | 26.7 | 1.16 | 6.5 | 1.00, 1.00 | 0, 0 |
| light | Liberation Sans | 12pt 2x | 32.0 | 1.16 | 6.3 | 1.00, 1.00 | 0, 0 |
| light | Liberation Sans | 18pt 2x | 48.0 | 1.17 | 6.4 | 1.00, 1.00 | 0, 0 |
| light | Liberation Sans | 10pt 0.5x | 6.7 | 1.11 | 6.5 | 0.80, 0.91 | 0, 0 |
| dark | Carlito | 10pt 1x | 13.3 | 1.45 | 14.3 | 0.93, 0.93 | 0, 0 |
| dark | Carlito | 12pt 1x | 16.0 | 1.41 | 12.9 | 0.93, 0.93 | 0, 0 |
| dark | Carlito | 18pt 1x | 24.0 | 1.33 | 10.7 | 0.93, 0.93 | 0, 0 |
| dark | Carlito | 10pt 2x | 26.7 | 1.32 | 10.1 | 0.93, 0.93 | 0, 0 |
| dark | Carlito | 12pt 2x | 32.0 | 1.29 | 9.5 | 0.93, 0.93 | 0, 0 |
| dark | Carlito | 18pt 2x | 48.0 | 1.26 | 8.4 | 0.93, 0.93 | 0, 0 |
| dark | Carlito | 10pt 0.5x | 6.7 | 1.68 | 19.9 | 0.79, 0.93 | 0, 0 |
| dark | Liberation Sans | 10pt 1x | 13.3 | 1.42 | 14.5 | 0.93, 0.93 | 0, 0 |
| dark | Liberation Sans | 12pt 1x | 16.0 | 1.41 | 13.8 | 0.93, 0.93 | 0, 0 |
| dark | Liberation Sans | 18pt 1x | 24.0 | 1.32 | 11.0 | 0.93, 0.93 | 0, 0 |
| dark | Liberation Sans | 10pt 2x | 26.7 | 1.32 | 11.0 | 0.93, 0.93 | 0, 0 |
| dark | Liberation Sans | 12pt 2x | 32.0 | 1.28 | 10.0 | 0.93, 0.93 | 0, 0 |
| dark | Liberation Sans | 18pt 2x | 48.0 | 1.25 | 8.7 | 0.93, 0.93 | 0, 0 |
| dark | Liberation Sans | 10pt 0.5x | 6.7 | 1.68 | 20.9 | 0.74, 0.93 | 0, 0 |

Probe: centroid shift of one glyph against the requested sub-pixel offset

- Carlito path x: 0.00, 0.14, 0.25, 0.36, 0.47, 0.58, 0.69, 0.80
- Carlito path y: 0.00, 0.24, 0.49, 0.75
- Carlito fillText x: 0.00, 0.27, 0.27, 0.48, 0.48, 0.69, 0.69, 1.00
- Carlito fillText y: 0.00, 0.00, 1.00, 1.00
- Carlito fillText geometricPrecision x: 0.00, 0.26, 0.26, 0.47, 0.47, 0.69, 0.69, 1.00
- Carlito fillText geometricPrecision y: 0.00, 0.00, 1.00, 1.00
- Liberation Sans path x: 0.00, 0.19, 0.30, 0.40, 0.51, 0.62, 0.72, 0.83
- Liberation Sans path y: 0.00, 0.25, 0.53, 0.77
- Liberation Sans fillText x: 0.00, 0.30, 0.30, 0.51, 0.51, 0.72, 0.72, 1.00
- Liberation Sans fillText y: 0.00, 0.00, 1.00, 1.00
- Liberation Sans fillText geometricPrecision x: 0.00, 0.30, 0.30, 0.52, 0.52, 0.73, 0.73, 1.00
- Liberation Sans fillText geometricPrecision y: 0.00, 0.00, 1.00, 1.00

### Engine against fillText, chrome

| Case | Clusters | Whole-text width diff, browser - engine, px, by setting | First divergence | Clusters adv > 0.01 px | Mean, max adv diff | Words | Words adv > 0.05 px | Max word diff |
| --- | ---: | --- | --- | ---: | --- | ---: | ---: | ---: |
| Latin (Carlito) | 1910 | k=auto r=auto: 0.000; k=normal r=auto: 0.000; k=auto r=optimizeLegibility: 0.000; k=auto r=geometricPrecision: 0.000 | none | 368 | 0.053, 1.422 | 288 | 0 | 0.000 |
| Latin (Liberation Sans) | 2000 | k=auto r=auto: 10.211; k=normal r=auto: 0.000; k=auto r=optimizeLegibility: 0.000; k=auto r=geometricPrecision: 0.000 | char 83 `e of Typ` | 85 | 0.034, 1.773 | 288 | 3 | 0.883 |
| Arabic (Noto Sans Arabic) | 1934 | k=auto r=auto: -0.002; k=normal r=auto: -0.029; k=auto r=optimizeLegibility: -0.029; k=auto r=geometricPrecision: -0.029 | none | 1253 | 2.510, 11.584 | 332 | 0 | 0.000 |
| CJK (Noto Sans SC) | 2000 | k=auto r=auto: -215.633; k=normal r=auto: -215.633; k=auto r=optimizeLegibility: -215.633; k=auto r=geometricPrecision: -215.633 | char 17 `安全今年，，こん` | 85 | 0.092, 2.160 | 1 | 1 | 215.612 |

| Case | Wrapping | Distinct clusters | Wrong form | Occurrences | Wrong form | Examples |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| Latin (Carlito) | plain | 48 | 0 | 1623 | 0 |  |
| Latin (Liberation Sans) | plain | 42 | 0 | 1713 | 0 |  |
| Arabic (Noto Sans Arabic) | plain | 81 | 60 | 1602 | 1249 | ي ن ا ي ب ل |
| Arabic (Noto Sans Arabic) | zwj_both | 81 | 58 | 1602 | 1151 | ن ي ب ل ا م |
| Arabic (Noto Sans Arabic) | best_of_4_zwj | 81 | 44 | 1602 | 860 | ن ي ب ل م ي |
| CJK (Noto Sans SC) | plain | 140 | 5 | 2000 | 85 | 한 국 어 세 계 |

### Appearance metrics, shell

| Panel | Font | Size | Device px | Ink fillText / Path2D | Mean abs lum diff (0-255) | Peak contrast Path2D, fillText | LCD pixels Path2D, fillText (opaque canvas) |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| light | Carlito | 10pt 1x | 13.3 | 1.14 | 5.9 | 1.00, 1.00 | 0, 0 |
| light | Carlito | 12pt 1x | 16.0 | 1.15 | 5.9 | 1.00, 1.00 | 0, 0 |
| light | Carlito | 18pt 1x | 24.0 | 1.16 | 5.8 | 1.00, 1.00 | 0, 0 |
| light | Carlito | 10pt 2x | 26.7 | 1.16 | 5.9 | 1.00, 1.00 | 0, 0 |
| light | Carlito | 12pt 2x | 32.0 | 1.16 | 6.0 | 1.00, 1.00 | 0, 0 |
| light | Carlito | 18pt 2x | 48.0 | 1.17 | 6.0 | 1.00, 1.00 | 0, 0 |
| light | Carlito | 10pt 0.5x | 6.7 | 1.11 | 5.8 | 0.85, 0.91 | 0, 0 |
| light | Liberation Sans | 10pt 1x | 13.3 | 1.14 | 6.4 | 1.00, 1.00 | 0, 0 |
| light | Liberation Sans | 12pt 1x | 16.0 | 1.16 | 7.3 | 1.00, 1.00 | 0, 0 |
| light | Liberation Sans | 18pt 1x | 24.0 | 1.16 | 6.3 | 1.00, 1.00 | 0, 0 |
| light | Liberation Sans | 10pt 2x | 26.7 | 1.16 | 6.5 | 1.00, 1.00 | 0, 0 |
| light | Liberation Sans | 12pt 2x | 32.0 | 1.16 | 6.3 | 1.00, 1.00 | 0, 0 |
| light | Liberation Sans | 18pt 2x | 48.0 | 1.17 | 6.4 | 1.00, 1.00 | 0, 0 |
| light | Liberation Sans | 10pt 0.5x | 6.7 | 1.11 | 6.5 | 0.80, 0.91 | 0, 0 |
| dark | Carlito | 10pt 1x | 13.3 | 1.45 | 14.3 | 0.93, 0.93 | 0, 0 |
| dark | Carlito | 12pt 1x | 16.0 | 1.41 | 12.9 | 0.93, 0.93 | 0, 0 |
| dark | Carlito | 18pt 1x | 24.0 | 1.33 | 10.7 | 0.93, 0.93 | 0, 0 |
| dark | Carlito | 10pt 2x | 26.7 | 1.32 | 10.1 | 0.93, 0.93 | 0, 0 |
| dark | Carlito | 12pt 2x | 32.0 | 1.29 | 9.5 | 0.93, 0.93 | 0, 0 |
| dark | Carlito | 18pt 2x | 48.0 | 1.26 | 8.4 | 0.93, 0.93 | 0, 0 |
| dark | Carlito | 10pt 0.5x | 6.7 | 1.68 | 19.9 | 0.79, 0.93 | 0, 0 |
| dark | Liberation Sans | 10pt 1x | 13.3 | 1.42 | 14.5 | 0.93, 0.93 | 0, 0 |
| dark | Liberation Sans | 12pt 1x | 16.0 | 1.41 | 13.8 | 0.93, 0.93 | 0, 0 |
| dark | Liberation Sans | 18pt 1x | 24.0 | 1.32 | 11.0 | 0.93, 0.93 | 0, 0 |
| dark | Liberation Sans | 10pt 2x | 26.7 | 1.32 | 11.0 | 0.93, 0.93 | 0, 0 |
| dark | Liberation Sans | 12pt 2x | 32.0 | 1.28 | 10.0 | 0.93, 0.93 | 0, 0 |
| dark | Liberation Sans | 18pt 2x | 48.0 | 1.25 | 8.7 | 0.93, 0.93 | 0, 0 |
| dark | Liberation Sans | 10pt 0.5x | 6.7 | 1.68 | 20.9 | 0.74, 0.93 | 0, 0 |

Probe: centroid shift of one glyph against the requested sub-pixel offset

- Carlito path x: 0.00, 0.14, 0.25, 0.36, 0.47, 0.58, 0.69, 0.80
- Carlito path y: 0.00, 0.24, 0.49, 0.75
- Carlito fillText x: 0.00, 0.27, 0.27, 0.48, 0.48, 0.69, 0.69, 1.00
- Carlito fillText y: 0.00, 0.00, 1.00, 1.00
- Carlito fillText geometricPrecision x: 0.00, 0.26, 0.26, 0.47, 0.47, 0.69, 0.69, 1.00
- Carlito fillText geometricPrecision y: 0.00, 0.00, 1.00, 1.00
- Liberation Sans path x: 0.00, 0.19, 0.30, 0.40, 0.51, 0.62, 0.72, 0.83
- Liberation Sans path y: 0.00, 0.25, 0.53, 0.77
- Liberation Sans fillText x: 0.00, 0.30, 0.30, 0.51, 0.51, 0.72, 0.72, 1.00
- Liberation Sans fillText y: 0.00, 0.00, 1.00, 1.00
- Liberation Sans fillText geometricPrecision x: 0.00, 0.30, 0.30, 0.52, 0.52, 0.73, 0.73, 1.00
- Liberation Sans fillText geometricPrecision y: 0.00, 0.00, 1.00, 1.00

### Engine against fillText, shell

| Case | Clusters | Whole-text width diff, browser - engine, px, by setting | First divergence | Clusters adv > 0.01 px | Mean, max adv diff | Words | Words adv > 0.05 px | Max word diff |
| --- | ---: | --- | --- | ---: | --- | ---: | ---: | ---: |
| Latin (Carlito) | 1910 | k=auto r=auto: 0.000; k=normal r=auto: 0.000; k=auto r=optimizeLegibility: 0.000; k=auto r=geometricPrecision: 0.000 | none | 368 | 0.053, 1.422 | 288 | 0 | 0.000 |
| Latin (Liberation Sans) | 2000 | k=auto r=auto: 10.211; k=normal r=auto: 0.000; k=auto r=optimizeLegibility: 0.000; k=auto r=geometricPrecision: 0.000 | char 83 `e of Typ` | 85 | 0.034, 1.773 | 288 | 3 | 0.883 |
| Arabic (Noto Sans Arabic) | 1934 | k=auto r=auto: -0.002; k=normal r=auto: -0.029; k=auto r=optimizeLegibility: -0.029; k=auto r=geometricPrecision: -0.029 | none | 1253 | 2.510, 11.584 | 332 | 0 | 0.000 |
| CJK (Noto Sans SC) | 2000 | k=auto r=auto: -215.633; k=normal r=auto: -215.633; k=auto r=optimizeLegibility: -215.633; k=auto r=geometricPrecision: -215.633 | char 17 `安全今年，，こん` | 85 | 0.092, 2.160 | 1 | 1 | 215.612 |

| Case | Wrapping | Distinct clusters | Wrong form | Occurrences | Wrong form | Examples |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| Latin (Carlito) | plain | 48 | 0 | 1623 | 0 |  |
| Latin (Liberation Sans) | plain | 42 | 0 | 1713 | 0 |  |
| Arabic (Noto Sans Arabic) | plain | 81 | 60 | 1602 | 1249 | ي ن ا ي ب ل |
| Arabic (Noto Sans Arabic) | zwj_both | 81 | 58 | 1602 | 1151 | ن ي ب ل ا م |
| Arabic (Noto Sans Arabic) | best_of_4_zwj | 81 | 44 | 1602 | 860 | ن ي ب ل م ي |
| CJK (Noto Sans SC) | plain | 140 | 5 | 2000 | 85 | 한 국 어 세 계 |

### Appearance metrics, linux

| Panel | Font | Size | Device px | Ink fillText / Path2D | Mean abs lum diff (0-255) | Peak contrast Path2D, fillText | LCD pixels Path2D, fillText (opaque canvas) |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| light | Carlito | 10pt 1x | 13.3 | 0.94 | 13.9 | 1.00, 1.00 | 0, 1814 |
| light | Carlito | 12pt 1x | 16.0 | 1.08 | 14.8 | 1.00, 1.00 | 0, 2749 |
| light | Carlito | 18pt 1x | 24.0 | 1.07 | 10.7 | 1.00, 1.00 | 0, 4546 |
| light | Carlito | 10pt 2x | 26.7 | 1.05 | 9.6 | 1.00, 1.00 | 0, 4986 |
| light | Carlito | 12pt 2x | 32.0 | 1.06 | 9.5 | 1.00, 1.00 | 0, 6002 |
| light | Carlito | 18pt 2x | 48.0 | 1.01 | 5.8 | 1.00, 1.00 | 0, 8794 |
| light | Carlito | 10pt 0.5x | 6.7 | 1.28 | 24.1 | 0.85, 0.84 | 0, 746 |
| light | Liberation Sans | 10pt 1x | 13.3 | 1.00 | 16.1 | 1.00, 1.00 | 0, 2234 |
| light | Liberation Sans | 12pt 1x | 16.0 | 1.07 | 16.3 | 1.00, 1.00 | 0, 3154 |
| light | Liberation Sans | 18pt 1x | 24.0 | 1.04 | 9.5 | 1.00, 1.00 | 0, 4959 |
| light | Liberation Sans | 10pt 2x | 26.7 | 1.08 | 13.3 | 1.00, 1.00 | 0, 5782 |
| light | Liberation Sans | 12pt 2x | 32.0 | 1.02 | 7.0 | 1.00, 1.00 | 0, 6749 |
| light | Liberation Sans | 18pt 2x | 48.0 | 1.02 | 5.6 | 1.00, 1.00 | 0, 9759 |
| light | Liberation Sans | 10pt 0.5x | 6.7 | 1.17 | 19.0 | 0.80, 0.78 | 0, 724 |
| dark | Carlito | 10pt 1x | 13.3 | 1.05 | 13.3 | 0.93, 0.93 | 0, 1883 |
| dark | Carlito | 12pt 1x | 16.0 | 1.18 | 14.4 | 0.93, 0.93 | 0, 2820 |
| dark | Carlito | 18pt 1x | 24.0 | 1.14 | 10.4 | 0.93, 0.93 | 0, 4516 |
| dark | Carlito | 10pt 2x | 26.7 | 1.11 | 9.2 | 0.93, 0.93 | 0, 4970 |
| dark | Carlito | 12pt 2x | 32.0 | 1.11 | 9.1 | 0.93, 0.93 | 0, 6025 |
| dark | Carlito | 18pt 2x | 48.0 | 1.04 | 5.5 | 0.93, 0.93 | 0, 8976 |
| dark | Carlito | 10pt 0.5x | 6.7 | 1.51 | 24.9 | 0.79, 0.82 | 0, 749 |
| dark | Liberation Sans | 10pt 1x | 13.3 | 1.11 | 15.5 | 0.93, 0.93 | 0, 2294 |
| dark | Liberation Sans | 12pt 1x | 16.0 | 1.17 | 15.9 | 0.93, 0.93 | 0, 3307 |
| dark | Liberation Sans | 18pt 1x | 24.0 | 1.10 | 9.2 | 0.93, 0.93 | 0, 4868 |
| dark | Liberation Sans | 10pt 2x | 26.7 | 1.14 | 12.8 | 0.93, 0.93 | 0, 5841 |
| dark | Liberation Sans | 12pt 2x | 32.0 | 1.06 | 6.8 | 0.93, 0.93 | 0, 6691 |
| dark | Liberation Sans | 18pt 2x | 48.0 | 1.05 | 5.5 | 0.93, 0.93 | 0, 10224 |
| dark | Liberation Sans | 10pt 0.5x | 6.7 | 1.38 | 19.8 | 0.74, 0.78 | 0, 730 |

Probe: centroid shift of one glyph against the requested sub-pixel offset

- Carlito path x: 0.00, 0.14, 0.25, 0.36, 0.47, 0.58, 0.69, 0.80
- Carlito path y: 0.00, 0.24, 0.49, 0.75
- Carlito fillText x: 0.00, 0.00, 0.00, 0.00, 1.00, 1.00, 1.00, 1.00
- Carlito fillText y: 0.00, 0.00, 1.00, 1.00
- Carlito fillText geometricPrecision x: 0.00, 0.26, 0.26, 0.47, 0.47, 0.69, 0.69, 1.00
- Carlito fillText geometricPrecision y: 0.00, 0.00, 1.00, 1.00
- Liberation Sans path x: 0.00, 0.19, 0.30, 0.40, 0.51, 0.62, 0.72, 0.83
- Liberation Sans path y: 0.00, 0.25, 0.53, 0.77
- Liberation Sans fillText x: 0.00, 0.00, 0.00, 0.00, 1.00, 1.00, 1.00, 1.00
- Liberation Sans fillText y: 0.00, 0.00, 1.00, 1.00
- Liberation Sans fillText geometricPrecision x: 0.00, 0.31, 0.31, 0.52, 0.52, 0.73, 0.73, 1.00
- Liberation Sans fillText geometricPrecision y: 0.00, 0.00, 1.00, 1.00

### Engine against fillText, linux

| Case | Clusters | Whole-text width diff, browser - engine, px, by setting | First divergence | Clusters adv > 0.01 px | Mean, max adv diff | Words | Words adv > 0.05 px | Max word diff |
| --- | ---: | --- | --- | ---: | --- | ---: | ---: | ---: |
| Latin (Carlito) | 1910 | k=auto r=auto: 60.445; k=normal r=auto: 60.445; k=auto r=optimizeLegibility: 60.445; k=auto r=geometricPrecision: 0.000 | char 1 `mar` | 1910 | 0.335, 1.625 | 288 | 284 | 1.859 |
| Latin (Liberation Sans) | 2000 | k=auto r=auto: -86.992; k=normal r=auto: -92.992; k=auto r=optimizeLegibility: -92.992; k=auto r=geometricPrecision: 0.000 | char 1 `mar` | 1801 | 0.270, 2.000 | 288 | 263 | 2.734 |
| Arabic (Noto Sans Arabic) | 1934 | k=auto r=auto: -11.229; k=normal r=auto: -11.229; k=auto r=optimizeLegibility: -11.229; k=auto r=geometricPrecision: 0.013 | char 1 `الأ` | 1934 | 2.534, 11.696 | 332 | 332 | 1.400 |
| CJK (Noto Sans SC) | 2000 | k=auto r=auto: -23.236; k=normal r=auto: -23.236; k=auto r=optimizeLegibility: -23.236; k=auto r=geometricPrecision: -31.971 | char 17 `安全今年，，こん` | 66 | 0.004, 0.264 | 1 | 1 | 23.216 |

| Case | Wrapping | Distinct clusters | Wrong form | Occurrences | Wrong form | Examples |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| Latin (Carlito) | plain | 48 | 0 | 1623 | 0 |  |
| Latin (Liberation Sans) | plain | 42 | 0 | 1713 | 0 |  |
| Arabic (Noto Sans Arabic) | plain | 81 | 60 | 1602 | 1249 | ي ن ا ي ب ل |
| Arabic (Noto Sans Arabic) | zwj_both | 81 | 58 | 1602 | 1151 | ن ي ب ل ا م |
| Arabic (Noto Sans Arabic) | best_of_4_zwj | 81 | 44 | 1602 | 860 | ن ي ب ل م ي |
| CJK (Noto Sans SC) | plain | 140 | 5 | 2000 | 85 | 한 국 어 세 계 |


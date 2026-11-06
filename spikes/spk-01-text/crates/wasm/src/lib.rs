//! wasm-bindgen surface of the spike. Each feature adds one export that really uses
//! the stage, so the linker cannot drop what the size ladder is trying to price.
#![forbid(unsafe_code)]
use wasm_bindgen::prelude::*;

/// Keeps the baseline honest: bindgen string and byte marshalling are present.
#[wasm_bindgen]
pub fn baseline_echo(text: &str, bytes: Vec<u8>) -> String {
    format!("{}:{}", text.len(), bytes.len())
}

#[cfg(feature = "embed-carlito")]
#[wasm_bindgen]
pub fn embedded_carlito() -> Vec<u8> {
    include_bytes!("../../../fonts/Carlito-Regular.ttf").to_vec()
}
#[cfg(feature = "embed-liberation")]
#[wasm_bindgen]
pub fn embedded_liberation() -> Vec<u8> {
    include_bytes!("../../../fonts/LiberationSans-Regular.ttf").to_vec()
}

#[cfg(feature = "outline")]
#[wasm_bindgen]
pub fn ladder_outline(font: Vec<u8>, gid: u32) -> String {
    spk_text_core::Face::parse(font).map_or_else(String::new, |f| f.glyph_svg(gid))
}

#[cfg(feature = "shape")]
#[wasm_bindgen]
pub fn ladder_shape(font: Vec<u8>, text: &str) -> Vec<u32> {
    spk_text_core::Face::parse(font)
        .map_or_else(Vec::new, |f| spk_text_core::ladder_shape(&f, text))
}

#[cfg(feature = "itemize")]
#[wasm_bindgen]
pub fn ladder_itemize(text: &str) -> usize {
    spk_text_core::ladder_itemize(text)
}

#[cfg(any(feature = "lb-uax14", feature = "lb-icu"))]
#[wasm_bindgen]
pub fn ladder_breaks(text: &str) -> usize {
    spk_text_core::line_breaks(text).len()
}

#[cfg(all(feature = "layout", any(feature = "lb-uax14", feature = "lb-icu")))]
pub use engine::*;

#[cfg(all(feature = "layout", any(feature = "lb-uax14", feature = "lb-icu")))]
mod engine {
    use spk_text_core as core_;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct Engine(core_::Engine);

    #[wasm_bindgen]
    pub struct LayoutResult(core_::Layout, Vec<u32>);

    fn stack(s: &[u32]) -> Vec<usize> {
        s.iter().map(|x| *x as usize).collect()
    }

    #[wasm_bindgen]
    impl Engine {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Engine {
            Engine(core_::Engine::new())
        }

        /// Face index, or -1 when the bytes are not a readable face.
        pub fn register_font(&mut self, bytes: Vec<u8>) -> i32 {
            self.0.register(bytes).map_or(-1, |i| i as i32)
        }

        pub fn layout(&mut self, text: &str, faces: &[u32], size: f32, width: f32) -> LayoutResult {
            let l = self.0.layout(text, &stack(faces), size, width);
            LayoutResult(l, core_::utf16_table(text))
        }

        /// SVG path data in font units, y up.
        pub fn glyph_svg(&self, face: u32, gid: u32) -> String {
            self.0.faces[face as usize].glyph_svg(gid)
        }

        pub fn units_per_em(&self, face: u32) -> f32 {
            self.0.faces[face as usize].upem
        }
    }

    macro_rules! getters {
        ($($n:ident : $t:ty),*) => {
            #[wasm_bindgen]
            impl LayoutResult { $(pub fn $n(&self) -> Vec<$t> { self.0.$n.clone() })* }
        };
    }
    getters!(gids: u32, faces: u32, xs: f32, ys: f32, cl_x: f32, cl_y: f32, cl_adv: f32,
             cl_face: u32, cl_rtl: u8, cl_n: u32, line_x: f32, line_y: f32, line_w: f32, line_rtl: u8);

    #[wasm_bindgen]
    impl LayoutResult {
        pub fn cl_start16(&self) -> Vec<u32> {
            self.0
                .cl_start
                .iter()
                .map(|b| self.1[*b as usize])
                .collect()
        }
        pub fn cl_end16(&self) -> Vec<u32> {
            self.0.cl_end.iter().map(|b| self.1[*b as usize]).collect()
        }
        pub fn line_start16(&self) -> Vec<u32> {
            self.0
                .line_start
                .iter()
                .map(|b| self.1[*b as usize])
                .collect()
        }
        pub fn line_end16(&self) -> Vec<u32> {
            self.0
                .line_end
                .iter()
                .map(|b| self.1[*b as usize])
                .collect()
        }
    }

    #[cfg(feature = "bench")]
    #[wasm_bindgen]
    impl Engine {
        /// Full layout `iters` times inside WASM; nothing crosses to JS per iteration.
        pub fn bench_layout(
            &mut self,
            text: &str,
            faces: &[u32],
            size: f32,
            width: f32,
            iters: u32,
        ) -> u32 {
            let s = stack(faces);
            let mut n = 0;
            for _ in 0..iters {
                n += self.0.layout(text, &s, size, width).gids.len() as u32;
            }
            n
        }
        pub fn bench_itemize(&mut self, text: &str, faces: &[u32], iters: u32) -> u32 {
            let s = stack(faces);
            (0..iters)
                .map(|_| self.0.bench_itemize(text, &s) as u32)
                .sum()
        }
        pub fn bench_shape(&mut self, text: &str, faces: &[u32], size: f32, iters: u32) -> u32 {
            let s = stack(faces);
            (0..iters)
                .map(|_| self.0.bench_shape(text, &s, size) as u32)
                .sum()
        }
    }

    #[cfg(feature = "bench")]
    #[wasm_bindgen]
    pub fn bench_breaks(text: &str, iters: u32) -> u32 {
        (0..iters)
            .map(|_| core_::line_breaks(text).len() as u32)
            .sum()
    }

    #[cfg(feature = "bench")]
    #[wasm_bindgen]
    pub fn corpus(kind: &str, chars: u32, seed: u32) -> String {
        let (n, s) = (chars as usize, u64::from(seed) | 1);
        match kind {
            "latin" => core_::corpus::latin(n, s),
            "arabic" => core_::corpus::arabic(n, s),
            "cjk" => core_::corpus::cjk(n, s),
            "mixed" => core_::corpus::mixed(n, s),
            "han" => core_::corpus::distinct_han(n),
            _ => String::new(),
        }
    }
}

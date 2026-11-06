//! Deterministic test text. Word lists plus a small PRNG, so the bodies are not a
//! single repeated sentence (which would flatter any cache) and every run of every
//! benchmark shapes the same bytes.

const LATIN: &[&str] = &[
    "the",
    "office",
    "first",
    "quarter",
    "revenue",
    "fiscal",
    "AVATAR",
    "Typography",
    "workflow",
    "efficient",
    "affinity",
    "fjord",
    "waffle",
    "presentation",
    "slide",
    "master",
    "layout",
    "theme",
    "chart",
    "table",
    "colour",
    "gradient",
    "shadow",
    "reflection",
    "transition",
    "animation",
    "Yearly",
    "WAVE",
    "To",
    "VA",
    "reviewed",
    "shipping",
    "customers",
    "markets",
    "growth",
    "(2026)",
    "42%",
    "and",
    "of",
    "in",
    "with",
    "for",
    "on",
    "by",
    "from",
    "is",
    "was",
    "that",
    "which",
    "because",
    "résumé",
    "naïve",
    "déjà",
    "über",
];

const ARABIC: &[&str] = &[
    "السلام",
    "عليكم",
    "العربية",
    "لغة",
    "جميلة",
    "الشركة",
    "المالية",
    "الربع",
    "الأول",
    "من",
    "السنة",
    "إلى",
    "في",
    "على",
    "هذا",
    "التقرير",
    "يوضح",
    "النمو",
    "المستمر",
    "للأرباح",
    "والمبيعات",
    "مُحَمَّد",
    "كِتَابٌ",
    "لا",
    "الله",
    "لأن",
    "مع",
    "العرض",
    "التقديمي",
    "الشريحة",
    "الجدول",
    "الرسم",
    "البياني",
    "الألوان",
    "الخطوط",
    "النص",
    "٢٠٢٦",
    "٤٢٪",
    "بسم",
    "الرحمن",
    "الرحيم",
    "فلسطين",
    "مصر",
    "المغرب",
    "الخليج",
    "شكراً",
    "جزيلاً",
    "،",
    "؛",
    "تجربة",
];

const CJK: &[&str] = &[
    "演示文稿",
    "幻灯片",
    "第一季度",
    "财务报告",
    "收入增长",
    "客户满意度",
    "市场份额",
    "主题颜色",
    "版式设计",
    "文本框",
    "图表",
    "表格",
    "动画效果",
    "切换方式",
    "我们",
    "公司",
    "今年",
    "继续",
    "提高",
    "产品质量",
    "服务水平",
    "。",
    "，",
    "、",
    "「重要」",
    "2026年",
    "42%",
    "北京",
    "上海",
    "深圳",
    "国际化",
    "团队协作",
    "云端存储",
    "数据安全",
    "性能优化",
    "用户体验",
    "人工智能",
    "机器学习",
    "排版引擎",
    "字体渲染",
    "换行规则",
    "禁则处理",
    "标点符号",
    "汉字",
    "こんにちは",
    "カタカナ",
    "한국어",
    "세계",
];

struct Rng(u64);
impl Rng {
    fn next(&mut self, n: usize) -> usize {
        // xorshift64; deterministic across platforms.
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % n as u64) as usize
    }
}

fn from_words(words: &[&str], sep: &str, chars: usize, seed: u64) -> String {
    let mut rng = Rng(seed);
    let mut out = String::new();
    let mut count = 0;
    while count < chars {
        let w = words[rng.next(words.len())];
        out.push_str(w);
        out.push_str(sep);
        count += w.chars().count() + sep.chars().count();
    }
    out.chars().take(chars).collect()
}

#[must_use]
pub fn latin(chars: usize, seed: u64) -> String {
    from_words(LATIN, " ", chars, seed)
}
#[must_use]
pub fn arabic(chars: usize, seed: u64) -> String {
    from_words(ARABIC, " ", chars, seed)
}
#[must_use]
pub fn cjk(chars: usize, seed: u64) -> String {
    from_words(CJK, "", chars, seed)
}
/// Alternating Latin, Arabic and CJK sentences: a bidi and font-fallback workout.
#[must_use]
pub fn mixed(chars: usize, seed: u64) -> String {
    let mut rng = Rng(seed);
    let mut out = String::new();
    let mut count = 0;
    while count < chars {
        let (w, sep): (&[&str], &str) = match rng.next(3) {
            0 => (LATIN, " "),
            1 => (ARABIC, " "),
            _ => (CJK, ""),
        };
        for _ in 0..4 {
            let word = w[rng.next(w.len())];
            out.push_str(word);
            out.push_str(sep);
            count += word.chars().count() + sep.chars().count();
        }
        out.push(' ');
        count += 1;
    }
    out.chars().take(chars).collect()
}
/// `n` distinct common Han characters: the worst case for a per-glyph path cache.
#[must_use]
pub fn distinct_han(n: usize) -> String {
    (0x4E00u32..).take(n).filter_map(char::from_u32).collect()
}

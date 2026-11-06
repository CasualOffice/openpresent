#!/bin/sh
# Fonts are not committed. Carlito and Liberation come from the sister repo; the
# Arabic and CJK faces exist only to measure those scripts (OFL-1.1, not bundled).
set -eu
cd "$(dirname "$0")"
mkdir -p fonts
cd fonts
CALC="${OPENCALC:-/Users/sachin/Desktop/melp/services/opencalc}"
for s in Regular Bold Italic BoldItalic; do
  cp "$CALC/webapp/fonts/Carlito-$s.ttf" "$CALC/webapp/fonts/LiberationSans-$s.ttf" .
done
[ -f NotoSansArabic.ttf ] || curl -fsSL -o NotoSansArabic.ttf \
  'https://raw.githubusercontent.com/google/fonts/main/ofl/notosansarabic/NotoSansArabic%5Bwdth,wght%5D.ttf'
[ -f NotoSansSC.otf ] || curl -fsSL -o NotoSansSC.otf \
  'https://raw.githubusercontent.com/notofonts/noto-cjk/main/Sans/SubsetOTF/SC/NotoSansSC-Regular.otf'
ls -l

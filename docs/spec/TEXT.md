# TEXT: Text bodies, paragraphs, runs, lists

Generated. Tier 1F; render 2D; edit 5; level Edit. 317 rows.


## `a:CT_GvmlTextShape`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `txBody` | `CT_TextBody` |  |  |  |
| element | `useSpRect` | `CT_GvmlUseShapeRectangle` | choice |  |  |
| element | `xfrm` | `CT_Transform2D` | choice |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `a:CT_TextParagraph`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `pPr` | `CT_TextParagraphProperties` | 0..1 |  |  |
| group | `EG_TextRun` | `EG_TextRun` | 0..unbounded |  |  |
| element | `endParaRPr` | `CT_TextCharacterProperties` | 0..1 |  |  |

## `a:ST_TextAnchoringType`
Restriction: `base=token`
- `t`
- `ctr`
- `b`
- `just`
- `dist`

## `a:ST_TextVertOverflowType`
Restriction: `base=token`
- `overflow`
- `ellipsis`
- `clip`

## `a:ST_TextHorzOverflowType`
Restriction: `base=token`
- `overflow`
- `clip`

## `a:ST_TextVerticalType`
Restriction: `base=token`
- `horz`
- `vert`
- `vert270`
- `wordArtVert`
- `eaVert`
- `mongolianVert`
- `wordArtVertRtl`

## `a:ST_TextWrappingType`
Restriction: `base=token`
- `none`
- `square`

## `a:ST_TextColumnCount`
Restriction: `base=int minInclusive=1 maxInclusive=16`

## `a:CT_TextListStyle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `defPPr` | `CT_TextParagraphProperties` | 0..1 |  |  |
| element | `lvl1pPr` | `CT_TextParagraphProperties` | 0..1 |  |  |
| element | `lvl2pPr` | `CT_TextParagraphProperties` | 0..1 |  |  |
| element | `lvl3pPr` | `CT_TextParagraphProperties` | 0..1 |  |  |
| element | `lvl4pPr` | `CT_TextParagraphProperties` | 0..1 |  |  |
| element | `lvl5pPr` | `CT_TextParagraphProperties` | 0..1 |  |  |
| element | `lvl6pPr` | `CT_TextParagraphProperties` | 0..1 |  |  |
| element | `lvl7pPr` | `CT_TextParagraphProperties` | 0..1 |  |  |
| element | `lvl8pPr` | `CT_TextParagraphProperties` | 0..1 |  |  |
| element | `lvl9pPr` | `CT_TextParagraphProperties` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `a:ST_TextFontScalePercentOrPercentString`
Restriction: `union=ST_TextFontScalePercent ST_Percentage`

## `a:ST_TextFontScalePercent`
Restriction: `base=ST_PercentageDecimal minInclusive=1000 maxInclusive=100000`

## `a:CT_TextNormalAutofit`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@fontScale` | `ST_TextFontScalePercentOrPercentString` | optional | 100% |  |
| attribute | `@lnSpcReduction` | `ST_TextSpacingPercentOrPercentString` | optional | 0% |  |

## `a:CT_TextShapeAutofit`

## `a:CT_TextNoAutofit`

## `a:EG_TextAutofit`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `noAutofit` | `CT_TextNoAutofit` | choice |  |  |
| element | `normAutofit` | `CT_TextNormalAutofit` | choice |  |  |
| element | `spAutoFit` | `CT_TextShapeAutofit` | choice |  |  |

## `a:CT_TextBodyProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `prstTxWarp` | `CT_PresetTextShape` | 0..1 |  |  |
| group | `EG_TextAutofit` | `EG_TextAutofit` | 0..1 |  |  |
| element | `scene3d` | `CT_Scene3D` | 0..1 |  |  |
| group | `EG_Text3D` | `EG_Text3D` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@rot` | `ST_Angle` | optional |  |  |
| attribute | `@spcFirstLastPara` | `boolean` | optional |  |  |
| attribute | `@vertOverflow` | `ST_TextVertOverflowType` | optional |  |  |
| attribute | `@horzOverflow` | `ST_TextHorzOverflowType` | optional |  |  |
| attribute | `@vert` | `ST_TextVerticalType` | optional |  |  |
| attribute | `@wrap` | `ST_TextWrappingType` | optional |  |  |
| attribute | `@lIns` | `ST_Coordinate32` | optional |  |  |
| attribute | `@tIns` | `ST_Coordinate32` | optional |  |  |
| attribute | `@rIns` | `ST_Coordinate32` | optional |  |  |
| attribute | `@bIns` | `ST_Coordinate32` | optional |  |  |
| attribute | `@numCol` | `ST_TextColumnCount` | optional |  |  |
| attribute | `@spcCol` | `ST_PositiveCoordinate32` | optional |  |  |
| attribute | `@rtlCol` | `boolean` | optional |  |  |
| attribute | `@fromWordArt` | `boolean` | optional |  |  |
| attribute | `@anchor` | `ST_TextAnchoringType` | optional |  |  |
| attribute | `@anchorCtr` | `boolean` | optional |  |  |
| attribute | `@forceAA` | `boolean` | optional |  |  |
| attribute | `@upright` | `boolean` | optional | false |  |
| attribute | `@compatLnSpc` | `boolean` | optional |  |  |

## `a:CT_TextBody`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `bodyPr` | `CT_TextBodyProperties` |  |  |  |
| element | `lstStyle` | `CT_TextListStyle` | 0..1 |  |  |
| element | `p` | `CT_TextParagraph` | 1..unbounded |  |  |

## `a:ST_TextBulletStartAtNum`
Restriction: `base=int minInclusive=1 maxInclusive=32767`

## `a:ST_TextAutonumberScheme`
Restriction: `base=token`
- `alphaLcParenBoth`
- `alphaUcParenBoth`
- `alphaLcParenR`
- `alphaUcParenR`
- `alphaLcPeriod`
- `alphaUcPeriod`
- `arabicParenBoth`
- `arabicParenR`
- `arabicPeriod`
- `arabicPlain`
- `romanLcParenBoth`
- `romanUcParenBoth`
- `romanLcParenR`
- `romanUcParenR`
- `romanLcPeriod`
- `romanUcPeriod`
- `circleNumDbPlain`
- `circleNumWdBlackPlain`
- `circleNumWdWhitePlain`
- `arabicDbPeriod`
- `arabicDbPlain`
- `ea1ChsPeriod`
- `ea1ChsPlain`
- `ea1ChtPeriod`
- `ea1ChtPlain`
- `ea1JpnChsDbPeriod`
- `ea1JpnKorPlain`
- `ea1JpnKorPeriod`
- `arabic1Minus`
- `arabic2Minus`
- `hebrew2Minus`
- `thaiAlphaPeriod`
- `thaiAlphaParenR`
- `thaiAlphaParenBoth`
- `thaiNumPeriod`
- `thaiNumParenR`
- `thaiNumParenBoth`
- `hindiAlphaPeriod`
- `hindiNumPeriod`
- `hindiNumParenR`
- `hindiAlpha1Period`

## `a:CT_TextBulletColorFollowText`

## `a:EG_TextBulletColor`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `buClrTx` | `CT_TextBulletColorFollowText` | choice |  |  |
| element | `buClr` | `CT_Color` | choice |  |  |

## `a:ST_TextBulletSize`
Restriction: `union=ST_TextBulletSizePercent ST_TextBulletSizeDecimal`

## `a:ST_TextBulletSizePercent`
Restriction: `base=string pattern=0*((2[5-9])\|([3-9][0-9])\|([1-3][0-9][0-9])\|400)%`

## `a:ST_TextBulletSizeDecimal`
Restriction: `base=ST_PercentageDecimal minInclusive=25000 maxInclusive=400000`

## `a:CT_TextBulletSizeFollowText`

## `a:CT_TextBulletSizePercent`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_TextBulletSizePercent` | required |  |  |

## `a:CT_TextBulletSizePoint`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_TextFontSize` | required |  |  |

## `a:EG_TextBulletSize`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `buSzTx` | `CT_TextBulletSizeFollowText` | choice |  |  |
| element | `buSzPct` | `CT_TextBulletSizePercent` | choice |  |  |
| element | `buSzPts` | `CT_TextBulletSizePoint` | choice |  |  |

## `a:CT_TextBulletTypefaceFollowText`

## `a:EG_TextBulletTypeface`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `buFontTx` | `CT_TextBulletTypefaceFollowText` | choice |  |  |
| element | `buFont` | `CT_TextFont` | choice |  |  |

## `a:CT_TextAutonumberBullet`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@type` | `ST_TextAutonumberScheme` | required |  |  |
| attribute | `@startAt` | `ST_TextBulletStartAtNum` | optional | 1 |  |

## `a:CT_TextCharBullet`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@char` | `string` | required |  |  |

## `a:CT_TextBlipBullet`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `blip` | `CT_Blip` |  |  |  |

## `a:CT_TextNoBullet`

## `a:EG_TextBullet`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `buNone` | `CT_TextNoBullet` | choice |  |  |
| element | `buAutoNum` | `CT_TextAutonumberBullet` | choice |  |  |
| element | `buChar` | `CT_TextCharBullet` | choice |  |  |
| element | `buBlip` | `CT_TextBlipBullet` | choice |  |  |

## `a:ST_TextPoint`
Restriction: `union=ST_TextPointUnqualified ST_UniversalMeasure`

## `a:ST_TextPointUnqualified`
Restriction: `base=int minInclusive=-400000 maxInclusive=400000`

## `a:ST_TextNonNegativePoint`
Restriction: `base=int minInclusive=0 maxInclusive=400000`

## `a:ST_TextFontSize`
Restriction: `base=int minInclusive=100 maxInclusive=400000`

## `a:ST_TextTypeface`
Restriction: `base=string`

## `a:CT_TextFont`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@typeface` | `ST_TextTypeface` | required |  |  |
| attribute | `@panose` | `ST_Panose` | optional |  |  |
| attribute | `@pitchFamily` | `ST_PitchFamily` | optional | 0 |  |
| attribute | `@charset` | `byte` | optional | 1 |  |

## `a:ST_TextUnderlineType`
Restriction: `base=token`
- `none`
- `words`
- `sng`
- `dbl`
- `heavy`
- `dotted`
- `dottedHeavy`
- `dash`
- `dashHeavy`
- `dashLong`
- `dashLongHeavy`
- `dotDash`
- `dotDashHeavy`
- `dotDotDash`
- `dotDotDashHeavy`
- `wavy`
- `wavyHeavy`
- `wavyDbl`

## `a:CT_TextUnderlineLineFollowText`

## `a:CT_TextUnderlineFillFollowText`

## `a:CT_TextUnderlineFillGroupWrapper`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_FillProperties` | `EG_FillProperties` |  |  |  |

## `a:EG_TextUnderlineLine`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `uLnTx` | `CT_TextUnderlineLineFollowText` | choice |  |  |
| element | `uLn` | `CT_LineProperties` | 0..1 |  |  |

## `a:EG_TextUnderlineFill`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `uFillTx` | `CT_TextUnderlineFillFollowText` | choice |  |  |
| element | `uFill` | `CT_TextUnderlineFillGroupWrapper` | choice |  |  |

## `a:ST_TextStrikeType`
Restriction: `base=token`
- `noStrike`
- `sngStrike`
- `dblStrike`

## `a:ST_TextCapsType`
Restriction: `base=token`
- `none`
- `small`
- `all`

## `a:CT_TextCharacterProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ln` | `CT_LineProperties` | 0..1 |  |  |
| group | `EG_FillProperties` | `EG_FillProperties` | 0..1 |  |  |
| group | `EG_EffectProperties` | `EG_EffectProperties` | 0..1 |  |  |
| element | `highlight` | `CT_Color` | 0..1 |  |  |
| group | `EG_TextUnderlineLine` | `EG_TextUnderlineLine` | 0..1 |  |  |
| group | `EG_TextUnderlineFill` | `EG_TextUnderlineFill` | 0..1 |  |  |
| element | `latin` | `CT_TextFont` | 0..1 |  |  |
| element | `ea` | `CT_TextFont` | 0..1 |  |  |
| element | `cs` | `CT_TextFont` | 0..1 |  |  |
| element | `sym` | `CT_TextFont` | 0..1 |  |  |
| element | `hlinkClick` | `CT_Hyperlink` | 0..1 |  |  |
| element | `hlinkMouseOver` | `CT_Hyperlink` | 0..1 |  |  |
| element | `rtl` | `CT_Boolean` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@kumimoji` | `boolean` | optional |  |  |
| attribute | `@lang` | `ST_Lang` | optional |  |  |
| attribute | `@altLang` | `ST_Lang` | optional |  |  |
| attribute | `@sz` | `ST_TextFontSize` | optional |  |  |
| attribute | `@b` | `boolean` | optional |  |  |
| attribute | `@i` | `boolean` | optional |  |  |
| attribute | `@u` | `ST_TextUnderlineType` | optional |  |  |
| attribute | `@strike` | `ST_TextStrikeType` | optional |  |  |
| attribute | `@kern` | `ST_TextNonNegativePoint` | optional |  |  |
| attribute | `@cap` | `ST_TextCapsType` | optional | none |  |
| attribute | `@spc` | `ST_TextPoint` | optional |  |  |
| attribute | `@normalizeH` | `boolean` | optional |  |  |
| attribute | `@baseline` | `ST_Percentage` | optional |  |  |
| attribute | `@noProof` | `boolean` | optional |  |  |
| attribute | `@dirty` | `boolean` | optional | true |  |
| attribute | `@err` | `boolean` | optional | false |  |
| attribute | `@smtClean` | `boolean` | optional | true |  |
| attribute | `@smtId` | `unsignedInt` | optional | 0 |  |
| attribute | `@bmk` | `string` | optional |  |  |

## `a:ST_TextSpacingPoint`
Restriction: `base=int minInclusive=0 maxInclusive=158400`

## `a:ST_TextSpacingPercentOrPercentString`
Restriction: `union=ST_TextSpacingPercent ST_Percentage`

## `a:ST_TextSpacingPercent`
Restriction: `base=ST_PercentageDecimal minInclusive=0 maxInclusive=13200000`

## `a:CT_TextSpacingPercent`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_TextSpacingPercentOrPercentString` | required |  |  |

## `a:CT_TextSpacingPoint`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_TextSpacingPoint` | required |  |  |

## `a:ST_TextMargin`
Restriction: `base=ST_Coordinate32Unqualified minInclusive=0 maxInclusive=51206400`

## `a:ST_TextIndent`
Restriction: `base=ST_Coordinate32Unqualified minInclusive=-51206400 maxInclusive=51206400`

## `a:ST_TextTabAlignType`
Restriction: `base=token`
- `l`
- `ctr`
- `r`
- `dec`

## `a:CT_TextTabStop`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@pos` | `ST_Coordinate32` | optional |  |  |
| attribute | `@algn` | `ST_TextTabAlignType` | optional |  |  |

## `a:CT_TextTabStopList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tab` | `CT_TextTabStop` | 0..32 |  |  |

## `a:CT_TextLineBreak`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `rPr` | `CT_TextCharacterProperties` | 0..1 |  |  |

## `a:CT_TextSpacing`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `spcPct` | `CT_TextSpacingPercent` | choice |  |  |
| element | `spcPts` | `CT_TextSpacingPoint` | choice |  |  |

## `a:ST_TextAlignType`
Restriction: `base=token`
- `l`
- `ctr`
- `r`
- `just`
- `justLow`
- `dist`
- `thaiDist`

## `a:ST_TextFontAlignType`
Restriction: `base=token`
- `auto`
- `t`
- `ctr`
- `base`
- `b`

## `a:ST_TextIndentLevelType`
Restriction: `base=int minInclusive=0 maxInclusive=8`

## `a:CT_TextParagraphProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `lnSpc` | `CT_TextSpacing` | 0..1 |  |  |
| element | `spcBef` | `CT_TextSpacing` | 0..1 |  |  |
| element | `spcAft` | `CT_TextSpacing` | 0..1 |  |  |
| group | `EG_TextBulletColor` | `EG_TextBulletColor` | 0..1 |  |  |
| group | `EG_TextBulletSize` | `EG_TextBulletSize` | 0..1 |  |  |
| group | `EG_TextBulletTypeface` | `EG_TextBulletTypeface` | 0..1 |  |  |
| group | `EG_TextBullet` | `EG_TextBullet` | 0..1 |  |  |
| element | `tabLst` | `CT_TextTabStopList` | 0..1 |  |  |
| element | `defRPr` | `CT_TextCharacterProperties` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@marL` | `ST_TextMargin` | optional |  |  |
| attribute | `@marR` | `ST_TextMargin` | optional |  |  |
| attribute | `@lvl` | `ST_TextIndentLevelType` | optional |  |  |
| attribute | `@indent` | `ST_TextIndent` | optional |  |  |
| attribute | `@algn` | `ST_TextAlignType` | optional |  |  |
| attribute | `@defTabSz` | `ST_Coordinate32` | optional |  |  |
| attribute | `@rtl` | `boolean` | optional |  |  |
| attribute | `@eaLnBrk` | `boolean` | optional |  |  |
| attribute | `@fontAlgn` | `ST_TextFontAlignType` | optional |  |  |
| attribute | `@latinLnBrk` | `boolean` | optional |  |  |
| attribute | `@hangingPunct` | `boolean` | optional |  |  |

## `a:CT_TextField`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `rPr` | `CT_TextCharacterProperties` | 0..1 |  |  |
| element | `pPr` | `CT_TextParagraphProperties` | 0..1 |  |  |
| element | `t` | `string` | 0..1 |  |  |
| attribute | `@id` | `ST_Guid` | required |  |  |
| attribute | `@type` | `string` | optional |  |  |

## `a:EG_TextRun`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `r` | `CT_RegularTextRun` | choice |  |  |
| element | `br` | `CT_TextLineBreak` | choice |  |  |
| element | `fld` | `CT_TextField` | choice |  |  |

## `a:CT_RegularTextRun`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `rPr` | `CT_TextCharacterProperties` | 0..1 |  |  |
| element | `t` | `string` |  |  |  |

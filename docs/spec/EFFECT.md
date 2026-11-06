# EFFECT: Effects and blip effects

Generated. Tier 1E; render 2C; edit 4; level Edit. 355 rows.


## `a:ST_RectAlignment`
Restriction: `base=token`
- `tl`
- `t`
- `tr`
- `l`
- `ctr`
- `r`
- `bl`
- `b`
- `br`

## `a:CT_AlphaBiLevelEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@thresh` | `ST_PositiveFixedPercentage` | required |  |  |

## `a:CT_AlphaCeilingEffect`

## `a:CT_AlphaFloorEffect`

## `a:CT_AlphaInverseEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorChoice` | `EG_ColorChoice` | 0..1 |  |  |

## `a:CT_AlphaModulateFixedEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@amt` | `ST_PositivePercentage` | optional | 100% |  |

## `a:CT_AlphaOutsetEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@rad` | `ST_Coordinate` | optional | 0 |  |

## `a:CT_AlphaReplaceEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@a` | `ST_PositiveFixedPercentage` | required |  |  |

## `a:CT_BiLevelEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@thresh` | `ST_PositiveFixedPercentage` | required |  |  |

## `a:CT_BlurEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@rad` | `ST_PositiveCoordinate` | optional | 0 |  |
| attribute | `@grow` | `boolean` | optional | true |  |

## `a:CT_DuotoneEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorChoice` | `EG_ColorChoice` | 2..2 |  |  |

## `a:CT_GlowEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorChoice` | `EG_ColorChoice` |  |  |  |
| attribute | `@rad` | `ST_PositiveCoordinate` | optional | 0 |  |

## `a:CT_GrayscaleEffect`

## `a:CT_HSLEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@hue` | `ST_PositiveFixedAngle` | optional | 0 |  |
| attribute | `@sat` | `ST_FixedPercentage` | optional | 0% |  |
| attribute | `@lum` | `ST_FixedPercentage` | optional | 0% |  |

## `a:CT_InnerShadowEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorChoice` | `EG_ColorChoice` |  |  |  |
| attribute | `@blurRad` | `ST_PositiveCoordinate` | optional | 0 |  |
| attribute | `@dist` | `ST_PositiveCoordinate` | optional | 0 |  |
| attribute | `@dir` | `ST_PositiveFixedAngle` | optional | 0 |  |

## `a:CT_LuminanceEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@bright` | `ST_FixedPercentage` | optional | 0% |  |
| attribute | `@contrast` | `ST_FixedPercentage` | optional | 0% |  |

## `a:CT_OuterShadowEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorChoice` | `EG_ColorChoice` |  |  |  |
| attribute | `@blurRad` | `ST_PositiveCoordinate` | optional | 0 |  |
| attribute | `@dist` | `ST_PositiveCoordinate` | optional | 0 |  |
| attribute | `@dir` | `ST_PositiveFixedAngle` | optional | 0 |  |
| attribute | `@sx` | `ST_Percentage` | optional | 100% |  |
| attribute | `@sy` | `ST_Percentage` | optional | 100% |  |
| attribute | `@kx` | `ST_FixedAngle` | optional | 0 |  |
| attribute | `@ky` | `ST_FixedAngle` | optional | 0 |  |
| attribute | `@algn` | `ST_RectAlignment` | optional | b |  |
| attribute | `@rotWithShape` | `boolean` | optional | true |  |

## `a:ST_PresetShadowVal`
Restriction: `base=token`
- `shdw1`
- `shdw2`
- `shdw3`
- `shdw4`
- `shdw5`
- `shdw6`
- `shdw7`
- `shdw8`
- `shdw9`
- `shdw10`
- `shdw11`
- `shdw12`
- `shdw13`
- `shdw14`
- `shdw15`
- `shdw16`
- `shdw17`
- `shdw18`
- `shdw19`
- `shdw20`

## `a:CT_PresetShadowEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorChoice` | `EG_ColorChoice` |  |  |  |
| attribute | `@prst` | `ST_PresetShadowVal` | required |  |  |
| attribute | `@dist` | `ST_PositiveCoordinate` | optional | 0 |  |
| attribute | `@dir` | `ST_PositiveFixedAngle` | optional | 0 |  |

## `a:CT_ReflectionEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@blurRad` | `ST_PositiveCoordinate` | optional | 0 |  |
| attribute | `@stA` | `ST_PositiveFixedPercentage` | optional | 100% |  |
| attribute | `@stPos` | `ST_PositiveFixedPercentage` | optional | 0% |  |
| attribute | `@endA` | `ST_PositiveFixedPercentage` | optional | 0% |  |
| attribute | `@endPos` | `ST_PositiveFixedPercentage` | optional | 100% |  |
| attribute | `@dist` | `ST_PositiveCoordinate` | optional | 0 |  |
| attribute | `@dir` | `ST_PositiveFixedAngle` | optional | 0 |  |
| attribute | `@fadeDir` | `ST_PositiveFixedAngle` | optional | 5400000 |  |
| attribute | `@sx` | `ST_Percentage` | optional | 100% |  |
| attribute | `@sy` | `ST_Percentage` | optional | 100% |  |
| attribute | `@kx` | `ST_FixedAngle` | optional | 0 |  |
| attribute | `@ky` | `ST_FixedAngle` | optional | 0 |  |
| attribute | `@algn` | `ST_RectAlignment` | optional | b |  |
| attribute | `@rotWithShape` | `boolean` | optional | true |  |

## `a:CT_RelativeOffsetEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@tx` | `ST_Percentage` | optional | 0% |  |
| attribute | `@ty` | `ST_Percentage` | optional | 0% |  |

## `a:CT_SoftEdgesEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@rad` | `ST_PositiveCoordinate` | required |  |  |

## `a:CT_TintEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@hue` | `ST_PositiveFixedAngle` | optional | 0 |  |
| attribute | `@amt` | `ST_FixedPercentage` | optional | 0% |  |

## `a:CT_TransformEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@sx` | `ST_Percentage` | optional | 100% |  |
| attribute | `@sy` | `ST_Percentage` | optional | 100% |  |
| attribute | `@kx` | `ST_FixedAngle` | optional | 0 |  |
| attribute | `@ky` | `ST_FixedAngle` | optional | 0 |  |
| attribute | `@tx` | `ST_Coordinate` | optional | 0 |  |
| attribute | `@ty` | `ST_Coordinate` | optional | 0 |  |

## `a:CT_FillEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_FillProperties` | `EG_FillProperties` |  |  |  |

## `a:ST_BlendMode`
Restriction: `base=token`
- `over`
- `mult`
- `screen`
- `darken`
- `lighten`

## `a:CT_FillOverlayEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_FillProperties` | `EG_FillProperties` |  |  |  |
| attribute | `@blend` | `ST_BlendMode` | required |  |  |

## `a:EG_Effect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cont` | `CT_EffectContainer` | choice |  |  |
| element | `effect` | `CT_EffectReference` | choice |  |  |
| element | `alphaBiLevel` | `CT_AlphaBiLevelEffect` | choice |  |  |
| element | `alphaCeiling` | `CT_AlphaCeilingEffect` | choice |  |  |
| element | `alphaFloor` | `CT_AlphaFloorEffect` | choice |  |  |
| element | `alphaInv` | `CT_AlphaInverseEffect` | choice |  |  |
| element | `alphaMod` | `CT_AlphaModulateEffect` | choice |  |  |
| element | `alphaModFix` | `CT_AlphaModulateFixedEffect` | choice |  |  |
| element | `alphaOutset` | `CT_AlphaOutsetEffect` | choice |  |  |
| element | `alphaRepl` | `CT_AlphaReplaceEffect` | choice |  |  |
| element | `biLevel` | `CT_BiLevelEffect` | choice |  |  |
| element | `blend` | `CT_BlendEffect` | choice |  |  |
| element | `blur` | `CT_BlurEffect` | choice |  |  |
| element | `clrChange` | `CT_ColorChangeEffect` | choice |  |  |
| element | `clrRepl` | `CT_ColorReplaceEffect` | choice |  |  |
| element | `duotone` | `CT_DuotoneEffect` | choice |  |  |
| element | `fill` | `CT_FillEffect` | choice |  |  |
| element | `fillOverlay` | `CT_FillOverlayEffect` | choice |  |  |
| element | `glow` | `CT_GlowEffect` | choice |  |  |
| element | `grayscl` | `CT_GrayscaleEffect` | choice |  |  |
| element | `hsl` | `CT_HSLEffect` | choice |  |  |
| element | `innerShdw` | `CT_InnerShadowEffect` | choice |  |  |
| element | `lum` | `CT_LuminanceEffect` | choice |  |  |
| element | `outerShdw` | `CT_OuterShadowEffect` | choice |  |  |
| element | `prstShdw` | `CT_PresetShadowEffect` | choice |  |  |
| element | `reflection` | `CT_ReflectionEffect` | choice |  |  |
| element | `relOff` | `CT_RelativeOffsetEffect` | choice |  |  |
| element | `softEdge` | `CT_SoftEdgesEffect` | choice |  |  |
| element | `tint` | `CT_TintEffect` | choice |  |  |
| element | `xfrm` | `CT_TransformEffect` | choice |  |  |

## `a:ST_EffectContainerType`
Restriction: `base=token`
- `sib`
- `tree`

## `a:CT_EffectContainer`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_Effect` | `EG_Effect` | 0..unbounded |  |  |
| attribute | `@type` | `ST_EffectContainerType` | optional | sib |  |
| attribute | `@name` | `token` | optional |  |  |

## `a:CT_AlphaModulateEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cont` | `CT_EffectContainer` |  |  |  |

## `a:CT_BlendEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cont` | `CT_EffectContainer` |  |  |  |
| attribute | `@blend` | `ST_BlendMode` | required |  |  |

## `a:CT_EffectList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `blur` | `CT_BlurEffect` | 0..1 |  |  |
| element | `fillOverlay` | `CT_FillOverlayEffect` | 0..1 |  |  |
| element | `glow` | `CT_GlowEffect` | 0..1 |  |  |
| element | `innerShdw` | `CT_InnerShadowEffect` | 0..1 |  |  |
| element | `outerShdw` | `CT_OuterShadowEffect` | 0..1 |  |  |
| element | `prstShdw` | `CT_PresetShadowEffect` | 0..1 |  |  |
| element | `reflection` | `CT_ReflectionEffect` | 0..1 |  |  |
| element | `softEdge` | `CT_SoftEdgesEffect` | 0..1 |  |  |

## `a:EG_EffectProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `effectLst` | `CT_EffectList` | choice |  |  |
| element | `effectDag` | `CT_EffectContainer` | choice |  |  |

## `a:CT_EffectProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_EffectProperties` | `EG_EffectProperties` |  |  |  |

## `x:drawing/2010/main:ST_LegacySpreadsheetColorIndex`
Restriction: `base=int minInclusive=0 maxInclusive=80`

## `x:drawing/2010/main:@legacySpreadsheetColorIndex` : `ST_LegacySpreadsheetColorIndex`

## `x:drawing/2010/main:CT_TextMath`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `m:oMath` | `oMath` |  |  |  |

## `x:drawing/2010/main:<m>` : `CT_TextMath`

## `x:drawing/2010/main:CT_ContentPartLocking`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attrgroup | `AG_Locking` | `AG_Locking` |  |  |  |

## `x:drawing/2010/main:CT_NonVisualInkContentPartProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cpLocks` | `CT_ContentPartLocking` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@isComment` | `boolean` | optional | true |  |

## `x:drawing/2010/main:CT_CameraTool`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@cellRange` | `string` | optional |  |  |
| attribute | `@spid` | `string` | optional | 0 |  |

## `x:drawing/2010/main:<cameraTool>` : `CT_CameraTool`

## `x:drawing/2010/main:CT_CompatExt`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@spid` | `string` | optional |  |  |

## `x:drawing/2010/main:<compatExt>` : `CT_CompatExt`

## `x:drawing/2010/main:CT_IsGvmlCanvas`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `boolean` | required |  |  |

## `x:drawing/2010/main:CT_GvmlContentPartNonVisual`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cNvPr` | `CT_NonVisualDrawingProps` |  |  |  |
| element | `cNvContentPartPr` | `CT_NonVisualInkContentPartProperties` | 0..1 |  |  |

## `x:drawing/2010/main:CT_GvmlContentPart`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `nvContentPartPr` | `CT_GvmlContentPartNonVisual` | 0..1 |  |  |
| element | `xfrm` | `CT_Transform2D` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@bwMode` | `ST_BlackWhiteMode` | optional |  |  |
| attribute | `@r:id` | `id` | required |  |  |

## `x:drawing/2010/main:<isCanvas>` : `CT_IsGvmlCanvas`

## `x:drawing/2010/main:<contentPart>` : `CT_GvmlContentPart`

## `x:drawing/2010/main:CT_ShadowObscured`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `boolean` | optional | false |  |

## `x:drawing/2010/main:<shadowObscured>` : `CT_ShadowObscured`

## `x:drawing/2010/main:<hiddenFill>` : `CT_FillProperties`

## `x:drawing/2010/main:<hiddenLine>` : `CT_LineProperties`

## `x:drawing/2010/main:<hiddenEffects>` : `CT_EffectProperties`

## `x:drawing/2010/main:<hiddenScene3d>` : `CT_Scene3D`

## `x:drawing/2010/main:<hiddenSp3d>` : `CT_Shape3D`

## `x:drawing/2010/main:ST_ArtisticEffectParam100`
Restriction: `base=int minInclusive=0 maxInclusive=100`

## `x:drawing/2010/main:ST_ArtisticEffectParam10`
Restriction: `base=int minInclusive=0 maxInclusive=10`

## `x:drawing/2010/main:ST_ArtisticEffectParam6`
Restriction: `base=int minInclusive=0 maxInclusive=6`

## `x:drawing/2010/main:ST_ArtisticEffectParam4`
Restriction: `base=int minInclusive=0 maxInclusive=4`

## `x:drawing/2010/main:CT_PictureEffectBlur`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@radius` | `ST_ArtisticEffectParam100` | optional | 10 |  |

## `x:drawing/2010/main:CT_PictureEffectCement`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@trans` | `ST_PositiveFixedPercentage` | optional | 0 |  |
| attribute | `@crackSpacing` | `ST_ArtisticEffectParam100` | optional | 24 |  |

## `x:drawing/2010/main:CT_PictureEffectChalkSketch`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@trans` | `ST_PositiveFixedPercentage` | optional | 0 |  |
| attribute | `@pressure` | `ST_ArtisticEffectParam4` | optional | 0 |  |

## `x:drawing/2010/main:CT_PictureEffectCrisscrossEtching`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@trans` | `ST_PositiveFixedPercentage` | optional | 75 |  |
| attribute | `@pressure` | `ST_ArtisticEffectParam100` | optional | 30 |  |

## `x:drawing/2010/main:CT_PictureEffectCutout`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@trans` | `ST_PositiveFixedPercentage` | optional | 0 |  |
| attribute | `@numberOfShades` | `ST_ArtisticEffectParam6` | optional | 2 |  |

## `x:drawing/2010/main:CT_PictureEffectFilmGrain`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@trans` | `ST_PositiveFixedPercentage` | optional | 0 |  |
| attribute | `@grainSize` | `ST_ArtisticEffectParam100` | optional | 40 |  |

## `x:drawing/2010/main:CT_PictureEffectGlass`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@trans` | `ST_PositiveFixedPercentage` | optional | 0 |  |
| attribute | `@scaling` | `ST_ArtisticEffectParam100` | optional | 34 |  |

## `x:drawing/2010/main:CT_PictureEffectGlowDiffused`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@trans` | `ST_PositiveFixedPercentage` | optional | 0 |  |
| attribute | `@intensity` | `ST_ArtisticEffectParam10` | optional | 5 |  |

## `x:drawing/2010/main:CT_PictureEffectGlowEdges`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@trans` | `ST_PositiveFixedPercentage` | optional | 15 |  |
| attribute | `@smoothness` | `ST_ArtisticEffectParam10` | optional | 3 |  |

## `x:drawing/2010/main:CT_PictureEffectLightScreen`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@trans` | `ST_PositiveFixedPercentage` | optional | 0 |  |
| attribute | `@gridSize` | `ST_ArtisticEffectParam10` | optional | 4 |  |

## `x:drawing/2010/main:CT_PictureEffectLineDrawing`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@trans` | `ST_PositiveFixedPercentage` | optional | 25 |  |
| attribute | `@pencilSize` | `ST_ArtisticEffectParam100` | optional | 0 |  |

## `x:drawing/2010/main:CT_PictureEffectMarker`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@trans` | `ST_PositiveFixedPercentage` | optional | 0 |  |
| attribute | `@size` | `ST_ArtisticEffectParam100` | optional | 97 |  |

## `x:drawing/2010/main:CT_PictureEffectMosiaicBubbles`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@trans` | `ST_PositiveFixedPercentage` | optional | 0 |  |
| attribute | `@pressure` | `ST_ArtisticEffectParam100` | optional | 14 |  |

## `x:drawing/2010/main:CT_PictureEffectPaintBrush`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@trans` | `ST_PositiveFixedPercentage` | optional | 0 |  |
| attribute | `@brushSize` | `ST_ArtisticEffectParam10` | optional | 2 |  |

## `x:drawing/2010/main:CT_PictureEffectPaintStrokes`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@trans` | `ST_PositiveFixedPercentage` | optional | 0 |  |
| attribute | `@intensity` | `ST_ArtisticEffectParam10` | optional | 5 |  |

## `x:drawing/2010/main:CT_PictureEffectPastelsSmooth`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@trans` | `ST_PositiveFixedPercentage` | optional | 0 |  |
| attribute | `@scaling` | `ST_ArtisticEffectParam100` | optional | 34 |  |

## `x:drawing/2010/main:CT_PictureEffectPencilGrayscale`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@trans` | `ST_PositiveFixedPercentage` | optional | 0 |  |
| attribute | `@pencilSize` | `ST_ArtisticEffectParam100` | optional | 27 |  |

## `x:drawing/2010/main:CT_PictureEffectPencilSketch`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@trans` | `ST_PositiveFixedPercentage` | optional | 0 |  |
| attribute | `@pressure` | `ST_ArtisticEffectParam100` | optional | 22 |  |

## `x:drawing/2010/main:CT_PictureEffectPhotocopy`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@trans` | `ST_PositiveFixedPercentage` | optional | 30 |  |
| attribute | `@detail` | `ST_ArtisticEffectParam10` | optional | 3 |  |

## `x:drawing/2010/main:CT_PictureEffectPlasticWrap`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@trans` | `ST_PositiveFixedPercentage` | optional | 0 |  |
| attribute | `@smoothness` | `ST_ArtisticEffectParam10` | optional | 5 |  |

## `x:drawing/2010/main:CT_PictureEffectTexturizer`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@trans` | `ST_PositiveFixedPercentage` | optional | 0 |  |
| attribute | `@scaling` | `ST_ArtisticEffectParam100` | optional | 34 |  |

## `x:drawing/2010/main:CT_PictureEffectWatercolorSponge`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@trans` | `ST_PositiveFixedPercentage` | optional | 0 |  |
| attribute | `@brushSize` | `ST_ArtisticEffectParam10` | optional | 2 |  |

## `x:drawing/2010/main:CT_PictureEffectBackgroundRemovalForegroundMark`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@x1` | `ST_PositiveFixedPercentage` | required |  |  |
| attribute | `@y1` | `ST_PositiveFixedPercentage` | required |  |  |
| attribute | `@x2` | `ST_PositiveFixedPercentage` | required |  |  |
| attribute | `@y2` | `ST_PositiveFixedPercentage` | required |  |  |

## `x:drawing/2010/main:CT_PictureEffectBackgroundRemovalBackgroundMark`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@x1` | `ST_PositiveFixedPercentage` | required |  |  |
| attribute | `@y1` | `ST_PositiveFixedPercentage` | required |  |  |
| attribute | `@x2` | `ST_PositiveFixedPercentage` | required |  |  |
| attribute | `@y2` | `ST_PositiveFixedPercentage` | required |  |  |

## `x:drawing/2010/main:CT_PictureEffectBackgroundRemoval`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `foregroundMark` | `CT_PictureEffectBackgroundRemovalForegroundMark` | 0..unbounded |  |  |
| element | `backgroundMark` | `CT_PictureEffectBackgroundRemovalBackgroundMark` | 0..unbounded |  |  |
| attribute | `@t` | `ST_PositiveFixedPercentage` | required |  |  |
| attribute | `@b` | `ST_PositiveFixedPercentage` | required |  |  |
| attribute | `@l` | `ST_PositiveFixedPercentage` | required |  |  |
| attribute | `@r` | `ST_PositiveFixedPercentage` | required |  |  |

## `x:drawing/2010/main:CT_PictureEffectBrightnessContrast`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@bright` | `ST_FixedPercentage` | optional | 0 |  |
| attribute | `@contrast` | `ST_FixedPercentage` | optional | 0 |  |

## `x:drawing/2010/main:ST_ColorTemperature`
Restriction: `base=int minInclusive=1500 maxInclusive=11500`

## `x:drawing/2010/main:CT_PictureEffectColorTemperature`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@colorTemp` | `ST_ColorTemperature` | optional | 6500 |  |

## `x:drawing/2010/main:ST_SaturationAmount`
Restriction: `base=ST_Percentage minInclusive=0 maxInclusive=400000`

## `x:drawing/2010/main:CT_PictureEffectSaturation`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@sat` | `ST_SaturationAmount` | optional | 100 |  |

## `x:drawing/2010/main:CT_PictureEffectSharpenSoften`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@amount` | `ST_FixedPercentage` | optional | 0 |  |

## `x:drawing/2010/main:CT_PictureEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `artisticBlur` | `CT_PictureEffectBlur` | choice |  |  |
| element | `artisticCement` | `CT_PictureEffectCement` | choice |  |  |
| element | `artisticChalkSketch` | `CT_PictureEffectChalkSketch` | choice |  |  |
| element | `artisticCrisscrossEtching` | `CT_PictureEffectCrisscrossEtching` | choice |  |  |
| element | `artisticCutout` | `CT_PictureEffectCutout` | choice |  |  |
| element | `artisticFilmGrain` | `CT_PictureEffectFilmGrain` | choice |  |  |
| element | `artisticGlass` | `CT_PictureEffectGlass` | choice |  |  |
| element | `artisticGlowDiffused` | `CT_PictureEffectGlowDiffused` | choice |  |  |
| element | `artisticGlowEdges` | `CT_PictureEffectGlowEdges` | choice |  |  |
| element | `artisticLightScreen` | `CT_PictureEffectLightScreen` | choice |  |  |
| element | `artisticLineDrawing` | `CT_PictureEffectLineDrawing` | choice |  |  |
| element | `artisticMarker` | `CT_PictureEffectMarker` | choice |  |  |
| element | `artisticMosiaicBubbles` | `CT_PictureEffectMosiaicBubbles` | choice |  |  |
| element | `artisticPaintStrokes` | `CT_PictureEffectPaintStrokes` | choice |  |  |
| element | `artisticPaintBrush` | `CT_PictureEffectPaintBrush` | choice |  |  |
| element | `artisticPastelsSmooth` | `CT_PictureEffectPastelsSmooth` | choice |  |  |
| element | `artisticPencilGrayscale` | `CT_PictureEffectPencilGrayscale` | choice |  |  |
| element | `artisticPencilSketch` | `CT_PictureEffectPencilSketch` | choice |  |  |
| element | `artisticPhotocopy` | `CT_PictureEffectPhotocopy` | choice |  |  |
| element | `artisticPlasticWrap` | `CT_PictureEffectPlasticWrap` | choice |  |  |
| element | `artisticTexturizer` | `CT_PictureEffectTexturizer` | choice |  |  |
| element | `artisticWatercolorSponge` | `CT_PictureEffectWatercolorSponge` | choice |  |  |
| element | `backgroundRemoval` | `CT_PictureEffectBackgroundRemoval` | choice |  |  |
| element | `brightnessContrast` | `CT_PictureEffectBrightnessContrast` | choice |  |  |
| element | `colorTemperature` | `CT_PictureEffectColorTemperature` | choice |  |  |
| element | `saturation` | `CT_PictureEffectSaturation` | choice |  |  |
| element | `sharpenSoften` | `CT_PictureEffectSharpenSoften` | choice |  |  |
| attribute | `@visible` | `boolean` | optional | true |  |

## `x:drawing/2010/main:CT_PictureLayer`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `imgEffect` | `CT_PictureEffect` | 0..unbounded |  |  |
| attribute | `@r:embed` | `embed` | optional |  |  |

## `x:drawing/2010/main:CT_Photo`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `imgLayer` | `CT_PictureLayer` |  |  |  |

## `x:drawing/2010/main:<imgProps>` : `CT_Photo`

## `x:drawing/2010/main:CT_UseLocalDpi`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `boolean` | optional | true |  |

## `x:drawing/2010/main:<useLocalDpi>` : `CT_UseLocalDpi`

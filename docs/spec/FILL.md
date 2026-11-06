# FILL: Fills

Generated. Tier 1E; render 2B; edit 4; level Edit. 125 rows.


## `a:CT_BackgroundFormatting`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_FillProperties` | `EG_FillProperties` | 0..1 |  |  |
| group | `EG_EffectProperties` | `EG_EffectProperties` | 0..1 |  |  |

## `a:CT_WholeE2oFormatting`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ln` | `CT_LineProperties` | 0..1 |  |  |
| group | `EG_EffectProperties` | `EG_EffectProperties` | 0..1 |  |  |

## `a:CT_NoFillProperties`

## `a:CT_SolidColorFillProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorChoice` | `EG_ColorChoice` | 0..1 |  |  |

## `a:CT_LinearShadeProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@ang` | `ST_PositiveFixedAngle` | optional |  |  |
| attribute | `@scaled` | `boolean` | optional |  |  |

## `a:ST_PathShadeType`
Restriction: `base=token`
- `shape`
- `circle`
- `rect`

## `a:CT_PathShadeProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `fillToRect` | `CT_RelativeRect` | 0..1 |  |  |
| attribute | `@path` | `ST_PathShadeType` | optional |  |  |

## `a:EG_ShadeProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `lin` | `CT_LinearShadeProperties` | choice |  |  |
| element | `path` | `CT_PathShadeProperties` | choice |  |  |

## `a:ST_TileFlipMode`
Restriction: `base=token`
- `none`
- `x`
- `y`
- `xy`

## `a:CT_GradientStop`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorChoice` | `EG_ColorChoice` |  |  |  |
| attribute | `@pos` | `ST_PositiveFixedPercentage` | required |  |  |

## `a:CT_GradientStopList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `gs` | `CT_GradientStop` | 2..unbounded |  |  |

## `a:CT_GradientFillProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `gsLst` | `CT_GradientStopList` | 0..1 |  |  |
| group | `EG_ShadeProperties` | `EG_ShadeProperties` | 0..1 |  |  |
| element | `tileRect` | `CT_RelativeRect` | 0..1 |  |  |
| attribute | `@flip` | `ST_TileFlipMode` | optional | none |  |
| attribute | `@rotWithShape` | `boolean` | optional |  |  |

## `a:CT_TileInfoProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@tx` | `ST_Coordinate` | optional |  |  |
| attribute | `@ty` | `ST_Coordinate` | optional |  |  |
| attribute | `@sx` | `ST_Percentage` | optional |  |  |
| attribute | `@sy` | `ST_Percentage` | optional |  |  |
| attribute | `@flip` | `ST_TileFlipMode` | optional | none |  |
| attribute | `@algn` | `ST_RectAlignment` | optional |  |  |

## `a:CT_StretchInfoProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `fillRect` | `CT_RelativeRect` | 0..1 |  |  |

## `a:EG_FillModeProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tile` | `CT_TileInfoProperties` | choice |  |  |
| element | `stretch` | `CT_StretchInfoProperties` | choice |  |  |

## `a:CT_BlipFillProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `blip` | `CT_Blip` | 0..1 |  |  |
| element | `srcRect` | `CT_RelativeRect` | 0..1 |  |  |
| group | `EG_FillModeProperties` | `EG_FillModeProperties` | 0..1 |  |  |
| attribute | `@dpi` | `unsignedInt` | optional |  |  |
| attribute | `@rotWithShape` | `boolean` | optional |  |  |

## `a:ST_PresetPatternVal`
Restriction: `base=token`
- `pct5`
- `pct10`
- `pct20`
- `pct25`
- `pct30`
- `pct40`
- `pct50`
- `pct60`
- `pct70`
- `pct75`
- `pct80`
- `pct90`
- `horz`
- `vert`
- `ltHorz`
- `ltVert`
- `dkHorz`
- `dkVert`
- `narHorz`
- `narVert`
- `dashHorz`
- `dashVert`
- `cross`
- `dnDiag`
- `upDiag`
- `ltDnDiag`
- `ltUpDiag`
- `dkDnDiag`
- `dkUpDiag`
- `wdDnDiag`
- `wdUpDiag`
- `dashDnDiag`
- `dashUpDiag`
- `diagCross`
- `smCheck`
- `lgCheck`
- `smGrid`
- `lgGrid`
- `dotGrid`
- `smConfetti`
- `lgConfetti`
- `horzBrick`
- `diagBrick`
- `solidDmnd`
- `openDmnd`
- `dotDmnd`
- `plaid`
- `sphere`
- `weave`
- `divot`
- `shingle`
- `wave`
- `trellis`
- `zigZag`

## `a:CT_PatternFillProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `fgClr` | `CT_Color` | 0..1 |  |  |
| element | `bgClr` | `CT_Color` | 0..1 |  |  |
| attribute | `@prst` | `ST_PresetPatternVal` | optional |  |  |

## `a:CT_GroupFillProperties`

## `a:EG_FillProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `noFill` | `CT_NoFillProperties` | choice |  |  |
| element | `solidFill` | `CT_SolidColorFillProperties` | choice |  |  |
| element | `gradFill` | `CT_GradientFillProperties` | choice |  |  |
| element | `blipFill` | `CT_BlipFillProperties` | choice |  |  |
| element | `pattFill` | `CT_PatternFillProperties` | choice |  |  |
| element | `grpFill` | `CT_GroupFillProperties` | choice |  |  |

## `a:CT_FillProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_FillProperties` | `EG_FillProperties` |  |  |  |

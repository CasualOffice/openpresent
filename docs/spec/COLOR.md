# COLOR: Colour model and transforms

Generated. Tier 1C; render 2A; edit 4; level Edit. 314 rows.


## `a:CT_ComplementTransform`

## `a:CT_InverseTransform`

## `a:CT_GrayscaleTransform`

## `a:CT_GammaTransform`

## `a:CT_InverseGammaTransform`

## `a:EG_ColorTransform`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tint` | `CT_PositiveFixedPercentage` | choice |  |  |
| element | `shade` | `CT_PositiveFixedPercentage` | choice |  |  |
| element | `comp` | `CT_ComplementTransform` | choice |  |  |
| element | `inv` | `CT_InverseTransform` | choice |  |  |
| element | `gray` | `CT_GrayscaleTransform` | choice |  |  |
| element | `alpha` | `CT_PositiveFixedPercentage` | choice |  |  |
| element | `alphaOff` | `CT_FixedPercentage` | choice |  |  |
| element | `alphaMod` | `CT_PositivePercentage` | choice |  |  |
| element | `hue` | `CT_PositiveFixedAngle` | choice |  |  |
| element | `hueOff` | `CT_Angle` | choice |  |  |
| element | `hueMod` | `CT_PositivePercentage` | choice |  |  |
| element | `sat` | `CT_Percentage` | choice |  |  |
| element | `satOff` | `CT_Percentage` | choice |  |  |
| element | `satMod` | `CT_Percentage` | choice |  |  |
| element | `lum` | `CT_Percentage` | choice |  |  |
| element | `lumOff` | `CT_Percentage` | choice |  |  |
| element | `lumMod` | `CT_Percentage` | choice |  |  |
| element | `red` | `CT_Percentage` | choice |  |  |
| element | `redOff` | `CT_Percentage` | choice |  |  |
| element | `redMod` | `CT_Percentage` | choice |  |  |
| element | `green` | `CT_Percentage` | choice |  |  |
| element | `greenOff` | `CT_Percentage` | choice |  |  |
| element | `greenMod` | `CT_Percentage` | choice |  |  |
| element | `blue` | `CT_Percentage` | choice |  |  |
| element | `blueOff` | `CT_Percentage` | choice |  |  |
| element | `blueMod` | `CT_Percentage` | choice |  |  |
| element | `gamma` | `CT_GammaTransform` | choice |  |  |
| element | `invGamma` | `CT_InverseGammaTransform` | choice |  |  |

## `a:CT_ScRgbColor`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorTransform` | `EG_ColorTransform` | 0..unbounded |  |  |
| attribute | `@r` | `ST_Percentage` | required |  |  |
| attribute | `@g` | `ST_Percentage` | required |  |  |
| attribute | `@b` | `ST_Percentage` | required |  |  |

## `a:CT_SRgbColor`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorTransform` | `EG_ColorTransform` | 0..unbounded |  |  |
| attribute | `@val` | `ST_HexColorRGB` | required |  |  |

## `a:CT_HslColor`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorTransform` | `EG_ColorTransform` | 0..unbounded |  |  |
| attribute | `@hue` | `ST_PositiveFixedAngle` | required |  |  |
| attribute | `@sat` | `ST_Percentage` | required |  |  |
| attribute | `@lum` | `ST_Percentage` | required |  |  |

## `a:ST_SystemColorVal`
Restriction: `base=token`
- `scrollBar`
- `background`
- `activeCaption`
- `inactiveCaption`
- `menu`
- `window`
- `windowFrame`
- `menuText`
- `windowText`
- `captionText`
- `activeBorder`
- `inactiveBorder`
- `appWorkspace`
- `highlight`
- `highlightText`
- `btnFace`
- `btnShadow`
- `grayText`
- `btnText`
- `inactiveCaptionText`
- `btnHighlight`
- `3dDkShadow`
- `3dLight`
- `infoText`
- `infoBk`
- `hotLight`
- `gradientActiveCaption`
- `gradientInactiveCaption`
- `menuHighlight`
- `menuBar`

## `a:CT_SystemColor`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorTransform` | `EG_ColorTransform` | 0..unbounded |  |  |
| attribute | `@val` | `ST_SystemColorVal` | required |  |  |
| attribute | `@lastClr` | `ST_HexColorRGB` | optional |  |  |

## `a:ST_SchemeColorVal`
Restriction: `base=token`
- `bg1`
- `tx1`
- `bg2`
- `tx2`
- `accent1`
- `accent2`
- `accent3`
- `accent4`
- `accent5`
- `accent6`
- `hlink`
- `folHlink`
- `phClr`
- `dk1`
- `lt1`
- `dk2`
- `lt2`

## `a:CT_SchemeColor`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorTransform` | `EG_ColorTransform` | 0..unbounded |  |  |
| attribute | `@val` | `ST_SchemeColorVal` | required |  |  |

## `a:ST_PresetColorVal`
Restriction: `base=token`
- `aliceBlue`
- `antiqueWhite`
- `aqua`
- `aquamarine`
- `azure`
- `beige`
- `bisque`
- `black`
- `blanchedAlmond`
- `blue`
- `blueViolet`
- `brown`
- `burlyWood`
- `cadetBlue`
- `chartreuse`
- `chocolate`
- `coral`
- `cornflowerBlue`
- `cornsilk`
- `crimson`
- `cyan`
- `darkBlue`
- `darkCyan`
- `darkGoldenrod`
- `darkGray`
- `darkGrey`
- `darkGreen`
- `darkKhaki`
- `darkMagenta`
- `darkOliveGreen`
- `darkOrange`
- `darkOrchid`
- `darkRed`
- `darkSalmon`
- `darkSeaGreen`
- `darkSlateBlue`
- `darkSlateGray`
- `darkSlateGrey`
- `darkTurquoise`
- `darkViolet`
- `dkBlue`
- `dkCyan`
- `dkGoldenrod`
- `dkGray`
- `dkGrey`
- `dkGreen`
- `dkKhaki`
- `dkMagenta`
- `dkOliveGreen`
- `dkOrange`
- `dkOrchid`
- `dkRed`
- `dkSalmon`
- `dkSeaGreen`
- `dkSlateBlue`
- `dkSlateGray`
- `dkSlateGrey`
- `dkTurquoise`
- `dkViolet`
- `deepPink`
- `deepSkyBlue`
- `dimGray`
- `dimGrey`
- `dodgerBlue`
- `firebrick`
- `floralWhite`
- `forestGreen`
- `fuchsia`
- `gainsboro`
- `ghostWhite`
- `gold`
- `goldenrod`
- `gray`
- `grey`
- `green`
- `greenYellow`
- `honeydew`
- `hotPink`
- `indianRed`
- `indigo`
- `ivory`
- `khaki`
- `lavender`
- `lavenderBlush`
- `lawnGreen`
- `lemonChiffon`
- `lightBlue`
- `lightCoral`
- `lightCyan`
- `lightGoldenrodYellow`
- `lightGray`
- `lightGrey`
- `lightGreen`
- `lightPink`
- `lightSalmon`
- `lightSeaGreen`
- `lightSkyBlue`
- `lightSlateGray`
- `lightSlateGrey`
- `lightSteelBlue`
- `lightYellow`
- `ltBlue`
- `ltCoral`
- `ltCyan`
- `ltGoldenrodYellow`
- `ltGray`
- `ltGrey`
- `ltGreen`
- `ltPink`
- `ltSalmon`
- `ltSeaGreen`
- `ltSkyBlue`
- `ltSlateGray`
- `ltSlateGrey`
- `ltSteelBlue`
- `ltYellow`
- `lime`
- `limeGreen`
- `linen`
- `magenta`
- `maroon`
- `medAquamarine`
- `medBlue`
- `medOrchid`
- `medPurple`
- `medSeaGreen`
- `medSlateBlue`
- `medSpringGreen`
- `medTurquoise`
- `medVioletRed`
- `mediumAquamarine`
- `mediumBlue`
- `mediumOrchid`
- `mediumPurple`
- `mediumSeaGreen`
- `mediumSlateBlue`
- `mediumSpringGreen`
- `mediumTurquoise`
- `mediumVioletRed`
- `midnightBlue`
- `mintCream`
- `mistyRose`
- `moccasin`
- `navajoWhite`
- `navy`
- `oldLace`
- `olive`
- `oliveDrab`
- `orange`
- `orangeRed`
- `orchid`
- `paleGoldenrod`
- `paleGreen`
- `paleTurquoise`
- `paleVioletRed`
- `papayaWhip`
- `peachPuff`
- `peru`
- `pink`
- `plum`
- `powderBlue`
- `purple`
- `red`
- `rosyBrown`
- `royalBlue`
- `saddleBrown`
- `salmon`
- `sandyBrown`
- `seaGreen`
- `seaShell`
- `sienna`
- `silver`
- `skyBlue`
- `slateBlue`
- `slateGray`
- `slateGrey`
- `snow`
- `springGreen`
- `steelBlue`
- `tan`
- `teal`
- `thistle`
- `tomato`
- `turquoise`
- `violet`
- `wheat`
- `white`
- `whiteSmoke`
- `yellow`
- `yellowGreen`

## `a:CT_PresetColor`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorTransform` | `EG_ColorTransform` | 0..unbounded |  |  |
| attribute | `@val` | `ST_PresetColorVal` | required |  |  |

## `a:EG_ColorChoice`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `scrgbClr` | `CT_ScRgbColor` | choice |  |  |
| element | `srgbClr` | `CT_SRgbColor` | choice |  |  |
| element | `hslClr` | `CT_HslColor` | choice |  |  |
| element | `sysClr` | `CT_SystemColor` | choice |  |  |
| element | `schemeClr` | `CT_SchemeColor` | choice |  |  |
| element | `prstClr` | `CT_PresetColor` | choice |  |  |

## `a:CT_Color`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorChoice` | `EG_ColorChoice` |  |  |  |

## `a:CT_ColorMRU`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorChoice` | `EG_ColorChoice` | 0..unbounded |  |  |

## `a:CT_ColorChangeEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `clrFrom` | `CT_Color` |  |  |  |
| element | `clrTo` | `CT_Color` |  |  |  |
| attribute | `@useA` | `boolean` | optional | true |  |

## `a:CT_ColorReplaceEffect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorChoice` | `EG_ColorChoice` |  |  |  |

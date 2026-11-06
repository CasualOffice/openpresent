# PIC: Pictures and image references

Generated. Tier 1H; render 2E; edit 4; level Edit. 69 rows.


## `a:CT_RelativeRect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@l` | `ST_Percentage` | optional | 0% |  |
| attribute | `@t` | `ST_Percentage` | optional | 0% |  |
| attribute | `@r` | `ST_Percentage` | optional | 0% |  |
| attribute | `@b` | `ST_Percentage` | optional | 0% |  |

## `a:AG_Blob`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@r:embed` | `embed` | optional |  |  |
| attribute | `@r:link` | `link` | optional |  |  |

## `a:CT_PictureLocking`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attrgroup | `AG_Locking` | `AG_Locking` |  |  |  |
| attribute | `@noCrop` | `boolean` | optional | false |  |

## `a:CT_NonVisualPictureProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `picLocks` | `CT_PictureLocking` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@preferRelativeResize` | `boolean` | optional | true |  |

## `a:CT_GvmlPictureNonVisual`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cNvPr` | `CT_NonVisualDrawingProps` |  |  |  |
| element | `cNvPicPr` | `CT_NonVisualPictureProperties` |  |  |  |

## `a:CT_GvmlPicture`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `nvPicPr` | `CT_GvmlPictureNonVisual` |  |  |  |
| element | `blipFill` | `CT_BlipFillProperties` |  |  |  |
| element | `spPr` | `CT_ShapeProperties` |  |  |  |
| element | `style` | `CT_ShapeStyle` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `a:ST_BlipCompression`
Restriction: `base=token`
- `email`
- `screen`
- `print`
- `hqprint`
- `none`

## `a:CT_Blip`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `alphaBiLevel` | `CT_AlphaBiLevelEffect` | choice |  |  |
| element | `alphaCeiling` | `CT_AlphaCeilingEffect` | choice |  |  |
| element | `alphaFloor` | `CT_AlphaFloorEffect` | choice |  |  |
| element | `alphaInv` | `CT_AlphaInverseEffect` | choice |  |  |
| element | `alphaMod` | `CT_AlphaModulateEffect` | choice |  |  |
| element | `alphaModFix` | `CT_AlphaModulateFixedEffect` | choice |  |  |
| element | `alphaRepl` | `CT_AlphaReplaceEffect` | choice |  |  |
| element | `biLevel` | `CT_BiLevelEffect` | choice |  |  |
| element | `blur` | `CT_BlurEffect` | choice |  |  |
| element | `clrChange` | `CT_ColorChangeEffect` | choice |  |  |
| element | `clrRepl` | `CT_ColorReplaceEffect` | choice |  |  |
| element | `duotone` | `CT_DuotoneEffect` | choice |  |  |
| element | `fillOverlay` | `CT_FillOverlayEffect` | choice |  |  |
| element | `grayscl` | `CT_GrayscaleEffect` | choice |  |  |
| element | `hsl` | `CT_HSLEffect` | choice |  |  |
| element | `lum` | `CT_LuminanceEffect` | choice |  |  |
| element | `tint` | `CT_TintEffect` | choice |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attrgroup | `AG_Blob` | `AG_Blob` |  |  |  |
| attribute | `@cstate` | `ST_BlipCompression` | optional | none |  |

## `a:<blip>` : `CT_Blip`

## `pic:CT_PictureNonVisual`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cNvPr` | `CT_NonVisualDrawingProps` |  |  |  |
| element | `cNvPicPr` | `CT_NonVisualPictureProperties` |  |  |  |

## `pic:CT_Picture`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `nvPicPr` | `CT_PictureNonVisual` |  |  |  |
| element | `blipFill` | `CT_BlipFillProperties` |  |  |  |
| element | `spPr` | `CT_ShapeProperties` |  |  |  |

## `pic:<pic>` : `CT_Picture`

## `x:drawing/2010/picture:<style>` : `CT_ShapeStyle`

## `x:drawing/2010/picture:<extLst>` : `CT_OfficeArtExtensionList`

## `x:drawing/2016/SVG/main:CT_SVGBlip`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_Blob` | `AG_Blob` |  |  |  |

## `x:drawing/2016/SVG/main:<svgBlip>` : `CT_SVGBlip`

## `x:drawing/2022/imageformula:CT_ImageFormula`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@formula` | `string` | optional |  |  |

## `x:drawing/2022/imageformula:<imageFormula>` : `CT_ImageFormula`

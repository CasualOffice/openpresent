# TABLE: Tables and table styles

Generated. Tier 1G; render 2E; edit 6; level Edit. 113 rows.


## `a:CT_TableCellProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `lnL` | `CT_LineProperties` | 0..1 |  |  |
| element | `lnR` | `CT_LineProperties` | 0..1 |  |  |
| element | `lnT` | `CT_LineProperties` | 0..1 |  |  |
| element | `lnB` | `CT_LineProperties` | 0..1 |  |  |
| element | `lnTlToBr` | `CT_LineProperties` | 0..1 |  |  |
| element | `lnBlToTr` | `CT_LineProperties` | 0..1 |  |  |
| element | `cell3D` | `CT_Cell3D` | 0..1 |  |  |
| group | `EG_FillProperties` | `EG_FillProperties` | 0..1 |  |  |
| element | `headers` | `CT_Headers` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@marL` | `ST_Coordinate32` | optional | 91440 |  |
| attribute | `@marR` | `ST_Coordinate32` | optional | 91440 |  |
| attribute | `@marT` | `ST_Coordinate32` | optional | 45720 |  |
| attribute | `@marB` | `ST_Coordinate32` | optional | 45720 |  |
| attribute | `@vert` | `ST_TextVerticalType` | optional | horz |  |
| attribute | `@anchor` | `ST_TextAnchoringType` | optional | t |  |
| attribute | `@anchorCtr` | `boolean` | optional | false |  |
| attribute | `@horzOverflow` | `ST_TextHorzOverflowType` | optional | clip |  |

## `a:CT_Headers`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `header` | `string` |  |  |  |

## `a:CT_TableCol`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@w` | `ST_Coordinate` | required |  |  |

## `a:CT_TableGrid`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `gridCol` | `CT_TableCol` | 0..unbounded |  |  |

## `a:CT_TableCell`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `txBody` | `CT_TextBody` | 0..1 |  |  |
| element | `tcPr` | `CT_TableCellProperties` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@rowSpan` | `int` | optional | 1 |  |
| attribute | `@gridSpan` | `int` | optional | 1 |  |
| attribute | `@hMerge` | `boolean` | optional | false |  |
| attribute | `@vMerge` | `boolean` | optional | false |  |
| attribute | `@id` | `string` | optional |  |  |

## `a:CT_TableRow`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tc` | `CT_TableCell` | 0..unbounded |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@h` | `ST_Coordinate` | required |  |  |

## `a:CT_TableProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_FillProperties` | `EG_FillProperties` | 0..1 |  |  |
| group | `EG_EffectProperties` | `EG_EffectProperties` | 0..1 |  |  |
| element | `tableStyle` | `CT_TableStyle` | choice |  |  |
| element | `tableStyleId` | `ST_Guid` | choice |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@rtl` | `boolean` | optional | false |  |
| attribute | `@firstRow` | `boolean` | optional | false |  |
| attribute | `@firstCol` | `boolean` | optional | false |  |
| attribute | `@lastRow` | `boolean` | optional | false |  |
| attribute | `@lastCol` | `boolean` | optional | false |  |
| attribute | `@bandRow` | `boolean` | optional | false |  |
| attribute | `@bandCol` | `boolean` | optional | false |  |

## `a:CT_Table`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tblPr` | `CT_TableProperties` | 0..1 |  |  |
| element | `tblGrid` | `CT_TableGrid` |  |  |  |
| element | `tr` | `CT_TableRow` | 0..unbounded |  |  |

## `a:<tbl>` : `CT_Table`

## `a:CT_Cell3D`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `bevel` | `CT_Bevel` |  |  |  |
| element | `lightRig` | `CT_LightRig` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@prstMaterial` | `ST_PresetMaterialType` | optional | plastic |  |

## `a:CT_ThemeableLineStyle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ln` | `CT_LineProperties` | choice |  |  |
| element | `lnRef` | `CT_StyleMatrixReference` | choice |  |  |

## `a:CT_TableStyleTextStyle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ThemeableFontStyles` | `EG_ThemeableFontStyles` | 0..1 |  |  |
| group | `EG_ColorChoice` | `EG_ColorChoice` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@b` | `ST_OnOffStyleType` | optional | def |  |
| attribute | `@i` | `ST_OnOffStyleType` | optional | def |  |

## `a:CT_TableCellBorderStyle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `left` | `CT_ThemeableLineStyle` | 0..1 |  |  |
| element | `right` | `CT_ThemeableLineStyle` | 0..1 |  |  |
| element | `top` | `CT_ThemeableLineStyle` | 0..1 |  |  |
| element | `bottom` | `CT_ThemeableLineStyle` | 0..1 |  |  |
| element | `insideH` | `CT_ThemeableLineStyle` | 0..1 |  |  |
| element | `insideV` | `CT_ThemeableLineStyle` | 0..1 |  |  |
| element | `tl2br` | `CT_ThemeableLineStyle` | 0..1 |  |  |
| element | `tr2bl` | `CT_ThemeableLineStyle` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `a:CT_TableBackgroundStyle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ThemeableFillStyle` | `EG_ThemeableFillStyle` | 0..1 |  |  |
| group | `EG_ThemeableEffectStyle` | `EG_ThemeableEffectStyle` | 0..1 |  |  |

## `a:CT_TableStyleCellStyle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tcBdr` | `CT_TableCellBorderStyle` | 0..1 |  |  |
| group | `EG_ThemeableFillStyle` | `EG_ThemeableFillStyle` | 0..1 |  |  |
| element | `cell3D` | `CT_Cell3D` | 0..1 |  |  |

## `a:CT_TablePartStyle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tcTxStyle` | `CT_TableStyleTextStyle` | 0..1 |  |  |
| element | `tcStyle` | `CT_TableStyleCellStyle` | 0..1 |  |  |

## `a:CT_TableStyle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tblBg` | `CT_TableBackgroundStyle` | 0..1 |  |  |
| element | `wholeTbl` | `CT_TablePartStyle` | 0..1 |  |  |
| element | `band1H` | `CT_TablePartStyle` | 0..1 |  |  |
| element | `band2H` | `CT_TablePartStyle` | 0..1 |  |  |
| element | `band1V` | `CT_TablePartStyle` | 0..1 |  |  |
| element | `band2V` | `CT_TablePartStyle` | 0..1 |  |  |
| element | `lastCol` | `CT_TablePartStyle` | 0..1 |  |  |
| element | `firstCol` | `CT_TablePartStyle` | 0..1 |  |  |
| element | `lastRow` | `CT_TablePartStyle` | 0..1 |  |  |
| element | `seCell` | `CT_TablePartStyle` | 0..1 |  |  |
| element | `swCell` | `CT_TablePartStyle` | 0..1 |  |  |
| element | `firstRow` | `CT_TablePartStyle` | 0..1 |  |  |
| element | `neCell` | `CT_TablePartStyle` | 0..1 |  |  |
| element | `nwCell` | `CT_TablePartStyle` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@styleId` | `ST_Guid` | required |  |  |
| attribute | `@styleName` | `string` | required |  |  |

## `a:CT_TableStyleList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tblStyle` | `CT_TableStyle` | 0..unbounded |  |  |
| attribute | `@def` | `ST_Guid` | required |  |  |

## `a:<tblStyleLst>` : `CT_TableStyleList`

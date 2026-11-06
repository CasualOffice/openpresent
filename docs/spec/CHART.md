# CHART: Charts

Generated. Tier 1J; render 2F; edit 6; level Edit. 1217 rows.


## `c:CT_Boolean`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `boolean` | optional | true |  |

## `c:CT_Double`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `double` | required |  |  |

## `c:CT_UnsignedInt`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `unsignedInt` | required |  |  |

## `c:CT_RelId`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@r:id` | `id` | required |  |  |

## `c:CT_Extension`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| any | `xsd:any` | `` |  |  | lax |
| attribute | `@uri` | `token` | optional |  |  |

## `c:CT_ExtensionList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ext` | `CT_Extension` | 0..unbounded |  |  |

## `c:CT_NumVal`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `v` | `ST_Xstring` |  |  |  |
| attribute | `@idx` | `unsignedInt` | required |  |  |
| attribute | `@formatCode` | `ST_Xstring` | optional |  |  |

## `c:CT_NumData`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `formatCode` | `ST_Xstring` | 0..1 |  |  |
| element | `ptCount` | `CT_UnsignedInt` | 0..1 |  |  |
| element | `pt` | `CT_NumVal` | 0..unbounded |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_NumRef`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `f` | `string` |  |  |  |
| element | `numCache` | `CT_NumData` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_NumDataSource`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `numRef` | `CT_NumRef` | choice |  |  |
| element | `numLit` | `CT_NumData` | choice |  |  |

## `c:CT_StrVal`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `v` | `ST_Xstring` |  |  |  |
| attribute | `@idx` | `unsignedInt` | required |  |  |

## `c:CT_StrData`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ptCount` | `CT_UnsignedInt` | 0..1 |  |  |
| element | `pt` | `CT_StrVal` | 0..unbounded |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_StrRef`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `f` | `string` |  |  |  |
| element | `strCache` | `CT_StrData` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_Tx`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `strRef` | `CT_StrRef` | choice |  |  |
| element | `rich` | `CT_TextBody` | choice |  |  |

## `c:CT_TextLanguageID`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_Lang` | required |  |  |

## `c:CT_Lvl`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `pt` | `CT_StrVal` | 0..unbounded |  |  |

## `c:CT_MultiLvlStrData`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ptCount` | `CT_UnsignedInt` | 0..1 |  |  |
| element | `lvl` | `CT_Lvl` | 0..unbounded |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_MultiLvlStrRef`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `f` | `string` |  |  |  |
| element | `multiLvlStrCache` | `CT_MultiLvlStrData` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_AxDataSource`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `multiLvlStrRef` | `CT_MultiLvlStrRef` | choice |  |  |
| element | `numRef` | `CT_NumRef` | choice |  |  |
| element | `numLit` | `CT_NumData` | choice |  |  |
| element | `strRef` | `CT_StrRef` | choice |  |  |
| element | `strLit` | `CT_StrData` | choice |  |  |

## `c:CT_SerTx`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `strRef` | `CT_StrRef` | choice |  |  |
| element | `v` | `ST_Xstring` | choice |  |  |

## `c:ST_LayoutTarget`
Restriction: `base=string`
- `inner`
- `outer`

## `c:CT_LayoutTarget`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_LayoutTarget` | optional | outer |  |

## `c:ST_LayoutMode`
Restriction: `base=string`
- `edge`
- `factor`

## `c:CT_LayoutMode`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_LayoutMode` | optional | factor |  |

## `c:CT_ManualLayout`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `layoutTarget` | `CT_LayoutTarget` | 0..1 |  |  |
| element | `xMode` | `CT_LayoutMode` | 0..1 |  |  |
| element | `yMode` | `CT_LayoutMode` | 0..1 |  |  |
| element | `wMode` | `CT_LayoutMode` | 0..1 |  |  |
| element | `hMode` | `CT_LayoutMode` | 0..1 |  |  |
| element | `x` | `CT_Double` | 0..1 |  |  |
| element | `y` | `CT_Double` | 0..1 |  |  |
| element | `w` | `CT_Double` | 0..1 |  |  |
| element | `h` | `CT_Double` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_Layout`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `manualLayout` | `CT_ManualLayout` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_Title`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tx` | `CT_Tx` | 0..1 |  |  |
| element | `layout` | `CT_Layout` | 0..1 |  |  |
| element | `overlay` | `CT_Boolean` | 0..1 |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `txPr` | `CT_TextBody` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:ST_RotX`
Restriction: `base=byte minInclusive=-90 maxInclusive=90`

## `c:CT_RotX`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_RotX` | optional | 0 |  |

## `c:ST_HPercent`
Restriction: `union=ST_HPercentWithSymbol ST_HPercentUShort`

## `c:ST_HPercentWithSymbol`
Restriction: `base=string pattern=0*(([5-9])\|([1-9][0-9])\|([1-4][0-9][0-9])\|500)%`

## `c:ST_HPercentUShort`
Restriction: `base=unsignedShort minInclusive=5 maxInclusive=500`

## `c:CT_HPercent`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_HPercent` | optional | 100% |  |

## `c:ST_RotY`
Restriction: `base=unsignedShort minInclusive=0 maxInclusive=360`

## `c:CT_RotY`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_RotY` | optional | 0 |  |

## `c:ST_DepthPercent`
Restriction: `union=ST_DepthPercentWithSymbol ST_DepthPercentUShort`

## `c:ST_DepthPercentWithSymbol`
Restriction: `base=string pattern=0*(([2-9][0-9])\|([1-9][0-9][0-9])\|(1[0-9][0-9][0-9])\|2000)%`

## `c:ST_DepthPercentUShort`
Restriction: `base=unsignedShort minInclusive=20 maxInclusive=2000`

## `c:CT_DepthPercent`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_DepthPercent` | optional | 100% |  |

## `c:ST_Perspective`
Restriction: `base=unsignedByte minInclusive=0 maxInclusive=240`

## `c:CT_Perspective`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_Perspective` | optional | 30 |  |

## `c:CT_View3D`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `rotX` | `CT_RotX` | 0..1 |  |  |
| element | `hPercent` | `CT_HPercent` | 0..1 |  |  |
| element | `rotY` | `CT_RotY` | 0..1 |  |  |
| element | `depthPercent` | `CT_DepthPercent` | 0..1 |  |  |
| element | `rAngAx` | `CT_Boolean` | 0..1 |  |  |
| element | `perspective` | `CT_Perspective` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_Surface`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `thickness` | `CT_Thickness` | 0..1 |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `pictureOptions` | `CT_PictureOptions` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:ST_Thickness`
Restriction: `union=ST_ThicknessPercent unsignedInt`

## `c:ST_ThicknessPercent`
Restriction: `base=string pattern=([0-9]+)%`

## `c:CT_Thickness`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_Thickness` | required |  |  |

## `c:CT_DTable`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `showHorzBorder` | `CT_Boolean` | 0..1 |  |  |
| element | `showVertBorder` | `CT_Boolean` | 0..1 |  |  |
| element | `showOutline` | `CT_Boolean` | 0..1 |  |  |
| element | `showKeys` | `CT_Boolean` | 0..1 |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `txPr` | `CT_TextBody` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:ST_GapAmount`
Restriction: `union=ST_GapAmountPercent ST_GapAmountUShort`

## `c:ST_GapAmountPercent`
Restriction: `base=string pattern=0*(([0-9])\|([1-9][0-9])\|([1-4][0-9][0-9])\|500)%`

## `c:ST_GapAmountUShort`
Restriction: `base=unsignedShort minInclusive=0 maxInclusive=500`

## `c:CT_GapAmount`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_GapAmount` | optional | 150% |  |

## `c:ST_Overlap`
Restriction: `union=ST_OverlapPercent ST_OverlapByte`

## `c:ST_OverlapPercent`
Restriction: `base=string pattern=(-?0*(([0-9])\|([1-9][0-9])\|100))%`

## `c:ST_OverlapByte`
Restriction: `base=byte minInclusive=-100 maxInclusive=100`

## `c:CT_Overlap`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_Overlap` | optional | 0% |  |

## `c:ST_BubbleScale`
Restriction: `union=ST_BubbleScalePercent ST_BubbleScaleUInt`

## `c:ST_BubbleScalePercent`
Restriction: `base=string pattern=0*(([0-9])\|([1-9][0-9])\|([1-2][0-9][0-9])\|300)%`

## `c:ST_BubbleScaleUInt`
Restriction: `base=unsignedInt minInclusive=0 maxInclusive=300`

## `c:CT_BubbleScale`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_BubbleScale` | optional | 100% |  |

## `c:ST_SizeRepresents`
Restriction: `base=string`
- `area`
- `w`

## `c:CT_SizeRepresents`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_SizeRepresents` | optional | area |  |

## `c:ST_FirstSliceAng`
Restriction: `base=unsignedShort minInclusive=0 maxInclusive=360`

## `c:CT_FirstSliceAng`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_FirstSliceAng` | optional | 0 |  |

## `c:ST_HoleSize`
Restriction: `union=ST_HoleSizePercent ST_HoleSizeUByte`

## `c:ST_HoleSizePercent`
Restriction: `base=string pattern=0*([1-9]\|([1-8][0-9])\|90)%`

## `c:ST_HoleSizeUByte`
Restriction: `base=unsignedByte minInclusive=1 maxInclusive=90`

## `c:CT_HoleSize`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_HoleSize` | optional | 10% |  |

## `c:ST_SplitType`
Restriction: `base=string`
- `auto`
- `cust`
- `percent`
- `pos`
- `val`

## `c:CT_SplitType`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_SplitType` | optional | auto |  |

## `c:CT_CustSplit`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `secondPiePt` | `CT_UnsignedInt` | 0..unbounded |  |  |

## `c:ST_SecondPieSize`
Restriction: `union=ST_SecondPieSizePercent ST_SecondPieSizeUShort`

## `c:ST_SecondPieSizePercent`
Restriction: `base=string pattern=0*(([5-9])\|([1-9][0-9])\|(1[0-9][0-9])\|200)%`

## `c:ST_SecondPieSizeUShort`
Restriction: `base=unsignedShort minInclusive=5 maxInclusive=200`

## `c:CT_SecondPieSize`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_SecondPieSize` | optional | 75% |  |

## `c:CT_NumFmt`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@formatCode` | `ST_Xstring` | required |  |  |
| attribute | `@sourceLinked` | `boolean` | optional |  |  |

## `c:ST_LblAlgn`
Restriction: `base=string`
- `ctr`
- `l`
- `r`

## `c:CT_LblAlgn`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_LblAlgn` | required |  |  |

## `c:ST_DLblPos`
Restriction: `base=string`
- `bestFit`
- `b`
- `ctr`
- `inBase`
- `inEnd`
- `l`
- `outEnd`
- `r`
- `t`

## `c:CT_DLblPos`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_DLblPos` | required |  |  |

## `c:EG_DLblShared`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `numFmt` | `CT_NumFmt` | 0..1 |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `txPr` | `CT_TextBody` | 0..1 |  |  |
| element | `dLblPos` | `CT_DLblPos` | 0..1 |  |  |
| element | `showLegendKey` | `CT_Boolean` | 0..1 |  |  |
| element | `showVal` | `CT_Boolean` | 0..1 |  |  |
| element | `showCatName` | `CT_Boolean` | 0..1 |  |  |
| element | `showSerName` | `CT_Boolean` | 0..1 |  |  |
| element | `showPercent` | `CT_Boolean` | 0..1 |  |  |
| element | `showBubbleSize` | `CT_Boolean` | 0..1 |  |  |
| element | `separator` | `string` | 0..1 |  |  |

## `c:Group_DLbl`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `layout` | `CT_Layout` | 0..1 |  |  |
| element | `tx` | `CT_Tx` | 0..1 |  |  |
| group | `EG_DLblShared` | `EG_DLblShared` |  |  |  |

## `c:CT_DLbl`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `idx` | `CT_UnsignedInt` |  |  |  |
| element | `delete` | `CT_Boolean` | choice |  |  |
| group | `Group_DLbl` | `Group_DLbl` | choice |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:Group_DLbls`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_DLblShared` | `EG_DLblShared` |  |  |  |
| element | `showLeaderLines` | `CT_Boolean` | 0..1 |  |  |
| element | `leaderLines` | `CT_ChartLines` | 0..1 |  |  |

## `c:CT_DLbls`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `dLbl` | `CT_DLbl` | 0..unbounded |  |  |
| element | `delete` | `CT_Boolean` | choice |  |  |
| group | `Group_DLbls` | `Group_DLbls` | choice |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:ST_MarkerStyle`
Restriction: `base=string`
- `circle`
- `dash`
- `diamond`
- `dot`
- `none`
- `picture`
- `plus`
- `square`
- `star`
- `triangle`
- `x`
- `auto`

## `c:CT_MarkerStyle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_MarkerStyle` | required |  |  |

## `c:ST_MarkerSize`
Restriction: `base=unsignedByte minInclusive=2 maxInclusive=72`

## `c:CT_MarkerSize`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_MarkerSize` | optional | 5 |  |

## `c:CT_Marker`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `symbol` | `CT_MarkerStyle` | 0..1 |  |  |
| element | `size` | `CT_MarkerSize` | 0..1 |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_DPt`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `idx` | `CT_UnsignedInt` |  |  |  |
| element | `invertIfNegative` | `CT_Boolean` | 0..1 |  |  |
| element | `marker` | `CT_Marker` | 0..1 |  |  |
| element | `bubble3D` | `CT_Boolean` | 0..1 |  |  |
| element | `explosion` | `CT_UnsignedInt` | 0..1 |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `pictureOptions` | `CT_PictureOptions` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:ST_TrendlineType`
Restriction: `base=string`
- `exp`
- `linear`
- `log`
- `movingAvg`
- `poly`
- `power`

## `c:CT_TrendlineType`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_TrendlineType` | optional | linear |  |

## `c:ST_Order`
Restriction: `base=unsignedByte minInclusive=2 maxInclusive=6`

## `c:CT_Order`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_Order` | optional | 2 |  |

## `c:ST_Period`
Restriction: `base=unsignedInt minInclusive=2`

## `c:CT_Period`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_Period` | optional | 2 |  |

## `c:CT_TrendlineLbl`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `layout` | `CT_Layout` | 0..1 |  |  |
| element | `tx` | `CT_Tx` | 0..1 |  |  |
| element | `numFmt` | `CT_NumFmt` | 0..1 |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `txPr` | `CT_TextBody` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_Trendline`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `name` | `string` | 0..1 |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `trendlineType` | `CT_TrendlineType` |  |  |  |
| element | `order` | `CT_Order` | 0..1 |  |  |
| element | `period` | `CT_Period` | 0..1 |  |  |
| element | `forward` | `CT_Double` | 0..1 |  |  |
| element | `backward` | `CT_Double` | 0..1 |  |  |
| element | `intercept` | `CT_Double` | 0..1 |  |  |
| element | `dispRSqr` | `CT_Boolean` | 0..1 |  |  |
| element | `dispEq` | `CT_Boolean` | 0..1 |  |  |
| element | `trendlineLbl` | `CT_TrendlineLbl` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:ST_ErrDir`
Restriction: `base=string`
- `x`
- `y`

## `c:CT_ErrDir`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_ErrDir` | required |  |  |

## `c:ST_ErrBarType`
Restriction: `base=string`
- `both`
- `minus`
- `plus`

## `c:CT_ErrBarType`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_ErrBarType` | optional | both |  |

## `c:ST_ErrValType`
Restriction: `base=string`
- `cust`
- `fixedVal`
- `percentage`
- `stdDev`
- `stdErr`

## `c:CT_ErrValType`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_ErrValType` | optional | fixedVal |  |

## `c:CT_ErrBars`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `errDir` | `CT_ErrDir` | 0..1 |  |  |
| element | `errBarType` | `CT_ErrBarType` |  |  |  |
| element | `errValType` | `CT_ErrValType` |  |  |  |
| element | `noEndCap` | `CT_Boolean` | 0..1 |  |  |
| element | `plus` | `CT_NumDataSource` | 0..1 |  |  |
| element | `minus` | `CT_NumDataSource` | 0..1 |  |  |
| element | `val` | `CT_Double` | 0..1 |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_UpDownBar`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |

## `c:CT_UpDownBars`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `gapWidth` | `CT_GapAmount` | 0..1 |  |  |
| element | `upBars` | `CT_UpDownBar` | 0..1 |  |  |
| element | `downBars` | `CT_UpDownBar` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:EG_SerShared`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `idx` | `CT_UnsignedInt` |  |  |  |
| element | `order` | `CT_UnsignedInt` |  |  |  |
| element | `tx` | `CT_SerTx` | 0..1 |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |

## `c:CT_LineSer`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_SerShared` | `EG_SerShared` |  |  |  |
| element | `marker` | `CT_Marker` | 0..1 |  |  |
| element | `dPt` | `CT_DPt` | 0..unbounded |  |  |
| element | `dLbls` | `CT_DLbls` | 0..1 |  |  |
| element | `trendline` | `CT_Trendline` | 0..unbounded |  |  |
| element | `errBars` | `CT_ErrBars` | 0..1 |  |  |
| element | `cat` | `CT_AxDataSource` | 0..1 |  |  |
| element | `val` | `CT_NumDataSource` | 0..1 |  |  |
| element | `smooth` | `CT_Boolean` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_ScatterSer`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_SerShared` | `EG_SerShared` |  |  |  |
| element | `marker` | `CT_Marker` | 0..1 |  |  |
| element | `dPt` | `CT_DPt` | 0..unbounded |  |  |
| element | `dLbls` | `CT_DLbls` | 0..1 |  |  |
| element | `trendline` | `CT_Trendline` | 0..unbounded |  |  |
| element | `errBars` | `CT_ErrBars` | 0..2 |  |  |
| element | `xVal` | `CT_AxDataSource` | 0..1 |  |  |
| element | `yVal` | `CT_NumDataSource` | 0..1 |  |  |
| element | `smooth` | `CT_Boolean` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_RadarSer`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_SerShared` | `EG_SerShared` |  |  |  |
| element | `marker` | `CT_Marker` | 0..1 |  |  |
| element | `dPt` | `CT_DPt` | 0..unbounded |  |  |
| element | `dLbls` | `CT_DLbls` | 0..1 |  |  |
| element | `cat` | `CT_AxDataSource` | 0..1 |  |  |
| element | `val` | `CT_NumDataSource` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_BarSer`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_SerShared` | `EG_SerShared` |  |  |  |
| element | `invertIfNegative` | `CT_Boolean` | 0..1 |  |  |
| element | `pictureOptions` | `CT_PictureOptions` | 0..1 |  |  |
| element | `dPt` | `CT_DPt` | 0..unbounded |  |  |
| element | `dLbls` | `CT_DLbls` | 0..1 |  |  |
| element | `trendline` | `CT_Trendline` | 0..unbounded |  |  |
| element | `errBars` | `CT_ErrBars` | 0..1 |  |  |
| element | `cat` | `CT_AxDataSource` | 0..1 |  |  |
| element | `val` | `CT_NumDataSource` | 0..1 |  |  |
| element | `shape` | `CT_Shape` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_AreaSer`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_SerShared` | `EG_SerShared` |  |  |  |
| element | `pictureOptions` | `CT_PictureOptions` | 0..1 |  |  |
| element | `dPt` | `CT_DPt` | 0..unbounded |  |  |
| element | `dLbls` | `CT_DLbls` | 0..1 |  |  |
| element | `trendline` | `CT_Trendline` | 0..unbounded |  |  |
| element | `errBars` | `CT_ErrBars` | 0..2 |  |  |
| element | `cat` | `CT_AxDataSource` | 0..1 |  |  |
| element | `val` | `CT_NumDataSource` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_PieSer`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_SerShared` | `EG_SerShared` |  |  |  |
| element | `explosion` | `CT_UnsignedInt` | 0..1 |  |  |
| element | `dPt` | `CT_DPt` | 0..unbounded |  |  |
| element | `dLbls` | `CT_DLbls` | 0..1 |  |  |
| element | `cat` | `CT_AxDataSource` | 0..1 |  |  |
| element | `val` | `CT_NumDataSource` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_BubbleSer`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_SerShared` | `EG_SerShared` |  |  |  |
| element | `invertIfNegative` | `CT_Boolean` | 0..1 |  |  |
| element | `dPt` | `CT_DPt` | 0..unbounded |  |  |
| element | `dLbls` | `CT_DLbls` | 0..1 |  |  |
| element | `trendline` | `CT_Trendline` | 0..unbounded |  |  |
| element | `errBars` | `CT_ErrBars` | 0..2 |  |  |
| element | `xVal` | `CT_AxDataSource` | 0..1 |  |  |
| element | `yVal` | `CT_NumDataSource` | 0..1 |  |  |
| element | `bubbleSize` | `CT_NumDataSource` | 0..1 |  |  |
| element | `bubble3D` | `CT_Boolean` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_SurfaceSer`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_SerShared` | `EG_SerShared` |  |  |  |
| element | `cat` | `CT_AxDataSource` | 0..1 |  |  |
| element | `val` | `CT_NumDataSource` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:ST_Grouping`
Restriction: `base=string`
- `percentStacked`
- `standard`
- `stacked`

## `c:CT_Grouping`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_Grouping` | optional | standard |  |

## `c:CT_ChartLines`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |

## `c:EG_LineChartShared`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `grouping` | `CT_Grouping` |  |  |  |
| element | `varyColors` | `CT_Boolean` | 0..1 |  |  |
| element | `ser` | `CT_LineSer` | 0..unbounded |  |  |
| element | `dLbls` | `CT_DLbls` | 0..1 |  |  |
| element | `dropLines` | `CT_ChartLines` | 0..1 |  |  |

## `c:CT_LineChart`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_LineChartShared` | `EG_LineChartShared` |  |  |  |
| element | `hiLowLines` | `CT_ChartLines` | 0..1 |  |  |
| element | `upDownBars` | `CT_UpDownBars` | 0..1 |  |  |
| element | `marker` | `CT_Boolean` | 0..1 |  |  |
| element | `smooth` | `CT_Boolean` | 0..1 |  |  |
| element | `axId` | `CT_UnsignedInt` | 2..2 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_Line3DChart`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_LineChartShared` | `EG_LineChartShared` |  |  |  |
| element | `gapDepth` | `CT_GapAmount` | 0..1 |  |  |
| element | `axId` | `CT_UnsignedInt` | 3..3 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_StockChart`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ser` | `CT_LineSer` | 3..4 |  |  |
| element | `dLbls` | `CT_DLbls` | 0..1 |  |  |
| element | `dropLines` | `CT_ChartLines` | 0..1 |  |  |
| element | `hiLowLines` | `CT_ChartLines` | 0..1 |  |  |
| element | `upDownBars` | `CT_UpDownBars` | 0..1 |  |  |
| element | `axId` | `CT_UnsignedInt` | 2..2 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:ST_ScatterStyle`
Restriction: `base=string`
- `none`
- `line`
- `lineMarker`
- `marker`
- `smooth`
- `smoothMarker`

## `c:CT_ScatterStyle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_ScatterStyle` | optional | marker |  |

## `c:CT_ScatterChart`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `scatterStyle` | `CT_ScatterStyle` |  |  |  |
| element | `varyColors` | `CT_Boolean` | 0..1 |  |  |
| element | `ser` | `CT_ScatterSer` | 0..unbounded |  |  |
| element | `dLbls` | `CT_DLbls` | 0..1 |  |  |
| element | `axId` | `CT_UnsignedInt` | 2..2 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:ST_RadarStyle`
Restriction: `base=string`
- `standard`
- `marker`
- `filled`

## `c:CT_RadarStyle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_RadarStyle` | optional | standard |  |

## `c:CT_RadarChart`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `radarStyle` | `CT_RadarStyle` |  |  |  |
| element | `varyColors` | `CT_Boolean` | 0..1 |  |  |
| element | `ser` | `CT_RadarSer` | 0..unbounded |  |  |
| element | `dLbls` | `CT_DLbls` | 0..1 |  |  |
| element | `axId` | `CT_UnsignedInt` | 2..2 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:ST_BarGrouping`
Restriction: `base=string`
- `percentStacked`
- `clustered`
- `standard`
- `stacked`

## `c:CT_BarGrouping`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_BarGrouping` | optional | clustered |  |

## `c:ST_BarDir`
Restriction: `base=string`
- `bar`
- `col`

## `c:CT_BarDir`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_BarDir` | optional | col |  |

## `c:ST_Shape`
Restriction: `base=string`
- `cone`
- `coneToMax`
- `box`
- `cylinder`
- `pyramid`
- `pyramidToMax`

## `c:CT_Shape`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_Shape` | optional | box |  |

## `c:EG_BarChartShared`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `barDir` | `CT_BarDir` |  |  |  |
| element | `grouping` | `CT_BarGrouping` | 0..1 |  |  |
| element | `varyColors` | `CT_Boolean` | 0..1 |  |  |
| element | `ser` | `CT_BarSer` | 0..unbounded |  |  |
| element | `dLbls` | `CT_DLbls` | 0..1 |  |  |

## `c:CT_BarChart`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_BarChartShared` | `EG_BarChartShared` |  |  |  |
| element | `gapWidth` | `CT_GapAmount` | 0..1 |  |  |
| element | `overlap` | `CT_Overlap` | 0..1 |  |  |
| element | `serLines` | `CT_ChartLines` | 0..unbounded |  |  |
| element | `axId` | `CT_UnsignedInt` | 2..2 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_Bar3DChart`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_BarChartShared` | `EG_BarChartShared` |  |  |  |
| element | `gapWidth` | `CT_GapAmount` | 0..1 |  |  |
| element | `gapDepth` | `CT_GapAmount` | 0..1 |  |  |
| element | `shape` | `CT_Shape` | 0..1 |  |  |
| element | `axId` | `CT_UnsignedInt` | 2..3 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:EG_AreaChartShared`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `grouping` | `CT_Grouping` | 0..1 |  |  |
| element | `varyColors` | `CT_Boolean` | 0..1 |  |  |
| element | `ser` | `CT_AreaSer` | 0..unbounded |  |  |
| element | `dLbls` | `CT_DLbls` | 0..1 |  |  |
| element | `dropLines` | `CT_ChartLines` | 0..1 |  |  |

## `c:CT_AreaChart`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_AreaChartShared` | `EG_AreaChartShared` |  |  |  |
| element | `axId` | `CT_UnsignedInt` | 2..2 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_Area3DChart`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_AreaChartShared` | `EG_AreaChartShared` |  |  |  |
| element | `gapDepth` | `CT_GapAmount` | 0..1 |  |  |
| element | `axId` | `CT_UnsignedInt` | 2..3 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:EG_PieChartShared`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `varyColors` | `CT_Boolean` | 0..1 |  |  |
| element | `ser` | `CT_PieSer` | 0..unbounded |  |  |
| element | `dLbls` | `CT_DLbls` | 0..1 |  |  |

## `c:CT_PieChart`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_PieChartShared` | `EG_PieChartShared` |  |  |  |
| element | `firstSliceAng` | `CT_FirstSliceAng` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_Pie3DChart`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_PieChartShared` | `EG_PieChartShared` |  |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_DoughnutChart`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_PieChartShared` | `EG_PieChartShared` |  |  |  |
| element | `firstSliceAng` | `CT_FirstSliceAng` | 0..1 |  |  |
| element | `holeSize` | `CT_HoleSize` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:ST_OfPieType`
Restriction: `base=string`
- `pie`
- `bar`

## `c:CT_OfPieType`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_OfPieType` | optional | pie |  |

## `c:CT_OfPieChart`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ofPieType` | `CT_OfPieType` |  |  |  |
| group | `EG_PieChartShared` | `EG_PieChartShared` |  |  |  |
| element | `gapWidth` | `CT_GapAmount` | 0..1 |  |  |
| element | `splitType` | `CT_SplitType` | 0..1 |  |  |
| element | `splitPos` | `CT_Double` | 0..1 |  |  |
| element | `custSplit` | `CT_CustSplit` | 0..1 |  |  |
| element | `secondPieSize` | `CT_SecondPieSize` | 0..1 |  |  |
| element | `serLines` | `CT_ChartLines` | 0..unbounded |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_BubbleChart`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `varyColors` | `CT_Boolean` | 0..1 |  |  |
| element | `ser` | `CT_BubbleSer` | 0..unbounded |  |  |
| element | `dLbls` | `CT_DLbls` | 0..1 |  |  |
| element | `bubble3D` | `CT_Boolean` | 0..1 |  |  |
| element | `bubbleScale` | `CT_BubbleScale` | 0..1 |  |  |
| element | `showNegBubbles` | `CT_Boolean` | 0..1 |  |  |
| element | `sizeRepresents` | `CT_SizeRepresents` | 0..1 |  |  |
| element | `axId` | `CT_UnsignedInt` | 2..2 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_BandFmt`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `idx` | `CT_UnsignedInt` |  |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |

## `c:CT_BandFmts`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `bandFmt` | `CT_BandFmt` | 0..unbounded |  |  |

## `c:EG_SurfaceChartShared`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `wireframe` | `CT_Boolean` | 0..1 |  |  |
| element | `ser` | `CT_SurfaceSer` | 0..unbounded |  |  |
| element | `bandFmts` | `CT_BandFmts` | 0..1 |  |  |

## `c:CT_SurfaceChart`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_SurfaceChartShared` | `EG_SurfaceChartShared` |  |  |  |
| element | `axId` | `CT_UnsignedInt` | 2..3 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_Surface3DChart`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_SurfaceChartShared` | `EG_SurfaceChartShared` |  |  |  |
| element | `axId` | `CT_UnsignedInt` | 3..3 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:ST_AxPos`
Restriction: `base=string`
- `b`
- `l`
- `r`
- `t`

## `c:CT_AxPos`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_AxPos` | required |  |  |

## `c:ST_Crosses`
Restriction: `base=string`
- `autoZero`
- `max`
- `min`

## `c:CT_Crosses`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_Crosses` | required |  |  |

## `c:ST_CrossBetween`
Restriction: `base=string`
- `between`
- `midCat`

## `c:CT_CrossBetween`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_CrossBetween` | required |  |  |

## `c:ST_TickMark`
Restriction: `base=string`
- `cross`
- `in`
- `none`
- `out`

## `c:CT_TickMark`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_TickMark` | optional | cross |  |

## `c:ST_TickLblPos`
Restriction: `base=string`
- `high`
- `low`
- `nextTo`
- `none`

## `c:CT_TickLblPos`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_TickLblPos` | optional | nextTo |  |

## `c:ST_Skip`
Restriction: `base=unsignedInt minInclusive=1`

## `c:CT_Skip`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_Skip` | required |  |  |

## `c:ST_TimeUnit`
Restriction: `base=string`
- `days`
- `months`
- `years`

## `c:CT_TimeUnit`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_TimeUnit` | optional | days |  |

## `c:ST_AxisUnit`
Restriction: `base=double minExclusive=0`

## `c:CT_AxisUnit`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_AxisUnit` | required |  |  |

## `c:ST_BuiltInUnit`
Restriction: `base=string`
- `hundreds`
- `thousands`
- `tenThousands`
- `hundredThousands`
- `millions`
- `tenMillions`
- `hundredMillions`
- `billions`
- `trillions`

## `c:CT_BuiltInUnit`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_BuiltInUnit` | optional | thousands |  |

## `c:ST_PictureFormat`
Restriction: `base=string`
- `stretch`
- `stack`
- `stackScale`

## `c:CT_PictureFormat`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_PictureFormat` | required |  |  |

## `c:ST_PictureStackUnit`
Restriction: `base=double minExclusive=0`

## `c:CT_PictureStackUnit`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_PictureStackUnit` | required |  |  |

## `c:CT_PictureOptions`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `applyToFront` | `CT_Boolean` | 0..1 |  |  |
| element | `applyToSides` | `CT_Boolean` | 0..1 |  |  |
| element | `applyToEnd` | `CT_Boolean` | 0..1 |  |  |
| element | `pictureFormat` | `CT_PictureFormat` | 0..1 |  |  |
| element | `pictureStackUnit` | `CT_PictureStackUnit` | 0..1 |  |  |

## `c:CT_DispUnitsLbl`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `layout` | `CT_Layout` | 0..1 |  |  |
| element | `tx` | `CT_Tx` | 0..1 |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `txPr` | `CT_TextBody` | 0..1 |  |  |

## `c:CT_DispUnits`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `custUnit` | `CT_Double` | choice |  |  |
| element | `builtInUnit` | `CT_BuiltInUnit` | choice |  |  |
| element | `dispUnitsLbl` | `CT_DispUnitsLbl` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:ST_Orientation`
Restriction: `base=string`
- `maxMin`
- `minMax`

## `c:CT_Orientation`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_Orientation` | optional | minMax |  |

## `c:ST_LogBase`
Restriction: `base=double minInclusive=2 maxInclusive=1000`

## `c:CT_LogBase`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_LogBase` | required |  |  |

## `c:CT_Scaling`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `logBase` | `CT_LogBase` | 0..1 |  |  |
| element | `orientation` | `CT_Orientation` | 0..1 |  |  |
| element | `max` | `CT_Double` | 0..1 |  |  |
| element | `min` | `CT_Double` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:ST_LblOffset`
Restriction: `union=ST_LblOffsetPercent ST_LblOffsetUShort`

## `c:ST_LblOffsetPercent`
Restriction: `base=string pattern=0*(([0-9])\|([1-9][0-9])\|([1-9][0-9][0-9])\|1000)%`

## `c:ST_LblOffsetUShort`
Restriction: `base=unsignedShort minInclusive=0 maxInclusive=1000`

## `c:CT_LblOffset`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_LblOffset` | optional | 100% |  |

## `c:EG_AxShared`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `axId` | `CT_UnsignedInt` |  |  |  |
| element | `scaling` | `CT_Scaling` |  |  |  |
| element | `delete` | `CT_Boolean` | 0..1 |  |  |
| element | `axPos` | `CT_AxPos` |  |  |  |
| element | `majorGridlines` | `CT_ChartLines` | 0..1 |  |  |
| element | `minorGridlines` | `CT_ChartLines` | 0..1 |  |  |
| element | `title` | `CT_Title` | 0..1 |  |  |
| element | `numFmt` | `CT_NumFmt` | 0..1 |  |  |
| element | `majorTickMark` | `CT_TickMark` | 0..1 |  |  |
| element | `minorTickMark` | `CT_TickMark` | 0..1 |  |  |
| element | `tickLblPos` | `CT_TickLblPos` | 0..1 |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `txPr` | `CT_TextBody` | 0..1 |  |  |
| element | `crossAx` | `CT_UnsignedInt` |  |  |  |
| element | `crosses` | `CT_Crosses` | choice |  |  |
| element | `crossesAt` | `CT_Double` | choice |  |  |

## `c:CT_CatAx`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_AxShared` | `EG_AxShared` |  |  |  |
| element | `auto` | `CT_Boolean` | 0..1 |  |  |
| element | `lblAlgn` | `CT_LblAlgn` | 0..1 |  |  |
| element | `lblOffset` | `CT_LblOffset` | 0..1 |  |  |
| element | `tickLblSkip` | `CT_Skip` | 0..1 |  |  |
| element | `tickMarkSkip` | `CT_Skip` | 0..1 |  |  |
| element | `noMultiLvlLbl` | `CT_Boolean` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_DateAx`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_AxShared` | `EG_AxShared` |  |  |  |
| element | `auto` | `CT_Boolean` | 0..1 |  |  |
| element | `lblOffset` | `CT_LblOffset` | 0..1 |  |  |
| element | `baseTimeUnit` | `CT_TimeUnit` | 0..1 |  |  |
| element | `majorUnit` | `CT_AxisUnit` | 0..1 |  |  |
| element | `majorTimeUnit` | `CT_TimeUnit` | 0..1 |  |  |
| element | `minorUnit` | `CT_AxisUnit` | 0..1 |  |  |
| element | `minorTimeUnit` | `CT_TimeUnit` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_SerAx`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_AxShared` | `EG_AxShared` |  |  |  |
| element | `tickLblSkip` | `CT_Skip` | 0..1 |  |  |
| element | `tickMarkSkip` | `CT_Skip` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_ValAx`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_AxShared` | `EG_AxShared` |  |  |  |
| element | `crossBetween` | `CT_CrossBetween` | 0..1 |  |  |
| element | `majorUnit` | `CT_AxisUnit` | 0..1 |  |  |
| element | `minorUnit` | `CT_AxisUnit` | 0..1 |  |  |
| element | `dispUnits` | `CT_DispUnits` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_PlotArea`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `layout` | `CT_Layout` | 0..1 |  |  |
| element | `areaChart` | `CT_AreaChart` | choice |  |  |
| element | `area3DChart` | `CT_Area3DChart` | choice |  |  |
| element | `lineChart` | `CT_LineChart` | choice |  |  |
| element | `line3DChart` | `CT_Line3DChart` | choice |  |  |
| element | `stockChart` | `CT_StockChart` | choice |  |  |
| element | `radarChart` | `CT_RadarChart` | choice |  |  |
| element | `scatterChart` | `CT_ScatterChart` | choice |  |  |
| element | `pieChart` | `CT_PieChart` | choice |  |  |
| element | `pie3DChart` | `CT_Pie3DChart` | choice |  |  |
| element | `doughnutChart` | `CT_DoughnutChart` | choice |  |  |
| element | `barChart` | `CT_BarChart` | choice |  |  |
| element | `bar3DChart` | `CT_Bar3DChart` | choice |  |  |
| element | `ofPieChart` | `CT_OfPieChart` | choice |  |  |
| element | `surfaceChart` | `CT_SurfaceChart` | choice |  |  |
| element | `surface3DChart` | `CT_Surface3DChart` | choice |  |  |
| element | `bubbleChart` | `CT_BubbleChart` | choice |  |  |
| element | `valAx` | `CT_ValAx` | choice |  |  |
| element | `catAx` | `CT_CatAx` | choice |  |  |
| element | `dateAx` | `CT_DateAx` | choice |  |  |
| element | `serAx` | `CT_SerAx` | choice |  |  |
| element | `dTable` | `CT_DTable` | 0..1 |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_PivotFmt`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `idx` | `CT_UnsignedInt` |  |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `txPr` | `CT_TextBody` | 0..1 |  |  |
| element | `marker` | `CT_Marker` | 0..1 |  |  |
| element | `dLbl` | `CT_DLbl` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_PivotFmts`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `pivotFmt` | `CT_PivotFmt` | 0..unbounded |  |  |

## `c:ST_LegendPos`
Restriction: `base=string`
- `b`
- `tr`
- `l`
- `r`
- `t`

## `c:CT_LegendPos`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_LegendPos` | optional | r |  |

## `c:EG_LegendEntryData`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `txPr` | `CT_TextBody` | 0..1 |  |  |

## `c:CT_LegendEntry`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `idx` | `CT_UnsignedInt` |  |  |  |
| element | `delete` | `CT_Boolean` | choice |  |  |
| group | `EG_LegendEntryData` | `EG_LegendEntryData` | choice |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:CT_Legend`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `legendPos` | `CT_LegendPos` | 0..1 |  |  |
| element | `legendEntry` | `CT_LegendEntry` | 0..unbounded |  |  |
| element | `layout` | `CT_Layout` | 0..1 |  |  |
| element | `overlay` | `CT_Boolean` | 0..1 |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `txPr` | `CT_TextBody` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:ST_DispBlanksAs`
Restriction: `base=string`
- `span`
- `gap`
- `zero`

## `c:CT_DispBlanksAs`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_DispBlanksAs` | optional | zero |  |

## `c:CT_Chart`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `title` | `CT_Title` | 0..1 |  |  |
| element | `autoTitleDeleted` | `CT_Boolean` | 0..1 |  |  |
| element | `pivotFmts` | `CT_PivotFmts` | 0..1 |  |  |
| element | `view3D` | `CT_View3D` | 0..1 |  |  |
| element | `floor` | `CT_Surface` | 0..1 |  |  |
| element | `sideWall` | `CT_Surface` | 0..1 |  |  |
| element | `backWall` | `CT_Surface` | 0..1 |  |  |
| element | `plotArea` | `CT_PlotArea` |  |  |  |
| element | `legend` | `CT_Legend` | 0..1 |  |  |
| element | `plotVisOnly` | `CT_Boolean` | 0..1 |  |  |
| element | `dispBlanksAs` | `CT_DispBlanksAs` | 0..1 |  |  |
| element | `showDLblsOverMax` | `CT_Boolean` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:ST_Style`
Restriction: `base=unsignedByte minInclusive=1 maxInclusive=48`

## `c:CT_Style`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_Style` | required |  |  |

## `c:CT_PivotSource`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `name` | `ST_Xstring` |  |  |  |
| element | `fmtId` | `CT_UnsignedInt` |  |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..unbounded |  |  |

## `c:CT_Protection`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `chartObject` | `CT_Boolean` | 0..1 |  |  |
| element | `data` | `CT_Boolean` | 0..1 |  |  |
| element | `formatting` | `CT_Boolean` | 0..1 |  |  |
| element | `selection` | `CT_Boolean` | 0..1 |  |  |
| element | `userInterface` | `CT_Boolean` | 0..1 |  |  |

## `c:CT_HeaderFooter`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `oddHeader` | `ST_Xstring` | 0..1 |  |  |
| element | `oddFooter` | `ST_Xstring` | 0..1 |  |  |
| element | `evenHeader` | `ST_Xstring` | 0..1 |  |  |
| element | `evenFooter` | `ST_Xstring` | 0..1 |  |  |
| element | `firstHeader` | `ST_Xstring` | 0..1 |  |  |
| element | `firstFooter` | `ST_Xstring` | 0..1 |  |  |
| attribute | `@alignWithMargins` | `boolean` | optional | true |  |
| attribute | `@differentOddEven` | `boolean` | optional | false |  |
| attribute | `@differentFirst` | `boolean` | optional | false |  |

## `c:CT_PageMargins`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@l` | `double` | required |  |  |
| attribute | `@r` | `double` | required |  |  |
| attribute | `@t` | `double` | required |  |  |
| attribute | `@b` | `double` | required |  |  |
| attribute | `@header` | `double` | required |  |  |
| attribute | `@footer` | `double` | required |  |  |

## `c:ST_PageSetupOrientation`
Restriction: `base=string`
- `default`
- `portrait`
- `landscape`

## `c:CT_ExternalData`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `autoUpdate` | `CT_Boolean` | 0..1 |  |  |
| attribute | `@r:id` | `id` | required |  |  |

## `c:CT_PageSetup`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@paperSize` | `unsignedInt` | optional | 1 |  |
| attribute | `@paperHeight` | `ST_PositiveUniversalMeasure` | optional |  |  |
| attribute | `@paperWidth` | `ST_PositiveUniversalMeasure` | optional |  |  |
| attribute | `@firstPageNumber` | `unsignedInt` | optional | 1 |  |
| attribute | `@orientation` | `ST_PageSetupOrientation` | optional | default |  |
| attribute | `@blackAndWhite` | `boolean` | optional | false |  |
| attribute | `@draft` | `boolean` | optional | false |  |
| attribute | `@useFirstPageNumber` | `boolean` | optional | false |  |
| attribute | `@horizontalDpi` | `int` | optional | 600 |  |
| attribute | `@verticalDpi` | `int` | optional | 600 |  |
| attribute | `@copies` | `unsignedInt` | optional | 1 |  |

## `c:CT_PrintSettings`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `headerFooter` | `CT_HeaderFooter` | 0..1 |  |  |
| element | `pageMargins` | `CT_PageMargins` | 0..1 |  |  |
| element | `pageSetup` | `CT_PageSetup` | 0..1 |  |  |
| element | `legacyDrawingHF` | `CT_RelId` | 0..1 |  |  |

## `c:CT_ChartSpace`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `date1904` | `CT_Boolean` | 0..1 |  |  |
| element | `lang` | `CT_TextLanguageID` | 0..1 |  |  |
| element | `roundedCorners` | `CT_Boolean` | 0..1 |  |  |
| element | `style` | `CT_Style` | 0..1 |  |  |
| element | `clrMapOvr` | `CT_ColorMapping` | 0..1 |  |  |
| element | `pivotSource` | `CT_PivotSource` | 0..1 |  |  |
| element | `protection` | `CT_Protection` | 0..1 |  |  |
| element | `chart` | `CT_Chart` |  |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `txPr` | `CT_TextBody` | 0..1 |  |  |
| element | `externalData` | `CT_ExternalData` | 0..1 |  |  |
| element | `printSettings` | `CT_PrintSettings` | 0..1 |  |  |
| element | `userShapes` | `CT_RelId` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `c:<chartSpace>` : `CT_ChartSpace`

## `c:<userShapes>` : `CT_Drawing`

## `c:<chart>` : `CT_RelId`

## `cdr:CT_ShapeNonVisual`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cNvPr` | `CT_NonVisualDrawingProps` |  |  |  |
| element | `cNvSpPr` | `CT_NonVisualDrawingShapeProps` |  |  |  |

## `cdr:CT_Shape`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `nvSpPr` | `CT_ShapeNonVisual` |  |  |  |
| element | `spPr` | `CT_ShapeProperties` |  |  |  |
| element | `style` | `CT_ShapeStyle` | 0..1 |  |  |
| element | `txBody` | `CT_TextBody` | 0..1 |  |  |
| attribute | `@macro` | `string` | optional |  |  |
| attribute | `@textlink` | `string` | optional |  |  |
| attribute | `@fLocksText` | `boolean` | optional | true |  |
| attribute | `@fPublished` | `boolean` | optional | false |  |

## `cdr:CT_ConnectorNonVisual`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cNvPr` | `CT_NonVisualDrawingProps` |  |  |  |
| element | `cNvCxnSpPr` | `CT_NonVisualConnectorProperties` |  |  |  |

## `cdr:CT_Connector`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `nvCxnSpPr` | `CT_ConnectorNonVisual` |  |  |  |
| element | `spPr` | `CT_ShapeProperties` |  |  |  |
| element | `style` | `CT_ShapeStyle` | 0..1 |  |  |
| attribute | `@macro` | `string` | optional |  |  |
| attribute | `@fPublished` | `boolean` | optional | false |  |

## `cdr:CT_PictureNonVisual`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cNvPr` | `CT_NonVisualDrawingProps` |  |  |  |
| element | `cNvPicPr` | `CT_NonVisualPictureProperties` |  |  |  |

## `cdr:CT_Picture`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `nvPicPr` | `CT_PictureNonVisual` |  |  |  |
| element | `blipFill` | `CT_BlipFillProperties` |  |  |  |
| element | `spPr` | `CT_ShapeProperties` |  |  |  |
| element | `style` | `CT_ShapeStyle` | 0..1 |  |  |
| attribute | `@macro` | `string` | optional |  |  |
| attribute | `@fPublished` | `boolean` | optional | false |  |

## `cdr:CT_GraphicFrameNonVisual`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cNvPr` | `CT_NonVisualDrawingProps` |  |  |  |
| element | `cNvGraphicFramePr` | `CT_NonVisualGraphicFrameProperties` |  |  |  |

## `cdr:CT_GraphicFrame`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `nvGraphicFramePr` | `CT_GraphicFrameNonVisual` |  |  |  |
| element | `xfrm` | `CT_Transform2D` |  |  |  |
| element | `a:graphic` | `graphic` |  |  |  |
| attribute | `@macro` | `string` | optional |  |  |
| attribute | `@fPublished` | `boolean` | optional | false |  |

## `cdr:CT_GroupShapeNonVisual`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cNvPr` | `CT_NonVisualDrawingProps` |  |  |  |
| element | `cNvGrpSpPr` | `CT_NonVisualGroupDrawingShapeProps` |  |  |  |

## `cdr:CT_GroupShape`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `nvGrpSpPr` | `CT_GroupShapeNonVisual` |  |  |  |
| element | `grpSpPr` | `CT_GroupShapeProperties` |  |  |  |
| element | `sp` | `CT_Shape` | choice |  |  |
| element | `grpSp` | `CT_GroupShape` | choice |  |  |
| element | `graphicFrame` | `CT_GraphicFrame` | choice |  |  |
| element | `cxnSp` | `CT_Connector` | choice |  |  |
| element | `pic` | `CT_Picture` | choice |  |  |

## `cdr:EG_ObjectChoices`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sp` | `CT_Shape` | choice |  |  |
| element | `grpSp` | `CT_GroupShape` | choice |  |  |
| element | `graphicFrame` | `CT_GraphicFrame` | choice |  |  |
| element | `cxnSp` | `CT_Connector` | choice |  |  |
| element | `pic` | `CT_Picture` | choice |  |  |

## `cdr:ST_MarkerCoordinate`
Restriction: `base=double minInclusive=0.0 maxInclusive=1.0`

## `cdr:CT_Marker`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `x` | `ST_MarkerCoordinate` |  |  |  |
| element | `y` | `ST_MarkerCoordinate` |  |  |  |

## `cdr:CT_RelSizeAnchor`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `from` | `CT_Marker` |  |  |  |
| element | `to` | `CT_Marker` |  |  |  |
| group | `EG_ObjectChoices` | `EG_ObjectChoices` |  |  |  |

## `cdr:CT_AbsSizeAnchor`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `from` | `CT_Marker` |  |  |  |
| element | `ext` | `CT_PositiveSize2D` |  |  |  |
| group | `EG_ObjectChoices` | `EG_ObjectChoices` |  |  |  |

## `cdr:EG_Anchor`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `relSizeAnchor` | `CT_RelSizeAnchor` | choice |  |  |
| element | `absSizeAnchor` | `CT_AbsSizeAnchor` | choice |  |  |

## `cdr:CT_Drawing`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_Anchor` | `EG_Anchor` | 0..unbounded |  |  |

## `x:drawing/2010/chartDrawing:CT_ApplicationNonVisualDrawingProps`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@macro` | `string` | optional |  |  |
| attribute | `@fPublished` | `boolean` | optional | false |  |

## `x:drawing/2010/chartDrawing:CT_ContentPartNonVisual`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cNvPr` | `CT_NonVisualDrawingProps` |  |  |  |
| element | `cNvContentPartPr` | `CT_NonVisualInkContentPartProperties` | 0..1 |  |  |

## `x:drawing/2010/chartDrawing:CT_ContentPart`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `nvContentPartPr` | `CT_ContentPartNonVisual` | 0..1 |  |  |
| element | `nvPr` | `CT_ApplicationNonVisualDrawingProps` | 0..1 |  |  |
| element | `xfrm` | `CT_Transform2D` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@r:id` | `id` | required |  |  |
| attribute | `@bwMode` | `ST_BlackWhiteMode` | optional | auto |  |

## `x:drawing/2010/chartDrawing:<contentPart>` : `CT_ContentPart`

## `x:drawing/2007/8/2/chart:CT_BooleanTrue`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `boolean` | optional | true |  |

## `x:drawing/2007/8/2/chart:CT_BooleanFalse`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `boolean` | optional | false |  |

## `x:drawing/2007/8/2/chart:CT_InvertSolidFillFmt`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `spPr` | `CT_ShapeProperties` |  |  |  |

## `x:drawing/2007/8/2/chart:CT_PivotOptions`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `dropZoneFilter` | `CT_BooleanFalse` | 0..1 |  |  |
| element | `dropZoneCategories` | `CT_BooleanFalse` | 0..1 |  |  |
| element | `dropZoneData` | `CT_BooleanFalse` | 0..1 |  |  |
| element | `dropZoneSeries` | `CT_BooleanFalse` | 0..1 |  |  |
| element | `dropZonesVisible` | `CT_BooleanFalse` | 0..1 |  |  |

## `x:drawing/2007/8/2/chart:ST_Style`
Restriction: `base=unsignedByte minInclusive=101 maxInclusive=148`

## `x:drawing/2007/8/2/chart:CT_Style`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_Style` | required |  |  |

## `x:drawing/2007/8/2/chart:<pivotOptions>` : `CT_PivotOptions`

## `x:drawing/2007/8/2/chart:<invertSolidFillFmt>` : `CT_InvertSolidFillFmt`

## `x:drawing/2007/8/2/chart:<style>` : `CT_Style`

## `x:drawing/2012/chart:<pivotSource>` : `CT_PivotSource`

## `x:drawing/2012/chart:<numFmt>` : `CT_NumFmt`

## `x:drawing/2012/chart:<spPr>` : `CT_ShapeProperties`

## `x:drawing/2012/chart:<layout>` : `CT_Layout`

## `x:drawing/2012/chart:<fullRef>` : `CT_FullRef`

## `x:drawing/2012/chart:CT_FullRef`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sqref` | `string` |  |  |  |

## `x:drawing/2012/chart:CT_LevelRef`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sqref` | `string` |  |  |  |

## `x:drawing/2012/chart:<levelRef>` : `CT_LevelRef`

## `x:drawing/2012/chart:CT_FormulaRef`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sqref` | `string` |  |  |  |

## `x:drawing/2012/chart:<formulaRef>` : `CT_FormulaRef`

## `x:drawing/2012/chart:CT_FilteredSeriesTitle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tx` | `CT_SerTx` |  |  |  |

## `x:drawing/2012/chart:<filteredSeriesTitle>` : `CT_FilteredSeriesTitle`

## `x:drawing/2012/chart:CT_FilteredCategoryTitle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cat` | `CT_AxDataSource` |  |  |  |

## `x:drawing/2012/chart:<filteredCategoryTitle>` : `CT_FilteredCategoryTitle`

## `x:drawing/2012/chart:CT_FilteredBarSer`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ser` | `CT_BarSer` |  |  |  |

## `x:drawing/2012/chart:CT_FilteredLineSer`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ser` | `CT_LineSer` |  |  |  |

## `x:drawing/2012/chart:CT_FilteredScatterSer`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ser` | `CT_ScatterSer` |  |  |  |

## `x:drawing/2012/chart:CT_FilteredAreaSer`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ser` | `CT_AreaSer` |  |  |  |

## `x:drawing/2012/chart:CT_FilteredPieSer`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ser` | `CT_PieSer` |  |  |  |

## `x:drawing/2012/chart:CT_FilteredBubbleSer`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ser` | `CT_BubbleSer` |  |  |  |

## `x:drawing/2012/chart:CT_FilteredRadarSer`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ser` | `CT_RadarSer` |  |  |  |

## `x:drawing/2012/chart:CT_FilteredSurfaceSer`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ser` | `CT_SurfaceSer` |  |  |  |

## `x:drawing/2012/chart:<filteredAreaSeries>` : `CT_FilteredAreaSer`

## `x:drawing/2012/chart:<filteredBarSeries>` : `CT_FilteredBarSer`

## `x:drawing/2012/chart:<filteredBubbleSeries>` : `CT_FilteredBubbleSer`

## `x:drawing/2012/chart:<filteredLineSeries>` : `CT_FilteredLineSer`

## `x:drawing/2012/chart:<filteredPieSeries>` : `CT_FilteredPieSer`

## `x:drawing/2012/chart:<filteredRadarSeries>` : `CT_FilteredRadarSer`

## `x:drawing/2012/chart:<filteredScatterSeries>` : `CT_FilteredScatterSer`

## `x:drawing/2012/chart:<filteredSurfaceSeries>` : `CT_FilteredSurfaceSer`

## `x:drawing/2012/chart:CT_SeriesDataLabelsRange`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `f` | `string` |  |  |  |
| element | `dlblRangeCache` | `CT_StrData` | 0..1 |  |  |

## `x:drawing/2012/chart:<datalabelsRange>` : `CT_SeriesDataLabelsRange`

## `x:drawing/2012/chart:CT_CategoryFilterException`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sqref` | `string` |  |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `explosion` | `CT_UnsignedInt` | 0..1 |  |  |
| element | `invertIfNegative` | `CT_Boolean` | 0..1 |  |  |
| element | `bubble3D` | `CT_Boolean` | 0..1 |  |  |
| element | `marker` | `CT_Marker` | 0..1 |  |  |
| element | `dLbl` | `CT_DLbl` | 0..1 |  |  |

## `x:drawing/2012/chart:CT_CategoryFilterExceptions`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `categoryFilterException` | `CT_CategoryFilterException` | 1..unbounded |  |  |

## `x:drawing/2012/chart:<categoryFilterExceptions>` : `CT_CategoryFilterExceptions`

## `x:drawing/2012/chart:CT_DataLabelFieldTableEntry`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `txfldGUID` | `string` |  |  |  |
| element | `f` | `string` |  |  |  |
| element | `dlblFieldTableCache` | `CT_StrData` | 0..1 |  |  |

## `x:drawing/2012/chart:CT_DataLabelFieldTable`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `dlblFTEntry` | `CT_DataLabelFieldTableEntry` | 0..unbounded |  |  |

## `x:drawing/2012/chart:<dlblFieldTable>` : `CT_DataLabelFieldTable`

## `x:drawing/2012/chart:<xForSave>` : `CT_Boolean`

## `x:drawing/2012/chart:<showDataLabelsRange>` : `CT_Boolean`

## `x:drawing/2012/chart:<tx>` : `CT_Tx`

## `x:drawing/2012/chart:<showLeaderLines>` : `CT_Boolean`

## `x:drawing/2012/chart:<leaderLines>` : `CT_ChartLines`

## `x:drawing/2012/chart:<autoCat>` : `CT_Boolean`

## `x:drawing/2012/chartStyle:ST_ColorStyleMethodEnum`
Restriction: `base=token`
- `cycle`
- `withinLinear`
- `acrossLinear`
- `withinLinearReversed`
- `acrossLinearReversed`

## `x:drawing/2012/chartStyle:ST_ColorStyleMethod`
Restriction: `union=ST_ColorStyleMethodEnum string`

## `x:drawing/2012/chartStyle:CT_ColorStyleVariation`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorTransform` | `EG_ColorTransform` | 0..unbounded |  |  |

## `x:drawing/2012/chartStyle:ST_StyleReferenceModifierEnum`
Restriction: `base=token`
- `ignoreCSTransforms`

## `x:drawing/2012/chartStyle:ST_StyleReferenceModifier`
Restriction: `union=ST_StyleReferenceModifierEnum string`

## `x:drawing/2012/chartStyle:ST_StyleReferenceModifierList`
Restriction: `list=ST_StyleReferenceModifier`

## `x:drawing/2012/chartStyle:CT_ColorStyle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorChoice` | `EG_ColorChoice` | 1..unbounded |  |  |
| element | `variation` | `CT_ColorStyleVariation` | 0..unbounded |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@meth` | `ST_ColorStyleMethod` | required |  |  |
| attribute | `@id` | `unsignedInt` | optional |  |  |

## `x:drawing/2012/chartStyle:<colorStyle>` : `CT_ColorStyle`

## `x:drawing/2012/chartStyle:ST_StyleColorEnum`
Restriction: `base=token`
- `auto`

## `x:drawing/2012/chartStyle:ST_StyleColorVal`
Restriction: `union=unsignedInt ST_StyleColorEnum string`

## `x:drawing/2012/chartStyle:CT_StyleColor`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorTransform` | `EG_ColorTransform` | 0..unbounded |  |  |
| attribute | `@val` | `ST_StyleColorVal` | optional |  |  |

## `x:drawing/2012/chartStyle:CT_StyleReference`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorChoice` | `EG_ColorChoice` | 0..1 |  |  |
| element | `styleClr` | `CT_StyleColor` | 0..1 |  |  |
| attribute | `@idx` | `ST_StyleMatrixColumnIndex` | required |  |  |
| attribute | `@mods` | `ST_StyleReferenceModifierList` | optional |  |  |

## `x:drawing/2012/chartStyle:CT_FontReference`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorChoice` | `EG_ColorChoice` | 0..1 |  |  |
| element | `styleClr` | `CT_StyleColor` | 0..1 |  |  |
| attribute | `@idx` | `ST_FontCollectionIndex` | required |  |  |
| attribute | `@mods` | `ST_StyleReferenceModifierList` | optional |  |  |

## `x:drawing/2012/chartStyle:ST_StyleEntryModifierEnum`
Restriction: `base=token`
- `allowNoFillOverride`
- `allowNoLineOverride`

## `x:drawing/2012/chartStyle:ST_StyleEntryModifier`
Restriction: `union=ST_StyleEntryModifierEnum string`

## `x:drawing/2012/chartStyle:ST_StyleEntryModifierList`
Restriction: `list=ST_StyleEntryModifier`

## `x:drawing/2012/chartStyle:ST_MarkerStyle`
Restriction: `base=token`
- `circle`
- `dash`
- `diamond`
- `dot`
- `plus`
- `square`
- `star`
- `triangle`
- `x`

## `x:drawing/2012/chartStyle:ST_MarkerSize`
Restriction: `base=unsignedByte minInclusive=2 maxInclusive=72`

## `x:drawing/2012/chartStyle:CT_MarkerLayout`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@symbol` | `ST_MarkerStyle` | optional |  |  |
| attribute | `@size` | `ST_MarkerSize` | optional |  |  |

## `x:drawing/2012/chartStyle:CT_StyleEntry`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `lnRef` | `CT_StyleReference` |  |  |  |
| element | `lineWidthScale` | `double` | 0..1 |  |  |
| element | `fillRef` | `CT_StyleReference` |  |  |  |
| element | `effectRef` | `CT_StyleReference` |  |  |  |
| element | `fontRef` | `CT_FontReference` |  |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `defRPr` | `CT_TextCharacterProperties` | 0..1 |  |  |
| element | `bodyPr` | `CT_TextBodyProperties` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@mods` | `ST_StyleEntryModifierList` | optional |  |  |

## `x:drawing/2012/chartStyle:CT_ChartStyle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `axisTitle` | `CT_StyleEntry` |  |  |  |
| element | `categoryAxis` | `CT_StyleEntry` |  |  |  |
| element | `chartArea` | `CT_StyleEntry` |  |  |  |
| element | `dataLabel` | `CT_StyleEntry` |  |  |  |
| element | `dataLabelCallout` | `CT_StyleEntry` | 0..1 |  |  |
| element | `dataPoint` | `CT_StyleEntry` |  |  |  |
| element | `dataPoint3D` | `CT_StyleEntry` |  |  |  |
| element | `dataPointLine` | `CT_StyleEntry` |  |  |  |
| element | `dataPointMarker` | `CT_StyleEntry` |  |  |  |
| element | `dataPointMarkerLayout` | `CT_MarkerLayout` | 0..1 |  |  |
| element | `dataPointWireframe` | `CT_StyleEntry` |  |  |  |
| element | `dataTable` | `CT_StyleEntry` |  |  |  |
| element | `downBar` | `CT_StyleEntry` |  |  |  |
| element | `dropLine` | `CT_StyleEntry` |  |  |  |
| element | `errorBar` | `CT_StyleEntry` |  |  |  |
| element | `floor` | `CT_StyleEntry` |  |  |  |
| element | `gridlineMajor` | `CT_StyleEntry` |  |  |  |
| element | `gridlineMinor` | `CT_StyleEntry` |  |  |  |
| element | `hiLoLine` | `CT_StyleEntry` |  |  |  |
| element | `leaderLine` | `CT_StyleEntry` |  |  |  |
| element | `legend` | `CT_StyleEntry` |  |  |  |
| element | `plotArea` | `CT_StyleEntry` |  |  |  |
| element | `plotArea3D` | `CT_StyleEntry` |  |  |  |
| element | `seriesAxis` | `CT_StyleEntry` |  |  |  |
| element | `seriesLine` | `CT_StyleEntry` |  |  |  |
| element | `title` | `CT_StyleEntry` |  |  |  |
| element | `trendline` | `CT_StyleEntry` |  |  |  |
| element | `trendlineLabel` | `CT_StyleEntry` |  |  |  |
| element | `upBar` | `CT_StyleEntry` |  |  |  |
| element | `valueAxis` | `CT_StyleEntry` |  |  |  |
| element | `wall` | `CT_StyleEntry` |  |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@id` | `unsignedInt` | optional |  |  |

## `x:drawing/2012/chartStyle:<chartStyle>` : `CT_ChartStyle`

## `x:drawing/2014/chart:CT_ChartUniqueID`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `string` | required |  |  |

## `x:drawing/2014/chart:CT_ChartDataPointUniqueIDMapEntry`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ptidx` | `unsignedInt` |  |  |  |
| element | `uniqueID` | `CT_ChartUniqueID` |  |  |  |

## `x:drawing/2014/chart:CT_ChartDataPointUniqueIDMap`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ptentry` | `CT_ChartDataPointUniqueIDMapEntry` | 0..unbounded |  |  |

## `x:drawing/2014/chart:<datapointuniqueidmap>` : `CT_ChartDataPointUniqueIDMap`

## `x:drawing/2014/chart:CT_BooleanFalse`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `boolean` | optional | false |  |

## `x:drawing/2014/chart:<spPr>` : `CT_ShapeProperties`

## `x:drawing/2014/chart:<explosion>` : `CT_UnsignedInt`

## `x:drawing/2014/chart:<invertIfNegative>` : `CT_Boolean`

## `x:drawing/2014/chart:<bubble3D>` : `CT_Boolean`

## `x:drawing/2014/chart:<marker>` : `CT_Marker`

## `x:drawing/2014/chart:<dLbl>` : `CT_DLbl`

## `x:drawing/2014/chart:CT_CategoryFilterException`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `uniqueId` | `CT_ChartUniqueID` |  |  |  |
| element | `spPr` | `spPr` | 0..1 |  |  |
| element | `explosion` | `explosion` | 0..1 |  |  |
| element | `invertIfNegative` | `invertIfNegative` | 0..1 |  |  |
| element | `bubble3D` | `bubble3D` | 0..1 |  |  |
| element | `marker` | `marker` | 0..1 |  |  |
| element | `dLbl` | `dLbl` | 0..1 |  |  |

## `x:drawing/2014/chart:CT_CategoryFilterExceptions`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `categoryFilterException` | `CT_CategoryFilterException` | 1..unbounded |  |  |

## `x:drawing/2014/chart:<categoryFilterExceptions>` : `CT_CategoryFilterExceptions`

## `x:drawing/2014/chart:CT_NumFilteredLiteralCache`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `numCache` | `CT_NumData` |  |  |  |

## `x:drawing/2014/chart:CT_StrFilteredLiteralCache`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `strCache` | `CT_StrData` |  |  |  |

## `x:drawing/2014/chart:CT_MultiLvlStrFilteredLiteralCache`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `multiLvlStrCache` | `CT_MultiLvlStrData` |  |  |  |

## `x:drawing/2014/chart:CT_LiteralDataChart`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `boolean` | required |  |  |

## `x:drawing/2014/chart:CT_PivotOptions16`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `showExpandCollapseFieldButtons` | `CT_BooleanFalse` | 0..1 |  |  |

## `x:drawing/2014/chart:<pivotOptions16>` : `CT_PivotOptions16`

## `x:drawing/2014/chart/ac:<multiLvlStrLit>` : `CT_MultiLvlStrData`

## `x:drawing/2017/03/chart:CT_BooleanFalse`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `boolean` | optional | false |  |

## `x:drawing/2017/03/chart:CT_DataDisplayOptions16`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `dispNaAsBlank` | `CT_BooleanFalse` | 0..1 |  |  |

## `x:drawing/2017/03/chart:<dataDisplayOptions16>` : `CT_DataDisplayOptions16`

## `x:drawing/2015/06/chart:@formatcode2` : `ST_Xstring`

## `x:drawing/2015/06/chart:<formatcode2>` : `ST_Xstring`

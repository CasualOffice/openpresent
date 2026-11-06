# SHAPE: Shape tree, identity, locks, links

Generated. Tier 1D; render 2B; edit 4; level Edit. 365 rows.


## `p:CT_ApplicationNonVisualDrawingProps`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ph` | `CT_Placeholder` | 0..1 |  |  |
| group | `EG_Media` | `EG_Media` | 0..1 |  |  |
| element | `custDataLst` | `CT_CustomerDataList` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@isPhoto` | `boolean` | optional | false |  |
| attribute | `@userDrawn` | `boolean` | optional | false |  |

## `p:CT_ShapeNonVisual`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cNvPr` | `CT_NonVisualDrawingProps` |  |  |  |
| element | `cNvSpPr` | `CT_NonVisualDrawingShapeProps` |  |  |  |
| element | `nvPr` | `CT_ApplicationNonVisualDrawingProps` |  |  |  |

## `p:CT_Shape`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `nvSpPr` | `CT_ShapeNonVisual` |  |  |  |
| element | `spPr` | `CT_ShapeProperties` |  |  |  |
| element | `style` | `CT_ShapeStyle` | 0..1 |  |  |
| element | `txBody` | `CT_TextBody` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionListModify` | 0..1 |  |  |
| attribute | `@useBgFill` | `boolean` | optional | false |  |

## `p:CT_ConnectorNonVisual`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cNvPr` | `CT_NonVisualDrawingProps` |  |  |  |
| element | `cNvCxnSpPr` | `CT_NonVisualConnectorProperties` |  |  |  |
| element | `nvPr` | `CT_ApplicationNonVisualDrawingProps` |  |  |  |

## `p:CT_Connector`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `nvCxnSpPr` | `CT_ConnectorNonVisual` |  |  |  |
| element | `spPr` | `CT_ShapeProperties` |  |  |  |
| element | `style` | `CT_ShapeStyle` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionListModify` | 0..1 |  |  |

## `p:CT_PictureNonVisual`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cNvPr` | `CT_NonVisualDrawingProps` |  |  |  |
| element | `cNvPicPr` | `CT_NonVisualPictureProperties` |  |  |  |
| element | `nvPr` | `CT_ApplicationNonVisualDrawingProps` |  |  |  |

## `p:CT_Picture`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `nvPicPr` | `CT_PictureNonVisual` |  |  |  |
| element | `blipFill` | `CT_BlipFillProperties` |  |  |  |
| element | `spPr` | `CT_ShapeProperties` |  |  |  |
| element | `style` | `CT_ShapeStyle` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionListModify` | 0..1 |  |  |

## `p:CT_GraphicalObjectFrameNonVisual`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cNvPr` | `CT_NonVisualDrawingProps` |  |  |  |
| element | `cNvGraphicFramePr` | `CT_NonVisualGraphicFrameProperties` |  |  |  |
| element | `nvPr` | `CT_ApplicationNonVisualDrawingProps` |  |  |  |

## `p:CT_GraphicalObjectFrame`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `nvGraphicFramePr` | `CT_GraphicalObjectFrameNonVisual` |  |  |  |
| element | `xfrm` | `CT_Transform2D` |  |  |  |
| element | `a:graphic` | `graphic` |  |  |  |
| element | `extLst` | `CT_ExtensionListModify` | 0..1 |  |  |
| attribute | `@bwMode` | `ST_BlackWhiteMode` | optional |  |  |

## `p:CT_GroupShapeNonVisual`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cNvPr` | `CT_NonVisualDrawingProps` |  |  |  |
| element | `cNvGrpSpPr` | `CT_NonVisualGroupDrawingShapeProps` |  |  |  |
| element | `nvPr` | `CT_ApplicationNonVisualDrawingProps` |  |  |  |

## `p:CT_GroupShape`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `nvGrpSpPr` | `CT_GroupShapeNonVisual` |  |  |  |
| element | `grpSpPr` | `CT_GroupShapeProperties` |  |  |  |
| element | `sp` | `CT_Shape` | choice |  |  |
| element | `grpSp` | `CT_GroupShape` | choice |  |  |
| element | `graphicFrame` | `CT_GraphicalObjectFrame` | choice |  |  |
| element | `cxnSp` | `CT_Connector` | choice |  |  |
| element | `pic` | `CT_Picture` | choice |  |  |
| element | `contentPart` | `CT_Rel` | choice |  |  |
| element | `extLst` | `CT_ExtensionListModify` | 0..1 |  |  |

## `a:CT_OfficeArtExtension`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| any | `xsd:any` | `` | 0..unbounded |  | lax |
| attribute | `@uri` | `token` | required |  |  |

## `a:ST_Coordinate`
Restriction: `union=ST_CoordinateUnqualified ST_UniversalMeasure`

## `a:ST_CoordinateUnqualified`
Restriction: `base=long minInclusive=-27273042329600 maxInclusive=27273042316900`

## `a:ST_Coordinate32`
Restriction: `union=ST_Coordinate32Unqualified ST_UniversalMeasure`

## `a:ST_Coordinate32Unqualified`
Restriction: `base=int`

## `a:ST_PositiveCoordinate`
Restriction: `base=long minInclusive=0 maxInclusive=27273042316900`

## `a:ST_PositiveCoordinate32`
Restriction: `base=ST_Coordinate32Unqualified minInclusive=0`

## `a:ST_Angle`
Restriction: `base=int`

## `a:CT_Angle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_Angle` | required |  |  |

## `a:ST_FixedAngle`
Restriction: `base=ST_Angle minExclusive=-5400000 maxExclusive=5400000`

## `a:ST_PositiveFixedAngle`
Restriction: `base=ST_Angle minInclusive=0 maxExclusive=21600000`

## `a:CT_PositiveFixedAngle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_PositiveFixedAngle` | required |  |  |

## `a:ST_Percentage`
Restriction: `union=ST_PercentageDecimal ST_Percentage`

## `a:ST_PercentageDecimal`
Restriction: `base=int`

## `a:CT_Percentage`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_Percentage` | required |  |  |

## `a:ST_PositivePercentage`
Restriction: `union=ST_PositivePercentageDecimal ST_PositivePercentage`

## `a:ST_PositivePercentageDecimal`
Restriction: `base=ST_PercentageDecimal minInclusive=0`

## `a:CT_PositivePercentage`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_PositivePercentage` | required |  |  |

## `a:ST_FixedPercentage`
Restriction: `union=ST_FixedPercentageDecimal ST_FixedPercentage`

## `a:ST_FixedPercentageDecimal`
Restriction: `base=ST_PercentageDecimal minInclusive=-100000 maxInclusive=100000`

## `a:CT_FixedPercentage`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_FixedPercentage` | required |  |  |

## `a:ST_PositiveFixedPercentage`
Restriction: `union=ST_PositiveFixedPercentageDecimal ST_PositiveFixedPercentage`

## `a:ST_PositiveFixedPercentageDecimal`
Restriction: `base=ST_PercentageDecimal minInclusive=0 maxInclusive=100000`

## `a:CT_PositiveFixedPercentage`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_PositiveFixedPercentage` | required |  |  |

## `a:CT_Ratio`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@n` | `long` | required |  |  |
| attribute | `@d` | `long` | required |  |  |

## `a:CT_Point2D`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@x` | `ST_Coordinate` | required |  |  |
| attribute | `@y` | `ST_Coordinate` | required |  |  |

## `a:CT_PositiveSize2D`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@cx` | `ST_PositiveCoordinate` | required |  |  |
| attribute | `@cy` | `ST_PositiveCoordinate` | required |  |  |

## `a:EG_OfficeArtExtensionList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ext` | `CT_OfficeArtExtension` | 0..unbounded |  |  |

## `a:CT_OfficeArtExtensionList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_OfficeArtExtensionList` | `EG_OfficeArtExtensionList` |  |  |  |

## `a:CT_Scale2D`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sx` | `CT_Ratio` |  |  |  |
| element | `sy` | `CT_Ratio` |  |  |  |

## `a:CT_Transform2D`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `off` | `CT_Point2D` | 0..1 |  |  |
| element | `ext` | `CT_PositiveSize2D` | 0..1 |  |  |
| attribute | `@rot` | `ST_Angle` | optional | 0 |  |
| attribute | `@flipH` | `boolean` | optional | false |  |
| attribute | `@flipV` | `boolean` | optional | false |  |

## `a:CT_GroupTransform2D`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `off` | `CT_Point2D` | 0..1 |  |  |
| element | `ext` | `CT_PositiveSize2D` | 0..1 |  |  |
| element | `chOff` | `CT_Point2D` | 0..1 |  |  |
| element | `chExt` | `CT_PositiveSize2D` | 0..1 |  |  |
| attribute | `@rot` | `ST_Angle` | optional | 0 |  |
| attribute | `@flipH` | `boolean` | optional | false |  |
| attribute | `@flipV` | `boolean` | optional | false |  |

## `a:ST_BlackWhiteMode`
Restriction: `base=token`
- `clr`
- `auto`
- `gray`
- `ltGray`
- `invGray`
- `grayWhite`
- `blackGray`
- `blackWhite`
- `black`
- `white`
- `hidden`

## `a:CT_Hyperlink`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `snd` | `CT_EmbeddedWAVAudioFile` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@r:id` | `id` | optional |  |  |
| attribute | `@invalidUrl` | `string` | optional |  |  |
| attribute | `@action` | `string` | optional |  |  |
| attribute | `@tgtFrame` | `string` | optional |  |  |
| attribute | `@tooltip` | `string` | optional |  |  |
| attribute | `@history` | `boolean` | optional | true |  |
| attribute | `@highlightClick` | `boolean` | optional | false |  |
| attribute | `@endSnd` | `boolean` | optional | false |  |

## `a:ST_DrawingElementId`
Restriction: `base=unsignedInt`

## `a:AG_Locking`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@noGrp` | `boolean` | optional | false |  |
| attribute | `@noSelect` | `boolean` | optional | false |  |
| attribute | `@noRot` | `boolean` | optional | false |  |
| attribute | `@noChangeAspect` | `boolean` | optional | false |  |
| attribute | `@noMove` | `boolean` | optional | false |  |
| attribute | `@noResize` | `boolean` | optional | false |  |
| attribute | `@noEditPoints` | `boolean` | optional | false |  |
| attribute | `@noAdjustHandles` | `boolean` | optional | false |  |
| attribute | `@noChangeArrowheads` | `boolean` | optional | false |  |
| attribute | `@noChangeShapeType` | `boolean` | optional | false |  |

## `a:CT_ConnectorLocking`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attrgroup | `AG_Locking` | `AG_Locking` |  |  |  |

## `a:CT_ShapeLocking`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attrgroup | `AG_Locking` | `AG_Locking` |  |  |  |
| attribute | `@noTextEdit` | `boolean` | optional | false |  |

## `a:CT_GroupLocking`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@noGrp` | `boolean` | optional | false |  |
| attribute | `@noUngrp` | `boolean` | optional | false |  |
| attribute | `@noSelect` | `boolean` | optional | false |  |
| attribute | `@noRot` | `boolean` | optional | false |  |
| attribute | `@noChangeAspect` | `boolean` | optional | false |  |
| attribute | `@noMove` | `boolean` | optional | false |  |
| attribute | `@noResize` | `boolean` | optional | false |  |

## `a:CT_GraphicalObjectFrameLocking`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@noGrp` | `boolean` | optional | false |  |
| attribute | `@noDrilldown` | `boolean` | optional | false |  |
| attribute | `@noSelect` | `boolean` | optional | false |  |
| attribute | `@noChangeAspect` | `boolean` | optional | false |  |
| attribute | `@noMove` | `boolean` | optional | false |  |
| attribute | `@noResize` | `boolean` | optional | false |  |

## `a:CT_ContentPartLocking`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attrgroup | `AG_Locking` | `AG_Locking` |  |  |  |

## `a:CT_NonVisualDrawingProps`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `hlinkClick` | `CT_Hyperlink` | 0..1 |  |  |
| element | `hlinkHover` | `CT_Hyperlink` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@id` | `ST_DrawingElementId` | required |  |  |
| attribute | `@name` | `string` | required |  |  |
| attribute | `@descr` | `string` | optional |  |  |
| attribute | `@hidden` | `boolean` | optional | false |  |
| attribute | `@title` | `string` | optional |  |  |

## `a:CT_NonVisualDrawingShapeProps`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `spLocks` | `CT_ShapeLocking` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@txBox` | `boolean` | optional | false |  |

## `a:CT_NonVisualConnectorProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cxnSpLocks` | `CT_ConnectorLocking` | 0..1 |  |  |
| element | `stCxn` | `CT_Connection` | 0..1 |  |  |
| element | `endCxn` | `CT_Connection` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `a:CT_NonVisualGroupDrawingShapeProps`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `grpSpLocks` | `CT_GroupLocking` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `a:CT_NonVisualGraphicFrameProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `graphicFrameLocks` | `CT_GraphicalObjectFrameLocking` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `a:CT_NonVisualContentPartProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cpLocks` | `CT_ContentPartLocking` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@isComment` | `boolean` | optional | true |  |

## `a:CT_GraphicalObjectData`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| any | `xsd:any` | `` | 0..unbounded |  | strict |
| attribute | `@uri` | `token` | required |  |  |

## `a:CT_GraphicalObject`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `graphicData` | `CT_GraphicalObjectData` |  |  |  |

## `a:<graphic>` : `CT_GraphicalObject`

## `a:CT_GvmlUseShapeRectangle`

## `a:CT_GvmlShapeNonVisual`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cNvPr` | `CT_NonVisualDrawingProps` |  |  |  |
| element | `cNvSpPr` | `CT_NonVisualDrawingShapeProps` |  |  |  |

## `a:CT_GvmlShape`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `nvSpPr` | `CT_GvmlShapeNonVisual` |  |  |  |
| element | `spPr` | `CT_ShapeProperties` |  |  |  |
| element | `txSp` | `CT_GvmlTextShape` | 0..1 |  |  |
| element | `style` | `CT_ShapeStyle` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `a:CT_GvmlConnectorNonVisual`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cNvPr` | `CT_NonVisualDrawingProps` |  |  |  |
| element | `cNvCxnSpPr` | `CT_NonVisualConnectorProperties` |  |  |  |

## `a:CT_GvmlConnector`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `nvCxnSpPr` | `CT_GvmlConnectorNonVisual` |  |  |  |
| element | `spPr` | `CT_ShapeProperties` |  |  |  |
| element | `style` | `CT_ShapeStyle` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `a:CT_GvmlGraphicFrameNonVisual`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cNvPr` | `CT_NonVisualDrawingProps` |  |  |  |
| element | `cNvGraphicFramePr` | `CT_NonVisualGraphicFrameProperties` |  |  |  |

## `a:CT_GvmlGraphicalObjectFrame`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `nvGraphicFramePr` | `CT_GvmlGraphicFrameNonVisual` |  |  |  |
| element | `graphic` | `graphic` |  |  |  |
| element | `xfrm` | `CT_Transform2D` |  |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `a:CT_GvmlGroupShapeNonVisual`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cNvPr` | `CT_NonVisualDrawingProps` |  |  |  |
| element | `cNvGrpSpPr` | `CT_NonVisualGroupDrawingShapeProps` |  |  |  |

## `a:CT_GvmlGroupShape`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `nvGrpSpPr` | `CT_GvmlGroupShapeNonVisual` |  |  |  |
| element | `grpSpPr` | `CT_GroupShapeProperties` |  |  |  |
| element | `txSp` | `CT_GvmlTextShape` | choice |  |  |
| element | `sp` | `CT_GvmlShape` | choice |  |  |
| element | `cxnSp` | `CT_GvmlConnector` | choice |  |  |
| element | `pic` | `CT_GvmlPicture` | choice |  |  |
| element | `graphicFrame` | `CT_GvmlGraphicalObjectFrame` | choice |  |  |
| element | `grpSp` | `CT_GvmlGroupShape` | choice |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `a:ST_FOVAngle`
Restriction: `base=ST_Angle minInclusive=0 maxInclusive=10800000`

## `a:ST_ShapeID`
Restriction: `base=token`

## `a:CT_ShapeProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `xfrm` | `CT_Transform2D` | 0..1 |  |  |
| group | `EG_Geometry` | `EG_Geometry` | 0..1 |  |  |
| group | `EG_FillProperties` | `EG_FillProperties` | 0..1 |  |  |
| element | `ln` | `CT_LineProperties` | 0..1 |  |  |
| group | `EG_EffectProperties` | `EG_EffectProperties` | 0..1 |  |  |
| element | `scene3d` | `CT_Scene3D` | 0..1 |  |  |
| element | `sp3d` | `CT_Shape3D` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@bwMode` | `ST_BlackWhiteMode` | optional |  |  |

## `a:CT_GroupShapeProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `xfrm` | `CT_GroupTransform2D` | 0..1 |  |  |
| group | `EG_FillProperties` | `EG_FillProperties` | 0..1 |  |  |
| group | `EG_EffectProperties` | `EG_EffectProperties` | 0..1 |  |  |
| element | `scene3d` | `CT_Scene3D` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@bwMode` | `ST_BlackWhiteMode` | optional |  |  |

## `a:ST_PitchFamily`
Restriction: `base=byte`
- `00`
- `01`
- `02`
- `16`
- `17`
- `18`
- `32`
- `33`
- `34`
- `48`
- `49`
- `50`
- `64`
- `65`
- `66`
- `80`
- `81`
- `82`

## `a:CT_Boolean`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_OnOff` | optional | 0 |  |

## `lc:<lockedCanvas>` : `CT_GvmlGroupShape`

## `x:drawing/2012/main:ST_TargetScreenSz`
Restriction: `base=token`
- `544x376`
- `640x480`
- `720x512`
- `800x600`
- `1024x768`
- `1152x882`
- `1152x900`
- `1280x1024`
- `1600x1200`
- `1800x1440`
- `1920x1200`

## `x:drawing/2012/main:CT_BackgroundPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@bwMode` | `ST_BlackWhiteMode` | optional |  |  |
| attribute | `@bwPure` | `ST_BlackWhiteMode` | optional |  |  |
| attribute | `@bwNormal` | `ST_BlackWhiteMode` | optional |  |  |
| attribute | `@targetScreenSize` | `ST_TargetScreenSz` | optional |  |  |

## `x:drawing/2012/main:<backgroundPr>` : `CT_BackgroundPr`

## `x:drawing/2012/main:CT_NonVisualGroupProps`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@isLegacyGroup` | `boolean` | optional |  |  |

## `x:drawing/2012/main:<nonVisualGroupProps>` : `CT_NonVisualGroupProps`

## `x:drawing/2012/main:CT_ObjectPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@objectId` | `string` | optional |  |  |
| attribute | `@isActiveX` | `boolean` | optional |  |  |
| attribute | `@linkType` | `string` | optional |  |  |

## `x:drawing/2012/main:<objectPr>` : `CT_ObjectPr`

## `x:drawing/2012/main:CT_SignatureLine`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@isSignatureLine` | `boolean` | optional |  |  |
| attribute | `@id` | `ST_Guid` | optional |  |  |
| attribute | `@provId` | `ST_Guid` | optional |  |  |
| attribute | `@signingInstructionsSet` | `boolean` | optional |  |  |
| attribute | `@allowComments` | `boolean` | optional |  |  |
| attribute | `@showSignDate` | `boolean` | optional |  |  |
| attribute | `@suggestedSigner` | `string` | optional |  |  |
| attribute | `@suggestedSigner2` | `string` | optional |  |  |
| attribute | `@suggestedSignerEmail` | `string` | optional |  |  |
| attribute | `@signingInstructions` | `string` | optional |  |  |
| attribute | `@addlXml` | `string` | optional |  |  |
| attribute | `@sigProvUrl` | `string` | optional |  |  |

## `x:drawing/2012/main:<signatureLine>` : `CT_SignatureLine`

## `x:drawing/2014/main:CT_CreationId`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@id` | `ST_Guid` | optional |  |  |

## `x:drawing/2014/main:CT_PredecessorDrawingElementReference`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@pred` | `ST_Guid` | optional |  |  |

## `x:drawing/2014/main:CT_ConnectableReferences`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@st` | `ST_Guid` | optional |  |  |
| attribute | `@end` | `ST_Guid` | optional |  |  |

## `x:drawing/2014/main:<creationId>` : `CT_CreationId`

## `x:drawing/2014/main:<predDERef>` : `CT_PredecessorDrawingElementReference`

## `x:drawing/2014/main:<cxnDERefs>` : `CT_ConnectableReferences`

## `x:drawing/2014/main:CT_Identifier`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `unsignedInt` | required |  |  |

## `x:drawing/2014/main:<rowId>` : `CT_Identifier`

## `x:drawing/2014/main:<colId>` : `CT_Identifier`

## `x:drawing/2016/11/main:CT_PictureAttributionSourceURL`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@r:id` | `id` | required |  |  |

## `x:drawing/2016/11/main:<picAttrSrcUrl>` : `CT_PictureAttributionSourceURL`

## `x:drawing/2017/decorative:CT_Decorative`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `boolean` | optional |  |  |

## `x:drawing/2017/decorative:<decorative>` : `CT_Decorative`

## `x:drawing/2018/hyperlinkcolor:ST_HyperlinkColor`
Restriction: `base=token`
- `hlink`
- `tx`

## `x:drawing/2018/hyperlinkcolor:CT_HyperlinkColor`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_HyperlinkColor` | required |  |  |

## `x:drawing/2018/hyperlinkcolor:<hlinkClr>` : `CT_HyperlinkColor`

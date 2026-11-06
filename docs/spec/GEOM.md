# GEOM: Preset and custom geometry

Generated. Tier 1E; render 2B; edit 4; level Edit. 328 rows.


## `a:ST_ShapeType`
Restriction: `base=token`
- `line`
- `lineInv`
- `triangle`
- `rtTriangle`
- `rect`
- `diamond`
- `parallelogram`
- `trapezoid`
- `nonIsoscelesTrapezoid`
- `pentagon`
- `hexagon`
- `heptagon`
- `octagon`
- `decagon`
- `dodecagon`
- `star4`
- `star5`
- `star6`
- `star7`
- `star8`
- `star10`
- `star12`
- `star16`
- `star24`
- `star32`
- `roundRect`
- `round1Rect`
- `round2SameRect`
- `round2DiagRect`
- `snipRoundRect`
- `snip1Rect`
- `snip2SameRect`
- `snip2DiagRect`
- `plaque`
- `ellipse`
- `teardrop`
- `homePlate`
- `chevron`
- `pieWedge`
- `pie`
- `blockArc`
- `donut`
- `noSmoking`
- `rightArrow`
- `leftArrow`
- `upArrow`
- `downArrow`
- `stripedRightArrow`
- `notchedRightArrow`
- `bentUpArrow`
- `leftRightArrow`
- `upDownArrow`
- `leftUpArrow`
- `leftRightUpArrow`
- `quadArrow`
- `leftArrowCallout`
- `rightArrowCallout`
- `upArrowCallout`
- `downArrowCallout`
- `leftRightArrowCallout`
- `upDownArrowCallout`
- `quadArrowCallout`
- `bentArrow`
- `uturnArrow`
- `circularArrow`
- `leftCircularArrow`
- `leftRightCircularArrow`
- `curvedRightArrow`
- `curvedLeftArrow`
- `curvedUpArrow`
- `curvedDownArrow`
- `swooshArrow`
- `cube`
- `can`
- `lightningBolt`
- `heart`
- `sun`
- `moon`
- `smileyFace`
- `irregularSeal1`
- `irregularSeal2`
- `foldedCorner`
- `bevel`
- `frame`
- `halfFrame`
- `corner`
- `diagStripe`
- `chord`
- `arc`
- `leftBracket`
- `rightBracket`
- `leftBrace`
- `rightBrace`
- `bracketPair`
- `bracePair`
- `straightConnector1`
- `bentConnector2`
- `bentConnector3`
- `bentConnector4`
- `bentConnector5`
- `curvedConnector2`
- `curvedConnector3`
- `curvedConnector4`
- `curvedConnector5`
- `callout1`
- `callout2`
- `callout3`
- `accentCallout1`
- `accentCallout2`
- `accentCallout3`
- `borderCallout1`
- `borderCallout2`
- `borderCallout3`
- `accentBorderCallout1`
- `accentBorderCallout2`
- `accentBorderCallout3`
- `wedgeRectCallout`
- `wedgeRoundRectCallout`
- `wedgeEllipseCallout`
- `cloudCallout`
- `cloud`
- `ribbon`
- `ribbon2`
- `ellipseRibbon`
- `ellipseRibbon2`
- `leftRightRibbon`
- `verticalScroll`
- `horizontalScroll`
- `wave`
- `doubleWave`
- `plus`
- `flowChartProcess`
- `flowChartDecision`
- `flowChartInputOutput`
- `flowChartPredefinedProcess`
- `flowChartInternalStorage`
- `flowChartDocument`
- `flowChartMultidocument`
- `flowChartTerminator`
- `flowChartPreparation`
- `flowChartManualInput`
- `flowChartManualOperation`
- `flowChartConnector`
- `flowChartPunchedCard`
- `flowChartPunchedTape`
- `flowChartSummingJunction`
- `flowChartOr`
- `flowChartCollate`
- `flowChartSort`
- `flowChartExtract`
- `flowChartMerge`
- `flowChartOfflineStorage`
- `flowChartOnlineStorage`
- `flowChartMagneticTape`
- `flowChartMagneticDisk`
- `flowChartMagneticDrum`
- `flowChartDisplay`
- `flowChartDelay`
- `flowChartAlternateProcess`
- `flowChartOffpageConnector`
- `actionButtonBlank`
- `actionButtonHome`
- `actionButtonHelp`
- `actionButtonInformation`
- `actionButtonForwardNext`
- `actionButtonBackPrevious`
- `actionButtonEnd`
- `actionButtonBeginning`
- `actionButtonReturn`
- `actionButtonDocument`
- `actionButtonSound`
- `actionButtonMovie`
- `gear6`
- `gear9`
- `funnel`
- `mathPlus`
- `mathMinus`
- `mathMultiply`
- `mathDivide`
- `mathEqual`
- `mathNotEqual`
- `cornerTabs`
- `squareTabs`
- `plaqueTabs`
- `chartX`
- `chartStar`
- `chartPlus`

## `a:ST_TextShapeType`
Restriction: `base=token`
- `textNoShape`
- `textPlain`
- `textStop`
- `textTriangle`
- `textTriangleInverted`
- `textChevron`
- `textChevronInverted`
- `textRingInside`
- `textRingOutside`
- `textArchUp`
- `textArchDown`
- `textCircle`
- `textButton`
- `textArchUpPour`
- `textArchDownPour`
- `textCirclePour`
- `textButtonPour`
- `textCurveUp`
- `textCurveDown`
- `textCanUp`
- `textCanDown`
- `textWave1`
- `textWave2`
- `textDoubleWave1`
- `textWave4`
- `textInflate`
- `textDeflate`
- `textInflateBottom`
- `textDeflateBottom`
- `textInflateTop`
- `textDeflateTop`
- `textDeflateInflate`
- `textDeflateInflateDeflate`
- `textFadeRight`
- `textFadeLeft`
- `textFadeUp`
- `textFadeDown`
- `textSlantUp`
- `textSlantDown`
- `textCascadeUp`
- `textCascadeDown`

## `a:ST_GeomGuideName`
Restriction: `base=token`

## `a:ST_GeomGuideFormula`
Restriction: `base=string`

## `a:CT_GeomGuide`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@name` | `ST_GeomGuideName` | required |  |  |
| attribute | `@fmla` | `ST_GeomGuideFormula` | required |  |  |

## `a:CT_GeomGuideList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `gd` | `CT_GeomGuide` | 0..unbounded |  |  |

## `a:ST_AdjCoordinate`
Restriction: `union=ST_Coordinate ST_GeomGuideName`

## `a:ST_AdjAngle`
Restriction: `union=ST_Angle ST_GeomGuideName`

## `a:CT_AdjPoint2D`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@x` | `ST_AdjCoordinate` | required |  |  |
| attribute | `@y` | `ST_AdjCoordinate` | required |  |  |

## `a:CT_GeomRect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@l` | `ST_AdjCoordinate` | required |  |  |
| attribute | `@t` | `ST_AdjCoordinate` | required |  |  |
| attribute | `@r` | `ST_AdjCoordinate` | required |  |  |
| attribute | `@b` | `ST_AdjCoordinate` | required |  |  |

## `a:CT_XYAdjustHandle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `pos` | `CT_AdjPoint2D` |  |  |  |
| attribute | `@gdRefX` | `ST_GeomGuideName` | optional |  |  |
| attribute | `@minX` | `ST_AdjCoordinate` | optional |  |  |
| attribute | `@maxX` | `ST_AdjCoordinate` | optional |  |  |
| attribute | `@gdRefY` | `ST_GeomGuideName` | optional |  |  |
| attribute | `@minY` | `ST_AdjCoordinate` | optional |  |  |
| attribute | `@maxY` | `ST_AdjCoordinate` | optional |  |  |

## `a:CT_PolarAdjustHandle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `pos` | `CT_AdjPoint2D` |  |  |  |
| attribute | `@gdRefR` | `ST_GeomGuideName` | optional |  |  |
| attribute | `@minR` | `ST_AdjCoordinate` | optional |  |  |
| attribute | `@maxR` | `ST_AdjCoordinate` | optional |  |  |
| attribute | `@gdRefAng` | `ST_GeomGuideName` | optional |  |  |
| attribute | `@minAng` | `ST_AdjAngle` | optional |  |  |
| attribute | `@maxAng` | `ST_AdjAngle` | optional |  |  |

## `a:CT_ConnectionSite`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `pos` | `CT_AdjPoint2D` |  |  |  |
| attribute | `@ang` | `ST_AdjAngle` | required |  |  |

## `a:CT_AdjustHandleList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ahXY` | `CT_XYAdjustHandle` | choice |  |  |
| element | `ahPolar` | `CT_PolarAdjustHandle` | choice |  |  |

## `a:CT_ConnectionSiteList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cxn` | `CT_ConnectionSite` | 0..unbounded |  |  |

## `a:CT_Connection`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@id` | `ST_DrawingElementId` | required |  |  |
| attribute | `@idx` | `unsignedInt` | required |  |  |

## `a:CT_Path2DMoveTo`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `pt` | `CT_AdjPoint2D` |  |  |  |

## `a:CT_Path2DLineTo`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `pt` | `CT_AdjPoint2D` |  |  |  |

## `a:CT_Path2DArcTo`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@wR` | `ST_AdjCoordinate` | required |  |  |
| attribute | `@hR` | `ST_AdjCoordinate` | required |  |  |
| attribute | `@stAng` | `ST_AdjAngle` | required |  |  |
| attribute | `@swAng` | `ST_AdjAngle` | required |  |  |

## `a:CT_Path2DQuadBezierTo`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `pt` | `CT_AdjPoint2D` | 2..2 |  |  |

## `a:CT_Path2DCubicBezierTo`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `pt` | `CT_AdjPoint2D` | 3..3 |  |  |

## `a:CT_Path2DClose`

## `a:ST_PathFillMode`
Restriction: `base=token`
- `none`
- `norm`
- `lighten`
- `lightenLess`
- `darken`
- `darkenLess`

## `a:CT_Path2D`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `close` | `CT_Path2DClose` | choice |  |  |
| element | `moveTo` | `CT_Path2DMoveTo` | choice |  |  |
| element | `lnTo` | `CT_Path2DLineTo` | choice |  |  |
| element | `arcTo` | `CT_Path2DArcTo` | choice |  |  |
| element | `quadBezTo` | `CT_Path2DQuadBezierTo` | choice |  |  |
| element | `cubicBezTo` | `CT_Path2DCubicBezierTo` | choice |  |  |
| attribute | `@w` | `ST_PositiveCoordinate` | optional | 0 |  |
| attribute | `@h` | `ST_PositiveCoordinate` | optional | 0 |  |
| attribute | `@fill` | `ST_PathFillMode` | optional | norm |  |
| attribute | `@stroke` | `boolean` | optional | true |  |
| attribute | `@extrusionOk` | `boolean` | optional | true |  |

## `a:CT_Path2DList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `path` | `CT_Path2D` | 0..unbounded |  |  |

## `a:CT_PresetGeometry2D`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `avLst` | `CT_GeomGuideList` | 0..1 |  |  |
| attribute | `@prst` | `ST_ShapeType` | required |  |  |

## `a:CT_PresetTextShape`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `avLst` | `CT_GeomGuideList` | 0..1 |  |  |
| attribute | `@prst` | `ST_TextShapeType` | required |  |  |

## `a:CT_CustomGeometry2D`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `avLst` | `CT_GeomGuideList` | 0..1 |  |  |
| element | `gdLst` | `CT_GeomGuideList` | 0..1 |  |  |
| element | `ahLst` | `CT_AdjustHandleList` | 0..1 |  |  |
| element | `cxnLst` | `CT_ConnectionSiteList` | 0..1 |  |  |
| element | `rect` | `CT_GeomRect` | 0..1 |  |  |
| element | `pathLst` | `CT_Path2DList` |  |  |  |

## `a:EG_Geometry`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `custGeom` | `CT_CustomGeometry2D` | choice |  |  |
| element | `prstGeom` | `CT_PresetGeometry2D` | choice |  |  |

## `a:EG_TextGeometry`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `custGeom` | `CT_CustomGeometry2D` | choice |  |  |
| element | `prstTxWarp` | `CT_PresetTextShape` | choice |  |  |

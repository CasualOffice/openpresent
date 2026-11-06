# CHARTEX: Chart extensions (2016+ types)

Generated. Tier 1J; render 2F; edit 6; level Edit. 547 rows.


## `x:drawing/2014/chartex:ST_ExtensionDropMode`
Restriction: `base=string`
- `never`
- `onModelChange`
- `onDataChange`
- `always`

## `x:drawing/2014/chartex:CT_FeatureExtension`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| any | `xsd:any` | `` |  |  | lax |
| attribute | `@feature` | `string` | required |  |  |
| attribute | `@drop` | `ST_ExtensionDropMode` | optional | never |  |

## `x:drawing/2014/chartex:CT_Extension`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| any | `xsd:any` | `` |  |  | lax |
| attribute | `@uri` | `token` | optional |  |  |

## `x:drawing/2014/chartex:CT_ExtensionList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ext` | `CT_Extension` | 0..unbounded |  |  |

## `x:drawing/2014/chartex:ST_DoubleOrAutomatic`
Restriction: `base=string union=double`
- `auto`

## `x:drawing/2014/chartex:CT_NumberFormat`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@formatCode` | `string` | required |  |  |
| attribute | `@sourceLinked` | `boolean` | optional |  |  |

## `x:drawing/2014/chartex:CT_RelId`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@r:id` | `id` | required |  |  |

## `x:drawing/2014/chartex:CT_ValueColors`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `minColor` | `CT_SolidColorFillProperties` | 0..1 |  |  |
| element | `midColor` | `CT_SolidColorFillProperties` | 0..1 |  |  |
| element | `maxColor` | `CT_SolidColorFillProperties` | 0..1 |  |  |

## `x:drawing/2014/chartex:CT_ExternalData`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@r:id` | `id` | required |  |  |
| attribute | `@autoUpdate` | `boolean` | optional |  |  |

## `x:drawing/2014/chartex:ST_FormulaDirection`
Restriction: `base=string`
- `col`
- `row`

## `x:drawing/2014/chartex:CT_Formula`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| base | `string` | `string` |  |  |  |
| attribute | `@dir` | `ST_FormulaDirection` | optional | col |  |

## `x:drawing/2014/chartex:ST_DataId`
Restriction: `base=unsignedInt`

## `x:drawing/2014/chartex:CT_DataId`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_DataId` | required |  |  |

## `x:drawing/2014/chartex:CT_StringValue`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| base | `string` | `string` |  |  |  |
| attribute | `@idx` | `unsignedInt` | required |  |  |

## `x:drawing/2014/chartex:CT_StringLevel`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `pt` | `CT_StringValue` | 0..unbounded |  |  |
| attribute | `@ptCount` | `unsignedInt` | required |  |  |
| attribute | `@name` | `string` | optional |  |  |

## `x:drawing/2014/chartex:ST_StringDimensionType`
Restriction: `base=string`
- `cat`
- `colorStr`
- `entityId`

## `x:drawing/2014/chartex:CT_StringDimension`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `f` | `CT_Formula` | choice |  |  |
| element | `nf` | `CT_Formula` | 0..1 |  |  |
| element | `lvl` | `CT_StringLevel` | 0..unbounded |  |  |
| element | `lvl` | `CT_StringLevel` | 1..unbounded |  |  |
| attribute | `@type` | `ST_StringDimensionType` | required |  |  |

## `x:drawing/2014/chartex:CT_NumericValue`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| base | `double` | `double` |  |  |  |
| attribute | `@idx` | `unsignedInt` | required |  |  |

## `x:drawing/2014/chartex:CT_NumericLevel`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `pt` | `CT_NumericValue` | 0..unbounded |  |  |
| attribute | `@ptCount` | `unsignedInt` | required |  |  |
| attribute | `@formatCode` | `string` | optional |  |  |
| attribute | `@name` | `string` | optional |  |  |

## `x:drawing/2014/chartex:ST_NumericDimensionType`
Restriction: `base=string`
- `val`
- `x`
- `y`
- `size`
- `colorVal`

## `x:drawing/2014/chartex:CT_NumericDimension`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `f` | `CT_Formula` | choice |  |  |
| element | `nf` | `CT_Formula` | 0..1 |  |  |
| element | `lvl` | `CT_NumericLevel` | 0..unbounded |  |  |
| element | `lvl` | `CT_NumericLevel` | 1..unbounded |  |  |
| attribute | `@type` | `ST_NumericDimensionType` | required |  |  |

## `x:drawing/2014/chartex:CT_Data`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `numDim` | `CT_NumericDimension` | choice |  |  |
| element | `strDim` | `CT_StringDimension` | choice |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@id` | `ST_DataId` | required |  |  |

## `x:drawing/2014/chartex:CT_ChartData`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `externalData` | `CT_ExternalData` | 0..1 |  |  |
| element | `data` | `CT_Data` | 1..unbounded |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `x:drawing/2014/chartex:CT_TextData`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `f` | `CT_Formula` | choice |  |  |
| element | `v` | `string` | 0..1 |  |  |
| element | `v` | `string` | choice |  |  |

## `x:drawing/2014/chartex:CT_Text`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `txData` | `CT_TextData` | choice |  |  |
| element | `rich` | `CT_TextBody` | choice |  |  |

## `x:drawing/2014/chartex:ST_SidePos`
Restriction: `base=string`
- `l`
- `t`
- `r`
- `b`

## `x:drawing/2014/chartex:ST_PosAlign`
Restriction: `base=string`
- `min`
- `ctr`
- `max`

## `x:drawing/2014/chartex:CT_Offset`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@top` | `double` | required |  |  |
| attribute | `@left` | `double` | required |  |  |

## `x:drawing/2014/chartex:CT_ChartTitle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tx` | `CT_Text` | 0..1 |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `txPr` | `CT_TextBody` | 0..1 |  |  |
| element | `offset` | `CT_Offset` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@pos` | `ST_SidePos` | optional | t |  |
| attribute | `@align` | `ST_PosAlign` | optional | ctr |  |
| attribute | `@overlay` | `boolean` | optional | 0 |  |

## `x:drawing/2014/chartex:ST_AxisId`
Restriction: `base=unsignedInt`

## `x:drawing/2014/chartex:ST_GapWidthRatio`
Restriction: `base=double minInclusive=0 base=string union=`
- `auto`

## `x:drawing/2014/chartex:CT_CategoryAxisScaling`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@gapWidth` | `ST_GapWidthRatio` | optional |  |  |

## `x:drawing/2014/chartex:ST_ValueAxisUnit`
Restriction: `base=double minExclusive=0 base=string union=`
- `auto`

## `x:drawing/2014/chartex:CT_ValueAxisScaling`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@max` | `ST_DoubleOrAutomatic` | optional |  |  |
| attribute | `@min` | `ST_DoubleOrAutomatic` | optional |  |  |
| attribute | `@majorUnit` | `ST_ValueAxisUnit` | optional |  |  |
| attribute | `@minorUnit` | `ST_ValueAxisUnit` | optional |  |  |

## `x:drawing/2014/chartex:CT_AxisTitle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tx` | `CT_Text` | 0..1 |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `txPr` | `CT_TextBody` | 0..1 |  |  |
| element | `offset` | `CT_Offset` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `x:drawing/2014/chartex:CT_AxisUnitsLabel`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tx` | `CT_Text` | 0..1 |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `txPr` | `CT_TextBody` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `x:drawing/2014/chartex:ST_AxisUnit`
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
- `percentage`

## `x:drawing/2014/chartex:CT_AxisUnits`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `unitsLabel` | `CT_AxisUnitsLabel` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@unit` | `ST_AxisUnit` | optional |  |  |

## `x:drawing/2014/chartex:CT_Gridlines`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `x:drawing/2014/chartex:ST_TickMarksType`
Restriction: `base=string`
- `in`
- `out`
- `cross`
- `none`

## `x:drawing/2014/chartex:CT_TickMarks`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@type` | `ST_TickMarksType` | optional |  |  |

## `x:drawing/2014/chartex:CT_TickLabels`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `x:drawing/2014/chartex:CT_Axis`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `catScaling` | `CT_CategoryAxisScaling` | choice |  |  |
| element | `valScaling` | `CT_ValueAxisScaling` | choice |  |  |
| element | `title` | `CT_AxisTitle` | 0..1 |  |  |
| element | `units` | `CT_AxisUnits` | 0..1 |  |  |
| element | `majorGridlines` | `CT_Gridlines` | 0..1 |  |  |
| element | `minorGridlines` | `CT_Gridlines` | 0..1 |  |  |
| element | `majorTickMarks` | `CT_TickMarks` | 0..1 |  |  |
| element | `minorTickMarks` | `CT_TickMarks` | 0..1 |  |  |
| element | `tickLabels` | `CT_TickLabels` | 0..1 |  |  |
| element | `numFmt` | `CT_NumberFormat` | 0..1 |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `txPr` | `CT_TextBody` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@id` | `ST_AxisId` | required |  |  |
| attribute | `@hidden` | `boolean` | optional | 0 |  |

## `x:drawing/2014/chartex:ST_SeriesLayout`
Restriction: `base=string`
- `boxWhisker`
- `clusteredColumn`
- `funnel`
- `paretoLine`
- `regionMap`
- `sunburst`
- `treemap`
- `waterfall`

## `x:drawing/2014/chartex:ST_ParentLabelLayout`
Restriction: `base=string`
- `none`
- `banner`
- `overlapping`

## `x:drawing/2014/chartex:CT_ParentLabelLayout`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_ParentLabelLayout` | required |  |  |

## `x:drawing/2014/chartex:ST_RegionLabelLayout`
Restriction: `base=string`
- `none`
- `bestFitOnly`
- `showAll`

## `x:drawing/2014/chartex:CT_RegionLabelLayout`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_RegionLabelLayout` | required |  |  |

## `x:drawing/2014/chartex:CT_SeriesElementVisibilities`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@connectorLines` | `boolean` | optional |  |  |
| attribute | `@meanLine` | `boolean` | optional |  |  |
| attribute | `@meanMarker` | `boolean` | optional |  |  |
| attribute | `@nonoutliers` | `boolean` | optional |  |  |
| attribute | `@outliers` | `boolean` | optional |  |  |

## `x:drawing/2014/chartex:ST_IntervalClosedSide`
Restriction: `base=string`
- `l`
- `r`

## `x:drawing/2014/chartex:CT_Aggregation`

## `x:drawing/2014/chartex:CT_Binning`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `binSize` | `double` | choice |  |  |
| element | `binCount` | `unsignedInt` | choice |  |  |
| attribute | `@intervalClosed` | `ST_IntervalClosedSide` | optional |  |  |
| attribute | `@underflow` | `ST_DoubleOrAutomatic` | optional |  |  |
| attribute | `@overflow` | `ST_DoubleOrAutomatic` | optional |  |  |

## `x:drawing/2014/chartex:ST_EntityType`
Restriction: `base=string`
- `Address`
- `AdminDistrict`
- `AdminDistrict2`
- `AdminDistrict3`
- `Continent`
- `CountryRegion`
- `Locality`
- `Ocean`
- `Planet`
- `PostalCode`
- `Region`
- `Unsupported`

## `x:drawing/2014/chartex:CT_GeoLocationQuery`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@countryRegion` | `string` | optional |  |  |
| attribute | `@adminDistrict1` | `string` | optional |  |  |
| attribute | `@adminDistrict2` | `string` | optional |  |  |
| attribute | `@postalCode` | `string` | optional |  |  |
| attribute | `@entityType` | `ST_EntityType` | required |  |  |

## `x:drawing/2014/chartex:CT_Address`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@address1` | `string` | optional |  |  |
| attribute | `@countryRegion` | `string` | optional |  |  |
| attribute | `@adminDistrict1` | `string` | optional |  |  |
| attribute | `@adminDistrict2` | `string` | optional |  |  |
| attribute | `@postalCode` | `string` | optional |  |  |
| attribute | `@locality` | `string` | optional |  |  |
| attribute | `@isoCountryCode` | `string` | optional |  |  |

## `x:drawing/2014/chartex:CT_GeoLocation`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `address` | `CT_Address` | 0..1 |  |  |
| attribute | `@latitude` | `double` | optional |  |  |
| attribute | `@longitude` | `double` | optional |  |  |
| attribute | `@entityName` | `string` | required |  |  |
| attribute | `@entityType` | `ST_EntityType` | required |  |  |

## `x:drawing/2014/chartex:CT_GeoLocations`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `geoLocation` | `CT_GeoLocation` | 0..1 |  |  |

## `x:drawing/2014/chartex:CT_GeoLocationQueryResult`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `geoLocationQuery` | `CT_GeoLocationQuery` | 0..1 |  |  |
| element | `geoLocations` | `CT_GeoLocations` | 0..1 |  |  |

## `x:drawing/2014/chartex:CT_GeoLocationQueryResults`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `geoLocationQueryResult` | `CT_GeoLocationQueryResult` | 0..unbounded |  |  |

## `x:drawing/2014/chartex:CT_GeoPolygon`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@polygonId` | `string` | required |  |  |
| attribute | `@numPoints` | `integer` | required |  |  |
| attribute | `@pcaRings` | `string` | required |  |  |

## `x:drawing/2014/chartex:CT_GeoPolygons`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `geoPolygon` | `CT_GeoPolygon` | 0..unbounded |  |  |

## `x:drawing/2014/chartex:CT_Copyrights`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `copyright` | `string` | 0..unbounded |  |  |

## `x:drawing/2014/chartex:CT_GeoData`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `geoPolygons` | `CT_GeoPolygons` | 0..1 |  |  |
| element | `copyrights` | `CT_Copyrights` | 0..1 |  |  |
| attribute | `@entityName` | `string` | required |  |  |
| attribute | `@entityId` | `string` | required |  |  |
| attribute | `@east` | `double` | required |  |  |
| attribute | `@west` | `double` | required |  |  |
| attribute | `@north` | `double` | required |  |  |
| attribute | `@south` | `double` | required |  |  |

## `x:drawing/2014/chartex:CT_GeoDataEntityQuery`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@entityType` | `ST_EntityType` | required |  |  |
| attribute | `@entityId` | `string` | required |  |  |

## `x:drawing/2014/chartex:CT_GeoDataEntityQueryResult`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `geoDataEntityQuery` | `CT_GeoDataEntityQuery` | 0..1 |  |  |
| element | `geoData` | `CT_GeoData` | 0..1 |  |  |

## `x:drawing/2014/chartex:CT_GeoDataEntityQueryResults`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `geoDataEntityQueryResult` | `CT_GeoDataEntityQueryResult` | 0..unbounded |  |  |

## `x:drawing/2014/chartex:CT_GeoDataPointQuery`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@entityType` | `ST_EntityType` | required |  |  |
| attribute | `@latitude` | `double` | required |  |  |
| attribute | `@longitude` | `double` | required |  |  |

## `x:drawing/2014/chartex:CT_GeoDataPointToEntityQuery`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@entityType` | `ST_EntityType` | required |  |  |
| attribute | `@entityId` | `string` | required |  |  |

## `x:drawing/2014/chartex:CT_GeoDataPointToEntityQueryResult`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `geoDataPointQuery` | `CT_GeoDataPointQuery` | 0..1 |  |  |
| element | `geoDataPointToEntityQuery` | `CT_GeoDataPointToEntityQuery` | 0..1 |  |  |

## `x:drawing/2014/chartex:CT_GeoDataPointToEntityQueryResults`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `geoDataPointToEntityQueryResult` | `CT_GeoDataPointToEntityQueryResult` | 0..unbounded |  |  |

## `x:drawing/2014/chartex:CT_GeoChildTypes`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `entityType` | `ST_EntityType` | 0..unbounded |  |  |

## `x:drawing/2014/chartex:CT_GeoChildEntitiesQuery`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `geoChildTypes` | `CT_GeoChildTypes` | 0..1 |  |  |
| attribute | `@entityId` | `string` | required |  |  |

## `x:drawing/2014/chartex:CT_GeoHierarchyEntity`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@entityName` | `string` | required |  |  |
| attribute | `@entityId` | `string` | required |  |  |
| attribute | `@entityType` | `ST_EntityType` | required |  |  |

## `x:drawing/2014/chartex:CT_GeoChildEntities`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `geoHierarchyEntity` | `CT_GeoHierarchyEntity ` | 0..unbounded |  |  |

## `x:drawing/2014/chartex:CT_GeoChildEntitiesQueryResult`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `geoChildEntitiesQuery` | `CT_GeoChildEntitiesQuery` | 0..1 |  |  |
| element | `geoChildEntities` | `CT_GeoChildEntities` | 0..1 |  |  |

## `x:drawing/2014/chartex:CT_GeoChildEntitiesQueryResults`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `geoChildEntitiesQueryResult` | `CT_GeoChildEntitiesQueryResult` | 0..unbounded |  |  |

## `x:drawing/2014/chartex:CT_GeoParentEntitiesQuery`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@entityId` | `string` | required |  |  |

## `x:drawing/2014/chartex:CT_GeoEntity`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@entityName` | `string` | required |  |  |
| attribute | `@entityType` | `ST_EntityType` | required |  |  |

## `x:drawing/2014/chartex:CT_GeoParentEntity`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@entityId` | `string` | required |  |  |

## `x:drawing/2014/chartex:CT_GeoParentEntitiesQueryResult`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `geoParentEntitiesQuery` | `CT_GeoParentEntitiesQuery` |  |  |  |
| element | `geoEntity` | `CT_GeoEntity` | 0..1 |  |  |
| element | `geoParentEntity` | `CT_GeoParentEntity` | 0..1 |  |  |

## `x:drawing/2014/chartex:CT_GeoParentEntitiesQueryResults`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `geoParentEntitiesQueryResult` | `CT_GeoParentEntitiesQueryResult` | 0..unbounded |  |  |

## `x:drawing/2014/chartex:CT_Clear`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `geoLocationQueryResults` | `CT_GeoLocationQueryResults` | 0..1 |  |  |
| element | `geoDataEntityQueryResults` | `CT_GeoDataEntityQueryResults` | 0..1 |  |  |
| element | `geoDataPointToEntityQueryResults` | `CT_GeoDataPointToEntityQueryResults` | 0..1 |  |  |
| element | `geoChildEntitiesQueryResults` | `CT_GeoChildEntitiesQueryResults` | 0..1 |  |  |
| element | `geoParentEntitiesQueryResults` | `CT_GeoParentEntitiesQueryResults` | 0..1 |  |  |

## `x:drawing/2014/chartex:CT_GeoCache`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `binary` | `base64Binary` | 0..1 |  |  |
| element | `clear` | `CT_Clear` | 0..1 |  |  |
| attribute | `@provider` | `string` | required |  |  |

## `x:drawing/2014/chartex:ST_GeoProjectionType`
Restriction: `base=string`
- `mercator`
- `miller`
- `robinson`
- `albers`

## `x:drawing/2014/chartex:ST_GeoMappingLevel`
Restriction: `base=string`
- `dataOnly`
- `postalCode`
- `county`
- `state`
- `countryRegion`
- `countryRegionList`
- `world`

## `x:drawing/2014/chartex:CT_Geography`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `geoCache` | `CT_GeoCache` | 0..1 |  |  |
| attribute | `@projectionType` | `ST_GeoProjectionType` | optional |  |  |
| attribute | `@viewedRegionType` | `ST_GeoMappingLevel` | optional |  |  |
| attribute | `@cultureLanguage` | `language` | required |  |  |
| attribute | `@cultureRegion` | `string` | required |  |  |
| attribute | `@attribution` | `string` | required |  |  |

## `x:drawing/2014/chartex:ST_QuartileMethod`
Restriction: `base=string`
- `inclusive`
- `exclusive`

## `x:drawing/2014/chartex:CT_Statistics`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@quartileMethod` | `ST_QuartileMethod` | optional |  |  |

## `x:drawing/2014/chartex:CT_SubtotalIndex`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `unsignedInt` | required |  |  |

## `x:drawing/2014/chartex:CT_Subtotals`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `idx` | `CT_SubtotalIndex` | 0..unbounded |  |  |

## `x:drawing/2014/chartex:CT_SeriesLayoutProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `parentLabelLayout` | `CT_ParentLabelLayout` | 0..1 |  |  |
| element | `regionLabelLayout` | `CT_RegionLabelLayout` | 0..1 |  |  |
| element | `visibility` | `CT_SeriesElementVisibilities` | 0..1 |  |  |
| element | `aggregation` | `CT_Aggregation` | 0..1 |  |  |
| element | `binning` | `CT_Binning` | 0..1 |  |  |
| element | `geography` | `CT_Geography` | 0..1 |  |  |
| element | `statistics` | `CT_Statistics` | 0..1 |  |  |
| element | `subtotals` | `CT_Subtotals` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `x:drawing/2014/chartex:CT_DataPoint`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@idx` | `unsignedInt` | required |  |  |

## `x:drawing/2014/chartex:ST_DataLabelPos`
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

## `x:drawing/2014/chartex:ST_ValueColorPositionCount`
Restriction: `base=int minInclusive=2 maxInclusive=3`

## `x:drawing/2014/chartex:CT_ExtremeValueColorPosition`

## `x:drawing/2014/chartex:CT_NumberColorPosition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `double` | required |  |  |

## `x:drawing/2014/chartex:CT_PercentageColorPosition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `double` | required |  |  |

## `x:drawing/2014/chartex:CT_ValueColorEndPosition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extremeValue` | `CT_ExtremeValueColorPosition` | choice |  |  |
| element | `number` | `CT_NumberColorPosition` | choice |  |  |
| element | `percent` | `CT_PercentageColorPosition` | choice |  |  |

## `x:drawing/2014/chartex:CT_ValueColorMiddlePosition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `number` | `CT_NumberColorPosition` | choice |  |  |
| element | `percent` | `CT_PercentageColorPosition` | choice |  |  |

## `x:drawing/2014/chartex:CT_ValueColorPositions`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `min` | `CT_ValueColorEndPosition` | 0..1 |  |  |
| element | `mid` | `CT_ValueColorMiddlePosition` | 0..1 |  |  |
| element | `max` | `CT_ValueColorEndPosition` | 0..1 |  |  |
| attribute | `@count` | `ST_ValueColorPositionCount` | optional | 2 |  |

## `x:drawing/2014/chartex:CT_DataLabelVisibilities`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@seriesName` | `boolean` | optional |  |  |
| attribute | `@categoryName` | `boolean` | optional |  |  |
| attribute | `@value` | `boolean` | optional |  |  |

## `x:drawing/2014/chartex:CT_DataLabel`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `numFmt` | `CT_NumberFormat` | 0..1 |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `txPr` | `CT_TextBody` | 0..1 |  |  |
| element | `visibility` | `CT_DataLabelVisibilities` | 0..1 |  |  |
| element | `separator` | `string` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@idx` | `unsignedInt` | required |  |  |
| attribute | `@pos` | `ST_DataLabelPos` | optional |  |  |

## `x:drawing/2014/chartex:CT_DataLabelHidden`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@idx` | `unsignedInt` | required |  |  |

## `x:drawing/2014/chartex:CT_DataLabels`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `numFmt` | `CT_NumberFormat` | 0..1 |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `txPr` | `CT_TextBody` | 0..1 |  |  |
| element | `visibility` | `CT_DataLabelVisibilities` | 0..1 |  |  |
| element | `separator` | `string` | 0..1 |  |  |
| element | `dataLabel` | `CT_DataLabel` | 0..unbounded |  |  |
| element | `dataLabelHidden` | `CT_DataLabelHidden` | 0..unbounded |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@pos` | `ST_DataLabelPos` | optional |  |  |

## `x:drawing/2014/chartex:CT_Series`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tx` | `CT_Text` | 0..1 |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `valueColors` | `CT_ValueColors` | 0..1 |  |  |
| element | `valueColorPositions` | `CT_ValueColorPositions` | 0..1 |  |  |
| element | `dataPt` | `CT_DataPoint` | 0..unbounded |  |  |
| element | `dataLabels` | `CT_DataLabels` | 0..1 |  |  |
| element | `dataId` | `CT_DataId` | 0..1 |  |  |
| element | `layoutPr` | `CT_SeriesLayoutProperties` | 0..1 |  |  |
| element | `axisId` | `ST_AxisId` | 0..unbounded |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@layoutId` | `ST_SeriesLayout` | required |  |  |
| attribute | `@hidden` | `boolean` | optional | 0 |  |
| attribute | `@ownerIdx` | `unsignedInt` | optional |  |  |
| attribute | `@uniqueId` | `string` | optional |  |  |
| attribute | `@formatIdx` | `unsignedInt` | optional |  |  |

## `x:drawing/2014/chartex:CT_Legend`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `txPr` | `CT_TextBody` | 0..1 |  |  |
| element | `offset` | `CT_Offset` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@pos` | `ST_SidePos` | optional | r |  |
| attribute | `@align` | `ST_PosAlign` | optional | ctr |  |
| attribute | `@overlay` | `boolean` | optional | 0 |  |

## `x:drawing/2014/chartex:CT_PlotSurface`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `x:drawing/2014/chartex:CT_PlotAreaRegion`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `plotSurface` | `CT_PlotSurface` | 0..1 |  |  |
| element | `series` | `CT_Series` | 0..unbounded |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `x:drawing/2014/chartex:CT_PlotArea`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `plotAreaRegion` | `CT_PlotAreaRegion` |  |  |  |
| element | `axis` | `CT_Axis` | 0..unbounded |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `x:drawing/2014/chartex:CT_Chart`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `title` | `CT_ChartTitle` | 0..1 |  |  |
| element | `plotArea` | `CT_PlotArea` |  |  |  |
| element | `legend` | `CT_Legend` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `x:drawing/2014/chartex:ST_BindingType`
Restriction: `union=ST_NumericDimensionType ST_StringDimensionType`

## `x:drawing/2014/chartex:CT_Binding`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@name` | `string` | required |  |  |
| attribute | `@id` | `ST_DataId` | required |  |  |
| attribute | `@type` | `ST_BindingType` | required |  |  |

## `x:drawing/2014/chartex:CT_Bindings`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `binding` | `CT_Binding` | 0..unbounded |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `x:drawing/2014/chartex:ST_ArtifactType`
Restriction: `base=string`
- `html`

## `x:drawing/2014/chartex:CT_Artifact`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| base | `string` | `string` |  |  |  |
| attribute | `@source` | `string` | optional |  |  |
| attribute | `@name` | `string` | required |  |  |
| attribute | `@type` | `ST_ArtifactType` | required |  |  |

## `x:drawing/2014/chartex:CT_Artifacts`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `artifact` | `CT_Artifact` | 1..unbounded |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |

## `x:drawing/2014/chartex:CT_Dashboard`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `bindings` | `CT_Bindings` |  |  |  |
| element | `artifacts` | `CT_Artifacts` |  |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@entrypoint` | `string` | required |  |  |

## `x:drawing/2014/chartex:CT_FormatOverride`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@idx` | `unsignedInt` | required |  |  |

## `x:drawing/2014/chartex:CT_FormatOverrides`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `fmtOvr` | `CT_FormatOverride` | 0..unbounded |  |  |

## `x:drawing/2014/chartex:CT_HeaderFooter`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `oddHeader` | `string` | 0..1 |  |  |
| element | `oddFooter` | `string` | 0..1 |  |  |
| element | `evenHeader` | `string` | 0..1 |  |  |
| element | `evenFooter` | `string` | 0..1 |  |  |
| element | `firstHeader` | `string` | 0..1 |  |  |
| element | `firstFooter` | `string` | 0..1 |  |  |
| attribute | `@alignWithMargins` | `boolean` | optional | true |  |
| attribute | `@differentOddEven` | `boolean` | optional | false |  |
| attribute | `@differentFirst` | `boolean` | optional | false |  |

## `x:drawing/2014/chartex:CT_PageMargins`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@l` | `double` | required |  |  |
| attribute | `@r` | `double` | required |  |  |
| attribute | `@t` | `double` | required |  |  |
| attribute | `@b` | `double` | required |  |  |
| attribute | `@header` | `double` | required |  |  |
| attribute | `@footer` | `double` | required |  |  |

## `x:drawing/2014/chartex:ST_PageOrientation`
Restriction: `base=string`
- `default`
- `portrait`
- `landscape`

## `x:drawing/2014/chartex:CT_PageSetup`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@paperSize` | `unsignedInt` | optional | 1 |  |
| attribute | `@firstPageNumber` | `unsignedInt` | optional | 1 |  |
| attribute | `@orientation` | `ST_PageOrientation` | optional | default |  |
| attribute | `@blackAndWhite` | `boolean` | optional | false |  |
| attribute | `@draft` | `boolean` | optional | false |  |
| attribute | `@useFirstPageNumber` | `boolean` | optional | false |  |
| attribute | `@horizontalDpi` | `int` | optional | 600 |  |
| attribute | `@verticalDpi` | `int` | optional | 600 |  |
| attribute | `@copies` | `unsignedInt` | optional | 1 |  |

## `x:drawing/2014/chartex:CT_PrintSettings`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `headerFooter` | `CT_HeaderFooter` | 0..1 |  |  |
| element | `pageMargins` | `CT_PageMargins` | 0..1 |  |  |
| element | `pageSetup` | `CT_PageSetup` | 0..1 |  |  |

## `x:drawing/2014/chartex:CT_ChartSpace`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `chartData` | `CT_ChartData` |  |  |  |
| element | `chart` | `CT_Chart` | choice |  |  |
| element | `dashboard` | `CT_Dashboard` | choice |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `txPr` | `CT_TextBody` | 0..1 |  |  |
| element | `clrMapOvr` | `CT_ColorMapping` | 0..1 |  |  |
| element | `fmtOvrs` | `CT_FormatOverrides` | 0..1 |  |  |
| element | `printSettings` | `CT_PrintSettings` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionList` | 0..1 |  |  |
| attribute | `@version` | `string` | optional | 0.0 |  |
| attribute | `@featureList` | `string` | optional |  |  |
| attribute | `@fallbackImg` | `string` | optional |  |  |

## `x:drawing/2014/chartex:<chartSpace>` : `CT_ChartSpace`

## `x:drawing/2014/chartex:<chart>` : `CT_RelId`

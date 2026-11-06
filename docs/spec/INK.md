# INK: Ink

Generated. Tier 1K; render 2F; edit 6; level Edit. 152 rows.


## `x:ink/2010/main:ST_KnownCtxNodeType`
Restriction: `base=string`
- `root`
- `unclassifiedInk`
- `writingRegion`
- `analysisHint`
- `object`
- `inkDrawing`
- `image`
- `paragraph`
- `line`
- `inkBullet`
- `inkWord`
- `textWord`
- `customRecognizer`
- `mathRegion`
- `mathEquation`
- `mathStruct`
- `mathSymbol`
- `mathIdentifier`
- `mathOperator`
- `mathNumber`
- `nonInkDrawing`
- `groupNode`
- `mixedDrawing`

## `x:ink/2010/main:ST_Guid`
Restriction: `base=token pattern=\{[0-9A-F]{8}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{12}\}`

## `x:ink/2010/main:ST_Ref`
Restriction: `union=ST_Guid unsignedInt`

## `x:ink/2010/main:ST_CtxNodeType`
Restriction: `union=ST_KnownCtxNodeType ST_Guid`

## `x:ink/2010/main:ST_Dir`
Restriction: `base=string`
- `to`
- `from`
- `with`

## `x:ink/2010/main:ST_KnownSemanticType`
Restriction: `base=string`
- `none`
- `underline`
- `strikethrough`
- `highlight`
- `scratchOut`
- `verticalRange`
- `callout`
- `enclosure`
- `comment`
- `container`
- `connector`

## `x:ink/2010/main:ST_SemanticType`
Restriction: `union=ST_KnownSemanticType unsignedInt`

## `x:ink/2010/main:ST_Point`
Restriction: `base=string pattern=-?[0-9]+,-?[0-9]+`

## `x:ink/2010/main:ST_Points`
Restriction: `list=ST_Point`

## `x:ink/2010/main:CT_Property`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| base | `hexBinary` | `hexBinary` |  |  |  |
| attribute | `@type` | `ST_Guid` | optional |  |  |

## `x:ink/2010/main:CT_CtxLink`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@direction` | `ST_Dir` | optional |  |  |
| attribute | `@ref` | `ST_Ref` | optional |  |  |

## `x:ink/2010/main:CT_CtxNode`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `property` | `CT_Property` | 0..unbounded |  |  |
| element | `sourceLink` | `CT_CtxLink` | 0..unbounded |  |  |
| element | `destinationLink` | `CT_CtxLink` | 0..unbounded |  |  |
| attribute | `@id` | `ST_Guid` | optional |  |  |
| attribute | `@type` | `ST_CtxNodeType` | required |  |  |
| attribute | `@rotatedBoundingBox` | `ST_Points` | optional |  |  |
| attribute | `@alignmentLevel` | `int` | optional | 0 |  |
| attribute | `@contentType` | `int` | optional | 0 |  |
| attribute | `@ascender` | `ST_Points` | optional | 0,0 |  |
| attribute | `@descender` | `ST_Points` | optional | 0,0 |  |
| attribute | `@baseline` | `ST_Points` | optional | 0,0 |  |
| attribute | `@midline` | `ST_Points` | optional | 0,0 |  |
| attribute | `@customRecognizerId` | `ST_Guid` | optional |  |  |
| attribute | `@mathML` | `string` | optional |  |  |
| attribute | `@mathStruct` | `string` | optional |  |  |
| attribute | `@mathSymbol` | `string` | optional |  |  |
| attribute | `@beginModifierType` | `string` | optional |  |  |
| attribute | `@endModifierType` | `string` | optional |  |  |
| attribute | `@rotationAngle` | `int` | optional | 0 |  |
| attribute | `@hotPoints` | `ST_Points` | optional |  |  |
| attribute | `@centroid` | `ST_Point` | optional |  |  |
| attribute | `@semanticType` | `ST_SemanticType` | optional | none |  |
| attribute | `@shapeName` | `string` | optional |  |  |
| attribute | `@shapeGeometry` | `ST_Points` | optional |  |  |

## `x:ink/2010/main:<context>` : `CT_CtxNode`

## `x:powerpoint/2014/inkAction:ST_DataNameReserved`
Restriction: `base=string`
- `stroke`
- `path`
- `target`

## `x:powerpoint/2014/inkAction:ST_DataNameUser`
Restriction: `base=string`

## `x:powerpoint/2014/inkAction:ST_DataName`
Restriction: `union=ST_DataNameReserved ST_DataNameUser`

## `x:powerpoint/2014/inkAction:ST_PropertyNameReserved`
Restriction: `base=string`
- `dataType`
- `style`

## `x:powerpoint/2014/inkAction:ST_PropertyNameUser`
Restriction: `base=string`

## `x:powerpoint/2014/inkAction:ST_PropertyName`
Restriction: `union=ST_PropertyNameReserved ST_PropertyNameUser`

## `x:powerpoint/2014/inkAction:ST_PropertyValueReserved`
Restriction: `base=string`
- `ink`
- `pointEraser`
- `strokeEraser`
- `instant`

## `x:powerpoint/2014/inkAction:ST_PropertyValueUser`
Restriction: `base=string`

## `x:powerpoint/2014/inkAction:ST_PropertyValue`
Restriction: `union=ST_PropertyValueReserved ST_PropertyValueUser`

## `x:powerpoint/2014/inkAction:ST_ActionTypeReserved`
Restriction: `base=string`
- `add`
- `remove`
- `transform`

## `x:powerpoint/2014/inkAction:ST_ActionTypeUser`
Restriction: `base=string`

## `x:powerpoint/2014/inkAction:ST_ActionType`
Restriction: `union=ST_ActionTypeReserved ST_ActionTypeUser`

## `x:powerpoint/2014/inkAction:CT_ActionData`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `transform` | `CT_Matrix` | 0..1 |  |  |
| element | `inkml:trace` | `trace` | choice |  |  |
| element | `inkml:traceView` | `traceView` | choice |  |  |
| attribute | `@xml:id` | `id` | optional |  |  |
| attribute | `@name` | `ST_DataName` | optional | stroke |  |
| attribute | `@ref` | `anyURI` | optional |  |  |

## `x:powerpoint/2014/inkAction:CT_ActionDataGroup`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `actionData` | `CT_ActionData` | 1..unbounded |  |  |
| attribute | `@xml:id` | `id` | optional |  |  |
| attribute | `@name` | `ST_DataName` | optional | stroke |  |

## `x:powerpoint/2014/inkAction:CT_ActionProperty`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@name` | `ST_PropertyName` | required |  |  |
| attribute | `@value` | `ST_PropertyValue` | optional | ink |  |

## `x:powerpoint/2014/inkAction:CT_Action`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `property` | `CT_ActionProperty` | 0..unbounded |  |  |
| element | `actionData` | `CT_ActionData` | choice |  |  |
| element | `actionDataGroup` | `CT_ActionDataGroup` | choice |  |  |
| attribute | `@xml:id` | `id` | optional |  |  |
| attribute | `@type` | `ST_ActionType` | required |  |  |
| attribute | `@startTime` | `decimal` | required |  |  |

## `x:powerpoint/2014/inkAction:CT_ActionGroup`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `action` | `CT_Action` | 1..unbounded |  |  |
| attribute | `@xml:id` | `id` | optional |  |  |
| attribute | `@type` | `ST_ActionType` | required |  |  |
| attribute | `@startTime` | `decimal` | required |  |  |

## `x:powerpoint/2014/inkAction:CT_Actions`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `inkml:definitions` | `definitions` | 0..1 |  |  |
| element | `actionGroup` | `CT_ActionGroup` | choice |  |  |
| element | `action` | `CT_Action` | choice |  |  |
| attribute | `@xml:id` | `id` | optional |  |  |
| attribute | `@lengthUnit` | `ST_StandardLengthUnits` | required |  |  |
| attribute | `@timeUnit` | `ST_StandardTimeUnits` | required |  |  |

## `x:powerpoint/2014/inkAction:<actions>` : `CT_Actions`

## `x:drawing/2016/ink:ST_ExtendedBrushPropertyName`
Restriction: `base=string`
- `inkEffects`
- `anchorX`
- `anchorY`
- `scaleFactor`

## `x:drawing/2016/ink:ST_InkEffectsType`
Restriction: `base=string`
- `none`
- `pencil`
- `rainbow`
- `galaxy`
- `gold`
- `silver`
- `lava`
- `ocean`
- `rosegold`
- `bronze`

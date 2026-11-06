# DGM: SmartArt diagrams

Generated. Tier 1K; render 2F; edit 6; level Edit. 785 rows.


## `dgm:CT_CTName`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@lang` | `string` | optional |  |  |
| attribute | `@val` | `string` | required |  |  |

## `dgm:CT_CTDescription`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@lang` | `string` | optional |  |  |
| attribute | `@val` | `string` | required |  |  |

## `dgm:CT_CTCategory`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@type` | `anyURI` | required |  |  |
| attribute | `@pri` | `unsignedInt` | required |  |  |

## `dgm:CT_CTCategories`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cat` | `CT_CTCategory` | 0..unbounded |  |  |

## `dgm:ST_ClrAppMethod`
Restriction: `base=token`
- `span`
- `cycle`
- `repeat`

## `dgm:ST_HueDir`
Restriction: `base=token`
- `cw`
- `ccw`

## `dgm:CT_Colors`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ColorChoice` | `EG_ColorChoice` | 0..unbounded |  |  |
| attribute | `@meth` | `ST_ClrAppMethod` | optional | span |  |
| attribute | `@hueDir` | `ST_HueDir` | optional | cw |  |

## `dgm:CT_CTStyleLabel`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `fillClrLst` | `CT_Colors` | 0..1 |  |  |
| element | `linClrLst` | `CT_Colors` | 0..1 |  |  |
| element | `effectClrLst` | `CT_Colors` | 0..1 |  |  |
| element | `txLinClrLst` | `CT_Colors` | 0..1 |  |  |
| element | `txFillClrLst` | `CT_Colors` | 0..1 |  |  |
| element | `txEffectClrLst` | `CT_Colors` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@name` | `string` | required |  |  |

## `dgm:CT_ColorTransform`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `title` | `CT_CTName` | 0..unbounded |  |  |
| element | `desc` | `CT_CTDescription` | 0..unbounded |  |  |
| element | `catLst` | `CT_CTCategories` | 0..1 |  |  |
| element | `styleLbl` | `CT_CTStyleLabel` | 0..unbounded |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@uniqueId` | `string` | optional |  |  |
| attribute | `@minVer` | `string` | optional |  |  |

## `dgm:<colorsDef>` : `CT_ColorTransform`

## `dgm:CT_ColorTransformHeader`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `title` | `CT_CTName` | 1..unbounded |  |  |
| element | `desc` | `CT_CTDescription` | 1..unbounded |  |  |
| element | `catLst` | `CT_CTCategories` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@uniqueId` | `string` | required |  |  |
| attribute | `@minVer` | `string` | optional |  |  |
| attribute | `@resId` | `int` | optional | 0 |  |

## `dgm:<colorsDefHdr>` : `CT_ColorTransformHeader`

## `dgm:CT_ColorTransformHeaderLst`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `colorsDefHdr` | `CT_ColorTransformHeader` | 0..unbounded |  |  |

## `dgm:<colorsDefHdrLst>` : `CT_ColorTransformHeaderLst`

## `dgm:ST_PtType`
Restriction: `base=token`
- `node`
- `asst`
- `doc`
- `pres`
- `parTrans`
- `sibTrans`

## `dgm:CT_Pt`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `prSet` | `CT_ElemPropSet` | 0..1 |  |  |
| element | `spPr` | `CT_ShapeProperties` | 0..1 |  |  |
| element | `t` | `CT_TextBody` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@modelId` | `ST_ModelId` | required |  |  |
| attribute | `@type` | `ST_PtType` | optional | node |  |
| attribute | `@cxnId` | `ST_ModelId` | optional | 0 |  |

## `dgm:CT_PtList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `pt` | `CT_Pt` | 0..unbounded |  |  |

## `dgm:ST_CxnType`
Restriction: `base=token`
- `parOf`
- `presOf`
- `presParOf`
- `unknownRelationship`

## `dgm:CT_Cxn`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@modelId` | `ST_ModelId` | required |  |  |
| attribute | `@type` | `ST_CxnType` | optional | parOf |  |
| attribute | `@srcId` | `ST_ModelId` | required |  |  |
| attribute | `@destId` | `ST_ModelId` | required |  |  |
| attribute | `@srcOrd` | `unsignedInt` | required |  |  |
| attribute | `@destOrd` | `unsignedInt` | required |  |  |
| attribute | `@parTransId` | `ST_ModelId` | optional | 0 |  |
| attribute | `@sibTransId` | `ST_ModelId` | optional | 0 |  |
| attribute | `@presId` | `string` | optional |  |  |

## `dgm:CT_CxnList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cxn` | `CT_Cxn` | 0..unbounded |  |  |

## `dgm:CT_DataModel`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ptLst` | `CT_PtList` |  |  |  |
| element | `cxnLst` | `CT_CxnList` | 0..1 |  |  |
| element | `bg` | `CT_BackgroundFormatting` | 0..1 |  |  |
| element | `whole` | `CT_WholeE2oFormatting` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `dgm:<dataModel>` : `CT_DataModel`

## `dgm:AG_IteratorAttributes`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@axis` | `ST_AxisTypes` | optional | none |  |
| attribute | `@ptType` | `ST_ElementTypes` | optional | all |  |
| attribute | `@hideLastTrans` | `ST_Booleans` | optional | true |  |
| attribute | `@st` | `ST_Ints` | optional | 1 |  |
| attribute | `@cnt` | `ST_UnsignedInts` | optional | 0 |  |
| attribute | `@step` | `ST_Ints` | optional | 1 |  |

## `dgm:AG_ConstraintAttributes`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@type` | `ST_ConstraintType` | required |  |  |
| attribute | `@for` | `ST_ConstraintRelationship` | optional | self |  |
| attribute | `@forName` | `string` | optional |  |  |
| attribute | `@ptType` | `ST_ElementType` | optional | all |  |

## `dgm:AG_ConstraintRefAttributes`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@refType` | `ST_ConstraintType` | optional | none |  |
| attribute | `@refFor` | `ST_ConstraintRelationship` | optional | self |  |
| attribute | `@refForName` | `string` | optional |  |  |
| attribute | `@refPtType` | `ST_ElementType` | optional | all |  |

## `dgm:CT_Constraint`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attrgroup | `AG_ConstraintAttributes` | `AG_ConstraintAttributes` |  |  |  |
| attrgroup | `AG_ConstraintRefAttributes` | `AG_ConstraintRefAttributes` |  |  |  |
| attribute | `@op` | `ST_BoolOperator` | optional | none |  |
| attribute | `@val` | `double` | optional | 0 |  |
| attribute | `@fact` | `double` | optional | 1 |  |

## `dgm:CT_Constraints`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `constr` | `CT_Constraint` | 0..unbounded |  |  |

## `dgm:CT_NumericRule`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attrgroup | `AG_ConstraintAttributes` | `AG_ConstraintAttributes` |  |  |  |
| attribute | `@val` | `double` | optional | NaN |  |
| attribute | `@fact` | `double` | optional | NaN |  |
| attribute | `@max` | `double` | optional | NaN |  |

## `dgm:CT_Rules`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `rule` | `CT_NumericRule` | 0..unbounded |  |  |

## `dgm:CT_PresentationOf`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attrgroup | `AG_IteratorAttributes` | `AG_IteratorAttributes` |  |  |  |

## `dgm:ST_LayoutShapeType`
Restriction: `union=ST_ShapeType ST_OutputShapeType`

## `dgm:ST_Index1`
Restriction: `base=unsignedInt minInclusive=1`

## `dgm:CT_Adj`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@idx` | `ST_Index1` | required |  |  |
| attribute | `@val` | `double` | required |  |  |

## `dgm:CT_AdjLst`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `adj` | `CT_Adj` | 0..unbounded |  |  |

## `dgm:CT_Shape`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `adjLst` | `CT_AdjLst` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@rot` | `double` | optional | 0 |  |
| attribute | `@type` | `ST_LayoutShapeType` | optional | none |  |
| attribute | `@r:blip` | `blip` | optional |  |  |
| attribute | `@zOrderOff` | `int` | optional | 0 |  |
| attribute | `@hideGeom` | `boolean` | optional | false |  |
| attribute | `@lkTxEntry` | `boolean` | optional | false |  |
| attribute | `@blipPhldr` | `boolean` | optional | false |  |

## `dgm:CT_Parameter`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@type` | `ST_ParameterId` | required |  |  |
| attribute | `@val` | `ST_ParameterVal` | required |  |  |

## `dgm:CT_Algorithm`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `param` | `CT_Parameter` | 0..unbounded |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@type` | `ST_AlgorithmType` | required |  |  |
| attribute | `@rev` | `unsignedInt` | optional | 0 |  |

## `dgm:CT_LayoutNode`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `alg` | `CT_Algorithm` | 0..1 |  |  |
| element | `shape` | `CT_Shape` | 0..1 |  |  |
| element | `presOf` | `CT_PresentationOf` | 0..1 |  |  |
| element | `constrLst` | `CT_Constraints` | 0..1 |  |  |
| element | `ruleLst` | `CT_Rules` | 0..1 |  |  |
| element | `varLst` | `CT_LayoutVariablePropertySet` | 0..1 |  |  |
| element | `forEach` | `CT_ForEach` | choice |  |  |
| element | `layoutNode` | `CT_LayoutNode` | choice |  |  |
| element | `choose` | `CT_Choose` | choice |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@name` | `string` | optional |  |  |
| attribute | `@styleLbl` | `string` | optional |  |  |
| attribute | `@chOrder` | `ST_ChildOrderType` | optional | b |  |
| attribute | `@moveWith` | `string` | optional |  |  |

## `dgm:CT_ForEach`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `alg` | `CT_Algorithm` | 0..1 |  |  |
| element | `shape` | `CT_Shape` | 0..1 |  |  |
| element | `presOf` | `CT_PresentationOf` | 0..1 |  |  |
| element | `constrLst` | `CT_Constraints` | 0..1 |  |  |
| element | `ruleLst` | `CT_Rules` | 0..1 |  |  |
| element | `forEach` | `CT_ForEach` | choice |  |  |
| element | `layoutNode` | `CT_LayoutNode` | choice |  |  |
| element | `choose` | `CT_Choose` | choice |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@name` | `string` | optional |  |  |
| attribute | `@ref` | `string` | optional |  |  |
| attrgroup | `AG_IteratorAttributes` | `AG_IteratorAttributes` |  |  |  |

## `dgm:CT_When`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `alg` | `CT_Algorithm` | 0..1 |  |  |
| element | `shape` | `CT_Shape` | 0..1 |  |  |
| element | `presOf` | `CT_PresentationOf` | 0..1 |  |  |
| element | `constrLst` | `CT_Constraints` | 0..1 |  |  |
| element | `ruleLst` | `CT_Rules` | 0..1 |  |  |
| element | `forEach` | `CT_ForEach` | choice |  |  |
| element | `layoutNode` | `CT_LayoutNode` | choice |  |  |
| element | `choose` | `CT_Choose` | choice |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@name` | `string` | optional |  |  |
| attrgroup | `AG_IteratorAttributes` | `AG_IteratorAttributes` |  |  |  |
| attribute | `@func` | `ST_FunctionType` | required |  |  |
| attribute | `@arg` | `ST_FunctionArgument` | optional | none |  |
| attribute | `@op` | `ST_FunctionOperator` | required |  |  |
| attribute | `@val` | `ST_FunctionValue` | required |  |  |

## `dgm:CT_Otherwise`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `alg` | `CT_Algorithm` | 0..1 |  |  |
| element | `shape` | `CT_Shape` | 0..1 |  |  |
| element | `presOf` | `CT_PresentationOf` | 0..1 |  |  |
| element | `constrLst` | `CT_Constraints` | 0..1 |  |  |
| element | `ruleLst` | `CT_Rules` | 0..1 |  |  |
| element | `forEach` | `CT_ForEach` | choice |  |  |
| element | `layoutNode` | `CT_LayoutNode` | choice |  |  |
| element | `choose` | `CT_Choose` | choice |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@name` | `string` | optional |  |  |

## `dgm:CT_Choose`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `if` | `CT_When` | 1..unbounded |  |  |
| element | `else` | `CT_Otherwise` | 0..1 |  |  |
| attribute | `@name` | `string` | optional |  |  |

## `dgm:CT_SampleData`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `dataModel` | `CT_DataModel` | 0..1 |  |  |
| attribute | `@useDef` | `boolean` | optional | false |  |

## `dgm:CT_Category`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@type` | `anyURI` | required |  |  |
| attribute | `@pri` | `unsignedInt` | required |  |  |

## `dgm:CT_Categories`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cat` | `CT_Category` | 0..unbounded |  |  |

## `dgm:CT_Name`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@lang` | `string` | optional |  |  |
| attribute | `@val` | `string` | required |  |  |

## `dgm:CT_Description`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@lang` | `string` | optional |  |  |
| attribute | `@val` | `string` | required |  |  |

## `dgm:CT_DiagramDefinition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `title` | `CT_Name` | 0..unbounded |  |  |
| element | `desc` | `CT_Description` | 0..unbounded |  |  |
| element | `catLst` | `CT_Categories` | 0..1 |  |  |
| element | `sampData` | `CT_SampleData` | 0..1 |  |  |
| element | `styleData` | `CT_SampleData` | 0..1 |  |  |
| element | `clrData` | `CT_SampleData` | 0..1 |  |  |
| element | `layoutNode` | `CT_LayoutNode` |  |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@uniqueId` | `string` | optional |  |  |
| attribute | `@minVer` | `string` | optional |  |  |
| attribute | `@defStyle` | `string` | optional |  |  |

## `dgm:<layoutDef>` : `CT_DiagramDefinition`

## `dgm:CT_DiagramDefinitionHeader`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `title` | `CT_Name` | 1..unbounded |  |  |
| element | `desc` | `CT_Description` | 1..unbounded |  |  |
| element | `catLst` | `CT_Categories` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@uniqueId` | `string` | required |  |  |
| attribute | `@minVer` | `string` | optional |  |  |
| attribute | `@defStyle` | `string` | optional |  |  |
| attribute | `@resId` | `int` | optional | 0 |  |

## `dgm:<layoutDefHdr>` : `CT_DiagramDefinitionHeader`

## `dgm:CT_DiagramDefinitionHeaderLst`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `layoutDefHdr` | `CT_DiagramDefinitionHeader` | 0..unbounded |  |  |

## `dgm:<layoutDefHdrLst>` : `CT_DiagramDefinitionHeaderLst`

## `dgm:CT_RelIds`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@r:dm` | `dm` | required |  |  |
| attribute | `@r:lo` | `lo` | required |  |  |
| attribute | `@r:qs` | `qs` | required |  |  |
| attribute | `@r:cs` | `cs` | required |  |  |

## `dgm:<relIds>` : `CT_RelIds`

## `dgm:ST_ParameterVal`
Restriction: `union=ST_DiagramHorizontalAlignment ST_VerticalAlignment ST_ChildDirection ST_ChildAlignment ST_SecondaryChildAlignment ST_LinearDirection ST_SecondaryLinearDirection ST_StartingElement ST_BendPoint ST_ConnectorRouting ST_ArrowheadStyle ST_ConnectorDimension ST_RotationPath ST_CenterShapeMapping ST_NodeHorizontalAlignment ST_NodeVerticalAlignment ST_FallbackDimension ST_TextDirection ST_PyramidAccentPosition ST_PyramidAccentTextMargin ST_TextBlockDirection ST_TextAnchorHorizontal ST_TextAnchorVertical ST_DiagramTextAlignment ST_AutoTextRotation ST_GrowDirection ST_FlowDirection ST_ContinueDirection ST_Breakpoint ST_Offset ST_HierarchyAlignment int double boolean string ST_ConnectorPoint`

## `dgm:ST_ModelId`
Restriction: `union=int ST_Guid`

## `dgm:ST_PrSetCustVal`
Restriction: `union=ST_Percentage int`

## `dgm:CT_ElemPropSet`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `presLayoutVars` | `CT_LayoutVariablePropertySet` | 0..1 |  |  |
| element | `style` | `CT_ShapeStyle` | 0..1 |  |  |
| attribute | `@presAssocID` | `ST_ModelId` | optional |  |  |
| attribute | `@presName` | `string` | optional |  |  |
| attribute | `@presStyleLbl` | `string` | optional |  |  |
| attribute | `@presStyleIdx` | `int` | optional |  |  |
| attribute | `@presStyleCnt` | `int` | optional |  |  |
| attribute | `@loTypeId` | `string` | optional |  |  |
| attribute | `@loCatId` | `string` | optional |  |  |
| attribute | `@qsTypeId` | `string` | optional |  |  |
| attribute | `@qsCatId` | `string` | optional |  |  |
| attribute | `@csTypeId` | `string` | optional |  |  |
| attribute | `@csCatId` | `string` | optional |  |  |
| attribute | `@coherent3DOff` | `boolean` | optional |  |  |
| attribute | `@phldrT` | `string` | optional |  |  |
| attribute | `@phldr` | `boolean` | optional |  |  |
| attribute | `@custAng` | `int` | optional |  |  |
| attribute | `@custFlipVert` | `boolean` | optional |  |  |
| attribute | `@custFlipHor` | `boolean` | optional |  |  |
| attribute | `@custSzX` | `int` | optional |  |  |
| attribute | `@custSzY` | `int` | optional |  |  |
| attribute | `@custScaleX` | `ST_PrSetCustVal` | optional |  |  |
| attribute | `@custScaleY` | `ST_PrSetCustVal` | optional |  |  |
| attribute | `@custT` | `boolean` | optional |  |  |
| attribute | `@custLinFactX` | `ST_PrSetCustVal` | optional |  |  |
| attribute | `@custLinFactY` | `ST_PrSetCustVal` | optional |  |  |
| attribute | `@custLinFactNeighborX` | `ST_PrSetCustVal` | optional |  |  |
| attribute | `@custLinFactNeighborY` | `ST_PrSetCustVal` | optional |  |  |
| attribute | `@custRadScaleRad` | `ST_PrSetCustVal` | optional |  |  |
| attribute | `@custRadScaleInc` | `ST_PrSetCustVal` | optional |  |  |

## `dgm:ST_Direction`
Restriction: `base=token`
- `norm`
- `rev`

## `dgm:ST_HierBranchStyle`
Restriction: `base=token`
- `l`
- `r`
- `hang`
- `std`
- `init`

## `dgm:ST_AnimOneStr`
Restriction: `base=token`
- `none`
- `one`
- `branch`

## `dgm:ST_AnimLvlStr`
Restriction: `base=token`
- `none`
- `lvl`
- `ctr`

## `dgm:CT_OrgChart`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `boolean` | optional | false |  |

## `dgm:ST_NodeCount`
Restriction: `base=int minInclusive=-1`

## `dgm:CT_ChildMax`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_NodeCount` | optional | -1 |  |

## `dgm:CT_ChildPref`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_NodeCount` | optional | -1 |  |

## `dgm:CT_BulletEnabled`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `boolean` | optional | false |  |

## `dgm:CT_Direction`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_Direction` | optional | norm |  |

## `dgm:CT_HierBranchStyle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_HierBranchStyle` | optional | std |  |

## `dgm:CT_AnimOne`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_AnimOneStr` | optional | one |  |

## `dgm:CT_AnimLvl`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_AnimLvlStr` | optional | none |  |

## `dgm:ST_ResizeHandlesStr`
Restriction: `base=token`
- `exact`
- `rel`

## `dgm:CT_ResizeHandles`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_ResizeHandlesStr` | optional | rel |  |

## `dgm:CT_LayoutVariablePropertySet`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `orgChart` | `CT_OrgChart` | 0..1 |  |  |
| element | `chMax` | `CT_ChildMax` | 0..1 |  |  |
| element | `chPref` | `CT_ChildPref` | 0..1 |  |  |
| element | `bulletEnabled` | `CT_BulletEnabled` | 0..1 |  |  |
| element | `dir` | `CT_Direction` | 0..1 |  |  |
| element | `hierBranch` | `CT_HierBranchStyle` | 0..1 |  |  |
| element | `animOne` | `CT_AnimOne` | 0..1 |  |  |
| element | `animLvl` | `CT_AnimLvl` | 0..1 |  |  |
| element | `resizeHandles` | `CT_ResizeHandles` | 0..1 |  |  |

## `dgm:CT_SDName`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@lang` | `string` | optional |  |  |
| attribute | `@val` | `string` | required |  |  |

## `dgm:CT_SDDescription`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@lang` | `string` | optional |  |  |
| attribute | `@val` | `string` | required |  |  |

## `dgm:CT_SDCategory`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@type` | `anyURI` | required |  |  |
| attribute | `@pri` | `unsignedInt` | required |  |  |

## `dgm:CT_SDCategories`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cat` | `CT_SDCategory` | 0..unbounded |  |  |

## `dgm:CT_TextProps`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_Text3D` | `EG_Text3D` | 0..1 |  |  |

## `dgm:CT_StyleLabel`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `scene3d` | `CT_Scene3D` | 0..1 |  |  |
| element | `sp3d` | `CT_Shape3D` | 0..1 |  |  |
| element | `txPr` | `CT_TextProps` | 0..1 |  |  |
| element | `style` | `CT_ShapeStyle` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@name` | `string` | required |  |  |

## `dgm:CT_StyleDefinition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `title` | `CT_SDName` | 0..unbounded |  |  |
| element | `desc` | `CT_SDDescription` | 0..unbounded |  |  |
| element | `catLst` | `CT_SDCategories` | 0..1 |  |  |
| element | `scene3d` | `CT_Scene3D` | 0..1 |  |  |
| element | `styleLbl` | `CT_StyleLabel` | 1..unbounded |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@uniqueId` | `string` | optional |  |  |
| attribute | `@minVer` | `string` | optional |  |  |

## `dgm:<styleDef>` : `CT_StyleDefinition`

## `dgm:CT_StyleDefinitionHeader`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `title` | `CT_SDName` | 1..unbounded |  |  |
| element | `desc` | `CT_SDDescription` | 1..unbounded |  |  |
| element | `catLst` | `CT_SDCategories` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@uniqueId` | `string` | required |  |  |
| attribute | `@minVer` | `string` | optional |  |  |
| attribute | `@resId` | `int` | optional | 0 |  |

## `dgm:<styleDefHdr>` : `CT_StyleDefinitionHeader`

## `dgm:CT_StyleDefinitionHeaderLst`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `styleDefHdr` | `CT_StyleDefinitionHeader` | 0..unbounded |  |  |

## `dgm:<styleDefHdrLst>` : `CT_StyleDefinitionHeaderLst`

## `dgm:ST_AlgorithmType`
Restriction: `base=token`
- `composite`
- `conn`
- `cycle`
- `hierChild`
- `hierRoot`
- `pyra`
- `lin`
- `sp`
- `tx`
- `snake`

## `dgm:ST_AxisType`
Restriction: `base=token`
- `self`
- `ch`
- `des`
- `desOrSelf`
- `par`
- `ancst`
- `ancstOrSelf`
- `followSib`
- `precedSib`
- `follow`
- `preced`
- `root`
- `none`

## `dgm:ST_AxisTypes`
Restriction: `list=ST_AxisType`

## `dgm:ST_BoolOperator`
Restriction: `base=token`
- `none`
- `equ`
- `gte`
- `lte`

## `dgm:ST_ChildOrderType`
Restriction: `base=token`
- `b`
- `t`

## `dgm:ST_ConstraintType`
Restriction: `base=token`
- `none`
- `alignOff`
- `begMarg`
- `bendDist`
- `begPad`
- `b`
- `bMarg`
- `bOff`
- `ctrX`
- `ctrXOff`
- `ctrY`
- `ctrYOff`
- `connDist`
- `diam`
- `endMarg`
- `endPad`
- `h`
- `hArH`
- `hOff`
- `l`
- `lMarg`
- `lOff`
- `r`
- `rMarg`
- `rOff`
- `primFontSz`
- `pyraAcctRatio`
- `secFontSz`
- `sibSp`
- `secSibSp`
- `sp`
- `stemThick`
- `t`
- `tMarg`
- `tOff`
- `userA`
- `userB`
- `userC`
- `userD`
- `userE`
- `userF`
- `userG`
- `userH`
- `userI`
- `userJ`
- `userK`
- `userL`
- `userM`
- `userN`
- `userO`
- `userP`
- `userQ`
- `userR`
- `userS`
- `userT`
- `userU`
- `userV`
- `userW`
- `userX`
- `userY`
- `userZ`
- `w`
- `wArH`
- `wOff`

## `dgm:ST_ConstraintRelationship`
Restriction: `base=token`
- `self`
- `ch`
- `des`

## `dgm:ST_ElementType`
Restriction: `base=token`
- `all`
- `doc`
- `node`
- `norm`
- `nonNorm`
- `asst`
- `nonAsst`
- `parTrans`
- `pres`
- `sibTrans`

## `dgm:ST_ElementTypes`
Restriction: `list=ST_ElementType`

## `dgm:ST_ParameterId`
Restriction: `base=token`
- `horzAlign`
- `vertAlign`
- `chDir`
- `chAlign`
- `secChAlign`
- `linDir`
- `secLinDir`
- `stElem`
- `bendPt`
- `connRout`
- `begSty`
- `endSty`
- `dim`
- `rotPath`
- `ctrShpMap`
- `nodeHorzAlign`
- `nodeVertAlign`
- `fallback`
- `txDir`
- `pyraAcctPos`
- `pyraAcctTxMar`
- `txBlDir`
- `txAnchorHorz`
- `txAnchorVert`
- `txAnchorHorzCh`
- `txAnchorVertCh`
- `parTxLTRAlign`
- `parTxRTLAlign`
- `shpTxLTRAlignCh`
- `shpTxRTLAlignCh`
- `autoTxRot`
- `grDir`
- `flowDir`
- `contDir`
- `bkpt`
- `off`
- `hierAlign`
- `bkPtFixedVal`
- `stBulletLvl`
- `stAng`
- `spanAng`
- `ar`
- `lnSpPar`
- `lnSpAfParP`
- `lnSpCh`
- `lnSpAfChP`
- `rtShortDist`
- `alignTx`
- `pyraLvlNode`
- `pyraAcctBkgdNode`
- `pyraAcctTxNode`
- `srcNode`
- `dstNode`
- `begPts`
- `endPts`

## `dgm:ST_Ints`
Restriction: `list=int`

## `dgm:ST_UnsignedInts`
Restriction: `list=unsignedInt`

## `dgm:ST_Booleans`
Restriction: `list=boolean`

## `dgm:ST_FunctionType`
Restriction: `base=token`
- `cnt`
- `pos`
- `revPos`
- `posEven`
- `posOdd`
- `var`
- `depth`
- `maxDepth`

## `dgm:ST_FunctionOperator`
Restriction: `base=token`
- `equ`
- `neq`
- `gt`
- `lt`
- `gte`
- `lte`

## `dgm:ST_DiagramHorizontalAlignment`
Restriction: `base=token`
- `l`
- `ctr`
- `r`
- `none`

## `dgm:ST_VerticalAlignment`
Restriction: `base=token`
- `t`
- `mid`
- `b`
- `none`

## `dgm:ST_ChildDirection`
Restriction: `base=token`
- `horz`
- `vert`

## `dgm:ST_ChildAlignment`
Restriction: `base=token`
- `t`
- `b`
- `l`
- `r`

## `dgm:ST_SecondaryChildAlignment`
Restriction: `base=token`
- `none`
- `t`
- `b`
- `l`
- `r`

## `dgm:ST_LinearDirection`
Restriction: `base=token`
- `fromL`
- `fromR`
- `fromT`
- `fromB`

## `dgm:ST_SecondaryLinearDirection`
Restriction: `base=token`
- `none`
- `fromL`
- `fromR`
- `fromT`
- `fromB`

## `dgm:ST_StartingElement`
Restriction: `base=token`
- `node`
- `trans`

## `dgm:ST_RotationPath`
Restriction: `base=token`
- `none`
- `alongPath`

## `dgm:ST_CenterShapeMapping`
Restriction: `base=token`
- `none`
- `fNode`

## `dgm:ST_BendPoint`
Restriction: `base=token`
- `beg`
- `def`
- `end`

## `dgm:ST_ConnectorRouting`
Restriction: `base=token`
- `stra`
- `bend`
- `curve`
- `longCurve`

## `dgm:ST_ArrowheadStyle`
Restriction: `base=token`
- `auto`
- `arr`
- `noArr`

## `dgm:ST_ConnectorDimension`
Restriction: `base=token`
- `1D`
- `2D`
- `cust`

## `dgm:ST_ConnectorPoint`
Restriction: `base=token`
- `auto`
- `bCtr`
- `ctr`
- `midL`
- `midR`
- `tCtr`
- `bL`
- `bR`
- `tL`
- `tR`
- `radial`

## `dgm:ST_NodeHorizontalAlignment`
Restriction: `base=token`
- `l`
- `ctr`
- `r`

## `dgm:ST_NodeVerticalAlignment`
Restriction: `base=token`
- `t`
- `mid`
- `b`

## `dgm:ST_FallbackDimension`
Restriction: `base=token`
- `1D`
- `2D`

## `dgm:ST_TextDirection`
Restriction: `base=token`
- `fromT`
- `fromB`

## `dgm:ST_PyramidAccentPosition`
Restriction: `base=token`
- `bef`
- `aft`

## `dgm:ST_PyramidAccentTextMargin`
Restriction: `base=token`
- `step`
- `stack`

## `dgm:ST_TextBlockDirection`
Restriction: `base=token`
- `horz`
- `vert`

## `dgm:ST_TextAnchorHorizontal`
Restriction: `base=token`
- `none`
- `ctr`

## `dgm:ST_TextAnchorVertical`
Restriction: `base=token`
- `t`
- `mid`
- `b`

## `dgm:ST_DiagramTextAlignment`
Restriction: `base=token`
- `l`
- `ctr`
- `r`

## `dgm:ST_AutoTextRotation`
Restriction: `base=token`
- `none`
- `upr`
- `grav`

## `dgm:ST_GrowDirection`
Restriction: `base=token`
- `tL`
- `tR`
- `bL`
- `bR`

## `dgm:ST_FlowDirection`
Restriction: `base=token`
- `row`
- `col`

## `dgm:ST_ContinueDirection`
Restriction: `base=token`
- `revDir`
- `sameDir`

## `dgm:ST_Breakpoint`
Restriction: `base=token`
- `endCnv`
- `bal`
- `fixed`

## `dgm:ST_Offset`
Restriction: `base=token`
- `ctr`
- `off`

## `dgm:ST_HierarchyAlignment`
Restriction: `base=token`
- `tL`
- `tR`
- `tCtrCh`
- `tCtrDes`
- `bL`
- `bR`
- `bCtrCh`
- `bCtrDes`
- `lT`
- `lB`
- `lCtrCh`
- `lCtrDes`
- `rT`
- `rB`
- `rCtrCh`
- `rCtrDes`

## `dgm:ST_FunctionValue`
Restriction: `union=int boolean ST_Direction ST_HierBranchStyle ST_AnimOneStr ST_AnimLvlStr ST_ResizeHandlesStr`

## `dgm:ST_VariableType`
Restriction: `base=token`
- `none`
- `orgChart`
- `chMax`
- `chPref`
- `bulEnabled`
- `dir`
- `hierBranch`
- `animOne`
- `animLvl`
- `resizeHandles`

## `dgm:ST_FunctionArgument`
Restriction: `union=ST_VariableType`

## `dgm:ST_OutputShapeType`
Restriction: `base=token`
- `none`
- `conn`

## `x:drawing/2008/diagram:CT_ShapeNonVisual`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cNvPr` | `CT_NonVisualDrawingProps` |  |  |  |
| element | `cNvSpPr` | `CT_NonVisualDrawingShapeProps` |  |  |  |

## `x:drawing/2008/diagram:CT_Shape`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `nvSpPr` | `CT_ShapeNonVisual` |  |  |  |
| element | `spPr` | `CT_ShapeProperties` |  |  |  |
| element | `style` | `CT_ShapeStyle` | 0..1 |  |  |
| element | `txBody` | `CT_TextBody` | 0..1 |  |  |
| element | `txXfrm` | `CT_Transform2D` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@modelId` | `ST_ModelId` | required |  |  |

## `x:drawing/2008/diagram:CT_GroupShapeNonVisual`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cNvPr` | `CT_NonVisualDrawingProps` |  |  |  |
| element | `cNvGrpSpPr` | `CT_NonVisualGroupDrawingShapeProps` |  |  |  |

## `x:drawing/2008/diagram:CT_GroupShape`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `nvGrpSpPr` | `CT_GroupShapeNonVisual` |  |  |  |
| element | `grpSpPr` | `CT_GroupShapeProperties` |  |  |  |
| element | `sp` | `CT_Shape` | choice |  |  |
| element | `grpSp` | `CT_GroupShape` | choice |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `x:drawing/2008/diagram:CT_Drawing`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `spTree` | `CT_GroupShape` |  |  |  |

## `x:drawing/2008/diagram:<drawing>` : `CT_Drawing`

## `x:drawing/2008/diagram:CT_DataModelExtBlock`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@relId` | `string` | optional |  |  |
| attribute | `@minVer` | `anyURI` | optional |  |  |

## `x:drawing/2008/diagram:<dataModelExt>` : `CT_DataModelExtBlock`

## `x:drawing/2010/diagram:<cNvPr>` : `CT_NonVisualDrawingProps`

## `x:drawing/2010/diagram:CT_Boolean`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `boolean` | optional | false |  |

## `x:drawing/2010/diagram:<recolorImg>` : `CT_Boolean`

## `x:drawing/2016/11/diagram:ST_STorageType`
Restriction: `base=token`
- `sibTrans`
- `parTrans`

## `x:drawing/2016/11/diagram:CT_DiagramAutoBullet`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_TextBullet` | `EG_TextBullet` |  |  |  |
| attribute | `@prefix` | `string` | optional |  |  |
| attribute | `@leadZeros` | `boolean` | optional |  |  |

## `x:drawing/2016/11/diagram:CT_NumberDiagramInfo`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `buPr` | `CT_DiagramAutoBullet` |  |  |  |
| attribute | `@lvl` | `unsignedInt` | required |  |  |
| attribute | `@ptType` | `ST_STorageType` | required |  |  |

## `x:drawing/2016/11/diagram:CT_NumberDiagramInfoList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `autoBuNodeInfo` | `CT_NumberDiagramInfo` | 0..unbounded |  |  |

## `x:drawing/2016/11/diagram:<autoBuNodeInfoLst>` : `CT_NumberDiagramInfoList`

## `x:drawing/2016/12/diagram:<spPr>` : `CT_ShapeProperties`

## `x:drawing/2016/12/diagram:<lstStyle>` : `CT_TextListStyle`

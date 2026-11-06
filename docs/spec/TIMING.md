# TIMING: Animations and timing tree

Generated. Tier 1I; render 8; edit 8; level Edit. 440 rows.


## `p:ST_TLTimeIndefinite`
Restriction: `base=token`
- `indefinite`

## `p:ST_TLTime`
Restriction: `union=unsignedInt ST_TLTimeIndefinite`

## `p:ST_TLTimeNodeID`
Restriction: `base=unsignedInt`

## `p:CT_TLIterateIntervalTime`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_TLTime` | required |  |  |

## `p:CT_TLIterateIntervalPercentage`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_PositivePercentage` | required |  |  |

## `p:ST_IterateType`
Restriction: `base=token`
- `el`
- `wd`
- `lt`

## `p:CT_TLIterateData`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tmAbs` | `CT_TLIterateIntervalTime` | choice |  |  |
| element | `tmPct` | `CT_TLIterateIntervalPercentage` | choice |  |  |
| attribute | `@type` | `ST_IterateType` | optional | el |  |
| attribute | `@backwards` | `boolean` | optional | false |  |

## `p:CT_TLSubShapeId`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@spid` | `ST_ShapeID` | required |  |  |

## `p:CT_TLTextTargetElement`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `charRg` | `CT_IndexRange` | choice |  |  |
| element | `pRg` | `CT_IndexRange` | choice |  |  |

## `p:ST_TLChartSubelementType`
Restriction: `base=token`
- `gridLegend`
- `series`
- `category`
- `ptInSeries`
- `ptInCategory`

## `p:CT_TLOleChartTargetElement`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@type` | `ST_TLChartSubelementType` | required |  |  |
| attribute | `@lvl` | `unsignedInt` | optional | 0 |  |

## `p:CT_TLShapeTargetElement`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `bg` | `CT_Empty` | choice |  |  |
| element | `subSp` | `CT_TLSubShapeId` | choice |  |  |
| element | `oleChartEl` | `CT_TLOleChartTargetElement` | choice |  |  |
| element | `txEl` | `CT_TLTextTargetElement` | choice |  |  |
| element | `graphicEl` | `CT_AnimationElementChoice` | choice |  |  |
| attribute | `@spid` | `ST_DrawingElementId` | required |  |  |

## `p:CT_TLTimeTargetElement`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sldTgt` | `CT_Empty` | choice |  |  |
| element | `sndTgt` | `CT_EmbeddedWAVAudioFile` | choice |  |  |
| element | `spTgt` | `CT_TLShapeTargetElement` | choice |  |  |
| element | `inkTgt` | `CT_TLSubShapeId` | choice |  |  |

## `p:CT_TLTriggerTimeNodeID`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_TLTimeNodeID` | required |  |  |

## `p:ST_TLTriggerRuntimeNode`
Restriction: `base=token`
- `first`
- `last`
- `all`

## `p:CT_TLTriggerRuntimeNode`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_TLTriggerRuntimeNode` | required |  |  |

## `p:ST_TLTriggerEvent`
Restriction: `base=token`
- `onBegin`
- `onEnd`
- `begin`
- `end`
- `onClick`
- `onDblClick`
- `onMouseOver`
- `onMouseOut`
- `onNext`
- `onPrev`
- `onStopAudio`

## `p:CT_TLTimeCondition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tgtEl` | `CT_TLTimeTargetElement` | choice |  |  |
| element | `tn` | `CT_TLTriggerTimeNodeID` | choice |  |  |
| element | `rtn` | `CT_TLTriggerRuntimeNode` | choice |  |  |
| attribute | `@evt` | `ST_TLTriggerEvent` | optional |  |  |
| attribute | `@delay` | `ST_TLTime` | optional |  |  |

## `p:CT_TLTimeConditionList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cond` | `CT_TLTimeCondition` | 1..unbounded |  |  |

## `p:CT_TimeNodeList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `par` | `CT_TLTimeNodeParallel` | choice |  |  |
| element | `seq` | `CT_TLTimeNodeSequence` | choice |  |  |
| element | `excl` | `CT_TLTimeNodeExclusive` | choice |  |  |
| element | `anim` | `CT_TLAnimateBehavior` | choice |  |  |
| element | `animClr` | `CT_TLAnimateColorBehavior` | choice |  |  |
| element | `animEffect` | `CT_TLAnimateEffectBehavior` | choice |  |  |
| element | `animMotion` | `CT_TLAnimateMotionBehavior` | choice |  |  |
| element | `animRot` | `CT_TLAnimateRotationBehavior` | choice |  |  |
| element | `animScale` | `CT_TLAnimateScaleBehavior` | choice |  |  |
| element | `cmd` | `CT_TLCommandBehavior` | choice |  |  |
| element | `set` | `CT_TLSetBehavior` | choice |  |  |
| element | `audio` | `CT_TLMediaNodeAudio` | choice |  |  |
| element | `video` | `CT_TLMediaNodeVideo` | choice |  |  |

## `p:ST_TLTimeNodePresetClassType`
Restriction: `base=token`
- `entr`
- `exit`
- `emph`
- `path`
- `verb`
- `mediacall`

## `p:ST_TLTimeNodeRestartType`
Restriction: `base=token`
- `always`
- `whenNotActive`
- `never`

## `p:ST_TLTimeNodeFillType`
Restriction: `base=token`
- `remove`
- `freeze`
- `hold`
- `transition`

## `p:ST_TLTimeNodeSyncType`
Restriction: `base=token`
- `canSlip`
- `locked`

## `p:ST_TLTimeNodeMasterRelation`
Restriction: `base=token`
- `sameClick`
- `lastClick`
- `nextClick`

## `p:ST_TLTimeNodeType`
Restriction: `base=token`
- `clickEffect`
- `withEffect`
- `afterEffect`
- `mainSeq`
- `interactiveSeq`
- `clickPar`
- `withGroup`
- `afterGroup`
- `tmRoot`

## `p:CT_TLCommonTimeNodeData`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `stCondLst` | `CT_TLTimeConditionList` | 0..1 |  |  |
| element | `endCondLst` | `CT_TLTimeConditionList` | 0..1 |  |  |
| element | `endSync` | `CT_TLTimeCondition` | 0..1 |  |  |
| element | `iterate` | `CT_TLIterateData` | 0..1 |  |  |
| element | `childTnLst` | `CT_TimeNodeList` | 0..1 |  |  |
| element | `subTnLst` | `CT_TimeNodeList` | 0..1 |  |  |
| attribute | `@id` | `ST_TLTimeNodeID` | optional |  |  |
| attribute | `@presetID` | `int` | optional |  |  |
| attribute | `@presetClass` | `ST_TLTimeNodePresetClassType` | optional |  |  |
| attribute | `@presetSubtype` | `int` | optional |  |  |
| attribute | `@dur` | `ST_TLTime` | optional |  |  |
| attribute | `@repeatCount` | `ST_TLTime` | optional | 1000 |  |
| attribute | `@repeatDur` | `ST_TLTime` | optional |  |  |
| attribute | `@spd` | `ST_Percentage` | optional | 100% |  |
| attribute | `@accel` | `ST_PositiveFixedPercentage` | optional | 0% |  |
| attribute | `@decel` | `ST_PositiveFixedPercentage` | optional | 0% |  |
| attribute | `@autoRev` | `boolean` | optional | false |  |
| attribute | `@restart` | `ST_TLTimeNodeRestartType` | optional |  |  |
| attribute | `@fill` | `ST_TLTimeNodeFillType` | optional |  |  |
| attribute | `@syncBehavior` | `ST_TLTimeNodeSyncType` | optional |  |  |
| attribute | `@tmFilter` | `string` | optional |  |  |
| attribute | `@evtFilter` | `string` | optional |  |  |
| attribute | `@display` | `boolean` | optional |  |  |
| attribute | `@masterRel` | `ST_TLTimeNodeMasterRelation` | optional |  |  |
| attribute | `@bldLvl` | `int` | optional |  |  |
| attribute | `@grpId` | `unsignedInt` | optional |  |  |
| attribute | `@afterEffect` | `boolean` | optional |  |  |
| attribute | `@nodeType` | `ST_TLTimeNodeType` | optional |  |  |
| attribute | `@nodePh` | `boolean` | optional |  |  |

## `p:CT_TLTimeNodeParallel`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cTn` | `CT_TLCommonTimeNodeData` |  |  |  |

## `p:ST_TLNextActionType`
Restriction: `base=token`
- `none`
- `seek`

## `p:ST_TLPreviousActionType`
Restriction: `base=token`
- `none`
- `skipTimed`

## `p:CT_TLTimeNodeSequence`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cTn` | `CT_TLCommonTimeNodeData` |  |  |  |
| element | `prevCondLst` | `CT_TLTimeConditionList` | 0..1 |  |  |
| element | `nextCondLst` | `CT_TLTimeConditionList` | 0..1 |  |  |
| attribute | `@concurrent` | `boolean` | optional |  |  |
| attribute | `@prevAc` | `ST_TLPreviousActionType` | optional |  |  |
| attribute | `@nextAc` | `ST_TLNextActionType` | optional |  |  |

## `p:CT_TLTimeNodeExclusive`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cTn` | `CT_TLCommonTimeNodeData` |  |  |  |

## `p:CT_TLBehaviorAttributeNameList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `attrName` | `string` | 1..unbounded |  |  |

## `p:ST_TLBehaviorAdditiveType`
Restriction: `base=token`
- `base`
- `sum`
- `repl`
- `mult`
- `none`

## `p:ST_TLBehaviorAccumulateType`
Restriction: `base=token`
- `none`
- `always`

## `p:ST_TLBehaviorTransformType`
Restriction: `base=token`
- `pt`
- `img`

## `p:ST_TLBehaviorOverrideType`
Restriction: `base=token`
- `normal`
- `childStyle`

## `p:CT_TLCommonBehaviorData`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cTn` | `CT_TLCommonTimeNodeData` |  |  |  |
| element | `tgtEl` | `CT_TLTimeTargetElement` |  |  |  |
| element | `attrNameLst` | `CT_TLBehaviorAttributeNameList` | 0..1 |  |  |
| attribute | `@additive` | `ST_TLBehaviorAdditiveType` | optional |  |  |
| attribute | `@accumulate` | `ST_TLBehaviorAccumulateType` | optional |  |  |
| attribute | `@xfrmType` | `ST_TLBehaviorTransformType` | optional |  |  |
| attribute | `@from` | `string` | optional |  |  |
| attribute | `@to` | `string` | optional |  |  |
| attribute | `@by` | `string` | optional |  |  |
| attribute | `@rctx` | `string` | optional |  |  |
| attribute | `@override` | `ST_TLBehaviorOverrideType` | optional |  |  |

## `p:CT_TLAnimVariantBooleanVal`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `boolean` | required |  |  |

## `p:CT_TLAnimVariantIntegerVal`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `int` | required |  |  |

## `p:CT_TLAnimVariantFloatVal`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `float` | required |  |  |

## `p:CT_TLAnimVariantStringVal`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `string` | required |  |  |

## `p:CT_TLAnimVariant`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `boolVal` | `CT_TLAnimVariantBooleanVal` | choice |  |  |
| element | `intVal` | `CT_TLAnimVariantIntegerVal` | choice |  |  |
| element | `fltVal` | `CT_TLAnimVariantFloatVal` | choice |  |  |
| element | `strVal` | `CT_TLAnimVariantStringVal` | choice |  |  |
| element | `clrVal` | `CT_Color` | choice |  |  |

## `p:ST_TLTimeAnimateValueTime`
Restriction: `union=ST_PositiveFixedPercentage ST_TLTimeIndefinite`

## `p:CT_TLTimeAnimateValue`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `val` | `CT_TLAnimVariant` | 0..1 |  |  |
| attribute | `@tm` | `ST_TLTimeAnimateValueTime` | optional | indefinite |  |
| attribute | `@fmla` | `string` | optional |  |  |

## `p:CT_TLTimeAnimateValueList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tav` | `CT_TLTimeAnimateValue` | 0..unbounded |  |  |

## `p:ST_TLAnimateBehaviorCalcMode`
Restriction: `base=token`
- `discrete`
- `lin`
- `fmla`

## `p:ST_TLAnimateBehaviorValueType`
Restriction: `base=token`
- `str`
- `num`
- `clr`

## `p:CT_TLAnimateBehavior`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cBhvr` | `CT_TLCommonBehaviorData` |  |  |  |
| element | `tavLst` | `CT_TLTimeAnimateValueList` | 0..1 |  |  |
| attribute | `@by` | `string` | optional |  |  |
| attribute | `@from` | `string` | optional |  |  |
| attribute | `@to` | `string` | optional |  |  |
| attribute | `@calcmode` | `ST_TLAnimateBehaviorCalcMode` | optional |  |  |
| attribute | `@valueType` | `ST_TLAnimateBehaviorValueType` | optional |  |  |

## `p:CT_TLByRgbColorTransform`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@r` | `ST_FixedPercentage` | required |  |  |
| attribute | `@g` | `ST_FixedPercentage` | required |  |  |
| attribute | `@b` | `ST_FixedPercentage` | required |  |  |

## `p:CT_TLByHslColorTransform`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@h` | `ST_Angle` | required |  |  |
| attribute | `@s` | `ST_FixedPercentage` | required |  |  |
| attribute | `@l` | `ST_FixedPercentage` | required |  |  |

## `p:CT_TLByAnimateColorTransform`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `rgb` | `CT_TLByRgbColorTransform` | choice |  |  |
| element | `hsl` | `CT_TLByHslColorTransform` | choice |  |  |

## `p:ST_TLAnimateColorSpace`
Restriction: `base=token`
- `rgb`
- `hsl`

## `p:ST_TLAnimateColorDirection`
Restriction: `base=token`
- `cw`
- `ccw`

## `p:CT_TLAnimateColorBehavior`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cBhvr` | `CT_TLCommonBehaviorData` |  |  |  |
| element | `by` | `CT_TLByAnimateColorTransform` | 0..1 |  |  |
| element | `from` | `CT_Color` | 0..1 |  |  |
| element | `to` | `CT_Color` | 0..1 |  |  |
| attribute | `@clrSpc` | `ST_TLAnimateColorSpace` | optional |  |  |
| attribute | `@dir` | `ST_TLAnimateColorDirection` | optional |  |  |

## `p:CT_TLAnimateEffectBehavior`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cBhvr` | `CT_TLCommonBehaviorData` |  |  |  |
| element | `progress` | `CT_TLAnimVariant` | 0..1 |  |  |
| attribute | `@transition` | `ST_TLAnimateEffectTransition` | optional | in |  |
| attribute | `@filter` | `string` | optional |  |  |
| attribute | `@prLst` | `string` | optional |  |  |

## `p:ST_TLAnimateMotionBehaviorOrigin`
Restriction: `base=token`
- `parent`
- `layout`

## `p:ST_TLAnimateMotionPathEditMode`
Restriction: `base=token`
- `relative`
- `fixed`

## `p:CT_TLPoint`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@x` | `ST_Percentage` | required |  |  |
| attribute | `@y` | `ST_Percentage` | required |  |  |

## `p:CT_TLAnimateMotionBehavior`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cBhvr` | `CT_TLCommonBehaviorData` |  |  |  |
| element | `by` | `CT_TLPoint` | 0..1 |  |  |
| element | `from` | `CT_TLPoint` | 0..1 |  |  |
| element | `to` | `CT_TLPoint` | 0..1 |  |  |
| element | `rCtr` | `CT_TLPoint` | 0..1 |  |  |
| attribute | `@origin` | `ST_TLAnimateMotionBehaviorOrigin` | optional |  |  |
| attribute | `@path` | `string` | optional |  |  |
| attribute | `@pathEditMode` | `ST_TLAnimateMotionPathEditMode` | optional |  |  |
| attribute | `@rAng` | `ST_Angle` | optional |  |  |
| attribute | `@ptsTypes` | `string` | optional |  |  |

## `p:CT_TLAnimateRotationBehavior`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cBhvr` | `CT_TLCommonBehaviorData` |  |  |  |
| attribute | `@by` | `ST_Angle` | optional |  |  |
| attribute | `@from` | `ST_Angle` | optional |  |  |
| attribute | `@to` | `ST_Angle` | optional |  |  |

## `p:CT_TLAnimateScaleBehavior`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cBhvr` | `CT_TLCommonBehaviorData` |  |  |  |
| element | `by` | `CT_TLPoint` | 0..1 |  |  |
| element | `from` | `CT_TLPoint` | 0..1 |  |  |
| element | `to` | `CT_TLPoint` | 0..1 |  |  |
| attribute | `@zoomContents` | `boolean` | optional |  |  |

## `p:ST_TLCommandType`
Restriction: `base=token`
- `evt`
- `call`
- `verb`

## `p:CT_TLCommandBehavior`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cBhvr` | `CT_TLCommonBehaviorData` |  |  |  |
| attribute | `@type` | `ST_TLCommandType` | optional |  |  |
| attribute | `@cmd` | `string` | optional |  |  |

## `p:CT_TLSetBehavior`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cBhvr` | `CT_TLCommonBehaviorData` |  |  |  |
| element | `to` | `CT_TLAnimVariant` | 0..1 |  |  |

## `p:CT_TLCommonMediaNodeData`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cTn` | `CT_TLCommonTimeNodeData` |  |  |  |
| element | `tgtEl` | `CT_TLTimeTargetElement` |  |  |  |
| attribute | `@vol` | `ST_PositiveFixedPercentage` | optional | 50% |  |
| attribute | `@mute` | `boolean` | optional | false |  |
| attribute | `@numSld` | `unsignedInt` | optional | 1 |  |
| attribute | `@showWhenStopped` | `boolean` | optional | true |  |

## `p:CT_TLMediaNodeAudio`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cMediaNode` | `CT_TLCommonMediaNodeData` |  |  |  |
| attribute | `@isNarration` | `boolean` | optional | false |  |

## `p:CT_TLMediaNodeVideo`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `cMediaNode` | `CT_TLCommonMediaNodeData` |  |  |  |
| attribute | `@fullScrn` | `boolean` | optional | false |  |

## `p:AG_TLBuild`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@spid` | `ST_DrawingElementId` | required |  |  |
| attribute | `@grpId` | `unsignedInt` | required |  |  |
| attribute | `@uiExpand` | `boolean` | optional | false |  |

## `p:CT_TLTemplate`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tnLst` | `CT_TimeNodeList` |  |  |  |
| attribute | `@lvl` | `unsignedInt` | optional | 0 |  |

## `p:CT_TLTemplateList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tmpl` | `CT_TLTemplate` | 0..9 |  |  |

## `p:ST_TLParaBuildType`
Restriction: `base=token`
- `allAtOnce`
- `p`
- `cust`
- `whole`

## `p:CT_TLBuildParagraph`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tmplLst` | `CT_TLTemplateList` | 0..1 |  |  |
| attrgroup | `AG_TLBuild` | `AG_TLBuild` |  |  |  |
| attribute | `@build` | `ST_TLParaBuildType` | optional | whole |  |
| attribute | `@bldLvl` | `unsignedInt` | optional | 1 |  |
| attribute | `@animBg` | `boolean` | optional | false |  |
| attribute | `@autoUpdateAnimBg` | `boolean` | optional | true |  |
| attribute | `@rev` | `boolean` | optional | false |  |
| attribute | `@advAuto` | `ST_TLTime` | optional | indefinite |  |

## `p:ST_TLDiagramBuildType`
Restriction: `base=token`
- `whole`
- `depthByNode`
- `depthByBranch`
- `breadthByNode`
- `breadthByLvl`
- `cw`
- `cwIn`
- `cwOut`
- `ccw`
- `ccwIn`
- `ccwOut`
- `inByRing`
- `outByRing`
- `up`
- `down`
- `allAtOnce`
- `cust`

## `p:CT_TLBuildDiagram`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_TLBuild` | `AG_TLBuild` |  |  |  |
| attribute | `@bld` | `ST_TLDiagramBuildType` | optional | whole |  |

## `p:ST_TLOleChartBuildType`
Restriction: `base=token`
- `allAtOnce`
- `series`
- `category`
- `seriesEl`
- `categoryEl`

## `p:CT_TLOleBuildChart`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_TLBuild` | `AG_TLBuild` |  |  |  |
| attribute | `@bld` | `ST_TLOleChartBuildType` | optional | allAtOnce |  |
| attribute | `@animBg` | `boolean` | optional | true |  |

## `p:CT_TLGraphicalObjectBuild`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `bldAsOne` | `CT_Empty` | choice |  |  |
| element | `bldSub` | `CT_AnimationGraphicalObjectBuildProperties` | choice |  |  |
| attrgroup | `AG_TLBuild` | `AG_TLBuild` |  |  |  |

## `p:CT_BuildList`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `bldP` | `CT_TLBuildParagraph` | choice |  |  |
| element | `bldDgm` | `CT_TLBuildDiagram` | choice |  |  |
| element | `bldOleChart` | `CT_TLOleBuildChart` | choice |  |  |
| element | `bldGraphic` | `CT_TLGraphicalObjectBuild` | choice |  |  |

## `p:CT_SlideTiming`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `tnLst` | `CT_TimeNodeList` | 0..1 |  |  |
| element | `bldLst` | `CT_BuildList` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionListModify` | 0..1 |  |  |

## `a:ST_ChartBuildStep`
Restriction: `base=token`
- `category`
- `ptInCategory`
- `series`
- `ptInSeries`
- `allPts`
- `gridLegend`

## `a:ST_DgmBuildStep`
Restriction: `base=token`
- `sp`
- `bg`

## `a:CT_AnimationDgmElement`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@id` | `ST_Guid` | optional | {00000000-0000-0000-0000-000000000000} |  |
| attribute | `@bldStep` | `ST_DgmBuildStep` | optional | sp |  |

## `a:CT_AnimationChartElement`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@seriesIdx` | `int` | optional | -1 |  |
| attribute | `@categoryIdx` | `int` | optional | -1 |  |
| attribute | `@bldStep` | `ST_ChartBuildStep` | required |  |  |

## `a:CT_AnimationElementChoice`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `dgm` | `CT_AnimationDgmElement` | choice |  |  |
| element | `chart` | `CT_AnimationChartElement` | choice |  |  |

## `a:ST_AnimationBuildType`
Restriction: `base=token`
- `allAtOnce`

## `a:ST_AnimationDgmOnlyBuildType`
Restriction: `base=token`
- `one`
- `lvlOne`
- `lvlAtOnce`

## `a:ST_AnimationDgmBuildType`
Restriction: `union=ST_AnimationBuildType ST_AnimationDgmOnlyBuildType`

## `a:CT_AnimationDgmBuildProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@bld` | `ST_AnimationDgmBuildType` | optional | allAtOnce |  |
| attribute | `@rev` | `boolean` | optional | false |  |

## `a:ST_AnimationChartOnlyBuildType`
Restriction: `base=token`
- `series`
- `category`
- `seriesEl`
- `categoryEl`

## `a:ST_AnimationChartBuildType`
Restriction: `union=ST_AnimationBuildType ST_AnimationChartOnlyBuildType`

## `a:CT_AnimationChartBuildProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@bld` | `ST_AnimationChartBuildType` | optional | allAtOnce |  |
| attribute | `@animBg` | `boolean` | optional | true |  |

## `a:CT_AnimationGraphicalObjectBuildProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `bldDgm` | `CT_AnimationDgmBuildProperties` | choice |  |  |
| element | `bldChart` | `CT_AnimationChartBuildProperties` | choice |  |  |

## `x:drawing/2018/animation:ST_AnimationName`
Restriction: `base=string`

## `x:drawing/2018/animation:ST_Indefinite`
Restriction: `base=token`
- `indefinite`

## `x:drawing/2018/animation:ST_PlayCount`
Restriction: `union=unsignedInt ST_Indefinite`

## `x:drawing/2018/animation:ST_UniversalTimeOffset`
Restriction: `base=string`

## `x:drawing/2018/animation:CT_AnimationProperties`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@name` | `ST_AnimationName` | optional |  |  |
| attribute | `@length` | `ST_UniversalTimeOffset` | required |  |  |
| attribute | `@count` | `ST_PlayCount` | optional | 1 |  |
| attribute | `@auto` | `boolean` | optional | true |  |
| attribute | `@offset` | `ST_UniversalTimeOffset` | optional | 0 |  |
| attribute | `@st` | `ST_UniversalTimeOffset` | optional | 0 |  |
| attribute | `@end` | `ST_UniversalTimeOffset` | optional | 0 |  |

## `x:drawing/2018/animation:<animPr>` : `CT_AnimationProperties`

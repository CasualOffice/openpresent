# TRANS: Slide transitions

Generated. Tier 1I; render 8; edit 8; level Edit. 82 rows.


## `p:ST_TransitionSideDirectionType`
Restriction: `base=token`
- `l`
- `u`
- `r`
- `d`

## `p:ST_TransitionCornerDirectionType`
Restriction: `base=token`
- `lu`
- `ru`
- `ld`
- `rd`

## `p:ST_TransitionInOutDirectionType`
Restriction: `base=token`
- `out`
- `in`

## `p:CT_SideDirectionTransition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@dir` | `ST_TransitionSideDirectionType` | optional | l |  |

## `p:CT_CornerDirectionTransition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@dir` | `ST_TransitionCornerDirectionType` | optional | lu |  |

## `p:ST_TransitionEightDirectionType`
Restriction: `union=ST_TransitionSideDirectionType ST_TransitionCornerDirectionType`

## `p:CT_EightDirectionTransition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@dir` | `ST_TransitionEightDirectionType` | optional | l |  |

## `p:CT_OrientationTransition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@dir` | `ST_Direction` | optional | horz |  |

## `p:CT_InOutTransition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@dir` | `ST_TransitionInOutDirectionType` | optional | out |  |

## `p:CT_OptionalBlackTransition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@thruBlk` | `boolean` | optional | false |  |

## `p:CT_SplitTransition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@orient` | `ST_Direction` | optional | horz |  |
| attribute | `@dir` | `ST_TransitionInOutDirectionType` | optional | out |  |

## `p:CT_WheelTransition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@spokes` | `unsignedInt` | optional | 4 |  |

## `p:CT_TransitionStartSoundAction`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `snd` | `CT_EmbeddedWAVAudioFile` |  |  |  |
| attribute | `@loop` | `boolean` | optional | false |  |

## `p:CT_TransitionSoundAction`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `stSnd` | `CT_TransitionStartSoundAction` | choice |  |  |
| element | `endSnd` | `CT_Empty` | choice |  |  |

## `p:ST_TransitionSpeed`
Restriction: `base=token`
- `slow`
- `med`
- `fast`

## `p:CT_SlideTransition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `blinds` | `CT_OrientationTransition` | choice |  |  |
| element | `checker` | `CT_OrientationTransition` | choice |  |  |
| element | `circle` | `CT_Empty` | choice |  |  |
| element | `dissolve` | `CT_Empty` | choice |  |  |
| element | `comb` | `CT_OrientationTransition` | choice |  |  |
| element | `cover` | `CT_EightDirectionTransition` | choice |  |  |
| element | `cut` | `CT_OptionalBlackTransition` | choice |  |  |
| element | `diamond` | `CT_Empty` | choice |  |  |
| element | `fade` | `CT_OptionalBlackTransition` | choice |  |  |
| element | `newsflash` | `CT_Empty` | choice |  |  |
| element | `plus` | `CT_Empty` | choice |  |  |
| element | `pull` | `CT_EightDirectionTransition` | choice |  |  |
| element | `push` | `CT_SideDirectionTransition` | choice |  |  |
| element | `random` | `CT_Empty` | choice |  |  |
| element | `randomBar` | `CT_OrientationTransition` | choice |  |  |
| element | `split` | `CT_SplitTransition` | choice |  |  |
| element | `strips` | `CT_CornerDirectionTransition` | choice |  |  |
| element | `wedge` | `CT_Empty` | choice |  |  |
| element | `wheel` | `CT_WheelTransition` | choice |  |  |
| element | `wipe` | `CT_SideDirectionTransition` | choice |  |  |
| element | `zoom` | `CT_InOutTransition` | choice |  |  |
| element | `sndAc` | `CT_TransitionSoundAction` | 0..1 |  |  |
| element | `extLst` | `CT_ExtensionListModify` | 0..1 |  |  |
| attribute | `@spd` | `ST_TransitionSpeed` | optional | fast |  |
| attribute | `@advClick` | `boolean` | optional | true |  |
| attribute | `@advTm` | `unsignedInt` | optional |  |  |

## `p:ST_TLAnimateEffectTransition`
Restriction: `base=token`
- `in`
- `out`
- `none`

## `p:ST_Direction`
Restriction: `base=token`
- `horz`
- `vert`

## `x:powerpoint/2015/09/main:ST_TransitionMorphOption`
Restriction: `base=token`
- `byObject`
- `byWord`
- `byChar`

## `x:powerpoint/2015/09/main:CT_MorphTransition`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@option` | `ST_TransitionMorphOption` | required |  |  |

## `x:powerpoint/2015/09/main:<morph>` : `CT_MorphTransition`

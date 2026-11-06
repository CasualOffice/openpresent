# MATH: Equations (OMML)

Generated. Tier 1K; render 2F; edit 6; level Edit. 326 rows.


## `m:ST_Integer255`
Restriction: `base=integer minInclusive=1 maxInclusive=255`

## `m:CT_Integer255`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_Integer255` | required |  |  |

## `m:ST_Integer2`
Restriction: `base=integer minInclusive=-2 maxInclusive=2`

## `m:CT_Integer2`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_Integer2` | required |  |  |

## `m:ST_SpacingRule`
Restriction: `base=integer minInclusive=0 maxInclusive=4`

## `m:CT_SpacingRule`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_SpacingRule` | required |  |  |

## `m:ST_UnSignedInteger`
Restriction: `base=unsignedInt`

## `m:CT_UnSignedInteger`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_UnSignedInteger` | required |  |  |

## `m:ST_Char`
Restriction: `base=string maxLength=1`

## `m:CT_Char`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_Char` | required |  |  |

## `m:CT_OnOff`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_OnOff` | optional |  |  |

## `m:CT_String`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_String` | optional |  |  |

## `m:CT_XAlign`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_XAlign` | required |  |  |

## `m:CT_YAlign`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_YAlign` | required |  |  |

## `m:ST_Shp`
Restriction: `base=string`
- `centered`
- `match`

## `m:CT_Shp`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_Shp` | required |  |  |

## `m:ST_FType`
Restriction: `base=string`
- `bar`
- `skw`
- `lin`
- `noBar`

## `m:CT_FType`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_FType` | required |  |  |

## `m:ST_LimLoc`
Restriction: `base=string`
- `undOvr`
- `subSup`

## `m:CT_LimLoc`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_LimLoc` | required |  |  |

## `m:ST_TopBot`
Restriction: `base=string`
- `top`
- `bot`

## `m:CT_TopBot`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_TopBot` | required |  |  |

## `m:ST_Script`
Restriction: `base=string`
- `roman`
- `script`
- `fraktur`
- `double-struck`
- `sans-serif`
- `monospace`

## `m:CT_Script`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_Script` | optional |  |  |

## `m:ST_Style`
Restriction: `base=string`
- `p`
- `b`
- `i`
- `bi`

## `m:CT_Style`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_Style` | optional |  |  |

## `m:CT_ManualBreak`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@alnAt` | `ST_Integer255` | optional |  |  |

## `m:EG_ScriptStyle`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `scr` | `CT_Script` | 0..1 |  |  |
| element | `sty` | `CT_Style` | 0..1 |  |  |

## `m:CT_RPR`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `lit` | `CT_OnOff` | 0..1 |  |  |
| element | `nor` | `CT_OnOff` | 0..1 |  |  |
| group | `EG_ScriptStyle` | `EG_ScriptStyle` | choice |  |  |
| element | `brk` | `CT_ManualBreak` | 0..1 |  |  |
| element | `aln` | `CT_OnOff` | 0..1 |  |  |

## `m:CT_Text`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| base | `ST_String` | `ST_String` |  |  |  |
| attribute | `@xml:space` | `space` | optional |  |  |

## `m:CT_R`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `rPr` | `CT_RPR` | 0..1 |  |  |
| group | `EG_RPr` | `EG_RPr` | 0..1 |  |  |
| group | `EG_RunInnerContent` | `EG_RunInnerContent` | choice |  |  |
| element | `t` | `CT_Text` | 0..1 |  |  |

## `m:CT_CtrlPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_RPrMath` | `EG_RPrMath` | 0..1 |  |  |

## `m:CT_AccPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `chr` | `CT_Char` | 0..1 |  |  |
| element | `ctrlPr` | `CT_CtrlPr` | 0..1 |  |  |

## `m:CT_Acc`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `accPr` | `CT_AccPr` | 0..1 |  |  |
| element | `e` | `CT_OMathArg` |  |  |  |

## `m:CT_BarPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `pos` | `CT_TopBot` | 0..1 |  |  |
| element | `ctrlPr` | `CT_CtrlPr` | 0..1 |  |  |

## `m:CT_Bar`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `barPr` | `CT_BarPr` | 0..1 |  |  |
| element | `e` | `CT_OMathArg` |  |  |  |

## `m:CT_BoxPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `opEmu` | `CT_OnOff` | 0..1 |  |  |
| element | `noBreak` | `CT_OnOff` | 0..1 |  |  |
| element | `diff` | `CT_OnOff` | 0..1 |  |  |
| element | `brk` | `CT_ManualBreak` | 0..1 |  |  |
| element | `aln` | `CT_OnOff` | 0..1 |  |  |
| element | `ctrlPr` | `CT_CtrlPr` | 0..1 |  |  |

## `m:CT_Box`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `boxPr` | `CT_BoxPr` | 0..1 |  |  |
| element | `e` | `CT_OMathArg` |  |  |  |

## `m:CT_BorderBoxPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `hideTop` | `CT_OnOff` | 0..1 |  |  |
| element | `hideBot` | `CT_OnOff` | 0..1 |  |  |
| element | `hideLeft` | `CT_OnOff` | 0..1 |  |  |
| element | `hideRight` | `CT_OnOff` | 0..1 |  |  |
| element | `strikeH` | `CT_OnOff` | 0..1 |  |  |
| element | `strikeV` | `CT_OnOff` | 0..1 |  |  |
| element | `strikeBLTR` | `CT_OnOff` | 0..1 |  |  |
| element | `strikeTLBR` | `CT_OnOff` | 0..1 |  |  |
| element | `ctrlPr` | `CT_CtrlPr` | 0..1 |  |  |

## `m:CT_BorderBox`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `borderBoxPr` | `CT_BorderBoxPr` | 0..1 |  |  |
| element | `e` | `CT_OMathArg` |  |  |  |

## `m:CT_DPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `begChr` | `CT_Char` | 0..1 |  |  |
| element | `sepChr` | `CT_Char` | 0..1 |  |  |
| element | `endChr` | `CT_Char` | 0..1 |  |  |
| element | `grow` | `CT_OnOff` | 0..1 |  |  |
| element | `shp` | `CT_Shp` | 0..1 |  |  |
| element | `ctrlPr` | `CT_CtrlPr` | 0..1 |  |  |

## `m:CT_D`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `dPr` | `CT_DPr` | 0..1 |  |  |
| element | `e` | `CT_OMathArg` | 1..unbounded |  |  |

## `m:CT_EqArrPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `baseJc` | `CT_YAlign` | 0..1 |  |  |
| element | `maxDist` | `CT_OnOff` | 0..1 |  |  |
| element | `objDist` | `CT_OnOff` | 0..1 |  |  |
| element | `rSpRule` | `CT_SpacingRule` | 0..1 |  |  |
| element | `rSp` | `CT_UnSignedInteger` | 0..1 |  |  |
| element | `ctrlPr` | `CT_CtrlPr` | 0..1 |  |  |

## `m:CT_EqArr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `eqArrPr` | `CT_EqArrPr` | 0..1 |  |  |
| element | `e` | `CT_OMathArg` | 1..unbounded |  |  |

## `m:CT_FPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `type` | `CT_FType` | 0..1 |  |  |
| element | `ctrlPr` | `CT_CtrlPr` | 0..1 |  |  |

## `m:CT_F`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `fPr` | `CT_FPr` | 0..1 |  |  |
| element | `num` | `CT_OMathArg` |  |  |  |
| element | `den` | `CT_OMathArg` |  |  |  |

## `m:CT_FuncPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ctrlPr` | `CT_CtrlPr` | 0..1 |  |  |

## `m:CT_Func`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `funcPr` | `CT_FuncPr` | 0..1 |  |  |
| element | `fName` | `CT_OMathArg` |  |  |  |
| element | `e` | `CT_OMathArg` |  |  |  |

## `m:CT_GroupChrPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `chr` | `CT_Char` | 0..1 |  |  |
| element | `pos` | `CT_TopBot` | 0..1 |  |  |
| element | `vertJc` | `CT_TopBot` | 0..1 |  |  |
| element | `ctrlPr` | `CT_CtrlPr` | 0..1 |  |  |

## `m:CT_GroupChr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `groupChrPr` | `CT_GroupChrPr` | 0..1 |  |  |
| element | `e` | `CT_OMathArg` |  |  |  |

## `m:CT_LimLowPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ctrlPr` | `CT_CtrlPr` | 0..1 |  |  |

## `m:CT_LimLow`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `limLowPr` | `CT_LimLowPr` | 0..1 |  |  |
| element | `e` | `CT_OMathArg` |  |  |  |
| element | `lim` | `CT_OMathArg` |  |  |  |

## `m:CT_LimUppPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ctrlPr` | `CT_CtrlPr` | 0..1 |  |  |

## `m:CT_LimUpp`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `limUppPr` | `CT_LimUppPr` | 0..1 |  |  |
| element | `e` | `CT_OMathArg` |  |  |  |
| element | `lim` | `CT_OMathArg` |  |  |  |

## `m:CT_MCPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `count` | `CT_Integer255` | 0..1 |  |  |
| element | `mcJc` | `CT_XAlign` | 0..1 |  |  |

## `m:CT_MC`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `mcPr` | `CT_MCPr` | 0..1 |  |  |

## `m:CT_MCS`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `mc` | `CT_MC` | 1..unbounded |  |  |

## `m:CT_MPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `baseJc` | `CT_YAlign` | 0..1 |  |  |
| element | `plcHide` | `CT_OnOff` | 0..1 |  |  |
| element | `rSpRule` | `CT_SpacingRule` | 0..1 |  |  |
| element | `cGpRule` | `CT_SpacingRule` | 0..1 |  |  |
| element | `rSp` | `CT_UnSignedInteger` | 0..1 |  |  |
| element | `cSp` | `CT_UnSignedInteger` | 0..1 |  |  |
| element | `cGp` | `CT_UnSignedInteger` | 0..1 |  |  |
| element | `mcs` | `CT_MCS` | 0..1 |  |  |
| element | `ctrlPr` | `CT_CtrlPr` | 0..1 |  |  |

## `m:CT_MR`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `e` | `CT_OMathArg` | 1..unbounded |  |  |

## `m:CT_M`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `mPr` | `CT_MPr` | 0..1 |  |  |
| element | `mr` | `CT_MR` | 1..unbounded |  |  |

## `m:CT_NaryPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `chr` | `CT_Char` | 0..1 |  |  |
| element | `limLoc` | `CT_LimLoc` | 0..1 |  |  |
| element | `grow` | `CT_OnOff` | 0..1 |  |  |
| element | `subHide` | `CT_OnOff` | 0..1 |  |  |
| element | `supHide` | `CT_OnOff` | 0..1 |  |  |
| element | `ctrlPr` | `CT_CtrlPr` | 0..1 |  |  |

## `m:CT_Nary`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `naryPr` | `CT_NaryPr` | 0..1 |  |  |
| element | `sub` | `CT_OMathArg` |  |  |  |
| element | `sup` | `CT_OMathArg` |  |  |  |
| element | `e` | `CT_OMathArg` |  |  |  |

## `m:CT_PhantPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `show` | `CT_OnOff` | 0..1 |  |  |
| element | `zeroWid` | `CT_OnOff` | 0..1 |  |  |
| element | `zeroAsc` | `CT_OnOff` | 0..1 |  |  |
| element | `zeroDesc` | `CT_OnOff` | 0..1 |  |  |
| element | `transp` | `CT_OnOff` | 0..1 |  |  |
| element | `ctrlPr` | `CT_CtrlPr` | 0..1 |  |  |

## `m:CT_Phant`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `phantPr` | `CT_PhantPr` | 0..1 |  |  |
| element | `e` | `CT_OMathArg` |  |  |  |

## `m:CT_RadPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `degHide` | `CT_OnOff` | 0..1 |  |  |
| element | `ctrlPr` | `CT_CtrlPr` | 0..1 |  |  |

## `m:CT_Rad`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `radPr` | `CT_RadPr` | 0..1 |  |  |
| element | `deg` | `CT_OMathArg` |  |  |  |
| element | `e` | `CT_OMathArg` |  |  |  |

## `m:CT_SPrePr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ctrlPr` | `CT_CtrlPr` | 0..1 |  |  |

## `m:CT_SPre`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sPrePr` | `CT_SPrePr` | 0..1 |  |  |
| element | `sub` | `CT_OMathArg` |  |  |  |
| element | `sup` | `CT_OMathArg` |  |  |  |
| element | `e` | `CT_OMathArg` |  |  |  |

## `m:CT_SSubPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ctrlPr` | `CT_CtrlPr` | 0..1 |  |  |

## `m:CT_SSub`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sSubPr` | `CT_SSubPr` | 0..1 |  |  |
| element | `e` | `CT_OMathArg` |  |  |  |
| element | `sub` | `CT_OMathArg` |  |  |  |

## `m:CT_SSubSupPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `alnScr` | `CT_OnOff` | 0..1 |  |  |
| element | `ctrlPr` | `CT_CtrlPr` | 0..1 |  |  |

## `m:CT_SSubSup`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sSubSupPr` | `CT_SSubSupPr` | 0..1 |  |  |
| element | `e` | `CT_OMathArg` |  |  |  |
| element | `sub` | `CT_OMathArg` |  |  |  |
| element | `sup` | `CT_OMathArg` |  |  |  |

## `m:CT_SSupPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `ctrlPr` | `CT_CtrlPr` | 0..1 |  |  |

## `m:CT_SSup`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sSupPr` | `CT_SSupPr` | 0..1 |  |  |
| element | `e` | `CT_OMathArg` |  |  |  |
| element | `sup` | `CT_OMathArg` |  |  |  |

## `m:EG_OMathMathElements`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `acc` | `CT_Acc` | choice |  |  |
| element | `bar` | `CT_Bar` | choice |  |  |
| element | `box` | `CT_Box` | choice |  |  |
| element | `borderBox` | `CT_BorderBox` | choice |  |  |
| element | `d` | `CT_D` | choice |  |  |
| element | `eqArr` | `CT_EqArr` | choice |  |  |
| element | `f` | `CT_F` | choice |  |  |
| element | `func` | `CT_Func` | choice |  |  |
| element | `groupChr` | `CT_GroupChr` | choice |  |  |
| element | `limLow` | `CT_LimLow` | choice |  |  |
| element | `limUpp` | `CT_LimUpp` | choice |  |  |
| element | `m` | `CT_M` | choice |  |  |
| element | `nary` | `CT_Nary` | choice |  |  |
| element | `phant` | `CT_Phant` | choice |  |  |
| element | `rad` | `CT_Rad` | choice |  |  |
| element | `sPre` | `CT_SPre` | choice |  |  |
| element | `sSub` | `CT_SSub` | choice |  |  |
| element | `sSubSup` | `CT_SSubSup` | choice |  |  |
| element | `sSup` | `CT_SSup` | choice |  |  |
| element | `r` | `CT_R` | choice |  |  |

## `m:EG_OMathElements`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_OMathMathElements` | `EG_OMathMathElements` | choice |  |  |
| group | `EG_PContentMath` | `EG_PContentMath` | choice |  |  |

## `m:CT_OMathArgPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `argSz` | `CT_Integer2` | 0..1 |  |  |

## `m:CT_OMathArg`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `argPr` | `CT_OMathArgPr` | 0..1 |  |  |
| group | `EG_OMathElements` | `EG_OMathElements` | 0..unbounded |  |  |
| element | `ctrlPr` | `CT_CtrlPr` | 0..1 |  |  |

## `m:ST_Jc`
Restriction: `base=string`
- `left`
- `right`
- `center`
- `centerGroup`

## `m:CT_OMathJc`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_Jc` | optional |  |  |

## `m:CT_OMathParaPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `jc` | `CT_OMathJc` | 0..1 |  |  |

## `m:CT_TwipsMeasure`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_TwipsMeasure` | required |  |  |

## `m:ST_BreakBin`
Restriction: `base=string`
- `before`
- `after`
- `repeat`

## `m:CT_BreakBin`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_BreakBin` | optional |  |  |

## `m:ST_BreakBinSub`
Restriction: `base=string`
- `--`
- `-+`
- `+-`

## `m:CT_BreakBinSub`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@val` | `ST_BreakBinSub` | optional |  |  |

## `m:CT_MathPr`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `mathFont` | `CT_String` | 0..1 |  |  |
| element | `brkBin` | `CT_BreakBin` | 0..1 |  |  |
| element | `brkBinSub` | `CT_BreakBinSub` | 0..1 |  |  |
| element | `smallFrac` | `CT_OnOff` | 0..1 |  |  |
| element | `dispDef` | `CT_OnOff` | 0..1 |  |  |
| element | `lMargin` | `CT_TwipsMeasure` | 0..1 |  |  |
| element | `rMargin` | `CT_TwipsMeasure` | 0..1 |  |  |
| element | `defJc` | `CT_OMathJc` | 0..1 |  |  |
| element | `preSp` | `CT_TwipsMeasure` | 0..1 |  |  |
| element | `postSp` | `CT_TwipsMeasure` | 0..1 |  |  |
| element | `interSp` | `CT_TwipsMeasure` | 0..1 |  |  |
| element | `intraSp` | `CT_TwipsMeasure` | 0..1 |  |  |
| element | `wrapIndent` | `CT_TwipsMeasure` | choice |  |  |
| element | `wrapRight` | `CT_OnOff` | choice |  |  |
| element | `intLim` | `CT_LimLoc` | 0..1 |  |  |
| element | `naryLim` | `CT_LimLoc` | 0..1 |  |  |

## `m:<mathPr>` : `CT_MathPr`

## `m:CT_OMathPara`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `oMathParaPr` | `CT_OMathParaPr` | 0..1 |  |  |
| element | `oMath` | `CT_OMath` | 1..unbounded |  |  |

## `m:CT_OMath`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_OMathElements` | `EG_OMathElements` | 0..unbounded |  |  |

## `m:<oMathPara>` : `CT_OMathPara`

## `m:<oMath>` : `CT_OMath`

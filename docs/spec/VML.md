# VML: Legacy VML

Generated. Tier 1L; render 2F; edit -; level Retain. 824 rows.


## `v:AG_Id`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@id` | `string` | optional |  |  |

## `v:AG_Style`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@style` | `string` | optional |  |  |

## `v:AG_Type`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@type` | `string` | optional |  |  |

## `v:AG_Adj`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@adj` | `string` | optional |  |  |

## `v:AG_Path`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@path` | `string` | optional |  |  |

## `v:AG_Fill`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@filled` | `ST_TrueFalse` | optional |  |  |
| attribute | `@fillcolor` | `ST_ColorType` | optional |  |  |

## `v:AG_Chromakey`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@chromakey` | `ST_ColorType` | optional |  |  |

## `v:AG_Ext`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@ext` | `ST_Ext` | optional |  |  |

## `v:AG_CoreAttributes`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_Id` | `AG_Id` |  |  |  |
| attrgroup | `AG_Style` | `AG_Style` |  |  |  |
| attribute | `@href` | `string` | optional |  |  |
| attribute | `@target` | `string` | optional |  |  |
| attribute | `@class` | `string` | optional |  |  |
| attribute | `@title` | `string` | optional |  |  |
| attribute | `@alt` | `string` | optional |  |  |
| attribute | `@coordsize` | `string` | optional |  |  |
| attribute | `@coordorigin` | `string` | optional |  |  |
| attribute | `@wrapcoords` | `string` | optional |  |  |
| attribute | `@print` | `ST_TrueFalse` | optional |  |  |

## `v:AG_ShapeAttributes`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_Chromakey` | `AG_Chromakey` |  |  |  |
| attrgroup | `AG_Fill` | `AG_Fill` |  |  |  |
| attribute | `@opacity` | `string` | optional |  |  |
| attribute | `@stroked` | `ST_TrueFalse` | optional |  |  |
| attribute | `@strokecolor` | `ST_ColorType` | optional |  |  |
| attribute | `@strokeweight` | `string` | optional |  |  |
| attribute | `@insetpen` | `ST_TrueFalse` | optional |  |  |

## `v:AG_OfficeCoreAttributes`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@o:spid` | `spid` | optional |  |  |
| attribute | `@o:oned` | `oned` | optional |  |  |
| attribute | `@o:regroupid` | `regroupid` | optional |  |  |
| attribute | `@o:doubleclicknotify` | `doubleclicknotify` | optional |  |  |
| attribute | `@o:button` | `button` | optional |  |  |
| attribute | `@o:userhidden` | `userhidden` | optional |  |  |
| attribute | `@o:bullet` | `bullet` | optional |  |  |
| attribute | `@o:hr` | `hr` | optional |  |  |
| attribute | `@o:hrstd` | `hrstd` | optional |  |  |
| attribute | `@o:hrnoshade` | `hrnoshade` | optional |  |  |
| attribute | `@o:hrpct` | `hrpct` | optional |  |  |
| attribute | `@o:hralign` | `hralign` | optional |  |  |
| attribute | `@o:allowincell` | `allowincell` | optional |  |  |
| attribute | `@o:allowoverlap` | `allowoverlap` | optional |  |  |
| attribute | `@o:userdrawn` | `userdrawn` | optional |  |  |
| attribute | `@o:bordertopcolor` | `bordertopcolor` | optional |  |  |
| attribute | `@o:borderleftcolor` | `borderleftcolor` | optional |  |  |
| attribute | `@o:borderbottomcolor` | `borderbottomcolor` | optional |  |  |
| attribute | `@o:borderrightcolor` | `borderrightcolor` | optional |  |  |
| attribute | `@o:dgmlayout` | `dgmlayout` | optional |  |  |
| attribute | `@o:dgmnodekind` | `dgmnodekind` | optional |  |  |
| attribute | `@o:dgmlayoutmru` | `dgmlayoutmru` | optional |  |  |
| attribute | `@o:insetmode` | `insetmode` | optional |  |  |

## `v:AG_OfficeShapeAttributes`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@o:spt` | `spt` | optional |  |  |
| attribute | `@o:connectortype` | `connectortype` | optional |  |  |
| attribute | `@o:bwmode` | `bwmode` | optional |  |  |
| attribute | `@o:bwpure` | `bwpure` | optional |  |  |
| attribute | `@o:bwnormal` | `bwnormal` | optional |  |  |
| attribute | `@o:forcedash` | `forcedash` | optional |  |  |
| attribute | `@o:oleicon` | `oleicon` | optional |  |  |
| attribute | `@o:ole` | `ole` | optional |  |  |
| attribute | `@o:preferrelative` | `preferrelative` | optional |  |  |
| attribute | `@o:cliptowrap` | `cliptowrap` | optional |  |  |
| attribute | `@o:clip` | `clip` | optional |  |  |

## `v:AG_AllCoreAttributes`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_CoreAttributes` | `AG_CoreAttributes` |  |  |  |
| attrgroup | `AG_OfficeCoreAttributes` | `AG_OfficeCoreAttributes` |  |  |  |

## `v:AG_AllShapeAttributes`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_ShapeAttributes` | `AG_ShapeAttributes` |  |  |  |
| attrgroup | `AG_OfficeShapeAttributes` | `AG_OfficeShapeAttributes` |  |  |  |

## `v:AG_ImageAttributes`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@src` | `string` | optional |  |  |
| attribute | `@cropleft` | `string` | optional |  |  |
| attribute | `@croptop` | `string` | optional |  |  |
| attribute | `@cropright` | `string` | optional |  |  |
| attribute | `@cropbottom` | `string` | optional |  |  |
| attribute | `@gain` | `string` | optional |  |  |
| attribute | `@blacklevel` | `string` | optional |  |  |
| attribute | `@gamma` | `string` | optional |  |  |
| attribute | `@grayscale` | `ST_TrueFalse` | optional |  |  |
| attribute | `@bilevel` | `ST_TrueFalse` | optional |  |  |

## `v:AG_StrokeAttributes`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@on` | `ST_TrueFalse` | optional |  |  |
| attribute | `@weight` | `string` | optional |  |  |
| attribute | `@color` | `ST_ColorType` | optional |  |  |
| attribute | `@opacity` | `string` | optional |  |  |
| attribute | `@linestyle` | `ST_StrokeLineStyle` | optional |  |  |
| attribute | `@miterlimit` | `decimal` | optional |  |  |
| attribute | `@joinstyle` | `ST_StrokeJoinStyle` | optional |  |  |
| attribute | `@endcap` | `ST_StrokeEndCap` | optional |  |  |
| attribute | `@dashstyle` | `string` | optional |  |  |
| attribute | `@filltype` | `ST_FillType` | optional |  |  |
| attribute | `@src` | `string` | optional |  |  |
| attribute | `@imageaspect` | `ST_ImageAspect` | optional |  |  |
| attribute | `@imagesize` | `string` | optional |  |  |
| attribute | `@imagealignshape` | `ST_TrueFalse` | optional |  |  |
| attribute | `@color2` | `ST_ColorType` | optional |  |  |
| attribute | `@startarrow` | `ST_StrokeArrowType` | optional |  |  |
| attribute | `@startarrowwidth` | `ST_StrokeArrowWidth` | optional |  |  |
| attribute | `@startarrowlength` | `ST_StrokeArrowLength` | optional |  |  |
| attribute | `@endarrow` | `ST_StrokeArrowType` | optional |  |  |
| attribute | `@endarrowwidth` | `ST_StrokeArrowWidth` | optional |  |  |
| attribute | `@endarrowlength` | `ST_StrokeArrowLength` | optional |  |  |
| attribute | `@o:href` | `href` | optional |  |  |
| attribute | `@o:althref` | `althref` | optional |  |  |
| attribute | `@o:title` | `title` | optional |  |  |
| attribute | `@o:forcedash` | `forcedash` | optional |  |  |
| attribute | `@r:id` | `id` | optional |  |  |
| attribute | `@insetpen` | `ST_TrueFalse` | optional |  |  |
| attribute | `@o:relid` | `relid` | optional |  |  |

## `v:EG_ShapeElements`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `path` | `path` | choice |  |  |
| element | `formulas` | `formulas` | choice |  |  |
| element | `handles` | `handles` | choice |  |  |
| element | `fill` | `fill` | choice |  |  |
| element | `stroke` | `stroke` | choice |  |  |
| element | `shadow` | `shadow` | choice |  |  |
| element | `textbox` | `textbox` | choice |  |  |
| element | `textpath` | `textpath` | choice |  |  |
| element | `imagedata` | `imagedata` | choice |  |  |
| element | `o:skew` | `skew` | choice |  |  |
| element | `o:extrusion` | `extrusion` | choice |  |  |
| element | `o:callout` | `callout` | choice |  |  |
| element | `o:lock` | `lock` | choice |  |  |
| element | `o:clippath` | `clippath` | choice |  |  |
| element | `o:signatureline` | `signatureline` | choice |  |  |
| element | `w10:wrap` | `wrap` | choice |  |  |
| element | `w10:anchorlock` | `anchorlock` | choice |  |  |
| element | `w10:bordertop` | `bordertop` | choice |  |  |
| element | `w10:borderbottom` | `borderbottom` | choice |  |  |
| element | `w10:borderleft` | `borderleft` | choice |  |  |
| element | `w10:borderright` | `borderright` | choice |  |  |
| element | `x:ClientData` | `ClientData` | 0..1 |  |  |
| element | `pvml:textdata` | `textdata` | 0..1 |  |  |

## `v:<shape>` : `CT_Shape`

## `v:<shapetype>` : `CT_Shapetype`

## `v:<group>` : `CT_Group`

## `v:<background>` : `CT_Background`

## `v:CT_Shape`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ShapeElements` | `EG_ShapeElements` | choice |  |  |
| element | `o:ink` | `ink` | choice |  |  |
| element | `pvml:iscomment` | `iscomment` | choice |  |  |
| element | `o:equationxml` | `equationxml` | choice |  |  |
| attrgroup | `AG_AllCoreAttributes` | `AG_AllCoreAttributes` |  |  |  |
| attrgroup | `AG_AllShapeAttributes` | `AG_AllShapeAttributes` |  |  |  |
| attrgroup | `AG_Type` | `AG_Type` |  |  |  |
| attrgroup | `AG_Adj` | `AG_Adj` |  |  |  |
| attrgroup | `AG_Path` | `AG_Path` |  |  |  |
| attribute | `@o:gfxdata` | `gfxdata` | optional |  |  |
| attribute | `@equationxml` | `string` | optional |  |  |

## `v:CT_Shapetype`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ShapeElements` | `EG_ShapeElements` | 0..unbounded |  |  |
| element | `o:complex` | `complex` | 0..1 |  |  |
| attrgroup | `AG_AllCoreAttributes` | `AG_AllCoreAttributes` |  |  |  |
| attrgroup | `AG_AllShapeAttributes` | `AG_AllShapeAttributes` |  |  |  |
| attrgroup | `AG_Adj` | `AG_Adj` |  |  |  |
| attrgroup | `AG_Path` | `AG_Path` |  |  |  |
| attribute | `@o:master` | `master` | optional |  |  |

## `v:CT_Group`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ShapeElements` | `EG_ShapeElements` | choice |  |  |
| element | `group` | `group` | choice |  |  |
| element | `shape` | `shape` | choice |  |  |
| element | `shapetype` | `shapetype` | choice |  |  |
| element | `arc` | `arc` | choice |  |  |
| element | `curve` | `curve` | choice |  |  |
| element | `image` | `image` | choice |  |  |
| element | `line` | `line` | choice |  |  |
| element | `oval` | `oval` | choice |  |  |
| element | `polyline` | `polyline` | choice |  |  |
| element | `rect` | `rect` | choice |  |  |
| element | `roundrect` | `roundrect` | choice |  |  |
| element | `o:diagram` | `diagram` | choice |  |  |
| attrgroup | `AG_AllCoreAttributes` | `AG_AllCoreAttributes` |  |  |  |
| attrgroup | `AG_Fill` | `AG_Fill` |  |  |  |
| attribute | `@editas` | `ST_EditAs` | optional |  |  |
| attribute | `@o:tableproperties` | `tableproperties` | optional |  |  |
| attribute | `@o:tablelimits` | `tablelimits` | optional |  |  |

## `v:CT_Background`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `fill` | `fill` | 0..1 |  |  |
| attrgroup | `AG_Id` | `AG_Id` |  |  |  |
| attrgroup | `AG_Fill` | `AG_Fill` |  |  |  |
| attribute | `@o:bwmode` | `bwmode` | optional |  |  |
| attribute | `@o:bwpure` | `bwpure` | optional |  |  |
| attribute | `@o:bwnormal` | `bwnormal` | optional |  |  |
| attribute | `@o:targetscreensize` | `targetscreensize` | optional |  |  |

## `v:<fill>` : `CT_Fill`

## `v:<formulas>` : `CT_Formulas`

## `v:<handles>` : `CT_Handles`

## `v:<imagedata>` : `CT_ImageData`

## `v:<path>` : `CT_Path`

## `v:<textbox>` : `CT_Textbox`

## `v:<shadow>` : `CT_Shadow`

## `v:<stroke>` : `CT_Stroke`

## `v:<textpath>` : `CT_TextPath`

## `v:CT_Fill`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `o:fill` | `fill` | 0..1 |  |  |
| attrgroup | `AG_Id` | `AG_Id` |  |  |  |
| attribute | `@type` | `ST_FillType` | optional |  |  |
| attribute | `@on` | `ST_TrueFalse` | optional |  |  |
| attribute | `@color` | `ST_ColorType` | optional |  |  |
| attribute | `@opacity` | `string` | optional |  |  |
| attribute | `@color2` | `ST_ColorType` | optional |  |  |
| attribute | `@src` | `string` | optional |  |  |
| attribute | `@o:href` | `href` | optional |  |  |
| attribute | `@o:althref` | `althref` | optional |  |  |
| attribute | `@size` | `string` | optional |  |  |
| attribute | `@origin` | `string` | optional |  |  |
| attribute | `@position` | `string` | optional |  |  |
| attribute | `@aspect` | `ST_ImageAspect` | optional |  |  |
| attribute | `@colors` | `string` | optional |  |  |
| attribute | `@angle` | `decimal` | optional |  |  |
| attribute | `@alignshape` | `ST_TrueFalse` | optional |  |  |
| attribute | `@focus` | `string` | optional |  |  |
| attribute | `@focussize` | `string` | optional |  |  |
| attribute | `@focusposition` | `string` | optional |  |  |
| attribute | `@method` | `ST_FillMethod` | optional |  |  |
| attribute | `@o:detectmouseclick` | `detectmouseclick` | optional |  |  |
| attribute | `@o:title` | `title` | optional |  |  |
| attribute | `@o:opacity2` | `opacity2` | optional |  |  |
| attribute | `@recolor` | `ST_TrueFalse` | optional |  |  |
| attribute | `@rotate` | `ST_TrueFalse` | optional |  |  |
| attribute | `@r:id` | `id` | optional |  |  |
| attribute | `@o:relid` | `relid` | optional |  |  |

## `v:CT_Formulas`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `f` | `CT_F` | 0..unbounded |  |  |

## `v:CT_F`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@eqn` | `string` | optional |  |  |

## `v:CT_Handles`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `h` | `CT_H` | 0..unbounded |  |  |

## `v:CT_H`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@position` | `string` | optional |  |  |
| attribute | `@polar` | `string` | optional |  |  |
| attribute | `@map` | `string` | optional |  |  |
| attribute | `@invx` | `ST_TrueFalse` | optional |  |  |
| attribute | `@invy` | `ST_TrueFalse` | optional |  |  |
| attribute | `@switch` | `ST_TrueFalseBlank` | optional |  |  |
| attribute | `@xrange` | `string` | optional |  |  |
| attribute | `@yrange` | `string` | optional |  |  |
| attribute | `@radiusrange` | `string` | optional |  |  |

## `v:CT_ImageData`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_Id` | `AG_Id` |  |  |  |
| attrgroup | `AG_ImageAttributes` | `AG_ImageAttributes` |  |  |  |
| attrgroup | `AG_Chromakey` | `AG_Chromakey` |  |  |  |
| attribute | `@embosscolor` | `ST_ColorType` | optional |  |  |
| attribute | `@recolortarget` | `ST_ColorType` | optional |  |  |
| attribute | `@o:href` | `href` | optional |  |  |
| attribute | `@o:althref` | `althref` | optional |  |  |
| attribute | `@o:title` | `title` | optional |  |  |
| attribute | `@o:oleid` | `oleid` | optional |  |  |
| attribute | `@o:detectmouseclick` | `detectmouseclick` | optional |  |  |
| attribute | `@o:movie` | `movie` | optional |  |  |
| attribute | `@o:relid` | `relid` | optional |  |  |
| attribute | `@r:id` | `id` | optional |  |  |
| attribute | `@r:pict` | `pict` | optional |  |  |
| attribute | `@r:href` | `href` | optional |  |  |

## `v:CT_Path`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_Id` | `AG_Id` |  |  |  |
| attribute | `@v` | `string` | optional |  |  |
| attribute | `@limo` | `string` | optional |  |  |
| attribute | `@textboxrect` | `string` | optional |  |  |
| attribute | `@fillok` | `ST_TrueFalse` | optional |  |  |
| attribute | `@strokeok` | `ST_TrueFalse` | optional |  |  |
| attribute | `@shadowok` | `ST_TrueFalse` | optional |  |  |
| attribute | `@arrowok` | `ST_TrueFalse` | optional |  |  |
| attribute | `@gradientshapeok` | `ST_TrueFalse` | optional |  |  |
| attribute | `@textpathok` | `ST_TrueFalse` | optional |  |  |
| attribute | `@insetpenok` | `ST_TrueFalse` | optional |  |  |
| attribute | `@o:connecttype` | `connecttype` | optional |  |  |
| attribute | `@o:connectlocs` | `connectlocs` | optional |  |  |
| attribute | `@o:connectangles` | `connectangles` | optional |  |  |
| attribute | `@o:extrusionok` | `extrusionok` | optional |  |  |

## `v:CT_Shadow`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_Id` | `AG_Id` |  |  |  |
| attribute | `@on` | `ST_TrueFalse` | optional |  |  |
| attribute | `@type` | `ST_ShadowType` | optional |  |  |
| attribute | `@obscured` | `ST_TrueFalse` | optional |  |  |
| attribute | `@color` | `ST_ColorType` | optional |  |  |
| attribute | `@opacity` | `string` | optional |  |  |
| attribute | `@offset` | `string` | optional |  |  |
| attribute | `@color2` | `ST_ColorType` | optional |  |  |
| attribute | `@offset2` | `string` | optional |  |  |
| attribute | `@origin` | `string` | optional |  |  |
| attribute | `@matrix` | `string` | optional |  |  |

## `v:CT_Stroke`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `o:left` | `left` | 0..1 |  |  |
| element | `o:top` | `top` | 0..1 |  |  |
| element | `o:right` | `right` | 0..1 |  |  |
| element | `o:bottom` | `bottom` | 0..1 |  |  |
| element | `o:column` | `column` | 0..1 |  |  |
| attrgroup | `AG_Id` | `AG_Id` |  |  |  |
| attrgroup | `AG_StrokeAttributes` | `AG_StrokeAttributes` |  |  |  |

## `v:CT_Textbox`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `w:txbxContent` | `txbxContent` | 0..1 |  |  |
| any | `xsd:any` | `##local` |  |  | skip |
| attrgroup | `AG_Id` | `AG_Id` |  |  |  |
| attrgroup | `AG_Style` | `AG_Style` |  |  |  |
| attribute | `@inset` | `string` | optional |  |  |
| attribute | `@o:singleclick` | `singleclick` | optional |  |  |
| attribute | `@o:insetmode` | `insetmode` | optional |  |  |

## `v:CT_TextPath`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_Id` | `AG_Id` |  |  |  |
| attrgroup | `AG_Style` | `AG_Style` |  |  |  |
| attribute | `@on` | `ST_TrueFalse` | optional |  |  |
| attribute | `@fitshape` | `ST_TrueFalse` | optional |  |  |
| attribute | `@fitpath` | `ST_TrueFalse` | optional |  |  |
| attribute | `@trim` | `ST_TrueFalse` | optional |  |  |
| attribute | `@xscale` | `ST_TrueFalse` | optional |  |  |
| attribute | `@string` | `string` | optional |  |  |

## `v:<arc>` : `CT_Arc`

## `v:<curve>` : `CT_Curve`

## `v:<image>` : `CT_Image`

## `v:<line>` : `CT_Line`

## `v:<oval>` : `CT_Oval`

## `v:<polyline>` : `CT_PolyLine`

## `v:<rect>` : `CT_Rect`

## `v:<roundrect>` : `CT_RoundRect`

## `v:CT_Arc`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ShapeElements` | `EG_ShapeElements` | 0..unbounded |  |  |
| attrgroup | `AG_AllCoreAttributes` | `AG_AllCoreAttributes` |  |  |  |
| attrgroup | `AG_AllShapeAttributes` | `AG_AllShapeAttributes` |  |  |  |
| attribute | `@startAngle` | `decimal` | optional |  |  |
| attribute | `@endAngle` | `decimal` | optional |  |  |

## `v:CT_Curve`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ShapeElements` | `EG_ShapeElements` | 0..unbounded |  |  |
| attrgroup | `AG_AllCoreAttributes` | `AG_AllCoreAttributes` |  |  |  |
| attrgroup | `AG_AllShapeAttributes` | `AG_AllShapeAttributes` |  |  |  |
| attribute | `@from` | `string` | optional |  |  |
| attribute | `@control1` | `string` | optional |  |  |
| attribute | `@control2` | `string` | optional |  |  |
| attribute | `@to` | `string` | optional |  |  |

## `v:CT_Image`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ShapeElements` | `EG_ShapeElements` | 0..unbounded |  |  |
| attrgroup | `AG_AllCoreAttributes` | `AG_AllCoreAttributes` |  |  |  |
| attrgroup | `AG_AllShapeAttributes` | `AG_AllShapeAttributes` |  |  |  |
| attrgroup | `AG_ImageAttributes` | `AG_ImageAttributes` |  |  |  |

## `v:CT_Line`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ShapeElements` | `EG_ShapeElements` | 0..unbounded |  |  |
| attrgroup | `AG_AllCoreAttributes` | `AG_AllCoreAttributes` |  |  |  |
| attrgroup | `AG_AllShapeAttributes` | `AG_AllShapeAttributes` |  |  |  |
| attribute | `@from` | `string` | optional |  |  |
| attribute | `@to` | `string` | optional |  |  |

## `v:CT_Oval`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ShapeElements` | `EG_ShapeElements` | 0..unbounded |  |  |
| attrgroup | `AG_AllCoreAttributes` | `AG_AllCoreAttributes` |  |  |  |
| attrgroup | `AG_AllShapeAttributes` | `AG_AllShapeAttributes` |  |  |  |

## `v:CT_PolyLine`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ShapeElements` | `EG_ShapeElements` | choice |  |  |
| element | `o:ink` | `ink` | choice |  |  |
| attrgroup | `AG_AllCoreAttributes` | `AG_AllCoreAttributes` |  |  |  |
| attrgroup | `AG_AllShapeAttributes` | `AG_AllShapeAttributes` |  |  |  |
| attribute | `@points` | `string` | optional |  |  |

## `v:CT_Rect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ShapeElements` | `EG_ShapeElements` | 0..unbounded |  |  |
| attrgroup | `AG_AllCoreAttributes` | `AG_AllCoreAttributes` |  |  |  |
| attrgroup | `AG_AllShapeAttributes` | `AG_AllShapeAttributes` |  |  |  |

## `v:CT_RoundRect`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| group | `EG_ShapeElements` | `EG_ShapeElements` | 0..unbounded |  |  |
| attrgroup | `AG_AllCoreAttributes` | `AG_AllCoreAttributes` |  |  |  |
| attrgroup | `AG_AllShapeAttributes` | `AG_AllShapeAttributes` |  |  |  |
| attribute | `@arcsize` | `string` | optional |  |  |

## `v:ST_Ext`
Restriction: `base=string`
- `view`
- `edit`
- `backwardCompatible`

## `v:ST_FillType`
Restriction: `base=string`
- `solid`
- `gradient`
- `gradientRadial`
- `tile`
- `pattern`
- `frame`

## `v:ST_FillMethod`
Restriction: `base=string`
- `none`
- `linear`
- `sigma`
- `any`
- `linear sigma`

## `v:ST_ShadowType`
Restriction: `base=string`
- `single`
- `double`
- `emboss`
- `perspective`

## `v:ST_StrokeLineStyle`
Restriction: `base=string`
- `single`
- `thinThin`
- `thinThick`
- `thickThin`
- `thickBetweenThin`

## `v:ST_StrokeJoinStyle`
Restriction: `base=string`
- `round`
- `bevel`
- `miter`

## `v:ST_StrokeEndCap`
Restriction: `base=string`
- `flat`
- `square`
- `round`

## `v:ST_StrokeArrowLength`
Restriction: `base=string`
- `short`
- `medium`
- `long`

## `v:ST_StrokeArrowWidth`
Restriction: `base=string`
- `narrow`
- `medium`
- `wide`

## `v:ST_StrokeArrowType`
Restriction: `base=string`
- `none`
- `block`
- `classic`
- `oval`
- `diamond`
- `open`

## `v:ST_ImageAspect`
Restriction: `base=string`
- `ignore`
- `atMost`
- `atLeast`

## `v:ST_EditAs`
Restriction: `base=string`
- `canvas`
- `orgchart`
- `radial`
- `cycle`
- `stacked`
- `venn`
- `bullseye`

## `o:@bwmode` : `ST_BWMode`

## `o:@bwpure` : `ST_BWMode`

## `o:@bwnormal` : `ST_BWMode`

## `o:@targetscreensize` : `ST_ScreenSize`

## `o:@insetmode` : `ST_InsetMode`

## `o:@spt` : `float`

## `o:@wrapcoords` : `string`

## `o:@oned` : `ST_TrueFalse`

## `o:@regroupid` : `integer`

## `o:@doubleclicknotify` : `ST_TrueFalse`

## `o:@connectortype` : `ST_ConnectorType`

## `o:@button` : `ST_TrueFalse`

## `o:@userhidden` : `ST_TrueFalse`

## `o:@forcedash` : `ST_TrueFalse`

## `o:@oleicon` : `ST_TrueFalse`

## `o:@ole` : `ST_TrueFalseBlank`

## `o:@preferrelative` : `ST_TrueFalse`

## `o:@cliptowrap` : `ST_TrueFalse`

## `o:@clip` : `ST_TrueFalse`

## `o:@bullet` : `ST_TrueFalse`

## `o:@hr` : `ST_TrueFalse`

## `o:@hrstd` : `ST_TrueFalse`

## `o:@hrnoshade` : `ST_TrueFalse`

## `o:@hrpct` : `float`

## `o:@hralign` : `ST_HrAlign`

## `o:@allowincell` : `ST_TrueFalse`

## `o:@allowoverlap` : `ST_TrueFalse`

## `o:@userdrawn` : `ST_TrueFalse`

## `o:@bordertopcolor` : `string`

## `o:@borderleftcolor` : `string`

## `o:@borderbottomcolor` : `string`

## `o:@borderrightcolor` : `string`

## `o:@connecttype` : `ST_ConnectType`

## `o:@connectlocs` : `string`

## `o:@connectangles` : `string`

## `o:@master` : `string`

## `o:@extrusionok` : `ST_TrueFalse`

## `o:@href` : `string`

## `o:@althref` : `string`

## `o:@title` : `string`

## `o:@singleclick` : `ST_TrueFalse`

## `o:@oleid` : `float`

## `o:@detectmouseclick` : `ST_TrueFalse`

## `o:@movie` : `float`

## `o:@spid` : `string`

## `o:@opacity2` : `string`

## `o:@relid` : `ST_RelationshipId`

## `o:@dgmlayout` : `ST_DiagramLayout`

## `o:@dgmnodekind` : `integer`

## `o:@dgmlayoutmru` : `ST_DiagramLayout`

## `o:@gfxdata` : `base64Binary`

## `o:@tableproperties` : `string`

## `o:@tablelimits` : `string`

## `o:<shapedefaults>` : `CT_ShapeDefaults`

## `o:<shapelayout>` : `CT_ShapeLayout`

## `o:<signatureline>` : `CT_SignatureLine`

## `o:<ink>` : `CT_Ink`

## `o:<diagram>` : `CT_Diagram`

## `o:<equationxml>` : `CT_EquationXml`

## `o:CT_ShapeDefaults`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `v:fill` | `fill` | 0..1 |  |  |
| element | `v:stroke` | `stroke` | 0..1 |  |  |
| element | `v:textbox` | `textbox` | 0..1 |  |  |
| element | `v:shadow` | `shadow` | 0..1 |  |  |
| element | `skew` | `skew` | 0..1 |  |  |
| element | `extrusion` | `extrusion` | 0..1 |  |  |
| element | `callout` | `callout` | 0..1 |  |  |
| element | `lock` | `lock` | 0..1 |  |  |
| element | `colormru` | `CT_ColorMru` | 0..1 |  |  |
| element | `colormenu` | `CT_ColorMenu` | 0..1 |  |  |
| attrgroup | `AG_Ext` | `AG_Ext` |  |  |  |
| attribute | `@spidmax` | `integer` | optional |  |  |
| attribute | `@style` | `string` | optional |  |  |
| attribute | `@fill` | `ST_TrueFalse` | optional |  |  |
| attribute | `@fillcolor` | `ST_ColorType` | optional |  |  |
| attribute | `@stroke` | `ST_TrueFalse` | optional |  |  |
| attribute | `@strokecolor` | `ST_ColorType` | optional |  |  |
| attribute | `@allowincell` | `ST_TrueFalse` | optional |  |  |

## `o:CT_Ink`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@i` | `string` | optional |  |  |
| attribute | `@annotation` | `ST_TrueFalse` | optional |  |  |
| attribute | `@contentType` | `ST_ContentType` | optional |  |  |

## `o:CT_SignatureLine`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_Ext` | `AG_Ext` |  |  |  |
| attribute | `@issignatureline` | `ST_TrueFalse` | optional |  |  |
| attribute | `@id` | `ST_Guid` | optional |  |  |
| attribute | `@provid` | `ST_Guid` | optional |  |  |
| attribute | `@signinginstructionsset` | `ST_TrueFalse` | optional |  |  |
| attribute | `@allowcomments` | `ST_TrueFalse` | optional |  |  |
| attribute | `@showsigndate` | `ST_TrueFalse` | optional |  |  |
| attribute | `@suggestedsigner` | `string` | optional |  |  |
| attribute | `@suggestedsigner2` | `string` | optional |  |  |
| attribute | `@suggestedsigneremail` | `string` | optional |  |  |
| attribute | `@signinginstructions` | `string` | optional |  |  |
| attribute | `@addlxml` | `string` | optional |  |  |
| attribute | `@sigprovurl` | `string` | optional |  |  |

## `o:CT_ShapeLayout`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `idmap` | `CT_IdMap` | 0..1 |  |  |
| element | `regrouptable` | `CT_RegroupTable` | 0..1 |  |  |
| element | `rules` | `CT_Rules` | 0..1 |  |  |
| attrgroup | `AG_Ext` | `AG_Ext` |  |  |  |

## `o:CT_IdMap`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_Ext` | `AG_Ext` |  |  |  |
| attribute | `@data` | `string` | optional |  |  |

## `o:CT_RegroupTable`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `entry` | `CT_Entry` | 0..unbounded |  |  |
| attrgroup | `AG_Ext` | `AG_Ext` |  |  |  |

## `o:CT_Entry`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@new` | `int` | optional |  |  |
| attribute | `@old` | `int` | optional |  |  |

## `o:CT_Rules`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `r` | `CT_R` | 0..unbounded |  |  |
| attrgroup | `AG_Ext` | `AG_Ext` |  |  |  |

## `o:CT_R`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `proxy` | `CT_Proxy` | 0..unbounded |  |  |
| attribute | `@id` | `string` | required |  |  |
| attribute | `@type` | `ST_RType` | optional |  |  |
| attribute | `@how` | `ST_How` | optional |  |  |
| attribute | `@idref` | `string` | optional |  |  |

## `o:CT_Proxy`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@start` | `ST_TrueFalseBlank` | optional | false |  |
| attribute | `@end` | `ST_TrueFalseBlank` | optional | false |  |
| attribute | `@idref` | `string` | optional |  |  |
| attribute | `@connectloc` | `int` | optional |  |  |

## `o:CT_Diagram`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `relationtable` | `CT_RelationTable` | 0..1 |  |  |
| attrgroup | `AG_Ext` | `AG_Ext` |  |  |  |
| attribute | `@dgmstyle` | `integer` | optional |  |  |
| attribute | `@autoformat` | `ST_TrueFalse` | optional |  |  |
| attribute | `@reverse` | `ST_TrueFalse` | optional |  |  |
| attribute | `@autolayout` | `ST_TrueFalse` | optional |  |  |
| attribute | `@dgmscalex` | `integer` | optional |  |  |
| attribute | `@dgmscaley` | `integer` | optional |  |  |
| attribute | `@dgmfontsize` | `integer` | optional |  |  |
| attribute | `@constrainbounds` | `string` | optional |  |  |
| attribute | `@dgmbasetextscale` | `integer` | optional |  |  |

## `o:CT_EquationXml`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| any | `xsd:any` | `##any` |  |  |  |
| attribute | `@contentType` | `ST_AlternateMathContentType` | optional |  |  |

## `o:ST_AlternateMathContentType`
Restriction: `base=string`

## `o:CT_RelationTable`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `rel` | `CT_Relation` | 0..unbounded |  |  |
| attrgroup | `AG_Ext` | `AG_Ext` |  |  |  |

## `o:CT_Relation`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_Ext` | `AG_Ext` |  |  |  |
| attribute | `@idsrc` | `string` | optional |  |  |
| attribute | `@iddest` | `string` | optional |  |  |
| attribute | `@idcntr` | `string` | optional |  |  |

## `o:CT_ColorMru`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_Ext` | `AG_Ext` |  |  |  |
| attribute | `@colors` | `string` | optional |  |  |

## `o:CT_ColorMenu`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_Ext` | `AG_Ext` |  |  |  |
| attribute | `@strokecolor` | `ST_ColorType` | optional |  |  |
| attribute | `@fillcolor` | `ST_ColorType` | optional |  |  |
| attribute | `@shadowcolor` | `ST_ColorType` | optional |  |  |
| attribute | `@extrusioncolor` | `ST_ColorType` | optional |  |  |

## `o:<skew>` : `CT_Skew`

## `o:<extrusion>` : `CT_Extrusion`

## `o:<callout>` : `CT_Callout`

## `o:<lock>` : `CT_Lock`

## `o:<OLEObject>` : `CT_OLEObject`

## `o:<complex>` : `CT_Complex`

## `o:<left>` : `CT_StrokeChild`

## `o:<top>` : `CT_StrokeChild`

## `o:<right>` : `CT_StrokeChild`

## `o:<bottom>` : `CT_StrokeChild`

## `o:<column>` : `CT_StrokeChild`

## `o:<clippath>` : `CT_ClipPath`

## `o:<fill>` : `CT_Fill`

## `o:CT_Skew`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_Ext` | `AG_Ext` |  |  |  |
| attribute | `@id` | `string` | optional |  |  |
| attribute | `@on` | `ST_TrueFalse` | optional |  |  |
| attribute | `@offset` | `string` | optional |  |  |
| attribute | `@origin` | `string` | optional |  |  |
| attribute | `@matrix` | `string` | optional |  |  |

## `o:CT_Extrusion`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_Ext` | `AG_Ext` |  |  |  |
| attribute | `@on` | `ST_TrueFalse` | optional |  |  |
| attribute | `@type` | `ST_ExtrusionType` | optional | parallel |  |
| attribute | `@render` | `ST_ExtrusionRender` | optional | solid |  |
| attribute | `@viewpointorigin` | `string` | optional |  |  |
| attribute | `@viewpoint` | `string` | optional |  |  |
| attribute | `@plane` | `ST_ExtrusionPlane` | optional | XY |  |
| attribute | `@skewangle` | `float` | optional |  |  |
| attribute | `@skewamt` | `string` | optional |  |  |
| attribute | `@foredepth` | `string` | optional |  |  |
| attribute | `@backdepth` | `string` | optional |  |  |
| attribute | `@orientation` | `string` | optional |  |  |
| attribute | `@orientationangle` | `float` | optional |  |  |
| attribute | `@lockrotationcenter` | `ST_TrueFalse` | optional |  |  |
| attribute | `@autorotationcenter` | `ST_TrueFalse` | optional |  |  |
| attribute | `@rotationcenter` | `string` | optional |  |  |
| attribute | `@rotationangle` | `string` | optional |  |  |
| attribute | `@colormode` | `ST_ColorMode` | optional |  |  |
| attribute | `@color` | `ST_ColorType` | optional |  |  |
| attribute | `@shininess` | `float` | optional |  |  |
| attribute | `@specularity` | `string` | optional |  |  |
| attribute | `@diffusity` | `string` | optional |  |  |
| attribute | `@metal` | `ST_TrueFalse` | optional |  |  |
| attribute | `@edge` | `string` | optional |  |  |
| attribute | `@facet` | `string` | optional |  |  |
| attribute | `@lightface` | `ST_TrueFalse` | optional |  |  |
| attribute | `@brightness` | `string` | optional |  |  |
| attribute | `@lightposition` | `string` | optional |  |  |
| attribute | `@lightlevel` | `string` | optional |  |  |
| attribute | `@lightharsh` | `ST_TrueFalse` | optional |  |  |
| attribute | `@lightposition2` | `string` | optional |  |  |
| attribute | `@lightlevel2` | `string` | optional |  |  |
| attribute | `@lightharsh2` | `ST_TrueFalse` | optional |  |  |

## `o:CT_Callout`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_Ext` | `AG_Ext` |  |  |  |
| attribute | `@on` | `ST_TrueFalse` | optional |  |  |
| attribute | `@type` | `string` | optional |  |  |
| attribute | `@gap` | `string` | optional |  |  |
| attribute | `@angle` | `ST_Angle` | optional |  |  |
| attribute | `@dropauto` | `ST_TrueFalse` | optional |  |  |
| attribute | `@drop` | `ST_CalloutDrop` | optional |  |  |
| attribute | `@distance` | `string` | optional |  |  |
| attribute | `@lengthspecified` | `ST_TrueFalse` | optional | f |  |
| attribute | `@length` | `string` | optional |  |  |
| attribute | `@accentbar` | `ST_TrueFalse` | optional |  |  |
| attribute | `@textborder` | `ST_TrueFalse` | optional |  |  |
| attribute | `@minusx` | `ST_TrueFalse` | optional |  |  |
| attribute | `@minusy` | `ST_TrueFalse` | optional |  |  |

## `o:CT_Lock`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_Ext` | `AG_Ext` |  |  |  |
| attribute | `@position` | `ST_TrueFalse` | optional |  |  |
| attribute | `@selection` | `ST_TrueFalse` | optional |  |  |
| attribute | `@grouping` | `ST_TrueFalse` | optional |  |  |
| attribute | `@ungrouping` | `ST_TrueFalse` | optional |  |  |
| attribute | `@rotation` | `ST_TrueFalse` | optional |  |  |
| attribute | `@cropping` | `ST_TrueFalse` | optional |  |  |
| attribute | `@verticies` | `ST_TrueFalse` | optional |  |  |
| attribute | `@adjusthandles` | `ST_TrueFalse` | optional |  |  |
| attribute | `@text` | `ST_TrueFalse` | optional |  |  |
| attribute | `@aspectratio` | `ST_TrueFalse` | optional |  |  |
| attribute | `@shapetype` | `ST_TrueFalse` | optional |  |  |

## `o:CT_OLEObject`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `LinkType` | `ST_OLELinkType` | 0..1 |  |  |
| element | `LockedField` | `ST_TrueFalseBlank` | 0..1 |  |  |
| element | `FieldCodes` | `string` | 0..1 |  |  |
| attribute | `@Type` | `ST_OLEType` | optional |  |  |
| attribute | `@ProgID` | `string` | optional |  |  |
| attribute | `@ShapeID` | `string` | optional |  |  |
| attribute | `@DrawAspect` | `ST_OLEDrawAspect` | optional |  |  |
| attribute | `@ObjectID` | `string` | optional |  |  |
| attribute | `@r:id` | `id` | optional |  |  |
| attribute | `@UpdateMode` | `ST_OLEUpdateMode` | optional |  |  |

## `o:CT_Complex`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_Ext` | `AG_Ext` |  |  |  |

## `o:CT_StrokeChild`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_Ext` | `AG_Ext` |  |  |  |
| attribute | `@on` | `ST_TrueFalse` | optional |  |  |
| attribute | `@weight` | `string` | optional |  |  |
| attribute | `@color` | `ST_ColorType` | optional |  |  |
| attribute | `@color2` | `ST_ColorType` | optional |  |  |
| attribute | `@opacity` | `string` | optional |  |  |
| attribute | `@linestyle` | `ST_StrokeLineStyle` | optional |  |  |
| attribute | `@miterlimit` | `decimal` | optional |  |  |
| attribute | `@joinstyle` | `ST_StrokeJoinStyle` | optional |  |  |
| attribute | `@endcap` | `ST_StrokeEndCap` | optional |  |  |
| attribute | `@dashstyle` | `string` | optional |  |  |
| attribute | `@insetpen` | `ST_TrueFalse` | optional |  |  |
| attribute | `@filltype` | `ST_FillType` | optional |  |  |
| attribute | `@src` | `string` | optional |  |  |
| attribute | `@imageaspect` | `ST_ImageAspect` | optional |  |  |
| attribute | `@imagesize` | `string` | optional |  |  |
| attribute | `@imagealignshape` | `ST_TrueFalse` | optional |  |  |
| attribute | `@startarrow` | `ST_StrokeArrowType` | optional |  |  |
| attribute | `@startarrowwidth` | `ST_StrokeArrowWidth` | optional |  |  |
| attribute | `@startarrowlength` | `ST_StrokeArrowLength` | optional |  |  |
| attribute | `@endarrow` | `ST_StrokeArrowType` | optional |  |  |
| attribute | `@endarrowwidth` | `ST_StrokeArrowWidth` | optional |  |  |
| attribute | `@endarrowlength` | `ST_StrokeArrowLength` | optional |  |  |
| attribute | `@href` | `href` | optional |  |  |
| attribute | `@althref` | `althref` | optional |  |  |
| attribute | `@title` | `title` | optional |  |  |
| attribute | `@forcedash` | `forcedash` | optional |  |  |

## `o:CT_ClipPath`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@v` | `string` | required |  |  |

## `o:CT_Fill`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attrgroup | `AG_Ext` | `AG_Ext` |  |  |  |
| attribute | `@type` | `ST_FillType` | optional |  |  |

## `o:ST_RType`
Restriction: `base=string`
- `arc`
- `callout`
- `connector`
- `align`

## `o:ST_How`
Restriction: `base=string`
- `top`
- `middle`
- `bottom`
- `left`
- `center`
- `right`

## `o:ST_BWMode`
Restriction: `base=string`
- `color`
- `auto`
- `grayScale`
- `lightGrayscale`
- `inverseGray`
- `grayOutline`
- `highContrast`
- `black`
- `white`
- `hide`
- `undrawn`
- `blackTextAndLines`

## `o:ST_ScreenSize`
Restriction: `base=string`
- `544,376`
- `640,480`
- `720,512`
- `800,600`
- `1024,768`
- `1152,862`

## `o:ST_InsetMode`
Restriction: `base=string`
- `auto`
- `custom`

## `o:ST_ColorMode`
Restriction: `base=string`
- `auto`
- `custom`

## `o:ST_ContentType`
Restriction: `base=string`

## `o:ST_DiagramLayout`
Restriction: `base=integer`
- `0`
- `1`
- `2`
- `3`

## `o:ST_ExtrusionType`
Restriction: `base=string`
- `perspective`
- `parallel`

## `o:ST_ExtrusionRender`
Restriction: `base=string`
- `solid`
- `wireFrame`
- `boundingCube`

## `o:ST_ExtrusionPlane`
Restriction: `base=string`
- `XY`
- `ZX`
- `YZ`

## `o:ST_Angle`
Restriction: `base=string`
- `any`
- `30`
- `45`
- `60`
- `90`
- `auto`

## `o:ST_CalloutDrop`
Restriction: `base=string`

## `o:ST_CalloutPlacement`
Restriction: `base=string`
- `top`
- `center`
- `bottom`
- `user`

## `o:ST_ConnectorType`
Restriction: `base=string`
- `none`
- `straight`
- `elbow`
- `curved`

## `o:ST_HrAlign`
Restriction: `base=string`
- `left`
- `right`
- `center`

## `o:ST_ConnectType`
Restriction: `base=string`
- `none`
- `rect`
- `segments`
- `custom`

## `o:ST_OLELinkType`
Restriction: `base=string`

## `o:ST_OLEType`
Restriction: `base=string`
- `Embed`
- `Link`

## `o:ST_OLEDrawAspect`
Restriction: `base=string`
- `Content`
- `Icon`

## `o:ST_OLEUpdateMode`
Restriction: `base=string`
- `Always`
- `OnCall`

## `o:ST_FillType`
Restriction: `base=string`
- `gradientCenter`
- `solid`
- `pattern`
- `tile`
- `frame`
- `gradientUnscaled`
- `gradientRadial`
- `gradient`
- `background`

## `pvml:<iscomment>` : `CT_Empty`

## `pvml:<textdata>` : `CT_Rel`

## `pvml:CT_Empty`

## `pvml:CT_Rel`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@id` | `string` | optional |  |  |

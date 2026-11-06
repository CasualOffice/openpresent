# MODEL3D: 3D models

Generated. Tier 1L; render 2F; edit -; level Retain. 97 rows.


## `x:drawing/2017/model3d:CT_PositiveRatio`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@n` | `unsignedLong` | required |  |  |
| attribute | `@d` | `unsignedLong` | required |  |  |

## `x:drawing/2017/model3d:CT_Rotate3D`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@ax` | `ST_Angle` | optional | 0 |  |
| attribute | `@ay` | `ST_Angle` | optional | 0 |  |
| attribute | `@az` | `ST_Angle` | optional | 0 |  |

## `x:drawing/2017/model3d:CT_Scale3D`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sx` | `CT_Ratio` |  |  |  |
| element | `sy` | `CT_Ratio` |  |  |  |
| element | `sz` | `CT_Ratio` |  |  |  |

## `x:drawing/2017/model3d:CT_Model3DTransform`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `meterPerModelUnit` | `CT_PositiveRatio` | 0..1 |  |  |
| element | `preTrans` | `CT_Vector3D` | 0..1 |  |  |
| element | `scale` | `CT_Scale3D` | 0..1 |  |  |
| element | `rot` | `CT_Rotate3D` | 0..1 |  |  |
| element | `postTrans` | `CT_Vector3D` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `x:drawing/2017/model3d:CT_OrthographicProjection`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `sz` | `CT_PositiveRatio` |  |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `x:drawing/2017/model3d:CT_PerspectiveProjection`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@fov` | `ST_FOVAngle` | required |  |  |

## `x:drawing/2017/model3d:CT_Model3DCamera`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `pos` | `CT_Point3D` |  |  |  |
| element | `up` | `CT_Vector3D` |  |  |  |
| element | `lookAt` | `CT_Point3D` |  |  |  |
| element | `orthographic` | `CT_OrthographicProjection` | choice |  |  |
| element | `perspective` | `CT_PerspectiveProjection` | choice |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `x:drawing/2017/model3d:CT_Model3DRaster`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `blip` | `CT_Blip` | 0..1 |  |  |
| attribute | `@rName` | `string` | required |  |  |
| attribute | `@rVer` | `string` | required |  |  |

## `x:drawing/2017/model3d:CT_ObjectViewport`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@viewportSz` | `ST_PositiveCoordinate` | required |  |  |

## `x:drawing/2017/model3d:CT_WindowViewport`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |

## `x:drawing/2017/model3d:CT_AmbientLight`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `clr` | `CT_Color` |  |  |  |
| element | `illuminance` | `CT_PositiveRatio` |  |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@enabled` | `boolean` | optional | true |  |

## `x:drawing/2017/model3d:CT_PointLight`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `clr` | `CT_Color` |  |  |  |
| element | `intensity` | `CT_PositiveRatio` |  |  |  |
| element | `pos` | `CT_Point3D` |  |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@enabled` | `boolean` | optional | true |  |
| attribute | `@rad` | `ST_PositiveCoordinate` | required |  |  |

## `x:drawing/2017/model3d:CT_SpotLight`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `clr` | `CT_Color` |  |  |  |
| element | `intensity` | `CT_PositiveRatio` |  |  |  |
| element | `pos` | `CT_Point3D` |  |  |  |
| element | `lookAt` | `CT_Point3D` |  |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@enabled` | `boolean` | optional | true |  |
| attribute | `@rad` | `ST_PositiveCoordinate` | required |  |  |
| attribute | `@spotAng` | `ST_FOVAngle` | required |  |  |

## `x:drawing/2017/model3d:ST_AngularRadius`
Restriction: `base=ST_Angle minInclusive=0 maxInclusive=5400000`

## `x:drawing/2017/model3d:CT_DirectionalLight`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `clr` | `CT_Color` |  |  |  |
| element | `illuminance` | `CT_PositiveRatio` |  |  |  |
| element | `pos` | `CT_Point3D` |  |  |  |
| element | `lookAt` | `CT_Point3D` |  |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@enabled` | `boolean` | optional | true |  |
| attribute | `@angularRad` | `ST_AngularRadius` | required |  |  |

## `x:drawing/2017/model3d:CT_UnknownLight`

## `x:drawing/2017/model3d:CT_Model3D`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `spPr` | `CT_ShapeProperties` |  |  |  |
| element | `camera` | `CT_Model3DCamera` |  |  |  |
| element | `trans` | `CT_Model3DTransform` |  |  |  |
| element | `attrSrcUrl` | `CT_PictureAttributionSourceURL` | 0..1 |  |  |
| element | `raster` | `CT_Model3DRaster` | 0..1 |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| element | `objViewport` | `CT_ObjectViewport` | choice |  |  |
| element | `winViewport` | `CT_WindowViewport` | choice |  |  |
| element | `ambientLight` | `CT_AmbientLight` | 0..1 |  |  |
| element | `ptLight` | `CT_PointLight` | choice |  |  |
| element | `spotLight` | `CT_SpotLight` | choice |  |  |
| element | `dirLight` | `CT_DirectionalLight` | choice |  |  |
| element | `unkLight` | `CT_UnknownLight` | choice |  |  |
| attrgroup | `AG_Blob` | `AG_Blob` |  |  |  |

## `x:drawing/2017/model3d:<model3d>` : `CT_Model3D`

## `x:drawing/2018/animation/model3d:ST_EmbeddedAnimationID`
Restriction: `base=unsignedInt`

## `x:drawing/2018/animation/model3d:CT_EmbeddedAnimation`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| element | `animPr` | `CT_AnimationProperties` |  |  |  |
| element | `extLst` | `CT_OfficeArtExtensionList` | 0..1 |  |  |
| attribute | `@animId` | `ST_EmbeddedAnimationID` | required |  |  |

## `x:drawing/2018/animation/model3d:CT_PosterFrame`

| Kind | Name | Type | Use | Default | Restriction |
| --- | --- | --- | --- | --- | --- |
| attribute | `@animId` | `ST_EmbeddedAnimationID` | required |  |  |
| attribute | `@frame` | `ST_PositiveFixedPercentage` | optional | 0 |  |

## `x:drawing/2018/animation/model3d:<embedAnim>` : `CT_EmbeddedAnimation`

## `x:drawing/2018/animation/model3d:<posterFrame>` : `CT_PosterFrame`

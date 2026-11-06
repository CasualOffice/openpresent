# Package parts and relationship types

Schemas describe what is inside a part. They do not list the parts. This list is extracted from the text of
ECMA-376 Part 1 (2016), filtered by hand to presentations, and is the checklist for Tier 1A's part graph. Extension
parts come from MS-PPTX and MS-ODRAWXML (`SPEC-04`: their part and relationship names are not yet enumerated here).

Strict files use `http://purl.oclc.org/ooxml/officeDocument/relationships/…`. Transitional files use
`http://schemas.openxmlformats.org/officeDocument/2006/relationships/…`. **Both are read; a file is written in the form it arrived in.**

## Content types

| Part | Content type |
| --- | --- |
| Presentation | `application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml` |
| Slide show (`.ppsx`) | `application/vnd.openxmlformats-officedocument.presentationml.slideshow.main+xml` |
| Template (`.potx`) | `application/vnd.openxmlformats-officedocument.presentationml.template.main+xml` |
| Slide | `…presentationml.slide+xml` |
| Slide layout | `…presentationml.slideLayout+xml` |
| Slide master | `…presentationml.slideMaster+xml` |
| Notes slide | `…presentationml.notesSlide+xml` |
| Notes master | `…presentationml.notesMaster+xml` |
| Handout master | `…presentationml.handoutMaster+xml` |
| Comments | `…presentationml.comments+xml` |
| Comment authors | `…presentationml.commentAuthors+xml` |
| Presentation properties | `…presentationml.presProps+xml` |
| View properties | `…presentationml.viewProps+xml` |
| Table styles | `…presentationml.tableStyles+xml` |
| Tags | `…presentationml.tags+xml` |
| Slide update info | `…presentationml.slideUpdateInfo+xml` |
| Theme | `application/vnd.openxmlformats-officedocument.theme+xml` |
| Theme override | `application/vnd.openxmlformats-officedocument.themeOverride+xml` |
| Chart | `…drawingml.chart+xml` |
| Chart user shapes | `…drawingml.chartshapes+xml` |
| Diagram data, layout, style, colours | `…drawingml.diagramData+xml`, `diagramLayout`, `diagramStyle`, `diagramColors` |
| Core properties | `application/vnd.openxmlformats-package.core-properties+xml` |
| Extended properties | `…officedocument.extended-properties+xml` |
| Custom properties | `…officedocument.custom-properties+xml` |
| Custom XML properties | `…officedocument.customXmlProperties+xml` |
| ActiveX | `application/vnd.ms-office.activeX+xml` |
| Relationships | `application/vnd.openxmlformats-package.relationships+xml` |

## Relationship types

Suffix after the base above: `officeDocument`, `extended-properties` and `custom-properties` in Transitional files (`extendedProperties` and `customProperties` in Strict), `metadata/thumbnail`, `slide`,
`slideLayout`, `slideMaster`, `notesSlide`, `notesMaster`, `handoutMaster`, `presProps`, `viewProps`, `tableStyles`, `tags`, `theme`,
`themeOverride`, `comments`, `commentAuthors`, `slideUpdateInfo`, `slideUpdateUrl`, `image`, `audio`, `video`, `movie`, `hyperlink`,
`chart`, `chartUserShapes`, `diagramData`, `diagramLayout`, `diagramQuickStyle`, `diagramColors`, `oleObject`, `package`, `control`,
`font`, `customXml`, `customXmlProps`, `printerSettings`, `vmlDrawing`, `htmlPubSaveAs`, `attachedTemplate`.

The part graph must also tolerate **unknown** relationship types: retained, named in the report, and never followed if external.

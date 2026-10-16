"""Clipboard exercise confined to the probe's explicitly isolated X display."""
import ctypes as C
import ctypes.util
import time


class Selection(C.Structure):
    _fields_ = [('type', C.c_int), ('serial', C.c_ulong), ('send_event', C.c_int), ('display', C.c_void_p), ('requestor', C.c_ulong), ('selection', C.c_ulong), ('target', C.c_ulong), ('property', C.c_ulong), ('time', C.c_ulong)]


def copy_report(display_name, origin):
    x = C.CDLL(ctypes.util.find_library('X11'))
    t = C.CDLL(ctypes.util.find_library('Xtst'))
    x.XOpenDisplay.argtypes = [C.c_char_p]
    x.XOpenDisplay.restype = C.c_void_p
    x.XCloseDisplay.argtypes = [C.c_void_p]
    x.XDefaultRootWindow.argtypes = [C.c_void_p]
    x.XDefaultRootWindow.restype = C.c_ulong
    x.XCreateSimpleWindow.argtypes = [C.c_void_p, C.c_ulong, C.c_int, C.c_int, C.c_uint, C.c_uint, C.c_uint, C.c_ulong, C.c_ulong]
    x.XCreateSimpleWindow.restype = C.c_ulong
    x.XDestroyWindow.argtypes = [C.c_void_p, C.c_ulong]
    x.XInternAtom.argtypes = [C.c_void_p, C.c_char_p, C.c_int]
    x.XInternAtom.restype = C.c_ulong
    x.XConvertSelection.argtypes = [C.c_void_p, C.c_ulong, C.c_ulong, C.c_ulong, C.c_ulong, C.c_ulong]
    x.XFlush.argtypes = [C.c_void_p]
    x.XPending.argtypes = [C.c_void_p]
    x.XNextEvent.argtypes = [C.c_void_p, C.c_void_p]
    x.XGetWindowProperty.argtypes = [C.c_void_p, C.c_ulong, C.c_ulong, C.c_long, C.c_long, C.c_int, C.c_ulong, C.POINTER(C.c_ulong), C.POINTER(C.c_int), C.POINTER(C.c_ulong), C.POINTER(C.c_ulong), C.POINTER(C.POINTER(C.c_ubyte))]
    x.XFree.argtypes = [C.c_void_p]
    t.XTestFakeMotionEvent.argtypes = [C.c_void_p, C.c_int, C.c_int, C.c_int, C.c_ulong]
    t.XTestFakeButtonEvent.argtypes = [C.c_void_p, C.c_uint, C.c_int, C.c_ulong]
    display = x.XOpenDisplay(display_name.encode())
    if not display:
        raise RuntimeError('Isolated clipboard display unavailable')
    window = None
    try:
        # Logical coordinates for the 960x640, 100% diagnostic fixture.
        t.XTestFakeMotionEvent(display, 0, origin['x'] + 80, origin['y'] + 200, 0)
        t.XTestFakeButtonEvent(display, 1, 1, 0)
        t.XTestFakeButtonEvent(display, 1, 0, 0)
        x.XFlush(display)
        time.sleep(0.3)
        window = x.XCreateSimpleWindow(display, x.XDefaultRootWindow(display), 0, 0, 1, 1, 0, 0, 0)
        selection = x.XInternAtom(display, b'CLIPBOARD', 0)
        target = x.XInternAtom(display, b'UTF8_STRING', 0)
        property_atom = x.XInternAtom(display, b'DLPAD_PROBE_CLIPBOARD', 0)
        x.XConvertSelection(display, selection, target, property_atom, window, 0)
        x.XFlush(display)
        deadline = time.monotonic() + 3
        event = (C.c_long * 24)()
        while time.monotonic() < deadline:
            if not x.XPending(display):
                time.sleep(0.01)
                continue
            x.XNextEvent(display, C.byref(event))
            notification = C.cast(C.byref(event), C.POINTER(Selection)).contents
            if notification.type != 31:
                continue
            if not notification.property:
                raise RuntimeError('Diagnostic clipboard conversion was refused')
            actual_type, count, remaining = C.c_ulong(), C.c_ulong(), C.c_ulong()
            format_bits, data = C.c_int(), C.POINTER(C.c_ubyte)()
            status = x.XGetWindowProperty(display, window, property_atom, 0, 2 * 1024 * 1024 // 4, 0, 0, C.byref(actual_type), C.byref(format_bits), C.byref(count), C.byref(remaining), C.byref(data))
            try:
                if status or format_bits.value != 8 or remaining.value or count.value > 2 * 1024 * 1024:
                    raise RuntimeError('Unsupported or oversized clipboard result')
                report = C.string_at(data, count.value).decode('utf-8')
            finally:
                if data:
                    x.XFree(data)
            if not report.startswith('# dlpad diagnostic report'):
                raise RuntimeError('Clipboard did not contain the diagnostic report')
            if 'media.invalid' in report or 'signature=fixture' in report:
                raise RuntimeError('Clipboard contained unredacted fixture source')
            return {'diagnostic_clipboard_verified': True, 'clipboard_utf8_bytes': len(report.encode())}
        raise RuntimeError('Diagnostic clipboard request timed out')
    finally:
        if window is not None:
            x.XDestroyWindow(display, window)
        x.XCloseDisplay(display)


def scroll_to_end(display_name, origin, steps=30):
    """Only called after the probe rejects desktop input injection."""
    x = C.CDLL(ctypes.util.find_library('X11'))
    t = C.CDLL(ctypes.util.find_library('Xtst'))
    x.XOpenDisplay.argtypes = [C.c_char_p]
    x.XOpenDisplay.restype = C.c_void_p
    x.XCloseDisplay.argtypes = [C.c_void_p]
    x.XFlush.argtypes = [C.c_void_p]
    t.XTestFakeMotionEvent.argtypes = [C.c_void_p, C.c_int, C.c_int, C.c_int, C.c_ulong]
    t.XTestFakeButtonEvent.argtypes = [C.c_void_p, C.c_uint, C.c_int, C.c_ulong]
    display = x.XOpenDisplay(display_name.encode())
    if not display:
        raise RuntimeError('Isolated scrolling display unavailable')
    try:
        t.XTestFakeMotionEvent(display, 0, origin['x'] + 400, origin['y'] + 400, 0)
        for _ in range(steps):
            t.XTestFakeButtonEvent(display, 5, 1, 0)
            t.XTestFakeButtonEvent(display, 5, 0, 0)
            x.XFlush(display)
            time.sleep(0.015)
        time.sleep(0.3)
    finally:
        x.XCloseDisplay(display)


def click_at(display_name, origin, dx, dy):
    """Caller must require isolated display before injecting any input."""
    x = C.CDLL(ctypes.util.find_library('X11'))
    t = C.CDLL(ctypes.util.find_library('Xtst'))
    x.XOpenDisplay.argtypes = [C.c_char_p]; x.XOpenDisplay.restype = C.c_void_p
    x.XCloseDisplay.argtypes = [C.c_void_p]; x.XFlush.argtypes = [C.c_void_p]
    t.XTestFakeMotionEvent.argtypes = [C.c_void_p, C.c_int, C.c_int, C.c_int, C.c_ulong]
    t.XTestFakeButtonEvent.argtypes = [C.c_void_p, C.c_uint, C.c_int, C.c_ulong]
    display = x.XOpenDisplay(display_name.encode())
    if not display: raise RuntimeError('Isolated input display unavailable')
    try:
        t.XTestFakeMotionEvent(display, 0, origin['x'] + dx, origin['y'] + dy, 0)
        t.XTestFakeButtonEvent(display, 1, 1, 0); t.XTestFakeButtonEvent(display, 1, 0, 0)
        x.XFlush(display)
    finally:
        x.XCloseDisplay(display)

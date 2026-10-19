// Logs StreamDock's HID traffic at the IOKit layer (hidapi is linked statically).
// Use: DYLD_INSERT_LIBRARIES=spy.dylib HIDSPY_LOG=file
#include <IOKit/hid/IOHIDDevice.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

static pthread_mutex_t mu = PTHREAD_MUTEX_INITIALIZER;
static void dump(const char *tag, IOHIDReportType t, CFIndex id, const uint8_t *b, CFIndex n) {
    static FILE *f;
    if (n <= 0) return;
    pthread_mutex_lock(&mu);
    if (!f) f = fopen(getenv("HIDSPY_LOG") ? getenv("HIDSPY_LOG") : "/tmp/hidspy.log", "a");
    struct timespec ts;
    clock_gettime(CLOCK_REALTIME, &ts);
    fprintf(f, "%ld.%03ld %s type=%d id=%ld len=%ld ", ts.tv_sec, ts.tv_nsec / 1000000, tag, t, (long)id, (long)n);
    CFIndex last = n;
    while (last > 0 && b[last - 1] == 0) last--;  // trim zero padding
    for (CFIndex i = 0; i < last && i < 64; i++) fprintf(f, "%02x", b[i]);
    fprintf(f, " |");
    for (CFIndex i = 0; i < last && i < 64; i++) fputc(b[i] >= 32 && b[i] < 127 ? b[i] : '.', f);
    fprintf(f, "|\n");
    fflush(f);
    pthread_mutex_unlock(&mu);
}

static IOReturn my_set(IOHIDDeviceRef d, IOHIDReportType t, CFIndex id, const uint8_t *b, CFIndex n) {
    dump("W", t, id, b, n);
    return IOHIDDeviceSetReport(d, t, id, b, n);
}
static IOReturn my_get(IOHIDDeviceRef d, IOHIDReportType t, CFIndex id, uint8_t *b, CFIndex *n) {
    IOReturn r = IOHIDDeviceGetReport(d, t, id, b, n);
    dump("GET", t, id, b, *n);
    return r;
}

// ponytail: one saved callback, StreamDock opens one device here; per-device table if it opens several
static IOHIDReportCallback cb1;
static IOHIDReportWithTimeStampCallback cb2;
static void wrap1(void *ctx, IOReturn r, void *s, IOHIDReportType t, uint32_t id, uint8_t *b, CFIndex n) {
    dump("R", t, id, b, n);
    cb1(ctx, r, s, t, id, b, n);
}
static void wrap2(void *ctx, IOReturn r, void *s, IOHIDReportType t, uint32_t id, uint8_t *b, CFIndex n, uint64_t ts) {
    dump("R", t, id, b, n);
    cb2(ctx, r, s, t, id, b, n, ts);
}
static void my_reg1(IOHIDDeviceRef d, uint8_t *b, CFIndex n, IOHIDReportCallback cb, void *ctx) {
    cb1 = cb;
    IOHIDDeviceRegisterInputReportCallback(d, b, n, cb ? wrap1 : NULL, ctx);
}
static void my_reg2(IOHIDDeviceRef d, uint8_t *b, CFIndex n, IOHIDReportWithTimeStampCallback cb, void *ctx) {
    cb2 = cb;
    IOHIDDeviceRegisterInputReportWithTimeStampCallback(d, b, n, cb ? wrap2 : NULL, ctx);
}

#define INTERPOSE(n, o) __attribute__((used)) static struct { const void *a, *b; } _i_##o \
    __attribute__((section("__DATA,__interpose"))) = {(const void *)n, (const void *)o};
INTERPOSE(my_set, IOHIDDeviceSetReport)
INTERPOSE(my_get, IOHIDDeviceGetReport)
INTERPOSE(my_reg1, IOHIDDeviceRegisterInputReportCallback)
INTERPOSE(my_reg2, IOHIDDeviceRegisterInputReportWithTimeStampCallback)

__attribute__((constructor)) static void loaded(void) { dump("LOADED", 0, 0, (const uint8_t *)"spy", 3); }

#ifndef SKEY_ENGINE_H
#define SKEY_ENGINE_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

#define SKEY_METHOD_TELEX    0
#define SKEY_METHOD_VNI      1
#define SKEY_METHOD_VIQR     2
#define SKEY_METHOD_TEIP_VNI 3

typedef void SkeyEngine;

// Lifecycle
SkeyEngine *skey_engine_new(int32_t method);
void        skey_engine_free(SkeyEngine *e);

// Configuration
void skey_engine_set_method(SkeyEngine *e, int32_t method);
void skey_engine_set_tone_style(SkeyEngine *e, int32_t modern);
void skey_engine_set_free_marking(SkeyEngine *e, int32_t free);
void skey_engine_set_short_w(SkeyEngine *e, int32_t enabled);
void skey_engine_set_bracket_uo(SkeyEngine *e, int32_t enabled);

// Core — stateless string transform
char *skey_engine_transform(SkeyEngine *e, const char *input);

// Validation — works on any composed string, not tied to engine state
int32_t skey_engine_is_valid(const char *s);

// Free a string returned by skey_engine_transform
void skey_free_string(char *s);

#ifdef __cplusplus
}
#endif

#endif // SKEY_ENGINE_H

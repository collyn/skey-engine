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
void skey_engine_set_auto_restore(SkeyEngine *e, int32_t enabled);
// Dictionary mode: auto-restore validates against the embedded Vietnamese
// word list instead of syllable rules.  User words can extend it.
void skey_engine_set_dict(SkeyEngine *e, int32_t enabled);
void skey_engine_add_word(SkeyEngine *e, const char *word);
void skey_engine_set_bracket_uo(SkeyEngine *e, int32_t enabled);

// Core — stateless string transform
char *skey_engine_transform(SkeyEngine *e, const char *input);

// Validation — works on any composed string, not tied to engine state
int32_t skey_engine_is_valid(const char *s);

// Free a string returned by skey_engine_transform
void skey_free_string(char *s);

// ── Charset conversion ────────────────────────────────────────────────
// Charset IDs (matches VietCharset enum in Rust):
//   0 = Unicode, 1 = TCVN3, 2 = VNI-WIN, 3 = WinCP1258, 4 = VIQR

// Encode UTF-8 string to target charset bytes.  out_len receives the
// number of bytes written.  Caller must free the returned buffer with
// skey_charset_free_buf().
uint8_t *skey_charset_encode(const char *input, int32_t charset,
                             size_t *out_len);

// Decode charset bytes to UTF-8 string.  Caller must free with
// skey_free_string().
char *skey_charset_decode(const uint8_t *input, size_t len, int32_t charset);

// Remove tone marks from UTF-8 Vietnamese text.
char *skey_charset_remove_tone(const char *input);

// Free buffer returned by skey_charset_encode.
void skey_charset_free_buf(uint8_t *buf);

#ifdef __cplusplus
}
#endif

#endif // SKEY_ENGINE_H

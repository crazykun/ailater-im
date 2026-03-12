/*
 * fcitx5-ai-im - AI-powered input method for fcitx5
 * 
 * Fcitx5 C API bindings for Rust
 */

#ifndef FCITX5_AI_IM_Fcitx5_H
#define FCITX5_AI_IM_Fcitx5_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Fcitx5 Key Symbols */
#define FCITX_KEY_None 0
#define FCITX_KEY_Shift_L 0xffe1
#define FCITX_KEY_Shift_R 0xffe2
#define FCITX_KEY_Control_L 0xffe3
#define FCITX_KEY_Control_R 0xffe4
#define FCITX_KEY_Alt_L 0xffe9
#define FCITX_KEY_Alt_R 0xffea
#define FCITX_KEY_Super_L 0xffeb
#define FCITX_KEY_Super_R 0xffec
#define FCITX_KEY_BackSpace 0xff08
#define FCITX_KEY_Tab 0xff09
#define FCITX_KEY_Return 0xff0d
#define FCITX_KEY_Escape 0xff1b
#define FCITX_KEY_space 0x0020
#define FCITX_KEY_Delete 0xffff
#define FCITX_KEY_Left 0xff51
#define FCITX_KEY_Up 0xff52
#define FCITX_KEY_Right 0xff53
#define FCITX_KEY_Down 0xff54
#define FCITX_KEY_Page_Up 0xff55
#define FCITX_KEY_Page_Down 0xff56

/* Key states */
#define FCITX_KEY_STATE_NONE 0
#define FCITX_KEY_STATE_SHIFT (1 << 0)
#define FCITX_KEY_STATE_CTRL (1 << 1)
#define FCITX_KEY_STATE_ALT (1 << 2)
#define FCITX_KEY_STATE_SUPER (1 << 3)

/* Input Method Return Values */
#define FCITX_IM_RETVAL_IGNORE 0
#define FCITX_IM_RETVAL_FORWARD 1
#define FCITX_IM_RETVAL_CONSUME 2

/* Forward declarations for opaque types */
typedef struct _FcitxInstance FcitxInstance;
typedef struct _FcitxInputContext FcitxInputContext;
typedef struct _FcitxAddon FcitxAddon;
typedef struct _FcitxIMClass FcitxIMClass;
typedef struct _FcitxInputMethod FcitxInputMethod;

/* Input Method Class Structure */
struct _FcitxIMClass {
    void* (*create)(FcitxInstance* instance);
    void (*destroy)(void* arg);
};

/* Input Method Entry */
typedef struct {
    const char* unique_name;
    const char* name;
    const char* icon_name;
    int priority;
    const char* lang_code;
    void* user_data;
} FcitxIMEntry;

/* Callback function types */
typedef int (*FcitxKeyEventHandler)(
    void* arg,
    FcitxInputContext* ic,
    uint32_t keysym,
    uint32_t keycode,
    uint32_t state,
    bool is_release
);

typedef void (*FcitxResetHandler)(void* arg, FcitxInputContext* ic);
typedef void (*FcitxFocusInHandler)(void* arg, FcitxInputContext* ic);
typedef void (*FcitxFocusOutHandler)(void* arg, FcitxInputContext* ic);
typedef void (*FcitxInitHandler)(void* arg, FcitxInstance* instance);
typedef void (*FcitxDestroyHandler)(void* arg);

/* API Functions - These are provided by fcitx5 */
void fcitx_instance_commit_string(FcitxInstance* instance, 
                                   FcitxInputContext* ic, 
                                   const char* str);

void fcitx_instance_set_preedit(FcitxInstance* instance,
                                 FcitxInputContext* ic,
                                 const char* str,
                                 int cursor_pos);

void fcitx_instance_update_preedit(FcitxInstance* instance,
                                    FcitxInputContext* ic);

void fcitx_instance_forward_key(FcitxInstance* instance,
                                 FcitxInputContext* ic,
                                 uint32_t keysym,
                                 uint32_t state,
                                 bool is_release);

/* Candidate List Functions */
typedef struct _FcitxCandidateList FcitxCandidateList;
typedef struct _FcitxCandidateWord FcitxCandidateWord;

FcitxCandidateList* fcitx_candidate_list_new(void);
void fcitx_candidate_list_free(FcitxCandidateList* list);
void fcitx_candidate_list_append(FcitxCandidateList* list, 
                                  const char* word,
                                  void* user_data);
int fcitx_candidate_list_size(FcitxCandidateList* list);
const char* fcitx_candidate_list_get(FcitxCandidateList* list, int index);

/* UI Update Functions */
void fcitx_instance_update_ui(FcitxInstance* instance, 
                               FcitxInputContext* ic);

/* Configuration Functions */
typedef struct _FcitxConfiguration FcitxConfiguration;

char* fcitx_config_get_string(FcitxConfiguration* config, 
                               const char* key,
                               const char* default_value);
int fcitx_config_get_int(FcitxConfiguration* config,
                          const char* key,
                          int default_value);
bool fcitx_config_get_bool(FcitxConfiguration* config,
                            const char* key,
                            bool default_value);

/* Addon Registration */
typedef struct {
    FcitxIMClass im_class;
    FcitxIMEntry* entries;
    int num_entries;
} FcitxAddonInfo;

/* Plugin entry point - must be exported */
FCITX_EXPORT void* fcitx_im_get_class(void);

#ifdef __cplusplus
}
#endif

#endif /* FCITX5_AI_IM_Fcitx5_H */

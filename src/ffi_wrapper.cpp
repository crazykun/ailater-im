/*
 * ailater-im - C++ wrapper for fcitx5 plugin
 *
 * This file provides the C++ interface that fcitx5 expects,
 * and forwards calls to the Rust implementation.
 */

#include <fcitx/addonfactory.h>
#include <fcitx/addonmanager.h>
#include <fcitx/instance.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/inputmethodentry.h>
#include <fcitx/inputpanel.h>
#include <fcitx/event.h>
#include <fcitx/candidatelist.h>
#include <fcitx-utils/i18n.h>
#include <fcitx-utils/log.h>

#include <string>
#include <memory>
#include <vector>

// External Rust functions
extern "C" {
    void* ailater_engine_create(void* instance);
    void ailater_engine_destroy(void* engine);
    int ailater_engine_handle_key(void* engine, void* ic, uint32_t keysym, uint32_t keycode, uint32_t state, bool is_release);
    void ailater_engine_reset(void* engine, void* ic);
    const char* ailater_engine_get_preedit(void* engine, void* ic);
    const char* ailater_engine_get_commit_text(void* engine, void* ic);
    const char** ailater_engine_get_candidates(void* engine, void* ic);
    size_t ailater_engine_get_candidate_count(void* engine, void* ic);
    size_t ailater_engine_get_current_page(void* engine, void* ic);
    size_t ailater_engine_get_selected_index(void* engine, void* ic);
    void ailater_engine_free_string(char* s);
}

namespace fcitx {

// CandidateWord that stores the index within the current page (0-based)
class AilaterCandidateWord : public CandidateWord {
public:
    AilaterCandidateWord(const std::string& text, int indexInPage, void* engine, InputContext* ic)
        : CandidateWord(Text(text)), indexInPage_(indexInPage), engine_(engine), ic_(ic) {}

    void select(InputContext* inputContext) const override {
        // Press the number key (1-9) corresponding to the index
        // indexInPage_ is 0-based, so add 1 to get the key number
        uint32_t keysym = 0x30 + (indexInPage_ + 1);
        ailater_engine_handle_key(engine_, ic_, keysym, 0, 0, false);
        inputContext->updateUserInterface(UserInterfaceComponent::InputPanel);
    }

private:
    int indexInPage_;
    void* engine_;
    InputContext* ic_;
};

class AilaterEngine : public InputMethodEngineV2 {
public:
    AilaterEngine(Instance* instance) : instance_(instance) {
        engine_ = ailater_engine_create(instance);
        FCITX_INFO() << "AilaterEngine created";
    }

    ~AilaterEngine() override {
        if (engine_) {
            ailater_engine_destroy(engine_);
        }
        FCITX_INFO() << "AilaterEngine destroyed";
    }

    void activate(const InputMethodEntry& entry, InputContextEvent& event) override {
        FCITX_UNUSED(entry);
        FCITX_UNUSED(event);
        FCITX_INFO() << "AilaterEngine activated";
    }

    void deactivate(const InputMethodEntry& entry, InputContextEvent& event) override {
        FCITX_UNUSED(entry);
        FCITX_UNUSED(event);
        FCITX_INFO() << "AilaterEngine deactivated";
    }

    void keyEvent(const InputMethodEntry& entry, KeyEvent& keyEvent) override {
        FCITX_UNUSED(entry);

        auto* ic = keyEvent.inputContext();
        Key key = keyEvent.key();

        if (keyEvent.isRelease()) {
            return;
        }

        uint32_t keysym = key.sym();
        uint32_t keycode = key.code();
        uint32_t state = 0;

        if (key.states().test(KeyState::Shift)) state |= (1 << 0);
        if (key.states().test(KeyState::Ctrl)) state |= (1 << 1);
        if (key.states().test(KeyState::Alt)) state |= (1 << 2);
        if (key.states().test(KeyState::Super)) state |= (1 << 3);

        // fcitx5 KeySym values for arrow and page keys
        const uint32_t FcitxKey_Up = 0xff52;
        const uint32_t FcitxKey_Down = 0xff54;
        const uint32_t FcitxKey_PageUp = 0xff55;
        const uint32_t FcitxKey_PageDown = 0xff56;
        const uint32_t FcitxKey_equal = 0x3d;   // = key
        const uint32_t FcitxKey_minus = 0x2d;   // - key
        const uint32_t FcitxKey_plus = 0x2b;    // + key (Shift+=)

        // Check if we have candidates
        size_t candidateCount = ailater_engine_get_candidate_count(engine_, ic);
        bool hasCandidates = (candidateCount > 0);

        // Handle paging keys when we have candidates
        // Up/Down for page navigation, Left/Right are passed to Rust for candidate selection
        if (hasCandidates) {
            int pageSize = instance_->globalConfig().defaultPageSize();
            size_t currentPage = ailater_engine_get_current_page(engine_, ic);
            size_t maxPage = (candidateCount - 1) / pageSize;

            // Next page keys: =, +, PageDown, Down
            // Only allow if not on last page
            if ((keysym == FcitxKey_equal || keysym == FcitxKey_plus ||
                 keysym == FcitxKey_PageDown || keysym == FcitxKey_Down) && currentPage < maxPage) {
                keyEvent.filterAndAccept();
                // Call Rust engine with Down keysym to advance page
                ailater_engine_handle_key(engine_, ic, FcitxKey_Down, 0, 0, false);
                updateUI(ic);
                return;
            }
            // Previous page keys: -, PageUp, Up
            // Only allow if not on first page
            if ((keysym == FcitxKey_minus || keysym == FcitxKey_PageUp ||
                 keysym == FcitxKey_Up) && currentPage > 0) {
                keyEvent.filterAndAccept();
                // Call Rust engine with Up keysym to go back
                ailater_engine_handle_key(engine_, ic, FcitxKey_Up, 0, 0, false);
                updateUI(ic);
                return;
            }
        }

        // Pass other keys to Rust engine
        int result = ailater_engine_handle_key(engine_, ic, keysym, keycode, state, keyEvent.isRelease());

        if (result == 2) {  // CONSUME
            keyEvent.filterAndAccept();
            updateUI(ic);
        } else if (result == 1) {  // FORWARD
            // Let fcitx5 framework handle it
        } else {
            // result == 0 (IGNORE)
            updateUI(ic);
        }
    }

    void reset(const InputMethodEntry& entry, InputContextEvent& event) override {
        FCITX_UNUSED(entry);
        ailater_engine_reset(engine_, event.inputContext());
        updateUI(event.inputContext());
    }

    void updateUI(InputContext* ic) {
        // Check if there's text to commit
        const char* commitText = ailater_engine_get_commit_text(engine_, ic);
        if (commitText && *commitText) {
            ic->commitString(std::string(commitText));
            // After commit, clear input panel
            ic->inputPanel().setPreedit(Text());
            ic->inputPanel().setCandidateList(nullptr);
            ic->updatePreedit();
            ic->updateUserInterface(UserInterfaceComponent::InputPanel);
            return;
        }

        // Get preedit text from Rust
        const char* preedit = ailater_engine_get_preedit(engine_, ic);
        if (preedit && *preedit) {
            ic->inputPanel().setPreedit(Text(preedit));
        } else {
            ic->inputPanel().setPreedit(Text());
        }

        // Get all candidates from Rust
        const char** candidates = ailater_engine_get_candidates(engine_, ic);
        if (candidates && *candidates) {
            int pageSize = instance_->globalConfig().defaultPageSize();
            size_t currentPage = ailater_engine_get_current_page(engine_, ic);

            // Count total candidates
            int totalCandidates = 0;
            for (int i = 0; candidates[i] != nullptr; i++) {
                totalCandidates++;
            }

            // Calculate start and end indices for current page
            int startIndex = static_cast<int>(currentPage * pageSize);
            int endIndex = std::min(startIndex + pageSize, totalCandidates);

            // Clamp current page to valid range
            if (startIndex >= totalCandidates && totalCandidates > 0) {
                // Current page is out of bounds, this shouldn't happen
                // But protect against it by not showing candidates
                FCITX_WARN() << "Current page out of bounds: page=" << currentPage 
                             << " maxPage=" << ((totalCandidates - 1) / pageSize);
            }

            // Only create candidate list if current page has candidates
            if (startIndex < totalCandidates) {
                auto candidateList = std::make_unique<CommonCandidateList>();
                candidateList->setPageSize(pageSize);

                // Set labels to 1-pageSize
                std::vector<std::string> labels;
                for (int i = 0; i < pageSize && i < 9; i++) {
                    labels.push_back(std::to_string(i + 1));
                }
                candidateList->setLabels(labels);

                // Add only candidates for current page
                for (int i = startIndex; i < endIndex; i++) {
                    // Index within current page (0-based)
                    int indexInPage = i - startIndex;
                    auto candidateWord = std::make_unique<AilaterCandidateWord>(
                        candidates[i], indexInPage, engine_, ic
                    );
                    candidateList->append(std::move(candidateWord));
                }

                // Set cursor to the selected candidate from Rust engine
                size_t selectedIndex = ailater_engine_get_selected_index(engine_, ic);
                candidateList->setCursorIndex(static_cast<int>(selectedIndex));

                ic->inputPanel().setCandidateList(std::move(candidateList));
            } else {
                // No candidates on current page, clear the panel
                ic->inputPanel().setCandidateList(nullptr);
            }
        } else {
            ic->inputPanel().setCandidateList(nullptr);
        }

        ic->updatePreedit();
        ic->updateUserInterface(UserInterfaceComponent::InputPanel);
    }

private:
    Instance* instance_;
    void* engine_;
};

class AilaterEngineFactory : public AddonFactory {
public:
    AddonInstance* create(AddonManager* manager) override {
        return new AilaterEngine(manager->instance());
    }
};

}  // namespace fcitx

// Use FCITX_ADDON_FACTORY macro - this creates the fcitx_addon_factory_instance symbol
FCITX_ADDON_FACTORY(fcitx::AilaterEngineFactory);
